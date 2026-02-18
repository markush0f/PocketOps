FROM rust:1.76-bookworm as builder

WORKDIR /usr/src/app
COPY . .

# Install dependencies for compilation
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Build the application
RUN cargo install --path .

FROM debian:bookworm-slim

# Install runtime dependencies
# - openssh-client: for ssh command execution
# - libssl3: for reqwest/openssl
# - ca-certificates: for https requests
# - sqlite3: for database inspection (optional but useful)
RUN apt-get update && apt-get install -y \
    openssh-client \
    libssl3 \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder
COPY --from=builder /usr/local/cargo/bin/pocket-sentinel /usr/local/bin/pocket-sentinel

# Copy scripts and templates
COPY scripts/ ./scripts/
COPY templates/ ./templates/

# Create a volume for the database and config
VOLUME ["/app/data"]

# Set environment variable for DB location to use the volume
# note: the code currently hardcodes "pocket_sentinel.db", we might need to symlink or chang workdir
# The app looks for "pocket_sentinel.db" in current dir.
# It also looks for "config/ai/..." (fallback) which we might need to handle if we want persistence there too, 
# but we moved to DB for most things.

# Entrypoint script to handle SSH key generation on startup if needed
COPY scripts/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["entrypoint.sh"]
CMD ["pocket-sentinel"]
