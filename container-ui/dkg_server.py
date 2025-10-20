#!/usr/bin/env python3
"""
DKG Server for Trustee Containers
Handles distributed key generation protocol (Pedersen DKG)
"""

import os
import sys
import json
import time
import requests
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse
import subprocess
import threading
from threading import Lock

# Configuration from environment
TRUSTEE_ID = os.getenv('TRUSTEE_ID', 'unknown')
TRUSTEE_INDEX = int(os.getenv('TRUSTEE_INDEX', '0'))
ELECTION_ID = os.getenv('ELECTION_ID', 'unknown')
THRESHOLD = int(os.getenv('THRESHOLD', '3'))
TOTAL_TRUSTEES = int(os.getenv('TOTAL_TRUSTEES', '5'))
DKG_PORT = int(os.getenv('DKG_PORT', '8000'))
PEERS = os.getenv('PEERS', '').split(',') if os.getenv('PEERS') else []
BACKEND_URL = os.getenv('BACKEND_URL', 'http://host.docker.internal:8080')

# State file
STATE_FILE = '/app/storage/dkg_state.json'

# Threading lock for state synchronization
state_lock = Lock()

# DKG State
dkg_state = {
    'session_id': None,
    'current_step': 0,
    'status': 'idle',  # idle, in_progress, completed, failed
    'my_index': TRUSTEE_INDEX,
    'threshold': THRESHOLD,
    'total_trustees': TOTAL_TRUSTEES,
    'peers': [],
    'my_polynomials': None,
    'my_commitments': None,
    'commitments_received': {},
    'shares_to_send': {},
    'shares_received': {},
    'complaints': [],
    'qualified_set': [],
    'mvk': None,
    'my_signing_key': None,
    'my_verification_keys': None
}

def load_state():
    """Load DKG state from file"""
    global dkg_state
    try:
        if os.path.exists(STATE_FILE):
            with open(STATE_FILE, 'r') as f:
                dkg_state = json.load(f)
            print(f"[DKG] State loaded from {STATE_FILE}")
    except Exception as e:
        print(f"[DKG] Error loading state: {e}")

def save_state():
    """Save DKG state to file (thread-safe)"""
    with state_lock:
        try:
            os.makedirs(os.path.dirname(STATE_FILE), exist_ok=True)
            with open(STATE_FILE, 'w') as f:
                json.dump(dkg_state, f, indent=2)
            print(f"[DKG] State saved to {STATE_FILE}")
        except Exception as e:
            print(f"[DKG] Error saving state: {e}")

def report_progress_to_backend():
    """Report DKG progress to backend"""
    if not dkg_state.get('session_id') or ELECTION_ID == 'unknown':
        return

    try:
        url = f"{BACKEND_URL}/api/elections/{ELECTION_ID}/keygen/progress"
        payload = {
            'trustee_id': TRUSTEE_ID,
            'session_id': dkg_state['session_id'],
            'current_step': dkg_state.get('current_step', 0),
            'status': dkg_state.get('status', 'idle'),
            'mvk': dkg_state.get('mvk'),
            'verification_key': dkg_state.get('my_verification_keys')
        }

        response = requests.post(url, json=payload, timeout=3)
        if response.status_code == 200:
            print(f"[DKG] Progress reported to backend (Step {payload['current_step']})")
        else:
            print(f"[DKG] Backend returned status {response.status_code}")
    except Exception as e:
        print(f"[DKG] Failed to report progress: {e}")

def call_crypto_library(function, *args):
    """Call C++ crypto library function"""
    try:
        # Call the compiled crypto binary
        cmd = ['/app/crypto/evoting_crypto', function] + list(args)
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

        if result.returncode != 0:
            print(f"[CRYPTO] Error: {result.stderr}")
            return None

        return result.stdout
    except Exception as e:
        print(f"[CRYPTO] Exception calling {function}: {e}")
        return None

