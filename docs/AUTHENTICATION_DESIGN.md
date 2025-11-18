# Authentication and User Management System Design

## Overview

This document outlines the architecture for adding **optional** user authentication to Fluxboard to support S3 file attachment uploads. The system is designed to preserve the current anonymous board creation workflow while enabling authenticated users to upload files.

## Design Principles

1. **Optional Authentication**: Users can continue using Fluxboard without authentication
2. **Backward Compatibility**: Existing anonymous boards must continue to work
3. **Stateless Architecture**: JWT-based authentication compatible with microservices
4. **Security First**: Industry-standard password hashing and token management
5. **Minimal Disruption**: Integrate seamlessly with existing board password system

## Goals

1. Enable user registration and login (email/password)
2. Track file ownership for quota management and security
3. Maintain existing board access control (share tokens + passwords)
4. Support future features requiring user identity (profiles, notifications, quotas)
5. Provide clear migration path from anonymous to authenticated usage

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Frontend                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Anonymous    │  │ Auth Dialog  │  │ User Profile │      │
│  │ Board Access │  │ (Optional)   │  │ (Future)     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
└────────────────────────────┼─────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  API Gateway    │
                    │  (Actix-Web)    │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
      ┌───────▼──────┐ ┌────▼─────┐ ┌─────▼──────┐
      │ Board API    │ │ Auth API │ │ Upload API │
      │ (Existing)   │ │  (New)   │ │  (Future)  │
      └───────┬──────┘ └────┬─────┘ └─────┬──────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                    ┌────────▼────────┐
                    │   PostgreSQL    │
                    │ ┌─────────────┐ │
                    │ │ users       │ │
                    │ │ sessions    │ │
                    │ │ boards      │ │
                    │ │ attachments │ │
                    │ └─────────────┘ │
                    └─────────────────┘
```

## Database Schema

### 1. Users Table

```sql
-- Users table for authentication
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Trigger for updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

**Rationale**:
- `email` as primary identifier (unique index for fast lookups)
- `password_hash` uses Argon2id (industry standard for Rust)
- `display_name` optional for user profiles
- `is_active` for soft deletion (GDPR compliance)
- Uses existing `update_updated_at_column()` trigger

### 2. User Sessions Table

```sql
-- User sessions for JWT-based authentication
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_agent TEXT,
    ip_address INET,
    last_activity_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

-- Cleanup expired sessions (run periodically)
CREATE INDEX idx_user_sessions_cleanup ON user_sessions(expires_at) 
    WHERE expires_at < NOW();
```

**Rationale**:
- `refresh_token_hash`: Long-lived token (30 days default) for obtaining new access tokens
- `access_token_hash`: Short-lived token (15 minutes) for API access (optional storage)
- `expires_at`: Refresh token expiration (can be refreshed indefinitely until explicit logout)
- `last_refreshed_at`: Track when access token was last refreshed
- `is_active`: Soft delete for logout (enables session revocation)
- `user_agent` and `ip_address` for security auditing and anomaly detection
- Cascade delete on user deletion (GDPR right to be forgotten)

### 3. Board Ownership (Optional Enhancement)

```sql
-- Add optional user relationship to boards
ALTER TABLE boards
ADD COLUMN created_by UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX idx_boards_created_by ON boards(created_by);
```

**Rationale**:
- `ON DELETE SET NULL` preserves anonymous boards when user deletes account
- Allows filtering "my boards" in future
- Optional: can be NULL for anonymous boards

### 4. File Attachments (Future S3 Integration)

