#!/bin/bash
set -e

KEY_FILE="$HOME/.ssh/id_rsa"
PUB_KEY_FILE="$HOME/.ssh/id_rsa.pub"
AUTH_KEYS_FILE="$HOME/.ssh/authorized_keys"

echo "Checking SSH configuration..."

# 1. Generate SSH Key if it doesn't exist
if [ ! -f "$KEY_FILE" ]; then
    echo "Generating SSH key in $KEY_FILE..."
    ssh-keygen -t rsa -N "" -f "$KEY_FILE"
else
    echo "SSH key already exists."
fi

# 2. Add Public Key to Authorized Keys
if [ -f "$PUB_KEY_FILE" ]; then
    PUB_KEY=$(cat "$PUB_KEY_FILE")
    
    if ! grep -qF "$PUB_KEY" "$AUTH_KEYS_FILE" 2>/dev/null; then
        echo "Adding public key to authorized_keys..."
        mkdir -p "$HOME/.ssh"
        chmod 700 "$HOME/.ssh"
        echo "$PUB_KEY" >> "$AUTH_KEYS_FILE"
        chmod 600 "$AUTH_KEYS_FILE"
        echo "Key authorized successfully."
    else
        echo "Key is already authorized."
    fi
else
    echo "Error: Public key file not found at $PUB_KEY_FILE"
    exit 1
fi

echo "SSH setup complete. You can now connect to localhost without a password."
