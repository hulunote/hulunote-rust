# Hulunote - Rust Backend

A high-performance Rust backend implementation for the Hulunote outline note-taking application, replacing the original Clojure backend.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Running the Server](#running-the-server)
- [API Reference](#api-reference)
- [Registration Code System](#registration-code-system)
- [Frontend Setup](#frontend-setup)
- [Deployment](#deployment)
- [Troubleshooting](#troubleshooting)

## Overview

Hulunote is an outline-based note-taking application that organizes your thoughts hierarchically. This Rust backend provides:

- User authentication with JWT tokens
- Note database (notebook) management
- Note CRUD operations
- Hierarchical outline navigation nodes
- Registration code-based user management with expiration

## Features

- **High Performance**: Built with Rust for minimal memory footprint and maximum throughput
- **Zero GC Pauses**: No garbage collection delays
- **Fast Startup**: Quick cold start times
- **API Compatible**: Drop-in replacement for the original Clojure backend
- **Registration System**: Code-based registration with configurable account expiration

## Tech Stack

| Component | Technology |
|-----------|------------|
| Web Framework | Axum 0.7 |
| Database | PostgreSQL + SQLx |
| Authentication | JWT (jsonwebtoken) |
| Password Hashing | bcrypt |
| Async Runtime | Tokio |
| Serialization | Serde |

## Project Structure

```
hulunote-rust/
├── Cargo.toml                    # Rust dependencies
├── Cargo.lock                    # Dependency lock file
├── .env                          # Environment configuration (create from template)
├── env-production-template.txt   # Production environment template
├── init.sql                      # Database initialization script
├── run.sh                        # Development run script
│
├── src/
│   ├── main.rs                   # Application entry point
│   ├── config.rs                 # Configuration management
│   ├── db.rs                     # Database connection pool
│   ├── error.rs                  # Error handling
│   ├── middleware.rs             # JWT authentication middleware
│   ├── models.rs                 # Data models and structures
│   ├── routes.rs                 # Route configuration
│   └── handlers/                 # API request handlers
│       ├── mod.rs
│       ├── auth.rs               # Login/registration handlers
│       ├── database.rs           # Note database operations
│       ├── note.rs               # Note CRUD operations
│       └── nav.rs                # Outline node operations
│
├── migrations/                   # Database migrations
│   └── 001_add_registration_codes.sql
│
├── scripts/
│   ├── deploy.sh                 # Deployment script
│   ├── generate_registration_code.sh  # Registration code generator
│   └── monitor.sh                # Monitoring script
│
├── resources/
│   └── public/                   # Static files (frontend assets)
│
└── nginx-deploy/                 # Nginx configuration files
```

## Prerequisites

Before you begin, ensure you have installed:

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **PostgreSQL** (12 or later) - [Install PostgreSQL](https://www.postgresql.org/download/)
- **Git** - [Install Git](https://git-scm.com/)

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/your-repo/hulunote-rust.git
cd hulunote-rust
```

### 2. Create the Database

```bash
# Create the database
createdb -U postgres hulunote_open

# Initialize the schema
psql -U postgres -d hulunote_open -f init.sql

# Run migrations (if any)
psql -U postgres -d hulunote_open -f migrations/001_add_registration_codes.sql
```

### 3. Configure Environment

```bash
# Copy the environment template
cp env-production-template.txt .env

# Edit the configuration
vim .env
```

### 4. Build the Project

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Configuration

Create a `.env` file in the project root with the following variables:

```env
# Database connection (required)
DATABASE_URL=postgres://postgres:your_password@localhost:5432/hulunote_open

# JWT configuration
JWT_SECRET=your-super-secret-key-change-this-in-production
JWT_EXPIRY_HOURS=720    # Token validity in hours (default: 30 days)

# Server configuration
PORT=6689               # Server port

# Logging level
RUST_LOG=hulunote_server=info,tower_http=info
```

### Environment Variables Reference

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | - | Yes |
| `JWT_SECRET` | Secret key for JWT signing | `hulunote-secret-key` | Yes (in production) |
| `JWT_EXPIRY_HOURS` | JWT token expiration in hours | `720` | No |
| `PORT` | Server listening port | `6689` | No |
| `RUST_LOG` | Logging configuration | `hulunote_server=debug` | No |

## Running the Server

### Development Mode

```bash
# Using the run script (recommended)
./run.sh

# Or directly with cargo
cargo run
```

### Production Mode

```bash
# Build release binary
cargo build --release

# Run the server
./target/release/hulunote-server
```

The server will start at `http://localhost:6689` by default.

## API Reference

All API endpoints return JSON responses with kebab-case field names for compatibility with the ClojureScript frontend.

### Authentication Endpoints (No login required)

#### Login
```http
POST /login/web-login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

#### Register
```http
POST /login/web-signup
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "username": "optional_username",
  "registration_code": "FA8E-AF6E-4578-9347"
}
```

#### Send Verification (Legacy)
```http
POST /login/send-ack-msg
Content-Type: application/json

{
  "email": "user@example.com"
}
```

### Note Database Endpoints (Login required)

All authenticated endpoints require the JWT token in the Authorization header:
```
Authorization: Bearer <jwt_token>
```

#### Create Note Database
```http
POST /hulunote/new-database
Content-Type: application/json

{
  "name": "My Notebook",
  "description": "My personal notes"
}
```

#### Get Database List
```http
POST /hulunote/get-database-list
Content-Type: application/json
```

#### Update Database
```http
POST /hulunote/update-database
Content-Type: application/json

{
  "database_id": "uuid",
  "name": "Updated Name"
}
```

### Note Endpoints (Login required)

#### Create Note
```http
POST /hulunote/new-note
Content-Type: application/json

{
  "database_id": "uuid",
  "title": "My Note"
}
```

#### Get Notes (Paginated)
```http
POST /hulunote/get-note-list
Content-Type: application/json

{
  "database_id": "uuid",
  "page": 1,
  "page_size": 20
}
```

#### Get All Notes
```http
POST /hulunote/get-all-note-list
Content-Type: application/json

{
  "database_id": "uuid"
}
```

#### Update Note
```http
POST /hulunote/update-hulunote-note
Content-Type: application/json

{
  "note_id": "uuid",
  "title": "Updated Title",
  "content": "Updated content"
}
```

### Outline Navigation Endpoints (Login required)

#### Create/Update Navigation Node
```http
POST /hulunote/create-or-update-nav
Content-Type: application/json

{
  "note_id": "uuid",
  "nav_id": "uuid",
  "content": "Node content",
  "parent_id": "parent_uuid_or_null"
}
```

#### Get Note Navigation Nodes
```http
POST /hulunote/get-note-navs
Content-Type: application/json

{
  "note_id": "uuid"
}
```

#### Get All Nodes (Paginated)
```http
POST /hulunote/get-all-nav-by-page
Content-Type: application/json

{
  "database_id": "uuid",
  "page": 1,
  "page_size": 100
}
```

#### Get All Nodes
```http
POST /hulunote/get-all-navs
Content-Type: application/json

{
  "database_id": "uuid"
}
```

## Registration Code System

Instead of email verification, Hulunote uses registration codes that control account expiration.

### Generating Registration Codes

```bash
# Generate a 6-month code
./scripts/generate_registration_code.sh 6months

# Generate a 1-year code
./scripts/generate_registration_code.sh 1year

# Generate a 2-year code
./scripts/generate_registration_code.sh 2years

# Generate a custom validity code
./scripts/generate_registration_code.sh custom
# Then enter the number of days when prompted
```

### Code Format

Registration codes follow the format: `XXXX-XXXX-XXXX-XXXX`
- 16 hexadecimal characters (uppercase)
- Separated by hyphens
- Example: `FA8E-AF6E-4578-9347`

### Code Properties

- **One-time use**: Each code can only be used once
- **Expiration control**: Determines account validity period
- **Usage tracking**: Records which user used the code and when

### Managing Codes via SQL

```sql
-- View all registration codes
SELECT code, validity_days, is_used, used_by_account_id, used_at, created_at
FROM registration_codes
ORDER BY created_at DESC;

-- View unused codes
SELECT code, validity_days, created_at
FROM registration_codes
WHERE is_used = false;

-- View expired accounts
SELECT id, username, mail, expires_at
FROM accounts
WHERE expires_at IS NOT NULL AND expires_at < NOW();

-- Extend an account's expiration
UPDATE accounts
SET expires_at = NOW() + INTERVAL '365 days'
WHERE id = <user_id>;
```

## Frontend Setup

The frontend is built with ClojureScript and needs to be compiled separately.

### Building the Frontend

```bash
# Navigate to the original Hulunote project
cd ../hulunote

# Build the frontend
npx shadow-cljs release hulunote

# Copy to Rust project
cp -r resources/public/hulunote ../hulunote-rust/resources/public/
```

The `run.sh` script will automatically attempt to copy frontend files if they don't exist.

## Deployment

### Using the Deploy Script

```bash
./scripts/deploy.sh
```

### Manual Deployment

1. Build the release binary:
```bash
cargo build --release
```

2. Copy files to server:
```bash
scp target/release/hulunote-server user@server:/opt/hulunote/
scp .env user@server:/opt/hulunote/
scp -r resources user@server:/opt/hulunote/
```

3. Set up systemd service (optional):
```ini
[Unit]
Description=Hulunote Server
After=network.target postgresql.service

[Service]
Type=simple
User=hulunote
WorkingDirectory=/opt/hulunote
ExecStart=/opt/hulunote/hulunote-server
Restart=always
Environment=RUST_LOG=hulunote_server=info

[Install]
WantedBy=multi-user.target
```

### Nginx Configuration

Sample Nginx reverse proxy configuration is available in the `nginx-deploy/` directory.

## Troubleshooting

### Database Connection Failed

**Error**: `error connecting to database`

**Solutions**:
1. Verify PostgreSQL is running: `pg_isready`
2. Check `DATABASE_URL` in `.env`
3. Ensure the database exists: `psql -l | grep hulunote_open`

### Invalid Registration Code

**Error**: `Invalid registration code`

**Solutions**:
1. Verify the code format (case-sensitive)
2. Check if the code exists in the database
3. Confirm the code hasn't been used

### Account Expired

**Error**: `Account has expired`

**Solutions**:
1. Generate a new registration code
2. Or extend the account manually:
```sql
UPDATE accounts SET expires_at = NOW() + INTERVAL '365 days' WHERE id = <user_id>;
```

### JWT Token Invalid

**Error**: `Invalid token` or `Token expired`

**Solutions**:
1. Re-login to get a new token
2. Check `JWT_SECRET` matches between deployments
3. Verify `JWT_EXPIRY_HOURS` setting

### Frontend Not Loading

**Error**: 404 on static files

**Solutions**:
1. Run `./run.sh` to auto-copy frontend files
2. Manually copy: `cp -r ../hulunote/resources/public/hulunote resources/public/`
3. Verify files exist in `resources/public/hulunote/`

## Compatibility

This Rust backend maintains full compatibility with:

- ✅ Same PostgreSQL database schema
- ✅ Same ClojureScript frontend
- ✅ Same JSON response format (kebab-case fields)
- ✅ Same API endpoint paths
- ✅ Same JWT authentication flow

## Performance Benefits

Compared to the Clojure backend:

| Metric | Improvement |
|--------|-------------|
| Memory Usage | 5-10x lower |
| Startup Time | 10-20x faster |
| Request Latency | 2-5x lower |
| Concurrent Connections | Higher capacity |
| GC Pauses | Zero |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