```sql
-- Card attachments (from S3_ATTACHMENT_DESIGN.md)
CREATE TABLE card_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    uploaded_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    file_size INTEGER NOT NULL,
    s3_key VARCHAR(512) NOT NULL,
    s3_bucket VARCHAR(255) NOT NULL,
    thumbnail_s3_key VARCHAR(512),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_card_attachments_card_id ON card_attachments(card_id);
CREATE INDEX idx_card_attachments_uploaded_by ON card_attachments(uploaded_by);
CREATE INDEX idx_card_attachments_created_at ON card_attachments(created_at);

-- Trigger for updated_at
CREATE TRIGGER update_card_attachments_updated_at
    BEFORE UPDATE ON card_attachments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

**Rationale**:
- `uploaded_by` tracks file ownership for quotas and security
- `ON DELETE SET NULL` preserves files when user deletes account
- Cascade delete when card is deleted (cleanup)

## Authentication Flow

### Registration Flow

```
┌─────────┐                 ┌─────────┐                 ┌──────────┐
│ Client  │                 │ Backend │                 │ Database │
└────┬────┘                 └────┬────┘                 └────┬─────┘
     │                           │                           │
     │ POST /api/auth/register   │                           │
     ├──────────────────────────>│                           │
     │ {email, password, name}   │                           │
     │                           │                           │
     │                           │ Validate email format     │
     │                           │ Check password strength   │
     │                           │                           │
     │                           │ Hash password (Argon2id)  │
     │                           │                           │
     │                           │ INSERT INTO users         │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ User created              │
     │                           │<──────────────────────────┤
     │                           │                           │
     │                           │ Generate access token     │
     │                           │ Generate refresh token    │
     │                           │ Store session             │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ Session stored            │
     │                           │<──────────────────────────┤
     │                           │                           │
     │ 201 Created               │                           │
     │ {access_token,            │                           │
     │  refresh_token,           │                           │
     │  user, expires_at}        │                           │
     │<──────────────────────────┤                           │
     │                           │                           │
     │ Store tokens in           │                           │
     │ localStorage              │                           │
     │                           │                           │
```

### Login Flow

```
┌─────────┐                 ┌─────────┐                 ┌──────────┐
│ Client  │                 │ Backend │                 │ Database │
└────┬────┘                 └────┬────┘                 └────┬─────┘
     │                           │                           │
     │ POST /api/auth/login      │                           │
     ├──────────────────────────>│                           │
     │ {email, password}         │                           │
     │                           │                           │
     │                           │ SELECT user WHERE email   │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ User record               │
     │                           │<──────────────────────────┤
     │                           │                           │
     │                           │ Verify password with      │
     │                           │ Argon2::verify_password() │
     │                           │                           │
     │                           │ Generate access token     │
     │                           │ Generate refresh token    │
     │                           │ Store session             │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ Session stored            │
     │                           │<──────────────────────────┤
     │                           │                           │
     │                           │ UPDATE last_login_at      │
     │                           ├──────────────────────────>│
     │                           │                           │
     │ 200 OK                    │                           │
     │ {access_token,            │                           │
     │  refresh_token,           │                           │
     │  user, expires_at}        │                           │
     │<──────────────────────────┤                           │
     │                           │                           │
     │ Store tokens in           │                           │
     │ localStorage              │                           │
     │                           │                           │
```

### Authenticated Request Flow

```
┌─────────┐                 ┌──────────────┐           ┌──────────┐
│ Client  │                 │ Auth         │           │ Database │
│         │                 │ Middleware   │           │          │
└────┬────┘                 └──────┬───────┘           └────┬─────┘
     │                             │                        │
     │ POST /api/cards/{id}/       │                        │
     │      attachments/upload-url │                        │
     │ Authorization: Bearer       │                        │
     │    <access_token>           │                        │
     ├────────────────────────────>│                        │
     │                             │                        │
     │                             │ Extract access token   │
     │                             │ Verify JWT signature   │
     │                             │ Check expiration       │
     │                             │ Extract user_id        │
     │                             │                        │
     │                             │ SELECT user            │
     │                             ├───────────────────────>│
     │                             │                        │
     │                             │ User record            │
     │                             │<───────────────────────┤
     │                             │                        │
     │                             │ Inject user into       │
     │                             │ request context        │
     │                             │                        │
     │                             ├──────────────┐         │
     │                             │ Call handler │         │
     │                             │<─────────────┘         │
     │                             │                        │
     │ 200 OK {upload_url}         │                        │
     │<────────────────────────────┤                        │
     │                             │                        │
