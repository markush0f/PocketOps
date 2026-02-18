#!/bin/bash
set -e

# Generate SSH keys if they don't exist
if [ ! -f "/root/.ssh/id_rsa" ]; then
    echo "Generating new SSH key..."
    ssh-keygen -t rsa -N "" -f "/root/.ssh/id_rsa"
    # echo "Public Key:"
    # cat /root/.ssh/id_rsa.pub
fi

# Ensure data directory exists
mkdir -p /app/data

# Symlink database if needed, or just let the app create it in current dir (/app)
# But we want it persistent. The app looks for `pocket_sentinel.db`.
# We should symlink it to /app/data/pocket_sentinel.db if not exists?
# Or we can just run the app inside /app/data?
# Let's simple create a symlink:
if [ ! -L "/app/pocket_sentinel.db" ]; then
    ln -s /app/data/pocket_sentinel.db /app/pocket_sentinel.db
fi

# Execute the main command
exec "$@"
