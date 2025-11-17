# SQLX Compile-Time Checking Setup

This guide explains how to enable sqlx compile-time checking for better IDE support, type safety, and catching SQL errors at compile time.

## Benefits

- **IDE Autocomplete**: Your IDE will know the exact column names and types
- **Compile-Time Safety**: SQL syntax errors and type mismatches are caught during compilation
- **Better Developer Experience**: No more runtime SQL errors!

## Setup Steps

### 1. Create `.env` file

Create a `.env` file in the `server/` directory:

```bash
DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing
```

**Important**: Make sure your database is running and migrations are applied before compiling.

### 2. Use Compile-Time Checked Macros

Replace `sqlx::query_as` with `sqlx::query_as!` (note the `!`):

**Before (runtime checking):**
```rust
let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM team WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;
```

**After (compile-time checking):**
```rust
let team = sqlx::query_as!(
    TeamDb,
    "SELECT * FROM team WHERE id = $1",
    id
)
.fetch_optional(pool)
.await?;
```

### 3. Key Differences

- **Macro syntax**: Use `query_as!` instead of `query_as::<_, Type>`
- **Type parameter**: Pass the type as the first argument: `query_as!(Type, "SQL", ...)`
- **Bindings**: Pass bind parameters as additional arguments instead of `.bind()`

### 4. Offline Mode (Required for CI/CD)

**This is already configured!** CI is set up to use offline mode via the `SQLX_OFFLINE=true` environment variable.

#### Generating Offline Metadata

When you add or modify queries, regenerate the offline metadata:

1. Make sure your database is running:
   ```bash
   docker-compose up -d
   ```

2. Install sqlx-cli (if not already installed):
   ```bash
   cargo install sqlx-cli
   ```

3. Generate offline metadata:
   ```bash
   cd server
   export DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing
   cargo sqlx prepare --database-url "$DATABASE_URL"
   ```

4. **Commit the `.sqlx/` directory to git** (it's not in `.gitignore`). This allows CI to compile without a database connection.

#### How It Works

- **Locally**: sqlx can connect to your database during compilation for validation
- **In CI**: With `SQLX_OFFLINE=true`, sqlx uses the committed `.sqlx/` metadata files instead
- **After query changes**: Regenerate and commit the `.sqlx/` directory

## Example Conversion

Here's a complete example of converting a query:

**Before:**
```rust
pub async fn get_team_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM team WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(team)
}
```

**After:**
```rust
pub async fn get_team_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as!(
        TeamDb,
        "SELECT * FROM team WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(team)
}
```

## Notes

- **Compile Time**: Compile-time checked queries require a database connection during compilation, which can slow down builds
- **Flexibility**: Runtime queries (`query_as`) are more flexible but don't provide IDE support
- **Hybrid Approach**: You can mix both approaches - use compile-time checking for critical queries and runtime for dynamic queries

## Troubleshooting

If you get errors like "error: error reading from the database":

1. Make sure your database is running: `docker-compose up -d`
2. Make sure migrations are applied
3. Check that `DATABASE_URL` in `.env` is correct
4. Try running `cargo clean` and rebuilding

## IDE Support

Once set up, your IDE (VS Code with rust-analyzer, IntelliJ Rust, etc.) will:
- Autocomplete column names in SQL queries
- Show type information for query results
- Highlight SQL syntax errors
- Warn about type mismatches

