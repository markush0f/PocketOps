mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env variables
    dotenv::dotenv().ok();

    // Start the communication bridge
    handlers::telegram::start_bot().await;

    Ok(())
}