```

### Logout Flow

```
┌─────────┐                 ┌─────────┐                 ┌──────────┐
│ Client  │                 │ Backend │                 │ Database │
└────┬────┘                 └────┬────┘                 └────┬─────┘
     │                           │                           │
     │ POST /api/auth/logout     │                           │
     │ Authorization: Bearer      │                           │
     │    <refresh_token>        │                           │
     ├──────────────────────────>│                           │
     │                           │                           │
     │                           │ Extract refresh token     │
     │                           │ Hash token                │
     │                           │                           │
     │                           │ UPDATE sessions           │
     │                           │ SET is_active = FALSE     │
     │                           │ WHERE refresh_token_hash  │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ Session deactivated       │
     │                           │<──────────────────────────┤
     │                           │                           │
     │ 204 No Content            │                           │
     │<──────────────────────────┤                           │
     │                           │                           │
     │ Clear localStorage        │                           │
     │                           │                           │
```

### Token Refresh Flow

```
┌─────────┐                 ┌─────────┐                 ┌──────────┐
│ Client  │                 │ Backend │                 │ Database │
└────┬────┘                 └────┬────┘                 └────┬─────┘
     │                           │                           │
     │ POST /api/auth/refresh    │                           │
     │ Authorization: Bearer      │                           │
     │    <refresh_token>        │                           │
     ├──────────────────────────>│                           │
     │                           │                           │
     │                           │ Verify refresh token      │
     │                           │ Hash token                │
     │                           │                           │
     │                           │ SELECT session            │
     │                           │ WHERE refresh_token_hash  │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ Session record            │
     │                           │<──────────────────────────┤
     │                           │                           │
     │                           │ Check is_active = TRUE    │
     │                           │ Check expires_at          │
     │                           │                           │
     │                           │ Generate new access token │
     │                           │                           │
     │                           │ UPDATE session            │
     │                           │ SET last_refreshed_at,    │
     │                           │     access_token_hash     │
     │                           ├──────────────────────────>│
     │                           │                           │
     │                           │ Session updated           │
     │                           │<──────────────────────────┤
     │                           │                           │
     │ 200 OK                    │                           │
     │ {access_token,            │                           │
     │  expires_at}              │                           │
     │<──────────────────────────┤                           │
     │                           │                           │
     │ Update access_token in    │                           │
     │ memory (not localStorage) │                           │
     │                           │                           │
```

## API Endpoints

### Authentication Endpoints

#### 1. Register

```
POST /api/auth/register
Content-Type: application/json

Request:
{
  "email": "user@example.com",
  "password": "secure_password_123",
  "display_name": "John Doe"  // Optional
}

Response: 201 Created
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",  // Short-lived (15 min)
  "refresh_token": "rt_1234567890abcdef...",   // Long-lived (30 days)
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "display_name": "John Doe",
    "created_at": "2025-01-18T00:00:00Z"
  },
  "access_token_expires_at": "2025-01-18T00:15:00Z",
  "refresh_token_expires_at": "2025-02-17T00:00:00Z"
}

Error Responses:
- 400 Bad Request: Invalid email format, weak password, missing fields
- 409 Conflict: Email already registered
- 500 Internal Server Error: Database error
```

**Validation Rules**:
- Email: RFC 5322 format validation
- Password: Minimum 8 characters, at least 1 uppercase, 1 lowercase, 1 number
- Display name: Optional, max 100 characters

#### 2. Login

```
POST /api/auth/login
Content-Type: application/json

Request:
{
  "email": "user@example.com",
  "password": "secure_password_123"
}

Response: 200 OK
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "rt_1234567890abcdef...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "display_name": "John Doe",
    "created_at": "2025-01-18T00:00:00Z",
    "last_login_at": "2025-01-18T00:00:00Z"
  },
  "access_token_expires_at": "2025-01-18T00:15:00Z",
  "refresh_token_expires_at": "2025-02-17T00:00:00Z"
}

Error Responses:
- 401 Unauthorized: Invalid email or password
- 403 Forbidden: Account disabled
- 500 Internal Server Error: Database error
```

#### 3. Logout

```
POST /api/auth/logout
Authorization: Bearer <refresh_token>

Response: 204 No Content

Error Responses:
- 401 Unauthorized: Invalid or expired refresh token
- 404 Not Found: Session not found
```

**Note**: Logout requires the refresh token (not access token) to revoke the session.

#### 4. Refresh Access Token

```
POST /api/auth/refresh
Authorization: Bearer <refresh_token>

