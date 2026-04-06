#!/bin/bash
set -e

# Initialize database from seed if it doesn't exist
DB_PATH="${DATABASE_URL:-/app/data/arak.db}"
DB_DIR=$(dirname "$DB_PATH")

# Create data directory if it doesn't exist
mkdir -p "$DB_DIR"

# Copy seed database if target doesn't exist
if [ ! -f "$DB_PATH" ] && [ -f "/opt/seed/arak.db" ]; then
    echo "Initializing database from seed..."
    cp /opt/seed/arak.db "$DB_PATH"
    echo "Database initialized at $DB_PATH"
else
    echo "Database already exists at $DB_PATH"
fi

# Execute the main command
exec "$@"

