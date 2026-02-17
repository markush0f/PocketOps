mod ai;
mod core;
mod db;
mod executor;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env variables
    dotenv::dotenv().ok();

    // Auto-configure SSH for local access
    println!("Configuring local SSH access...");
    let status = std::process::Command::new("bash")
        .arg("scripts/setup_ssh.sh")
        .status();

    match status {
        Ok(s) if s.success() => println!("SSH configuration completed successfully."),
        _ => eprintln!(
            "Warning: Failed to auto-configure SSH. You might need to set it up manually."
        ),
    }

    // Initialize Database
    let pool = db::Database::connect().await?;
    println!("Database connected successfully.");

    // Initialize default servers
    let manager = core::server_manager::ServerManager::new(pool.clone());
    if let Err(e) = manager.initialize_local_server().await {
        eprintln!("Failed to initialize local server: {}", e);
    }

    // Initialize Session Manager
    let session_manager = core::session::SessionManager::new();

    // Start the communication bridge
    handlers::telegram::start_bot(pool, session_manager).await;

    Ok(())
}