Response: 200 OK
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "access_token_expires_at": "2025-01-18T00:30:00Z"
}

Error Responses:
- 401 Unauthorized: Invalid or expired refresh token
- 403 Forbidden: Session revoked or inactive
```

**Token Refresh Strategy**:
- Frontend automatically refreshes access token when it's about to expire
- Refresh tokens can be used indefinitely (until logout or 30-day expiration)
- If refresh token expires, user must log in again

#### 5. Get Current User

```
GET /api/auth/me
Authorization: Bearer <access_token>

Response: 200 OK
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "display_name": "John Doe",
  "created_at": "2025-01-18T00:00:00Z",
  "last_login_at": "2025-01-18T00:00:00Z"
}

Error Responses:
- 401 Unauthorized: Invalid or expired access token
```

### Modified Attachment Endpoints (Future)

```
POST /api/cards/{card_id}/attachments/upload-url
Authorization: Bearer <access_token>
X-Board-Password: <password>  // Still required for locked boards
Content-Type: application/json

Request:
{
  "filename": "screenshot.png",
  "content_type": "image/png",
  "file_size": 2048576
}

Response: 200 OK
{
  "upload_url": "https://s3.amazonaws.com/...",
  "attachment_id": "650e8400-e29b-41d4-a716-446655440000",
  "s3_key": "attachments/board123/card456/uuid.png"
}
```

## Token Architecture

### Access Token (JWT)

Short-lived JWT for API authentication:

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",  // User ID
  "email": "user@example.com",
  "iat": 1737158400,  // Issued at (Unix timestamp)
  "exp": 1737159300,  // Expiration (15 minutes later)
  "type": "access"
}
```

**Properties**:
- Algorithm: HS256 (HMAC with SHA-256)
- Expiration: 15 minutes (configurable via `ACCESS_TOKEN_EXPIRY_MINUTES`)
- Stateless: No database lookup required for validation
- Cannot be revoked (short expiration mitigates risk)

### Refresh Token

Long-lived opaque token for obtaining new access tokens:

```
Format: rt_<32-byte-random-base64>
Example: rt_Xk7mP9nQ2vL8cW5jH4dF6gS1aZ3bY0eR...
```

**Properties**:
- Cryptographically secure random token (32 bytes)
- Hashed (SHA-256) before storage in database
- Expiration: 30 days (configurable via `REFRESH_TOKEN_EXPIRY_DAYS`)
- Can be refreshed indefinitely until logout or expiration
- Revocable: Stored in `user_sessions` table with `is_active` flag
- One-time use: Each refresh invalidates previous refresh token (optional security enhancement)

**Security Benefits**:
- Access tokens expire quickly (15 min) → limited window for token theft
- Refresh tokens stored in database → can be revoked on logout or suspicious activity
- Refresh tokens never sent with regular API requests → reduced exposure

## Integration with Existing Systems

### 1. Board Password System

**Current Behavior** (preserved):
- Anonymous users can create boards
- Boards have auto-generated passwords
- Board owners store passwords in localStorage
- Locked boards require `X-Board-Password` header

**New Behavior**:
- Authenticated users can optionally link boards to their account
- Board password system remains unchanged
- Both `Authorization: Bearer <token>` AND `X-Board-Password` may be required

**Example**: Upload attachment to locked board
```
POST /api/cards/{card_id}/attachments/upload-url
Authorization: Bearer <access_token>      // Required: identifies user
X-Board-Password: <board_password>        // Required: board is locked
```

### 2. Share Token System

**No Changes Required**:
- Share tokens remain the primary board access mechanism
- Anonymous users continue to access boards via share tokens
- Authenticated users also use share tokens for board access

### 3. SSE (Server-Sent Events)

**No Authentication Required**:
- SSE connections don't require authentication
- Board updates broadcast to all clients with share token
- Presence system remains anonymous (username-based)

**Rationale**: Real-time collaboration should work for anonymous users

### 4. Presence Service

