use sqlx::PgPool;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Migration not found: {0}")]
    NotFound(String),
    #[error("Invalid migration format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub up_path: PathBuf,
    pub down_path: Option<PathBuf>,
}

impl Migration {
    /// Load migration from a single .sql file (backward compatibility)
    fn from_single_file(path: &Path) -> Result<Self, MigrationError> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MigrationError::InvalidFormat("Invalid file name".to_string()))?;

        // Extract version and name from filename like "20251117000000_initial_schema.sql"
        let (version, name) = parse_migration_filename(file_name)?;

        Ok(Migration {
            version,
            name,
            up_path: path.to_path_buf(),
            down_path: None,
        })
    }

    /// Load migration from .up.sql and optionally .down.sql files
    fn from_up_down_files(up_path: &Path) -> Result<Self, MigrationError> {
        let file_name = up_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MigrationError::InvalidFormat("Invalid file name".to_string()))?;

        // Extract version and name from filename like "20251130000000_add_password_hash_to_player.up.sql"
        let base_name = file_name
            .strip_suffix(".up.sql")
            .ok_or_else(|| MigrationError::InvalidFormat("Not a .up.sql file".to_string()))?;

        let (version, name) = parse_migration_filename(base_name)?;

        // Look for corresponding .down.sql file
        let down_path = up_path
            .parent()
            .map(|p| p.join(format!("{}.down.sql", base_name)));

        let down_path = if let Some(ref dp) = down_path {
            if dp.exists() {
                Some(dp.clone())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Migration {
            version,
            name,
            up_path: up_path.to_path_buf(),
            down_path,
        })
    }

    /// Read the SQL content for the up migration
    pub fn read_up_sql(&self) -> Result<String, MigrationError> {
        fs::read_to_string(&self.up_path).map_err(MigrationError::Io)
    }

    /// Read the SQL content for the down migration
    pub fn read_down_sql(&self) -> Result<Option<String>, MigrationError> {
        match &self.down_path {
            Some(path) => fs::read_to_string(path)
                .map(Some)
                .map_err(MigrationError::Io),
            None => Ok(None),
        }
    }
}

/// Parse migration filename to extract version and name
/// Supports formats like:
/// - "20251117000000_initial_schema.sql" (YYYYMMDDHHMMSS format)
/// - "20251130000000_add_password_hash_to_player"
fn parse_migration_filename(filename: &str) -> Result<(i64, String), MigrationError> {
    // Remove .sql extension if present
    let base = filename.strip_suffix(".sql").unwrap_or(filename);

    // Split on first underscore
    if let Some(underscore_pos) = base.find('_') {
        let version_str = &base[..underscore_pos];
        let name = base[underscore_pos + 1..].to_string();

        // Validate that version is exactly 14 digits (YYYYMMDDHHMMSS)
        if version_str.len() != 14 {
            return Err(MigrationError::InvalidFormat(format!(
                "Migration version must be exactly 14 digits (YYYYMMDDHHMMSS), got: {} ({} digits)",
                version_str,
                version_str.len()
            )));
        }

        // Validate that all characters are digits
        if !version_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(MigrationError::InvalidFormat(format!(
                "Migration version must contain only digits, got: {}",
                version_str
            )));
        }

        let version = version_str.parse::<i64>().map_err(|_| {
            MigrationError::InvalidFormat(format!("Invalid version: {}", version_str))
        })?;

        Ok((version, name))
    } else {
        // Try to parse entire filename as version
        if base.len() != 14 {
            return Err(MigrationError::InvalidFormat(format!(
                "Migration filename must be in format YYYYMMDDHHMMSS_description, got: {}",
                filename
            )));
        }
        if !base.chars().all(|c| c.is_ascii_digit()) {
            return Err(MigrationError::InvalidFormat(format!(
                "Migration version must contain only digits, got: {}",
                base
            )));
        }
        let version = base.parse::<i64>().map_err(|_| {
            MigrationError::InvalidFormat(format!("Invalid migration filename: {}", filename))
        })?;
        Ok((version, "unnamed".to_string()))
    }
}

/// Discover all migrations in the migrations directory
pub fn discover_migrations(migrations_dir: &Path) -> Result<Vec<Migration>, MigrationError> {
    let mut migrations = Vec::new();
    let mut seen_versions = std::collections::HashSet::new();

    if !migrations_dir.exists() {
        return Ok(migrations);
    }

    let entries = fs::read_dir(migrations_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| MigrationError::InvalidFormat("Invalid file name".to_string()))?;

            // Handle .up.sql files (preferred format)
            if file_name.ends_with(".up.sql") {
                let migration = Migration::from_up_down_files(&path)?;
                if seen_versions.insert(migration.version) {
                    migrations.push(migration);
                }
            }
            // Handle single .sql files (backward compatibility, but skip if .up.sql exists)
            else if file_name.ends_with(".sql") && !file_name.ends_with(".down.sql") {
                let migration = Migration::from_single_file(&path)?;
                // Only add if we haven't seen this version as an .up.sql file
                if seen_versions.insert(migration.version) {
                    migrations.push(migration);
                }
            }
        }
    }

    // Sort by version
    migrations.sort_by_key(|m| m.version);

    Ok(migrations)
}