def broadcast_to_peers(endpoint, data):
    """Send data to all peers (excluding self)"""
    results = []
    for peer in dkg_state['peers']:
        # Skip sending to self
        if peer['index'] == TRUSTEE_INDEX:
            print(f"[DKG] Skipping broadcast to self (peer {peer['index']})")
            continue

        try:
            url = f"http://{peer['hostname']}:{peer['port']}{endpoint}"
            response = requests.post(url, json=data, timeout=5)
            results.append({
                'peer': peer['index'],
                'success': response.status_code == 200,
                'response': response.json() if response.status_code == 200 else None
            })
            print(f"[DKG] Broadcast to peer {peer['index']}: {response.status_code}")
        except Exception as e:
            print(f"[DKG] Error broadcasting to peer {peer['index']}: {e}")
            results.append({'peer': peer['index'], 'success': False, 'error': str(e)})
    return results

def send_to_peer(peer_index, endpoint, data):
    """Send data to specific peer"""
    try:
        peer = next((p for p in dkg_state['peers'] if p['index'] == peer_index), None)
        if not peer:
            print(f"[DKG] Peer {peer_index} not found")
            return False

        url = f"http://{peer['hostname']}:{peer['port']}{endpoint}"
        response = requests.post(url, json=data, timeout=5)
        print(f"[DKG] Sent to peer {peer_index}: {response.status_code}")
        return response.status_code == 200
    except Exception as e:
        print(f"[DKG] Error sending to peer {peer_index}: {e}")
        return False

