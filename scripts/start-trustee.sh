#!/bin/bash

echo "🚀 Starting Trustee Container..."
echo "Container ID: $(hostname)"
echo "API Port: ${API_PORT}"
echo "UI Port: ${UI_PORT}"
echo "Trustee ID: ${TRUSTEE_ID:-Not set}"
echo "Election ID: ${ELECTION_ID:-Not set}"

# Export container ID for UI
export CONTAINER_ID=$(hostname)

# Create initial storage file if it doesn't exist
if [ ! -f /app/storage/data.json ]; then
    echo "📝 Creating initial storage file..."
    cat > /app/storage/data.json <<EOF
{
    "container_id": "$(hostname)",
    "type": "Trustee",
    "trustee_id": "${TRUSTEE_ID:-unknown}",
    "election_id": "${ELECTION_ID:-unknown}",
    "public_key": null,
    "private_key_stored": false,
    "signatures_count": 0,
    "status": "pending",
    "started_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
fi

# Start Web UI in background
echo "🌐 Starting Web UI on port ${UI_PORT}..."
cd /app/ui
python3 web_server.py &
UI_PID=$!

# Wait a moment for UI to start
sleep 2

echo "✅ Trustee Container started successfully!"
echo "   - Web UI: http://localhost:${UI_PORT}/"
echo "   - API: http://localhost:${API_PORT}/"
echo ""
echo "📊 Container ready and waiting for requests..."

# Keep container running
wait $UI_PID
