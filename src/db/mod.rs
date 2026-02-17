use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs;
use std::path::Path;

/// Database connection pool alias.
pub type DbPool = Pool<Sqlite>;

/// Manages database initialization and connection.
pub struct Database;

impl Database {
    /// Connects to the SQLite database, creating it if it doesn't exist.
    pub async fn connect() -> Result<DbPool, sqlx::Error> {
        let db_url = "sqlite:pocket_sentinel.db";

        // Ensure the file exists for SQLite to connect initially
        if !Path::new("pocket_sentinel.db").exists() {
            fs::File::create("pocket_sentinel.db").expect("Failed to create DB file");
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // Initialize schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS servers (
                id TEXT PRIMARY KEY,
                alias TEXT UNIQUE NOT NULL,
                hostname TEXT NOT NULL,
                user TEXT NOT NULL,
                port INTEGER NOT NULL,
                password TEXT
            );
            
            CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                command TEXT NOT NULL,
                user_id INTEGER,
                output TEXT
            );

            CREATE TABLE IF NOT EXISTS server_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                cpu_load TEXT,
                memory_usage TEXT,
                disk_usage TEXT,
                FOREIGN KEY(server_id) REFERENCES servers(id)
            );
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(pool)
    }
}
