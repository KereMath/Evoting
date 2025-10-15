#!/bin/bash

echo "======================================"
echo "Trustee Node Starting"
echo "Trustee ID: ${TRUSTEE_ID}"
echo "Trustee Name: ${TRUSTEE_NAME}"
echo "Election ID: ${ELECTION_ID}"
echo "Port: ${PORT}"
echo "TTP Host: ${TTP_HOST}"
echo "======================================"

exec > >(tee -a /var/log/trustee.log)
exec 2>&1

echo "[$(date)] Trustee ${TRUSTEE_NAME} initialized"
echo "[$(date)] PBC library loaded"
echo "[$(date)] Generating key shares..."

# Simulate key generation
sleep 2
echo "[$(date)] Key share generated"

# Connect to TTP
echo "[$(date)] Connecting to TTP at ${TTP_HOST}..."
timeout 2 nc ${TTP_HOST} 9000 > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "[$(date)] Successfully connected to TTP"
    echo "[$(date)] Sent key share to TTP"
else
    echo "[$(date)] TTP connection pending..."
fi

# Keep container running and listen
echo "[$(date)] Trustee node ready, listening on port ${PORT}..."
while true; do
    echo "TRUSTEE-${TRUSTEE_NAME}-READY" | nc -l -p ${PORT}
    echo "[$(date)] Processed request from voter/TTP"
    sleep 1
done
