#!/bin/sh
set -eu

mkdir -p /app/data

# Ensure the SQLite file exists and is writable before app startup.
sqlite3 /app/data/data.db "PRAGMA journal_mode=WAL;" >/dev/null

exec /usr/local/bin/blog001
