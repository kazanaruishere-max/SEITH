#!/bin/bash
# Run database migration
# Usage: ./scripts/db_migrate.sh

echo "Running database migrations..."
cargo run --release -- --migrate
