# 🚀 E-Voting System - Quick Start Guide

## Prerequisites
- Docker Desktop installed and running
- PowerShell or Command Prompt

## Start the System

```bash
# Start all containers
docker-compose up -d

# Check container status
docker-compose ps

# View logs
docker-compose logs -f main-server
```

## Verify System is Running

### 1. Check Health
```bash
curl http://localhost:8080/health
```

Expected response:
```json
{"status":"ok","version":"0.1.0","database":"connected"}
```

### 2. List Elections
```bash
curl http://localhost:8080/api/elections
```

### 3. List Trustees
```bash
curl http://localhost:8080/api/trustees
```

### 4. List Voters
```bash
curl http://localhost:8080/api/voters
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | System health check |
| `/api/elections` | GET | List all elections |
| `/api/elections` | POST | Create new election |
| `/api/trustees` | GET | List all trustees |
| `/api/trustees` | POST | Register trustee |
| `/api/voters` | GET | List all voters |

## Create New Election

```bash
curl -X POST http://localhost:8080/api/elections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Election 2024",
    "description": "Test election",
    "threshold": 3,
    "total_trustees": 5
  }'
```

## PowerShell Examples

```powershell
# Health check
Invoke-WebRequest -Uri http://localhost:8080/health

# Get elections
Invoke-RestMethod -Uri http://localhost:8080/api/elections

# Create election
$body = @{
    name = "My Election 2024"
    description = "Test election"
    threshold = 3
    total_trustees = 5
} | ConvertTo-Json

Invoke-RestMethod -Uri http://localhost:8080/api/elections `
  -Method Post `
  -Body $body `
  -ContentType "application/json"
```

## Stop the System

```bash
# Stop containers
docker-compose down

# Stop and remove all data (CAUTION!)
docker-compose down -v
```

## Troubleshooting

### Container won't start
```bash
# Check logs
docker-compose logs main-server
docker-compose logs postgres

# Restart containers
docker-compose restart
```

### Database connection issues
```bash
# Check if postgres is healthy
docker-compose ps

# Connect to database
docker-compose exec postgres psql -U evoting -d evoting_db
```

### Reset everything
```bash
# Stop and remove everything
docker-compose down -v

# Rebuild and start
docker-compose build --no-cache
docker-compose up -d
```

## Access Points

- **Main API:** http://localhost:8080/api
- **Health Check:** http://localhost:8080/health
- **PostgreSQL:** localhost:5432
  - Database: `evoting_db`
  - User: `evoting`
  - Password: `evoting`

## Project Structure

```
E-votingAPP/
├── backend/              # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── models/
│   │   ├── handlers/
│   │   └── db/
│   └── Cargo.toml
├── crypto/               # C++ cryptography implementation
├── db/
│   └── init.sql         # Database schema
├── docker-compose.yml    # Docker orchestration
├── Dockerfile.server     # Main server image
└── README.md
```

## Next Steps

1. **Add Trustees:** Implement trustee registration and key generation
2. **Add Voters:** Create voter containers with DID generation
3. **Visualization:** Build real-time monitoring dashboard
4. **Integration:** Connect crypto library with backend API

## Support

For issues or questions, check:
- [TEST_RESULTS.md](TEST_RESULTS.md) - System test results
- [README.md](README.md) - Full documentation
- Logs: `docker-compose logs -f`