def step1_generate_polynomials():
    """Step 1: Generate polynomials and commitments (thread-safe)"""
    threshold = dkg_state.get('threshold', THRESHOLD)
    print(f"[DKG] Step 1: Generating polynomials (degree={threshold-1})")

    # Call crypto library to generate real polynomials and commitments
    try:
        result = subprocess.run(
            ['/app/crypto/dkg_cli', 'generate_polynomials', str(threshold)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Print stderr for debugging (contains dkg_cli status messages)
        if result.stderr:
            print(f"[CRYPTO] {result.stderr}")

        if result.returncode != 0:
            print(f"[CRYPTO] Error: {result.stderr}")
            raise Exception(f"Failed to generate polynomials: {result.stderr}")

        crypto_output = json.loads(result.stdout)

        with state_lock:
            dkg_state['my_polynomials'] = {
                'F': crypto_output['F_coeffs'],
                'G': crypto_output['G_coeffs']
            }

            dkg_state['my_commitments'] = crypto_output['commitments']

            # Add own commitments to received list
            dkg_state['commitments_received'][str(TRUSTEE_INDEX)] = dkg_state['my_commitments']

            dkg_state['current_step'] = 1

        save_state()
        print(f"[DKG] Generated real polynomials and commitments using PBC")

    except Exception as e:
        print(f"[DKG] Error generating polynomials: {e}")
        raise

    # Broadcast commitments to all peers
    broadcast_data = {
        'sender_index': TRUSTEE_INDEX,
        'commitments': dkg_state['my_commitments']
    }

    print(f"[DKG] Broadcasting commitments to all peers")
    broadcast_to_peers('/dkg/receive-commitment', broadcast_data)

    # Check if we can proceed to step 2
    check_step1_completion()

def check_step1_completion():
    """Check if all commitments received (thread-safe)"""
    with state_lock:
        received_count = len(dkg_state['commitments_received'])
        total_trustees = dkg_state.get('total_trustees', TOTAL_TRUSTEES)
        current_step = dkg_state.get('current_step', 0)
        print(f"[DKG] Step 1 progress: {received_count}/{total_trustees} commitments received")

        if received_count == total_trustees and current_step < 2:
            print(f"[DKG] ✅ All commitments received! Moving to Step 2")
            dkg_state['current_step'] = 2  # Mark step 2 as started
            threading.Thread(target=step2_distribute_shares).start()

def step2_distribute_shares():
    """Step 2: Calculate and distribute shares (thread-safe)"""
    print(f"[DKG] Step 2: Calculating and distributing shares")

    peers_list = None
    threshold = None
    F_coeffs = None
    G_coeffs = None

    with state_lock:
        peers_list = list(dkg_state['peers'])
        threshold = dkg_state.get('threshold', THRESHOLD)
        F_coeffs = dkg_state['my_polynomials']['F']
        G_coeffs = dkg_state['my_polynomials']['G']

    for peer in peers_list:
        peer_index = peer['index']

        # Evaluate polynomial at peer_index using crypto library
        try:
            cmd = ['/app/crypto/dkg_cli', 'evaluate_polynomial', str(threshold), str(peer_index)]
            cmd.extend(F_coeffs)
            cmd.extend(G_coeffs)

            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

            if result.returncode != 0:
                print(f"[CRYPTO] Error evaluating polynomial: {result.stderr}")
                continue

            share_data = json.loads(result.stdout)

            if peer_index == TRUSTEE_INDEX:
                # Own share - don't send to self
                with state_lock:
                    dkg_state['shares_received'][str(TRUSTEE_INDEX)] = share_data
            else:
                # Send share to peer
                share = {
                    'sender_index': TRUSTEE_INDEX,
                    'receiver_index': peer_index,
                    'F': share_data['F'],
                    'G': share_data['G']
                }
                send_to_peer(peer_index, '/dkg/receive-share', share)

        except Exception as e:
            print(f"[DKG] Error calculating share for peer {peer_index}: {e}")

    with state_lock:
        dkg_state['current_step'] = 2
    save_state()

    # Check if we can proceed to step 3
    check_step2_completion()

def check_step2_completion():
    """Check if all shares received (thread-safe)"""
    with state_lock:
        received_count = len(dkg_state['shares_received'])
        total_trustees = dkg_state.get('total_trustees', TOTAL_TRUSTEES)
        current_step = dkg_state.get('current_step', 0)
        print(f"[DKG] Step 2 progress: {received_count}/{total_trustees} shares received")

        if received_count == total_trustees and current_step < 3:
            print(f"[DKG] ✅ All shares received! Moving to Step 3")
            dkg_state['current_step'] = 3  # Mark step 3 as started
            threading.Thread(target=step3_verify_shares).start()

def step3_verify_shares():
    """Step 3: Verify received shares (thread-safe)"""
    print(f"[DKG] Step 3: Verifying shares")

    all_valid = True
    shares_to_verify = None
    commitments_received = None
    threshold = None

    with state_lock:
        shares_to_verify = dict(dkg_state['shares_received'])
        commitments_received = dict(dkg_state['commitments_received'])
        threshold = dkg_state.get('threshold', THRESHOLD)

    for sender_index, share in shares_to_verify.items():
        if sender_index == str(TRUSTEE_INDEX):
            continue  # Don't verify own share

        # Verify share using crypto library
        try:
            sender_commitments = commitments_received[sender_index]

            cmd = ['/app/crypto/dkg_cli', 'verify_share', str(threshold), str(TRUSTEE_INDEX)]
            cmd.append(share['F'])
            cmd.append(share['G'])
            cmd.extend(sender_commitments['V_x'])
            cmd.extend(sender_commitments['V_y'])
            cmd.extend(sender_commitments['V_y_prime'])

            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

            if result.returncode != 0:
                print(f"[CRYPTO] Error verifying share from {sender_index}: {result.stderr}")
                print(f"[CRYPTO] Command was: {' '.join(cmd[:10])}...")
                is_valid = False
            else:
                verify_result = json.loads(result.stdout)
                is_valid = verify_result.get('valid', False)
                print(f"[DKG] Share from {sender_index} verification result: {is_valid}")

        except Exception as e:
            print(f"[DKG] Error verifying share from {sender_index}: {e}")
            is_valid = False

        if not is_valid:
            # Broadcast complaint
            complaint = {
                'complainer_index': TRUSTEE_INDEX,
                'accused_index': int(sender_index),
                'reason': f"Share verification failed for trustee {sender_index}"
            }
            with state_lock:
                dkg_state['complaints'].append(complaint)
            broadcast_to_peers('/dkg/receive-complaint', complaint)
            all_valid = False

    with state_lock:
        dkg_state['current_step'] = 3

        # Track own verification completion
        if 'verifications_received' not in dkg_state:
            dkg_state['verifications_received'] = []
        if TRUSTEE_INDEX not in dkg_state['verifications_received']:
            dkg_state['verifications_received'].append(TRUSTEE_INDEX)

    save_state()
    report_progress_to_backend()

    # Broadcast verification complete
    broadcast_to_peers('/dkg/verification-done', {'sender_index': TRUSTEE_INDEX})

    complaints_count = 0
    with state_lock:
        complaints_count = len(dkg_state['complaints'])
    print(f"[DKG] Step 3 complete. Valid: {all_valid}, Complaints: {complaints_count}")

    # Check if all verifications are done
    check_step3_completion()

def check_step3_completion():
    """Check if all trustees completed verification (thread-safe)"""
    with state_lock:
        verifications = dkg_state.get('verifications_received', [])
        total_trustees = dkg_state.get('total_trustees', TOTAL_TRUSTEES)
        current_step = dkg_state.get('current_step', 0)

        print(f"[DKG] Step 3 progress: {len(verifications)}/{total_trustees} verifications received")

        if len(verifications) == total_trustees and current_step < 4:
            print(f"[DKG] ✅ All verifications received! Moving to Step 4")
            dkg_state['current_step'] = 4  # Mark step 4 as started
            threading.Thread(target=step4_calculate_qualified_set).start()

def step4_calculate_qualified_set():
    """Step 4: Calculate qualified set based on complaints (thread-safe)"""
    print(f"[DKG] Step 4: Calculating qualified set")

    with state_lock:
        total_trustees = dkg_state.get('total_trustees', TOTAL_TRUSTEES)
        threshold = dkg_state.get('threshold', THRESHOLD)

        # Count complaints against each trustee
        complaint_counts = {i: 0 for i in range(1, total_trustees + 1)}
        for complaint in dkg_state['complaints']:
            accused = complaint['accused_index']
            complaint_counts[accused] = complaint_counts.get(accused, 0) + 1

        # Qualified if complaints < threshold
        dkg_state['qualified_set'] = [
            i for i in range(1, total_trustees + 1)
            if complaint_counts[i] < threshold
        ]

        print(f"[DKG] Qualified set: {dkg_state['qualified_set']}")
        print(f"[DKG] Complaint counts: {complaint_counts}")

        if len(dkg_state['qualified_set']) < threshold:
            dkg_state['status'] = 'failed'
            dkg_state['error'] = 'Not enough qualified trustees'
            save_state()
            print(f"[DKG] ❌ DKG FAILED: Not enough qualified trustees")
            return

        dkg_state['current_step'] = 4

    save_state()

    # Move to step 5
    threading.Thread(target=step5_aggregate_mvk).start()

def step5_aggregate_mvk():
    """Step 5: Calculate Master Verification Key (thread-safe)"""
    print(f"[DKG] Step 5: Calculating Master Verification Key")

    qualified_set = None
    threshold = None
    commitments_received = None

    with state_lock:
        qualified_set = list(dkg_state['qualified_set'])
        threshold = dkg_state.get('threshold', THRESHOLD)
        commitments_received = dict(dkg_state['commitments_received'])

    # Aggregate MVK using crypto library
    try:
        cmd = ['/app/crypto/dkg_cli', 'aggregate_mvk', str(threshold), str(len(qualified_set))]
        cmd.extend([str(i) for i in qualified_set])

        # Add all commitments from qualified trustees
        for q_index in qualified_set:
            commitments = commitments_received[str(q_index)]
            cmd.extend(commitments['V_x'])
            cmd.extend(commitments['V_y'])
            cmd.extend(commitments['V_y_prime'])

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

        if result.returncode != 0:
            print(f"[CRYPTO] Error aggregating MVK: {result.stderr}")
            raise Exception(f"Failed to aggregate MVK: {result.stderr}")

        mvk_data = json.loads(result.stdout)

        with state_lock:
            dkg_state['mvk'] = mvk_data
            dkg_state['current_step'] = 5

        save_state()
        print(f"[DKG] MVK calculated using real PBC aggregation")

    except Exception as e:
        print(f"[DKG] Error aggregating MVK: {e}")
        raise

    # Move to step 6
    threading.Thread(target=step6_calculate_signing_key).start()

def step6_calculate_signing_key():
    """Step 6: Calculate own signing key share (thread-safe)"""
    print(f"[DKG] Step 6: Calculating signing key share")

    qualified_set = None
    threshold = None
    shares_received = None

    with state_lock:
        qualified_set = list(dkg_state['qualified_set'])
        threshold = dkg_state.get('threshold', THRESHOLD)
        shares_received = dict(dkg_state['shares_received'])

    # Compute signing key using crypto library
    try:
        cmd = ['/app/crypto/dkg_cli', 'compute_signing_key', str(threshold), str(len(qualified_set)), str(TRUSTEE_INDEX)]

        # Add all shares from qualified trustees
        for q_index in qualified_set:
            share = shares_received[str(q_index)]
            cmd.append(share['F'])
            cmd.append(share['G'])

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

        if result.returncode != 0:
            print(f"[CRYPTO] Error computing signing key: {result.stderr}")
            raise Exception(f"Failed to compute signing key: {result.stderr}")

        sgk_data = json.loads(result.stdout)

        with state_lock:
            dkg_state['my_signing_key'] = sgk_data
            dkg_state['current_step'] = 6

        save_state()
        print(f"[DKG] Signing key calculated using real PBC")

    except Exception as e:
        print(f"[DKG] Error computing signing key: {e}")
        raise

    # Move to step 7
    threading.Thread(target=step7_calculate_verification_keys).start()

def step7_calculate_verification_keys():
    """Step 7: Calculate verification keys (thread-safe)"""
    print(f"[DKG] Step 7: Calculating verification keys")

    qualified_set = None
    threshold = None
    commitments_received = None

    with state_lock:
        qualified_set = list(dkg_state['qualified_set'])
        threshold = dkg_state.get('threshold', THRESHOLD)
        commitments_received = dict(dkg_state['commitments_received'])

    # Compute verification keys using crypto library
    try:
        cmd = ['/app/crypto/dkg_cli', 'compute_verification_keys', str(threshold), str(len(qualified_set)), str(TRUSTEE_INDEX)]

        # Add all commitments from qualified trustees
        for q_index in qualified_set:
            commitments = commitments_received[str(q_index)]
            cmd.extend(commitments['V_x'])
            cmd.extend(commitments['V_y'])
            cmd.extend(commitments['V_y_prime'])

        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)

        if result.returncode != 0:
            print(f"[CRYPTO] Error computing verification keys: {result.stderr}")
            raise Exception(f"Failed to compute verification keys: {result.stderr}")

        vk_data = json.loads(result.stdout)

        with state_lock:
            dkg_state['my_verification_keys'] = vk_data
            dkg_state['current_step'] = 7
            dkg_state['status'] = 'completed'

        save_state()
        report_progress_to_backend()

        print(f"[DKG] ✅ DKG COMPLETED!")
        print(f"[DKG] Verification keys calculated using real PBC")

        with state_lock:
            mvk_preview = str(dkg_state['mvk'])[:50] + "..."
            vk_preview = str(vk_data)[:50] + "..."
        print(f"[DKG] MVK (preview): {mvk_preview}")
        print(f"[DKG] VK (preview): {vk_preview}")

    except Exception as e:
        print(f"[DKG] Error computing verification keys: {e}")
        raise

class DKGHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed_path = urlparse(self.path)

        if parsed_path.path == '/dkg/status':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()

            status = {
                'trustee_id': TRUSTEE_ID,
                'trustee_index': TRUSTEE_INDEX,
                'current_step': dkg_state['current_step'],
                'status': dkg_state['status'],
                'qualified_set': dkg_state['qualified_set'],
                'mvk': dkg_state['mvk'],
                'verification_keys': dkg_state['my_verification_keys']
            }
            self.wfile.write(json.dumps(status, indent=2).encode('utf-8'))

        elif parsed_path.path == '/health':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'healthy'}).encode('utf-8'))

        else:
            self.send_error(404, "Not Found")

    def do_POST(self):
        parsed_path = urlparse(self.path)
        content_length = int(self.headers['Content-Length'])
        body = self.rfile.read(content_length)
        data = json.loads(body.decode('utf-8'))

        if parsed_path.path == '/dkg/start':
            # Start DKG process (thread-safe)
            global TRUSTEE_INDEX
            print(f"[DKG] Starting DKG session")

            with state_lock:
                dkg_state['session_id'] = data.get('session_id')
                dkg_state['peers'] = data.get('peers', [])
                dkg_state['threshold'] = data.get('threshold', THRESHOLD)
                dkg_state['total_trustees'] = data.get('total_trustees', TOTAL_TRUSTEES)
                dkg_state['status'] = 'in_progress'
                dkg_state['current_step'] = 0

                # Update my_index from backend (overrides env variable)
                my_index = data.get('my_index')
                if my_index is not None:
                    TRUSTEE_INDEX = my_index
                    dkg_state['my_index'] = TRUSTEE_INDEX
                    print(f"[DKG] My index set to: {TRUSTEE_INDEX}")

                # Save crypto parameters from backend to file for dkg_cli
                crypto_params = data.get('crypto_params')
                if crypto_params:
                    try:
                        os.makedirs('/app/storage', exist_ok=True)
                        # The pairing_params already have literal \n in the string from database
                        # These were correctly stored, just need to save them
                        with open('/app/storage/crypto_params.json', 'w') as f:
                            json.dump(crypto_params, f)
                        print(f"[DKG] ✅ Crypto parameters saved from backend")
                    except Exception as e:
                        print(f"[DKG] ⚠️  Failed to save crypto parameters: {e}")
                else:
                    print(f"[DKG] ⚠️  No crypto parameters received from backend!")

                print(f"[DKG] Session parameters: threshold={dkg_state['threshold']}, total={dkg_state['total_trustees']}, peers={len(dkg_state['peers'])}, my_index={TRUSTEE_INDEX}")

            save_state()

            # Start Step 1 in background thread
            threading.Thread(target=step1_generate_polynomials).start()

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'started'}).encode('utf-8'))

        elif parsed_path.path == '/dkg/receive-commitment':
            # Receive commitment from peer (thread-safe)
            sender_index = data['sender_index']
            commitments = data['commitments']

            print(f"[DKG] Received commitment from trustee {sender_index}")

            with state_lock:
                dkg_state['commitments_received'][str(sender_index)] = commitments
            save_state()

            # Check if all commitments received
            check_step1_completion()

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'received'}).encode('utf-8'))

        elif parsed_path.path == '/dkg/receive-share':
            # Receive share from peer (thread-safe)
            sender_index = data['sender_index']
            share = {'F': data['F'], 'G': data['G']}

            print(f"[DKG] Received share from trustee {sender_index}")

            with state_lock:
                dkg_state['shares_received'][str(sender_index)] = share
            save_state()

            # Check if all shares received
            check_step2_completion()

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'received'}).encode('utf-8'))

        elif parsed_path.path == '/dkg/receive-complaint':
            # Receive complaint (thread-safe)
            complaint = data
            print(f"[DKG] Received complaint: Trustee {complaint['complainer_index']} complains about {complaint['accused_index']}")

            with state_lock:
                dkg_state['complaints'].append(complaint)
            save_state()

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'received'}).encode('utf-8'))

        elif parsed_path.path == '/dkg/verification-done':
            # Peer completed verification (thread-safe)
            sender_index = data['sender_index']
            print(f"[DKG] Trustee {sender_index} completed verification")

            # Track who completed verification
            with state_lock:
                if 'verifications_received' not in dkg_state:
                    dkg_state['verifications_received'] = []

                if sender_index not in dkg_state['verifications_received']:
                    dkg_state['verifications_received'].append(sender_index)
            save_state()

            # Check if all trustees completed verification
            check_step3_completion()

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'received'}).encode('utf-8'))

        else:
            self.send_error(404, "Not Found")

    def log_message(self, format, *args):
        """Custom log format"""
        sys.stderr.write(f"[DKG Server] {format % args}\n")

def run_server():
    """Start the DKG server"""
    load_state()

    server_address = ('', DKG_PORT)
    httpd = HTTPServer(server_address, DKGHandler)

    print(f"🔐 Starting DKG Server")
    print(f"   Trustee ID: {TRUSTEE_ID}")
    print(f"   Trustee Index: {TRUSTEE_INDEX}")
    print(f"   Election ID: {ELECTION_ID}")
    print(f"   Threshold: {THRESHOLD}/{TOTAL_TRUSTEES}")
    print(f"   Port: {DKG_PORT}")
    print(f"   Peers: {len(PEERS)}")
    print("-" * 50)

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print(f"\n🛑 Shutting down DKG server")
        httpd.shutdown()

if __name__ == '__main__':
    run_server()