**Remains Anonymous**:
- WebSocket connections don't require authentication
- Users identified by self-chosen usernames
- No link between authenticated users and presence

**Future Enhancement**: Optional user ID in presence messages for avatars

## Security Considerations

### 1. Password Security

**Hashing Algorithm**: Argon2id
```rust
// Recommended parameters (from OWASP)
let config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 65536,      // 64 MB
    time_cost: 3,         // 3 iterations
    lanes: 4,             // 4 parallel threads
    secret: &[],
    ad: &[],
    hash_length: 32
};
```

**Why Argon2id**:
- Winner of Password Hashing Competition (2015)
- Resistant to side-channel attacks
- Configurable memory and CPU cost
- Native Rust support via `argon2` crate

**Password Requirements**:
- Minimum 8 characters
- At least 1 uppercase letter
- At least 1 lowercase letter
- At least 1 number
- Optional: Check against common password lists (future)

### 2. Token Security

**JWT Secret Management**:
```env
# backend/.env
JWT_SECRET=<256-bit-random-key>  # Required
JWT_EXPIRY_DAYS=7                # Default: 7 days
```

**Token Generation**:
```rust
// Generate cryptographically secure random token
use rand::Rng;
let token: [u8; 32] = rand::thread_rng().gen();
let token_base64 = base64::encode(token);

// Hash token for storage (SHA-256)
use sha2::{Sha256, Digest};
let mut hasher = Sha256::new();
hasher.update(&token);
let token_hash = format!("{:x}", hasher.finalize());
```

**Token Revocation**:
- Store hashed tokens in `user_sessions` table
- DELETE session on logout
- Check session validity on every authenticated request
- Automatic cleanup of expired sessions (cron job)

### 3. Rate Limiting

**Authentication Endpoints**:
```rust
// Recommended limits
POST /api/auth/login:    5 attempts per 15 minutes per IP
POST /api/auth/register: 3 attempts per hour per IP
POST /api/auth/refresh:  30 attempts per hour per user  // Increased for auto-refresh
POST /api/auth/logout:   10 attempts per hour per user
```

**Implementation**: Use `actix-governor` crate for rate limiting middleware

### 4. HTTPS Enforcement

**Production Requirements**:
- Force HTTPS for all authentication endpoints
- Set `Secure` flag on cookies (if used)
- Use `Strict-Transport-Security` header (HSTS)

**Development**: Allow HTTP on localhost only

### 5. Session Security

**Session Properties**:
- Store `user_agent` and `ip_address` for each session
- Optional: Detect session hijacking (IP change alerts)
- Automatic expiration after 7 days
- Manual revocation on logout
- Revoke all sessions on password change

**Session Cleanup**:
```sql
-- Cron job to delete expired inactive sessions (run daily)
DELETE FROM user_sessions
WHERE expires_at < NOW()
  AND is_active = FALSE;

-- Optionally: Hard delete inactive sessions older than 90 days
DELETE FROM user_sessions
WHERE is_active = FALSE
  AND created_at < NOW() - INTERVAL '90 days';
```

### 6. Input Validation

**Email Validation**:
```rust
// Use validator crate
use validator::Validate;

#[derive(Deserialize, Validate)]
struct RegisterInput {
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8, max = 128))]
    password: String,
    
    #[validate(length(max = 100))]
    display_name: Option<String>,
}
```

**SQL Injection Prevention**:
- Use SQLx compile-time query validation
- Always use parameterized queries
- Never concatenate user input into SQL

## Rust Implementation Details

### 1. Dependencies

Add to `backend/Cargo.toml`:
```toml
[dependencies]
# Existing dependencies...

# Authentication
argon2 = "0.5"
jsonwebtoken = "9.3"
validator = { version = "0.18", features = ["derive"] }
rand = "0.8"
base64 = "0.22"
sha2 = "0.10"

# Rate limiting (optional)
actix-governor = "0.5"
```

### 2. Project Structure

