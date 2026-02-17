mod ai;
mod core;
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

    // Start the communication bridge
    handlers::telegram::start_bot().await;

    Ok(())
}
