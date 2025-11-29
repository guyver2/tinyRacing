use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Invalid configuration: {0}")]
    Configuration(String),
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    /// Run database migrations
    /// Note: The path is relative to the crate root (where Cargo.toml is located)
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Initialize database connection from environment variable
///
/// This function attempts to connect to the database using the `DATABASE_URL` environment variable.
/// If the connection is successful, it runs migrations and returns the pool.
/// If the connection fails or DATABASE_URL is not set, it returns None and logs a warning.
///
/// Returns `Some(PgPool)` if connection is successful, `None` otherwise.
pub async fn init_from_env() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing".to_string()
    });
    match Database::new(&database_url).await {
        Ok(db) => {
            if let Err(e) = db.migrate().await {
                eprintln!("Warning: Database migration failed: {}", e);
            }
            Some(db.pool().clone())
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to connect to database: {}. API database endpoints will not work.",
                e
            );
            None
        }
    }
}
