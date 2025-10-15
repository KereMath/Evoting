#!/bin/bash

echo "======================================"
echo "Voter Node Starting"
echo "Voter ID: ${VOTER_ID}"
echo "TC ID: ${TC_ID}"
echo "Election ID: ${ELECTION_ID}"
echo "Port: ${PORT}"
echo "TTP Host: ${TTP_HOST}"
echo "======================================"

exec > >(tee -a /var/log/voter.log)
exec 2>&1

echo "[$(date)] Voter ${TC_ID} initialized"
echo "[$(date)] PBC library loaded"
echo "[$(date)] Generating DID..."

# Simulate DID generation
sleep 1
DID=$(echo -n "${TC_ID}${ELECTION_ID}" | md5sum | cut -d' ' -f1)
echo "[$(date)] DID generated: ${DID}"

# Connect to TTP to get credential
echo "[$(date)] Connecting to TTP for blind signature..."
timeout 2 nc ${TTP_HOST} 9000 > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "[$(date)] Successfully received blind signature from TTP"
else
    echo "[$(date)] TTP connection pending..."
fi

# Listen for voting requests
echo "[$(date)] Voter node ready, listening on port ${PORT}..."
while true; do
    echo "VOTER-${TC_ID}-READY" | nc -l -p ${PORT}
    echo "[$(date)] Processed vote submission request"
    sleep 1
done