/// Get list of applied migrations from the database
pub async fn get_applied_migrations(pool: &PgPool) -> Result<Vec<i64>, MigrationError> {
    // Check if migrations table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = '_sqlx_migrations'
        )",
    )
    .fetch_one(pool)
    .await?;

    if !table_exists {
        return Ok(Vec::new());
    }

    let applied: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(pool)
            .await?;

    Ok(applied)
}

/// Ensure the migrations table exists
async fn ensure_migrations_table(pool: &PgPool) -> Result<(), MigrationError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL,
            execution_time BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Run a migration up
pub async fn run_migration_up(pool: &PgPool, migration: &Migration) -> Result<(), MigrationError> {
    ensure_migrations_table(pool).await?;

    let sql = migration.read_up_sql()?;
    let checksum = calculate_checksum(&sql);

    // Check if already applied
    let already_applied: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = $1)")
            .bind(migration.version)
            .fetch_one(pool)
            .await?;

    if already_applied {
        return Ok(()); // Already applied, skip
    }

    let start = SystemTime::now();

    // Execute the entire migration SQL as a batch
    // Note: Using raw SQL execution to properly handle functions with $$ delimiters
    sqlx::raw_sql(&sql).execute(pool).await?;

    let execution_time = start
        .elapsed()
        .map_err(|_| MigrationError::InvalidFormat("Time error".to_string()))?
        .as_millis() as i64;

    // Record the migration
    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(migration.version)
    .bind(&migration.name)
    .bind(true)
    .bind(&checksum)
    .bind(execution_time)
    .execute(pool)
    .await?;

    Ok(())
}

/// Run a migration down
pub async fn run_migration_down(
    pool: &PgPool,
    migration: &Migration,
) -> Result<(), MigrationError> {
    ensure_migrations_table(pool).await?;

    let down_sql = migration.read_down_sql()?;

    let down_sql = match down_sql {
        Some(sql) => sql,
        None => {
            return Err(MigrationError::InvalidFormat(format!(
                "Migration {} does not have a down migration",
                migration.name
            )));
        }
    };

    // Check if migration is applied
    let is_applied: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = $1)")
            .bind(migration.version)
            .fetch_one(pool)
            .await?;

    if !is_applied {
        return Err(MigrationError::NotFound(format!(
            "Migration {} is not applied",
            migration.name
        )));
    }

    // Execute the entire down migration SQL as a batch
    // Note: Using raw SQL execution to properly handle functions with $$ delimiters
    sqlx::raw_sql(&down_sql).execute(pool).await?;

    // Remove the migration record
    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
        .bind(migration.version)
        .execute(pool)
        .await?;

    Ok(())
}

/// Run all pending migrations up
pub async fn migrate_up(
    pool: &PgPool,
    migrations_dir: &Path,
) -> Result<Vec<Migration>, MigrationError> {
    ensure_migrations_table(pool).await?;

    let migrations = discover_migrations(migrations_dir)?;
    let applied = get_applied_migrations(pool).await?;
    let applied_set: std::collections::HashSet<i64> = applied.into_iter().collect();

    let mut applied_migrations = Vec::new();

    for migration in &migrations {
        if !applied_set.contains(&migration.version) {
            run_migration_up(pool, migration).await?;
            applied_migrations.push(migration.clone());
        }
    }

    Ok(applied_migrations)
}

/// Run migrations down (revert the last N migrations, or a specific migration)
pub async fn migrate_down(
    pool: &PgPool,
    migrations_dir: &Path,
    target_version: Option<i64>,
) -> Result<Vec<Migration>, MigrationError> {
    ensure_migrations_table(pool).await?;

    let migrations = discover_migrations(migrations_dir)?;
    let applied = get_applied_migrations(pool).await?;

    if applied.is_empty() {
        return Ok(Vec::new());
    }

    let mut reverted_migrations = Vec::new();

    if let Some(target) = target_version {
        // Revert all migrations with version >= target that have down migrations
        for migration in migrations.iter().rev() {
            if applied.contains(&migration.version) && migration.version >= target {
                if migration.down_path.is_some() {
                    run_migration_down(pool, migration).await?;
                    reverted_migrations.push(migration.clone());
                }
                // Skip migrations without down migrations, but don't error
                // This allows reverting to a target even if some migrations in between don't have down migrations
            }
        }
    } else {
        // Revert only the last applied migration that has a down migration
        // Find the last applied migration that has a down path
        for migration in migrations.iter().rev() {
            if applied.contains(&migration.version) && migration.down_path.is_some() {
                // Found the last migration with a down path, revert it
                run_migration_down(pool, migration).await?;
                reverted_migrations.push(migration.clone());
                break;
            }
        }
    }

    Ok(reverted_migrations)
}

/// Calculate a simple checksum for migration content
fn calculate_checksum(content: &str) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish().to_le_bytes().to_vec()
}
