#!/bin/bash
set -e

echo "🧪 Testing Voter UI WASM Loading Fix"
echo "====================================="

# Create election
echo "1️⃣ Creating test election..."
ELECTION_RESPONSE=$(curl -s -X POST http://localhost:8080/api/elections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "WASM Test Election",
    "description": "Testing WASM loading",
    "num_trustees": 3,
    "threshold": 2
  }')

ELECTION_ID=$(echo $ELECTION_RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])" 2>/dev/null || echo "")

if [ -z "$ELECTION_ID" ]; then
    echo "❌ Failed to create election"
    exit 1
fi

echo "✅ Election created: $ELECTION_ID"

# Create trustees
echo ""
echo "2️⃣ Creating trustees..."
for i in 1 2 3; do
    TRUSTEE_RESPONSE=$(curl -s -X POST http://localhost:8080/api/trustees \
      -H "Content-Type: application/json" \
      -d "{
        \"election_id\": \"$ELECTION_ID\",
        \"name\": \"Trustee $i\",
        \"email\": \"trustee$i@test.com\",
        \"docker_type\": \"TrusteeA\"
      }")
    echo "   Trustee $i created"
done

echo "✅ Trustees created"

# Wait for containers
echo ""
echo "3️⃣ Waiting for trustee containers..."
sleep 5

# Start keygen
echo ""
echo "4️⃣ Starting DKG keygen..."
curl -s -X POST "http://localhost:8080/api/elections/$ELECTION_ID/keygen/start" > /dev/null
echo "✅ Keygen started"

# Wait for keygen
echo ""
echo "5️⃣ Waiting for keygen to complete (30 seconds)..."
sleep 30

# Check keygen status
KEYGEN_STATUS=$(curl -s "http://localhost:8080/api/elections/$ELECTION_ID/keygen/status")
echo "✅ Keygen status:"
echo "$KEYGEN_STATUS" | python3 -m json.tool 2>/dev/null | head -20

# Create voter
echo ""
echo "6️⃣ Creating voter..."
VOTER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/voters \
  -H "Content-Type: application/json" \
  -d "{
    \"election_id\": \"$ELECTION_ID\",
    \"name\": \"Test Voter\",
    \"email\": \"voter@test.com\",
    \"tc_id\": \"12345678901\"
  }")

VOTER_ID=$(echo $VOTER_RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])" 2>/dev/null || echo "")
VOTER_UI_PORT=$(echo $VOTER_RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['ui_port'])" 2>/dev/null || echo "")

if [ -z "$VOTER_ID" ]; then
    echo "❌ Failed to create voter"
    exit 1
fi

echo "✅ Voter created: $VOTER_ID"
echo "   UI Port: $VOTER_UI_PORT"

# Wait for voter container
echo ""
echo "7️⃣ Waiting for voter container..."
sleep 5

# Test UI endpoint
echo ""
echo "8️⃣ Testing UI endpoints..."
echo ""
echo "   Testing main UI..."
curl -s "http://localhost:$VOTER_UI_PORT/" > /dev/null && echo "   ✅ Main UI accessible" || echo "   ❌ Main UI failed"

echo "   Testing /storage endpoint..."
curl -s "http://localhost:$VOTER_UI_PORT/storage" > /dev/null && echo "   ✅ Storage endpoint accessible" || echo "   ❌ Storage endpoint failed"

echo "   Testing WASM file (did_wasm.js)..."
curl -s "http://localhost:$VOTER_UI_PORT/wasm/did_wasm.js" > /dev/null && echo "   ✅ WASM JS file accessible" || echo "   ❌ WASM JS file failed"

echo "   Testing WASM file (did_wasm_bg.wasm)..."
curl -s "http://localhost:$VOTER_UI_PORT/wasm/did_wasm_bg.wasm" > /dev/null && echo "   ✅ WASM binary file accessible" || echo "   ❌ WASM binary file failed"

echo "   Testing WASM file (blindsign.js)..."
curl -s "http://localhost:$VOTER_UI_PORT/wasm/blindsign.js" > /dev/null && echo "   ✅ BlindSign JS file accessible" || echo "   ❌ BlindSign JS file failed"

echo "   Testing WASM file (blindsign.wasm)..."
curl -s "http://localhost:$VOTER_UI_PORT/wasm/blindsign.wasm" > /dev/null && echo "   ✅ BlindSign WASM file accessible" || echo "   ❌ BlindSign WASM file failed"

echo ""
echo "====================================="
echo "🎉 Test completed!"
echo ""
echo "🌐 Open voter UI in browser:"
echo "   http://localhost:$VOTER_UI_PORT/"
echo ""
echo "📋 Election ID: $ELECTION_ID"
echo "📋 Voter ID: $VOTER_ID"
echo ""
echo "📝 Check browser console for WASM loading messages"