```
backend/src/
├── main.rs
├── config.rs                    # Add JWT secret config
├── db.rs
├── error.rs
├── models/
│   ├── mod.rs
│   ├── board.rs
│   ├── user.rs                  # New: User model
│   └── session.rs               # New: Session model
├── services/
│   ├── mod.rs
│   ├── board_service.rs
│   ├── auth_service.rs          # New: Auth business logic
│   └── user_service.rs          # New: User management
├── handlers/
│   ├── mod.rs
│   ├── board_handlers.rs
│   └── auth_handlers.rs         # New: Auth endpoints
├── middleware/
│   ├── mod.rs
│   └── auth.rs                  # New: JWT verification middleware
└── utils/
    ├── mod.rs
    ├── password.rs              # New: Password hashing utilities
    └── jwt.rs                   # New: JWT utilities
```

### 3. Configuration Updates

Update [`backend/src/config.rs`](backend/src/config.rs):
```rust
#[derive(Clone, Debug)]
pub struct Config {
    // Existing fields...
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,
    pub cors_origin: Option<String>,
    pub gemini_api_key: Option<String>,
    
    // New: Authentication config
    pub jwt_secret: String,
    pub jwt_expiry_days: i64,
    pub password_min_length: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            // Existing...
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            // ...
            
            // New
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiry_days: env::var("JWT_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .expect("JWT_EXPIRY_DAYS must be valid i64"),
            password_min_length: env::var("PASSWORD_MIN_LENGTH")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .expect("PASSWORD_MIN_LENGTH must be valid usize"),
        }
    }
}
```

### 4. Middleware Structure

```rust
// backend/src/middleware/auth.rs
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub async fn validator(
        req: ServiceRequest,
        credentials: BearerAuth,
    ) -> Result<ServiceRequest, (Error, ServiceRequest)> {
        // Extract JWT token
        let token = credentials.token();
        
        // Verify JWT signature and expiration
        let claims = jwt::verify_token(token)?;
        
        // Check session in database
        let session = Session::find_by_token_hash(&claims.jti).await?;
        
        // Check expiration
        if session.expires_at < Utc::now() {
            return Err(AuthError::TokenExpired);
        }
        
        // Load user
        let user = User::find_by_id(session.user_id).await?;
        
        // Inject user into request extensions
        req.extensions_mut().insert(user);
        
        Ok(req)
    }
}
```

### 5. Handler Patterns

```rust
// backend/src/handlers/auth_handlers.rs
use actix_web::{web, HttpResponse};
use crate::models::User;

// Extract authenticated user from request
pub async fn get_current_user(
    user: web::ReqData<User>,  // Injected by middleware
) -> HttpResponse {
    HttpResponse::Ok().json(user.into_inner())
}

// Optional authentication
pub async fn upload_attachment(
    user: Option<web::ReqData<User>>,  // Optional
    share_token: web::Path<String>,
) -> HttpResponse {
    if let Some(user) = user {
        // Authenticated upload
    } else {
        // Reject: authentication required for uploads
        return HttpResponse::Unauthorized().json(json!({
            "error": "Authentication required to upload attachments"
        }));
    }
}
```

## Frontend Integration

### 1. Auth Store (Zustand)

Create [`frontend/src/store/auth-store.ts`](frontend/src/store/auth-store.ts):
```typescript
interface User {
  id: string;
  email: string;
  display_name: string | null;
  created_at: string;
}

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  
  // Actions
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, displayName?: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
  checkAuth: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
  
  login: async (email, password) => {
    const response = await api.post('/auth/login', { email, password });
    const { token, user, expires_at } = response.data;
    
    localStorage.setItem('auth_token', token);
    localStorage.setItem('token_expires_at', expires_at);
    
    set({ user, token, isAuthenticated: true });
  },
  
  // ... other actions
}));
```

### 2. API Client Updates

Update [`frontend/src/lib/api.ts`](frontend/src/lib/api.ts):
```typescript
// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle 401 errors (token expired)
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Clear auth state and redirect to login
      useAuthStore.getState().logout();
    }
    return Promise.reject(error);
  }
);
```

### 3. Auth Dialog Component

