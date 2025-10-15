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

class ContainerUIHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed_path = urlparse(self.path)

        if parsed_path.path == '/':
            self.serve_ui()
        elif parsed_path.path == '/storage':
            self.serve_storage_data()
        elif parsed_path.path == '/health':
            self.serve_health()
        else:
            self.send_error(404, "Not Found")

    def serve_ui(self):
        """Serve the main UI HTML"""
        try:
            template_path = os.path.join(os.path.dirname(__file__), 'template.html')
            with open(template_path, 'r', encoding='utf-8') as f:
                html = f.read()

            # Replace placeholders
            html = html.replace('{{CONTAINER_TYPE}}', CONTAINER_TYPE)
            html = html.replace('{{API_PORT}}', API_PORT)
            html = html.replace('{{UI_PORT}}', str(UI_PORT))
            html = html.replace('{{CONTAINER_ID}}', CONTAINER_ID[:12])

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
