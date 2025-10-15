# 🎨 E-Voting Admin Panel - Complete Guide

## 🚀 What's Been Built

### Complete Admin Frontend + Backend Orchestration System

---

## 📦 **Components**

### **1. Backend API (Rust + Axum)**

#### Authentication
- **POST /api/auth/login**
  - Hardcoded credentials: `admin` / `admin`
  - Returns JWT-like token

#### Elections Management
- **GET /api/elections** - List all elections
- **POST /api/elections** - Create new election
- **GET /api/elections/:id** - Get election details
- **DELETE /api/elections/:id** - Delete election

#### Voters Management
- **GET /api/voters?election_id=xxx** - List voters (optionally filtered)
- **POST /api/voters** - Add voter with TC ID
- **DELETE /api/voters/:id** - Remove voter

#### Trustees Management
- **GET /api/trustees** - List all trustees
- **POST /api/trustees** - Add trustee to election

#### System Monitoring
- **GET /api/events?limit=100** - Get system logs/events
- Real-time event logging for all operations

---

### **2. Frontend (React + Vite)**

#### Pages

**Login Page** (`/login`)
- Simple authentication
- Credentials: admin / admin
- Redirects to dashboard on success

**Dashboard** (`/dashboard`)
- Overview statistics
- Total elections, active elections
- Total voters, total trustees
- Recent system events (auto-refresh every 5s)

**Elections Page** (`/elections`)
- List all elections
- Create new election with:
  - Name
  - Description
  - Threshold (t)
  - Total Trustees (n)
- Delete elections
- Navigate to election details

**Election Detail Page** (`/elections/:id`)
- Election information
- Statistics (voters, trustees, votes)
- **Trustee Management**:
  - Add trustees (up to total_trustees limit)
  - View trustee list
- **Voter Management**:
  - Add voters with TC Kimlik No (11 digits)
  - View all registered voters
  - Remove voters
  - Track voting status

**System Logs** (`/logs`)
- Real-time event monitoring (auto-refresh every 2s)
- Event filtering and statistics
- Data flow visualization
- Color-coded event types

---

## 🎯 **Key Features**

### ✅ Election Management
- Create multiple elections
- Each election has unique ID
- Configure threshold (t) and total trustees (n)
- Delete elections (cascades to voters/trustees)

### ✅ Voter Management
- Register voters with TC Kimlik Numarası
- Each voter linked to specific election
- Track voter status (registered, voted, etc.)
- Remove voters if needed

### ✅ Trustee Management
- Add election authorities
- Enforce trustee limit (can't exceed total_trustees)
- Track trustee status

### ✅ Logging & Monitoring
- Every action logged to system_events table
- Real-time event stream
- Data flow tracking
- Event categorization:
  - `election_created`
  - `voter_registered`
  - `voter_deleted`
  - `trustee_added`
  - etc.

### ✅ User Experience
- Modern, responsive UI
- Modal dialogs for forms
- Real-time updates
- Status badges
- Auto-refresh on dashboard and logs

---

## 🐳 **Docker Setup**

### Services

```yaml
1. postgres          (Port 5432)
2. main-server       (Port 8080) - Rust Backend
3. frontend          (Port 3000) - React App
```

### Starting Everything

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f main-server
docker-compose logs -f frontend
```

### Access Points

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8080/api
- **Health Check**: http://localhost:8080/health

---

## 🔐 **Authentication**

### Hardcoded Credentials
```
Username: admin
Password: admin
Token: admin-token-12345 (static)
```

Token stored in localStorage, used for API requests.

---

## 📊 **Database Schema**

### Tables Created

```sql
elections (
  id, name, description,
  threshold, total_trustees,
  status, created_at, started_at, ended_at
)

voters (
  id, election_id, voter_id (TC ID),
  did, status, created_at, voted_at
)

trustees (
  id, election_id, name,
  public_key, status, created_at
)

system_events (
  id, event_type, entity_type,
  entity_id, data (JSONB), created_at
)
```

---

## 🎨 **UI Components**

### Styles
- Clean, modern design
- Color scheme:
  - Primary: #667eea (purple)
  - Success: #27ae60 (green)
  - Danger: #e74c3c (red)
  - Warning: #f39c12 (orange)

### Status Badges
- `setup` - Blue
- `active` - Green
- `completed` - Gray
- `registered` - Orange
- `voted` - Green

---

## 🔄 **Data Flow**

### Creating an Election
1. Admin fills form (name, description, t, n)
2. POST `/api/elections`
3. Backend creates election in DB
4. Logs `election_created` event
5. Returns election with UUID
6. Frontend refreshes list

### Adding a Voter
1. Admin selects election
2. Enters TC Kimlik No (11 digits)
3. POST `/api/voters`
4. Backend validates and creates voter
5. Logs `voter_registered` event with data
6. Frontend shows in voters table

### Monitoring
1. Events endpoint polls every 2s
2. New events appear in real-time
3. Color-coded by type
4. Shows data flow (JSONB data field)

---

## 🧪 **Testing the System**

### 1. Login
```
Navigate to http://localhost:3000
Enter: admin / admin
Click Login
```

### 2. Create Election
```
Go to Elections page
Click "+ Create Election"
Fill form:
  Name: "Test Election 2024"
  Description: "My first election"
  Threshold: 3
  Total Trustees: 5
Click "Create Election"
```

### 3. Add Voters
```
Click "Manage" on election
Click "+ Add Voter"
Enter TC ID: 12345678901 (11 digits)
Click "Add Voter"
Repeat for more voters
```

### 4. Add Trustees
```
In same election detail page
Click "+ Add Trustee"
Enter name: "Trustee 1"
Click "Add Trustee"
Repeat up to total_trustees limit
```

### 5. Monitor Logs
```
Go to System Logs page
See all events in real-time
Watch auto-refresh
Check event data
```

---

## 📱 **Screenshots Flow**

```
[Login] → [Dashboard] → [Elections]
             ↓
        [System Logs]

[Elections] → [Election Detail]
                   ↓
              [Add Voters]
                   ↓
              [Add Trustees]
```

---

## 🚀 **Next Steps**

This admin panel is now ready to:

1. **Manage Elections** - Full CRUD
2. **Register Voters** - With TC ID tracking
3. **Add Trustees** - Election authorities
4. **Monitor System** - Real-time logs and events

### Ready for Integration:
- Trustee Docker containers
- Voter Docker containers
- Crypto protocol implementation
- Vote casting and verification

---

## 🎯 **API Usage Examples**

### Create Election
```bash
curl -X POST http://localhost:8080/api/elections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Election 2024",
    "description": "Test",
    "threshold": 3,
    "total_trustees": 5
  }'
```

### Add Voter
```bash
curl -X POST http://localhost:8080/api/voters \
  -H "Content-Type: application/json" \
  -d '{
    "election_id": "uuid-here",
    "tc_id": "12345678901"
  }'
```

### Get Events
```bash
curl http://localhost:8080/api/events?limit=50
```

---

**System Status:** ✅ FULLY OPERATIONAL
**Next Phase:** Trustee & Voter Docker Containers
