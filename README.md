# PocketSentinel 🛡️

PocketSentinel (also known as PocketOps) is a powerful Telegram bot designed to manage and monitor your servers directly from your chat. It combines secure SSH execution with the intelligence of modern AI models (OpenAI, Gemini, Ollama) to help you troubleshoot issues, ask questions about your infrastructure, and execute commands safely.

## Features ✨

- **Server Management via SSH**: Add, remove, and manage multiple servers.
- **AI-Powered Assistance**: Ask questions about your server status, logs, or potential issues.
    - **Interactive Troubleshooting**: Use `/investigate` to let the AI diagnose server problems step-by-step.
    - **Smart Command Execution**: The AI can suggest commands (`RUN: <cmd>`) which you can approve or skip via interactive buttons.
- **Multi-Provider AI Support**:
    - **OpenAI** (GPT-4o, GPT-4-turbo, etc.)
    - **Google Gemini** (Gemini Pro)
    - **Ollama** (Local models like Llama 3)
- **Secure Configuration**:
    - **Interactive API Key Setup**: Configure your API keys securely within the chat using `/config_key`. Keys are stored encrypted/encoded in the database.
    - **Admin Whitelist**: The bot only responds to a specific Telegram User ID.
- **Database Backed**: Uses SQLite for persistent storage of server configurations and chat history.

## Installation 🚀

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Docker](https://docs.docker.com/get-docker/) & Docker Compose (optional, for containerized deployment)
- A Telegram Bot Token (from [@BotFather](https://t.me/BotFather))
- Your Telegram User ID (from [@userinfobot](https://t.me/userinfobot))

### Local Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/PocketOps.git
   cd PocketOps
   ```

2. **Set up environment variables:**
   Create a `.env` file in the root directory:
   ```bash
   TELOXIDE_TOKEN=your_telegram_bot_token
   ADMIN_ID=your_telegram_user_id
   DATABASE_URL=sqlite:pocket_sentinel.db
   ```

3. **Initialize the database:**
   Ensure `sqlx-cli` is installed or let the application create the database file on first run if configured (the current setup uses SQLx compile-time checks, so you might need to create the DB file first):
   ```bash
   # If you have sqlx-cli
   sqlx database create
   sqlx migrate run
   ```

4. **Run the bot:**
   ```bash
   cargo run
   ```

### Docker Deployment

1. **Build and run with Docker Compose:**
   ```bash
   docker-compose up -d --build
   ```

   Ensure your `docker-compose.yml` or `.env` file has the correct `TELOXIDE_TOKEN` and `ADMIN_ID`.

## Usage 💡

Interact with the bot using the following commands:

### Server Management
- `/add <alias> <host> <user>` - Add a new server (e.g., `/add prod 192.168.1.10 root`).
- `/remove <alias>` - Remove a server.
- `/servers` - List all configured servers.
- `/exec <alias> <command>` - Execute a shell command on a server.
- `/status` - Check if the bot is online.

### AI & Troubleshooting
- `/ask <question>` - Ask the AI a question (context-aware if a session is active).
    - Example: `/ask Why is the server load high?`
- `/investigate <alias>` - Start an interactive troubleshooting session for a specific server.
- `/explain` - Get an explanation of the system architecture.
- `/tokens <text>` - Count the estimated tokens for a given text.

### Configuration
- `/config_key` - Interactively configure API keys for OpenAI or Gemini.
- `/provider <name>` - Switch the active AI provider (openai, gemini, ollama).
    - Example: `/provider gemini`
- `/ai_info` - Show current AI provider information.
- `/models` - List available models for the current provider.

## Security 🔒

- **User Whitelisting**: The bot explicitly checks `msg.chat.id` against the `ADMIN_ID` environment variable. Any message from other users is ignored.
- **API Keys**: API keys are stored in the local SQLite database and are never exposed in logs.
- **SSH Keys**: The bot uses your local SSH configuration (or the container's) to connect. Ensure your public key is authorized on the target servers.

## License 📄

This project is licensed under the MIT License.