Create [`frontend/src/components/dialogs/auth-dialog.tsx`](frontend/src/components/dialogs/auth-dialog.tsx):
```typescript
export function AuthDialog({ isOpen, onClose, mode = 'login' }) {
  const [authMode, setAuthMode] = useState<'login' | 'register'>(mode);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [displayName, setDisplayName] = useState('');
  
  const { login, register } = useAuthStore();
  
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    try {
      if (authMode === 'login') {
        await login(email, password);
      } else {
        await register(email, password, displayName);
      }
      onClose();
    } catch (error) {
      // Show error toast
    }
  };
  
  // ... render form
}
```

### 4. Protected Actions

```typescript
// frontend/src/components/card/attachment-upload.tsx
export function AttachmentUpload({ cardId }) {
  const { isAuthenticated } = useAuthStore();
  const [showAuthDialog, setShowAuthDialog] = useState(false);
  
  const handleUpload = () => {
    if (!isAuthenticated) {
      setShowAuthDialog(true);
      return;
    }
    
    // Proceed with upload
  };
  
  return (
    <>
      <button onClick={handleUpload}>Upload Attachment</button>
      
      {showAuthDialog && (
        <AuthDialog 
          isOpen={showAuthDialog}
          onClose={() => setShowAuthDialog(false)}
          mode="login"
        />
      )}
    </>
  );
}
```

## Migration Strategy

### Phase 1: Add Authentication System (No Breaking Changes)

**Week 1-2: Database & Backend**
1. Create database migrations
2. Implement user model and service
3. Implement authentication handlers
4. Add JWT middleware
5. Update configuration

**Changes**:
- New tables: `users`, `user_sessions`
- New endpoints: `/api/auth/*`
- No changes to existing endpoints

**Compatibility**: 100% backward compatible

### Phase 2: Frontend Integration (Optional Features)

**Week 3-4: Frontend**
1. Create auth store
2. Build auth dialog component
3. Add "Login" button to navbar
4. Update API client with token interceptor

**Changes**:
- New components: AuthDialog, UserProfile
- New store: auth-store.ts
- No changes to existing board functionality

**Compatibility**: Anonymous users see no changes

### Phase 3: Link Boards to Users (Optional Enhancement)

**Week 5: Board Ownership**
1. Add `created_by` column to boards
2. Auto-link boards created by authenticated users
3. Add "My Boards" filter
4. Migrate existing anonymous boards (keep NULL)

**Migration Script**:
```sql
-- Add column (nullable for backward compatibility)
ALTER TABLE boards ADD COLUMN created_by UUID REFERENCES users(id) ON DELETE SET NULL;

-- Existing boards remain anonymous (created_by = NULL)
-- New boards auto-populate created_by if user is authenticated
```

**Compatibility**: Existing anonymous boards continue to work

### Phase 4: Enable File Uploads (Authentication Required)

**Week 6-8: S3 Integration**
1. Implement S3 service
2. Create attachments table with `uploaded_by` column
3. Add upload endpoints (require authentication)
4. Build upload UI

**Breaking Change**: File uploads require authentication

**User Communication**:
- Show "Login to upload attachments" prompt
- Allow registration directly from upload dialog
- Keep all other features anonymous

## Cost and Performance Considerations

### Database Impact

**Additional Storage**:
- Users: ~500 bytes/user
- Sessions: ~300 bytes/session (auto-expire after 7 days)
- Negligible impact for 1,000 users (~0.8 MB)

**Query Performance**:
- `idx_users_email`: Fast login lookups (unique index)
- `idx_user_sessions_token_hash`: Fast session validation
- Existing board queries unchanged

### API Latency

**Authentication Overhead**:
- JWT verification: ~1-2ms
- Session lookup: ~5-10ms (database query)
- **Total overhead**: ~10-15ms per authenticated request

**Optimization**:
- Cache active sessions in Redis (future)
- Use connection pooling (already implemented)

### Security Overhead

**Password Hashing**:
- Argon2id with recommended parameters: ~100-200ms
- Only during registration/login (not on every request)
- Non-blocking (use tokio::spawn_blocking)

## Testing Strategy

