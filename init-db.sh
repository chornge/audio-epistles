#!/bin/sh

if [ ! -f /app/videos.db ]; then
    echo "Initializing database from template..."
    cp /app/videos.db.template /app/videos.db
fi

# Execute the command passed as argument
exec "$@"