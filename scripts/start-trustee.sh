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

# Start DKG Server in background
echo "🔐 Starting DKG Server on port ${DKG_PORT:-8000}..."
cd /app/ui
python3 dkg_server.py &
DKG_PID=$!

# Start Web UI in background
echo "🌐 Starting Web UI on port ${UI_PORT}..."
python3 web_server.py &
UI_PID=$!

# Wait a moment for servers to start
sleep 2

echo "✅ Trustee Container started successfully!"
echo "   - Web UI: http://localhost:${UI_PORT}/"
echo "   - DKG API: http://localhost:${DKG_PORT:-8000}/"
echo "   - Trustee ID: ${TRUSTEE_ID}"
echo "   - Trustee Index: ${TRUSTEE_INDEX}"
echo ""
echo "📊 Container ready for DKG and voting operations..."

# Keep container running (wait for both processes)
wait $UI_PID $DKG_PID
