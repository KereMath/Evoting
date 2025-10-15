#!/bin/bash

echo "======================================"
echo "TTP (Trusted Third Party) Node Starting"
echo "Election ID: ${ELECTION_ID}"
echo "Port: 9000"
echo "======================================"

# Log to stdout for Docker logs
exec > >(tee -a /var/log/ttp.log)
exec 2>&1

echo "[$(date)] TTP node initialized"
echo "[$(date)] PBC library version: $(ldconfig -p | grep libpbc)"
echo "[$(date)] Waiting for connections..."

# Simple TCP server that logs all connections
while true; do
    echo "[$(date)] TTP listening on port 9000..."
    echo "TTP-READY" | nc -l -p 9000
    echo "[$(date)] Connection received and processed"
    sleep 1
done
