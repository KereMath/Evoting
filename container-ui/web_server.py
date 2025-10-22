#!/usr/bin/env python3
"""
Simple web server for container UI
Serves the storage & accessibility dashboard
"""

import os
import sys
import json
import urllib.request
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

# Configuration from environment variables
CONTAINER_TYPE = os.getenv('CONTAINER_TYPE', 'Unknown')
API_PORT = os.getenv('API_PORT', '8000')
UI_PORT = int(os.getenv('UI_PORT', '8001'))
CONTAINER_ID = os.getenv('CONTAINER_ID', 'unknown')

# Set storage file based on container type
if CONTAINER_TYPE == 'Trustee':
    STORAGE_FILE = '/app/storage/dkg_state.json'  # Trustees use DKG state
else:
    STORAGE_FILE = os.getenv('STORAGE_FILE', '/app/storage/data.json')

ELECTION_ID = os.getenv('ELECTION_ID', 'unknown')
BACKEND_URL = os.getenv('BACKEND_URL', 'http://host.docker.internal:8080')

def fetch_crypto_parameters():
    """Fetch crypto parameters from backend API"""
    if ELECTION_ID == 'unknown':
        return None

    try:
        url = f"{BACKEND_URL}/api/crypto/parameters/{ELECTION_ID}"
        with urllib.request.urlopen(url, timeout=5) as response:
            if response.status == 200:
                data = json.loads(response.read().decode('utf-8'))
                return data
    except Exception as e:
        print(f"⚠️  Failed to fetch crypto parameters: {e}")

    return None

def fetch_keygen_status():
    """Fetch DKG/keygen status from backend API including MVK and VKs"""
    if ELECTION_ID == 'unknown':
        return None

    try:
        url = f"{BACKEND_URL}/api/elections/{ELECTION_ID}/keygen/status"
        with urllib.request.urlopen(url, timeout=5) as response:
            if response.status == 200:
                data = json.loads(response.read().decode('utf-8'))
                return data
    except Exception as e:
        print(f"⚠️  Failed to fetch keygen status: {e}")

    return None

class ContainerUIHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed_path = urlparse(self.path)

        if parsed_path.path == '/':
            self.serve_ui()
        elif parsed_path.path == '/storage':
            self.serve_storage_data()
        elif parsed_path.path == '/health':
            self.serve_health()
        elif parsed_path.path.startswith('/wasm/'):
            self.serve_static_file(parsed_path.path)
        else:
            self.send_error(404, "Not Found")

    def do_OPTIONS(self):
        """Handle CORS preflight requests"""
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()

    def serve_ui(self):
        """Serve the main UI HTML"""
        try:
            # Use voter-specific template for voters, default template for trustees
            if CONTAINER_TYPE == 'Voter':
                template_path = os.path.join(os.path.dirname(__file__), 'template_voter.html')
            else:
                template_path = os.path.join(os.path.dirname(__file__), 'template.html')

            with open(template_path, 'r', encoding='utf-8') as f:
                html = f.read()

            # Replace placeholders
            html = html.replace('{{CONTAINER_TYPE}}', CONTAINER_TYPE)
            html = html.replace('{{API_PORT}}', API_PORT)
            html = html.replace('{{UI_PORT}}', str(UI_PORT))
            html = html.replace('{{CONTAINER_ID}}', CONTAINER_ID[:12])
            html = html.replace('{{ELECTION_ID}}', ELECTION_ID)
            html = html.replace('{{VOTER_ID}}', os.getenv('VOTER_ID', ''))
            html = html.replace('{{TC_ID}}', os.getenv('TC_ID', ''))

            self.send_response(200)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(html.encode('utf-8'))
        except Exception as e:
            self.send_error(500, f"Error serving UI: {str(e)}")

    def serve_storage_data(self):
        """Serve storage data as JSON"""
        try:
            # Try to read from storage file
            if os.path.exists(STORAGE_FILE):
                with open(STORAGE_FILE, 'r') as f:
                    data = json.load(f)
            else:
                # Return sample data if storage file doesn't exist
                data = self.get_sample_data()

            # Fetch and add crypto parameters
            crypto_params = fetch_crypto_parameters()
            if crypto_params:
                data['crypto_parameters'] = crypto_params
                data['crypto_parameters_loaded'] = True
            else:
                data['crypto_parameters_loaded'] = False

            # Fetch keygen status to get MVK and VKs
            keygen_status = fetch_keygen_status()
            if keygen_status and keygen_status.get('status') == 'completed':
                # Add MVK (available for both Trustees and Voters)
                if keygen_status.get('mvk'):
                    data['mvk'] = keygen_status['mvk']

                # For TRUSTEES: Add OTHER trustees' VKs (exclude self)
                if CONTAINER_TYPE == 'Trustee':
                    my_trustee_id = os.getenv('TRUSTEE_ID', 'unknown')
                    other_trustees_vks = []

                    if keygen_status.get('trustees_ready'):
                        for trustee in keygen_status['trustees_ready']:
                            # Skip self
                            if trustee.get('trustee_id') == my_trustee_id:
                                continue

                            # Only include completed trustees with VKs
                            if trustee.get('status') == 'completed' and trustee.get('verification_key'):
                                try:
                                    vk_data = json.loads(trustee['verification_key']) if isinstance(trustee['verification_key'], str) else trustee['verification_key']
                                    other_trustees_vks.append({
                                        'index': trustee.get('trustee_index'),
                                        'vk1': vk_data.get('vk1', ''),
                                        'vk2': vk_data.get('vk2', ''),
                                        'vk3': vk_data.get('vk3', '')
                                    })
                                except:
                                    pass

                    if other_trustees_vks:
                        data['other_trustees_vks'] = other_trustees_vks

                # For VOTERS: Add ALL trustees' VKs
                elif CONTAINER_TYPE == 'Voter':
                    all_trustees_vks = []

                    if keygen_status.get('trustees_ready'):
                        for trustee in keygen_status['trustees_ready']:
                            # Only include completed trustees with VKs
                            if trustee.get('status') == 'completed' and trustee.get('verification_key'):
                                try:
                                    vk_data = json.loads(trustee['verification_key']) if isinstance(trustee['verification_key'], str) else trustee['verification_key']
                                    all_trustees_vks.append({
                                        'index': trustee.get('trustee_index'),
                                        'vk1': vk_data.get('vk1', ''),
                                        'vk2': vk_data.get('vk2', ''),
                                        'vk3': vk_data.get('vk3', '')
                                    })
                                except:
                                    pass

                    if all_trustees_vks:
                        data['all_trustees_vks'] = all_trustees_vks

            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(data, indent=2).encode('utf-8'))
        except Exception as e:
            self.send_error(500, f"Error reading storage: {str(e)}")

    def serve_health(self):
        """Health check endpoint"""
        health = {
            'status': 'healthy',
            'container_type': CONTAINER_TYPE,
            'api_port': API_PORT,
            'ui_port': UI_PORT
        }

        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(health).encode('utf-8'))

    def serve_static_file(self, path):
        """Serve static files (WASM, JS, etc.)"""
        try:
            # Remove leading slash and get file path
            file_path = path.lstrip('/')
            full_path = os.path.join(os.path.dirname(__file__), file_path)

            if not os.path.exists(full_path):
                print(f"❌ File not found: {full_path}")
                self.send_error(404, "File not found")
                return

            # Determine content type
            content_type = 'application/octet-stream'
            if path.endswith('.js'):
                content_type = 'application/javascript; charset=utf-8'
            elif path.endswith('.wasm'):
                content_type = 'application/wasm'
            elif path.endswith('.d.ts'):
                content_type = 'text/plain; charset=utf-8'

            # Read and serve file
            with open(full_path, 'rb') as f:
                content = f.read()

            self.send_response(200)
            self.send_header('Content-Type', content_type)
            self.send_header('Access-Control-Allow-Origin', '*')
            self.send_header('Access-Control-Allow-Methods', 'GET, OPTIONS')
            self.send_header('Access-Control-Allow-Headers', 'Content-Type')
            self.send_header('Cache-Control', 'public, max-age=3600')
            self.send_header('Content-Length', len(content))
            self.end_headers()
            self.wfile.write(content)
            print(f"✅ Served file: {file_path} ({len(content)} bytes, {content_type})")
        except Exception as e:
            print(f"❌ Error serving file {path}: {str(e)}")
            self.send_error(500, f"Error serving file: {str(e)}")

    def get_sample_data(self):
        """Get sample data based on container type"""
        samples = {
            'TTP': {
                'container_id': CONTAINER_ID[:12],
                'type': 'TTP',
                'election_id': os.getenv('ELECTION_ID', 'N/A'),
                'crypto_parameters': {
                    'loaded': True,
                    'security_level': 256,
                    'prime_order_length': 64,
                    'g1_length': 256
                },
                'status': 'active',
                'uptime': 'N/A'
            },
            'Trustee': {
                'container_id': CONTAINER_ID[:12],
                'type': 'Trustee',
                'trustee_id': os.getenv('TRUSTEE_ID', 'N/A'),
                'election_id': os.getenv('ELECTION_ID', 'N/A'),
                'public_key': None,
                'private_key_stored': False,
                'signatures_count': 0,
                'status': 'pending'
            },
            'Voter': {
                'container_id': CONTAINER_ID[:12],
                'type': 'Voter',
                'voter_id': os.getenv('VOTER_ID', 'N/A'),
                'election_id': os.getenv('ELECTION_ID', 'N/A'),
                'credential': None,
                'has_voted': False,
                'vote_timestamp': None,
                'status': 'registered'
            }
        }

        return samples.get(CONTAINER_TYPE, {
            'container_id': CONTAINER_ID[:12],
            'type': CONTAINER_TYPE,
            'message': 'No specific data structure defined'
        })

    def log_message(self, format, *args):
        """Custom log format"""
        sys.stderr.write(f"[{CONTAINER_TYPE} UI] {format % args}\n")

def run_server():
    """Start the web server"""
    server_address = ('', UI_PORT)
    httpd = HTTPServer(server_address, ContainerUIHandler)

    print(f"🌐 Starting {CONTAINER_TYPE} Container UI on port {UI_PORT}")
    print(f"📊 API Port: {API_PORT}")
    print(f"🆔 Container ID: {CONTAINER_ID[:12]}")
    print(f"Access UI at: http://localhost:{UI_PORT}/")
    print("-" * 50)

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print(f"\n🛑 Shutting down {CONTAINER_TYPE} UI server")
        httpd.shutdown()

if __name__ == '__main__':
    run_server()