### Backend Tests

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[actix_web::test]
    async fn test_register_user() {
        // Test user registration
    }
    
    #[actix_web::test]
    async fn test_login_success() {
        // Test successful login
    }
    
    #[actix_web::test]
    async fn test_login_invalid_password() {
        // Test login with wrong password
    }
    
    #[actix_web::test]
    async fn test_jwt_validation() {
        // Test JWT verification
    }
    
    #[actix_web::test]
    async fn test_session_expiration() {
        // Test expired session rejection
    }
}
```

**Integration Tests**:
- Test complete registration → login → authenticated request flow
- Test session revocation on logout
- Test concurrent sessions for same user
- Test board creation with and without authentication

### Frontend Tests

**Component Tests**:
- AuthDialog: render, form validation, submission
- Login flow: success, error handling
- Register flow: success, error handling

**E2E Tests** (Playwright):
```typescript
test('anonymous user can create board', async ({ page }) => {
  // Verify existing functionality unchanged
});

test('authenticated user can upload attachment', async ({ page }) => {
  // Register → Login → Create board → Upload file
});

test('anonymous user sees login prompt for upload', async ({ page }) => {
  // Create board → Try upload → See auth dialog
});
```

### Security Tests

**Penetration Testing**:
- SQL injection in email/password fields
- JWT token tampering
- Session token theft
- Brute-force password attempts
- Cross-site scripting (XSS) in display names

**Rate Limiting Tests**:
- Verify login rate limits enforced
- Verify registration rate limits enforced

## Configuration

### Backend Environment Variables

Add to [`backend/.env.example`](backend/.env.example):
```env
# Existing...
DATABASE_URL=postgresql://postgres:password@localhost:5432/fluxboard
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
GEMINI_API_KEY=...

# New: Authentication
JWT_SECRET=<generate-with-openssl-rand-base64-32>
JWT_EXPIRY_DAYS=7
PASSWORD_MIN_LENGTH=8
AUTH_RATE_LIMIT_ATTEMPTS=5
AUTH_RATE_LIMIT_WINDOW_MINUTES=15
```

### Frontend Environment Variables

Add to [`frontend/.env.example`](frontend/.env.example):
```env
# Existing...
NEXT_PUBLIC_API_URL=http://localhost:8080/api
NEXT_PUBLIC_WS_URL=ws://localhost:3001

# New: Authentication (if needed)
NEXT_PUBLIC_ENABLE_AUTH=true
```

## Future Enhancements

### Short-term (3-6 months)
- Email verification on registration
- Password reset flow (forgot password)
- Account settings page (change password, delete account)
- User profile with avatar upload
- "My Boards" dashboard

### Medium-term (6-12 months)
- OAuth 2.0 providers (Google, GitHub)
- Two-factor authentication (2FA)
- User quotas (storage limits)
- Team workspaces
- Invitation system

### Long-term (12+ months)
- Role-based access control (RBAC)
- Board permissions (viewer, editor, admin)
- Activity audit logs
- SSO for enterprise customers

## Summary

This authentication design provides:

✅ **Optional Authentication**: Users can continue using Fluxboard anonymously  
✅ **Secure by Default**: Argon2id password hashing, JWT tokens, rate limiting  
✅ **Backward Compatible**: Zero breaking changes to existing functionality  
✅ **Scalable**: Stateless JWT architecture supports microservices  
✅ **Future-Ready**: Enables file uploads, user profiles, quotas, and more  
✅ **Developer-Friendly**: Clear API contracts, comprehensive error handling  
✅ **Production-Ready**: Security best practices, monitoring, testing strategy  

**Key Design Decisions**:

1. **JWT + Session Hybrid**: Stateless JWTs with database sessions for revocation
2. **Argon2id Password Hashing**: Industry-standard, resistant to attacks
3. **Optional Board Ownership**: Preserve anonymous boards, enable "My Boards"
4. **Dual Access Control**: Both JWT tokens AND board passwords may be required
5. **Gradual Migration**: Phased rollout without breaking existing users

**Next Steps**:

1. Review and approve this design document
2. Create implementation tasks based on migration phases
3. Set up development environment with JWT secret
4. Begin Phase 1: Database migrations and backend implementation

---

**Document Status**: Draft for Review  
**Author**: Architect Mode  
**Date**: 2025-01-18  
**Related**: [`S3_ATTACHMENT_DESIGN.md`](S3_ATTACHMENT_DESIGN.md)