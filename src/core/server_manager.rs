use crate::db::DbPool;
use crate::models::ManagedServer;
use sqlx::{sqlite::SqliteRow, Row};
use uuid::Uuid;

/// Manages the collection of servers using SQLite.
#[derive(Clone)]
pub struct ServerManager {
    pool: DbPool,
}

impl ServerManager {
    /// Creates a new `ServerManager` with the given database pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Adds a new server to the database.
    pub async fn add_server(
        &self,
        alias: String,
        host: String,
        user: String,
        port: u16,
        password: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO servers (id, alias, hostname, user, port, password) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(alias)
        .bind(host)
        .bind(user)
        .bind(port)
        .bind(password)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Removes a server by its alias.
    pub async fn remove_server(&self, alias: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM servers WHERE alias = ?")
            .bind(alias)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Retrieves a server configuration by its alias.
    pub async fn get_server(&self, alias: &str) -> Result<Option<ManagedServer>, sqlx::Error> {
        let row: Option<SqliteRow> = sqlx::query("SELECT * FROM servers WHERE alias = ?")
            .bind(alias)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(ManagedServer {
                id: row.get("id"),
                hostname: row.get("hostname"),
                ip_address: row.get("hostname"), // Mapping host to IP for now
                port: row.get::<u32, _>("port") as u16,
                ssh_user: row.get("user"),
                password: row.get("password"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists all configured servers.
    pub async fn list_servers(&self) -> Result<Vec<(String, ManagedServer)>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM servers")
            .fetch_all(&self.pool)
            .await?;

        let mut servers = Vec::new();
        for row in rows {
            let server = ManagedServer {
                id: row.get("id"),
                hostname: row.get("hostname"),
                ip_address: row.get("hostname"),
                port: row.get::<u32, _>("port") as u16,
                ssh_user: row.get("user"),
                password: row.get("password"),
            };
            let alias: String = row.get("alias");
            servers.push((alias, server));
        }
        Ok(servers)
    }
}
