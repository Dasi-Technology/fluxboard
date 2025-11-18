# Authentication API Testing Examples

This document provides examples for testing the authentication endpoints.

## Endpoints

### 1. Register a New User

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "display_name": "Test User"
  }'
```

**Expected Response (201 Created):**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "rt_a1b2c3d4...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "test@example.com",
    "display_name": "Test User",
    "created_at": "2024-01-18T01:00:00Z",
    "last_login_at": null
  },
  "access_token_expires_at": "2024-01-18T01:15:00Z",
  "refresh_token_expires_at": "2024-02-17T01:00:00Z"
}
```

### 2. Login

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

**Expected Response (200 OK):**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "rt_a1b2c3d4...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "test@example.com",
    "display_name": "Test User",
    "created_at": "2024-01-18T01:00:00Z",
    "last_login_at": "2024-01-18T01:10:00Z"
  },
  "access_token_expires_at": "2024-01-18T01:25:00Z",
  "refresh_token_expires_at": "2024-02-17T01:10:00Z"
}
```

### 3. Get Current User (Protected Route)

```bash
# Save access token from login/register response
ACCESS_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

**Expected Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "test@example.com",
  "display_name": "Test User",
  "created_at": "2024-01-18T01:00:00Z",
  "last_login_at": "2024-01-18T01:10:00Z"
}
```

**Without token (401 Unauthorized):**
```json
{
  "error": "401 Unauthorized",
  "message": "Missing authorization header"
}
```

### 4. Refresh Access Token

```bash
# Save refresh token from login/register response
REFRESH_TOKEN="rt_a1b2c3d4..."

curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$REFRESH_TOKEN\"
  }"
```

**Expected Response (200 OK):**
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refresh_token": "rt_a1b2c3d4...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "test@example.com",
    "display_name": "Test User",
    "created_at": "2024-01-18T01:00:00Z",
    "last_login_at": "2024-01-18T01:10:00Z"
  },
  "access_token_expires_at": "2024-01-18T01:30:00Z",
  "refresh_token_expires_at": "2024-02-17T01:10:00Z"
}
```

### 5. Logout

```bash
# Use the refresh token to logout
REFRESH_TOKEN="rt_a1b2c3d4..."

curl -X POST http://localhost:8080/api/auth/logout \
  -H "Content-Type: application/json" \
  -d "{
    \"refresh_token\": \"$REFRESH_TOKEN\"
  }"
```

**Expected Response (200 OK):**
```json
{
  "message": "Successfully logged out"
}
```

## Error Cases

### Invalid Credentials (Login)
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "wrongpassword"
  }'
```

**Expected Response (401 Unauthorized):**
```json
{
  "error": "401 Unauthorized",
  "message": "Invalid email or password"
}
```

### Duplicate Email (Register)
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

**Expected Response (409 Conflict):**
```json
{
  "error": "409 Conflict",
  "message": "Email already registered"
}
```

### Expired/Invalid Token
```bash
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer invalid_token"
```

**Expected Response (401 Unauthorized):**
```json
{
  "error": "401 Unauthorized",
  "message": "Invalid or expired token"
}
```

## Token Lifetimes

- **Access Token**: 15 minutes (900 seconds - configurable via `JWT_ACCESS_TOKEN_EXPIRY`)
- **Refresh Token**: 30 days (2,592,000 seconds - configurable via `JWT_REFRESH_TOKEN_EXPIRY`)

## Authentication Flow

1. **Register/Login**: Client receives both access and refresh tokens
2. **API Requests**: Client includes access token in `Authorization: Bearer <token>` header
3. **Token Expiry**: When access token expires (15 min), use refresh token to get new tokens
4. **Logout**: Client sends refresh token to invalidate the session
5. **Re-login**: If refresh token expires (30 days), user must login again

## Integration with Other Endpoints

Protected endpoints can use the `AuthenticatedUser` extractor:

```rust
use crate::auth_middleware::auth::AuthenticatedUser;

pub async fn protected_handler(
    user: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    // user.user_id is available here
    // ...
}
```

For optional authentication (user might or might not be logged in):

```rust
use crate::auth_middleware::auth::OptionalUser;

pub async fn optional_handler(
    optional_user: OptionalUser,
) -> Result<HttpResponse, AppError> {
    if let Some(user) = optional_user.0 {
        // User is authenticated: user.user_id
    } else {
        // User is not authenticated
    }
}
```

## Complete Test Sequence

```bash
# 1. Register a new user
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "securepass123",
    "display_name": "Alice"
  }' | jq '.'

# Save the tokens from the response
export ACCESS_TOKEN="<paste access_token here>"
export REFRESH_TOKEN="<paste refresh_token here>"

# 2. Get current user info
curl -X GET http://localhost:8080/api/auth/me \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq '.'

# 3. Wait 16 minutes or manually expire the token, then refresh
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" | jq '.'

# 4. Logout
curl -X POST http://localhost:8080/api/auth/logout \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" | jq '.'

# 5. Try to use the same refresh token again (should fail)
curl -X POST http://localhost:8080/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" | jq '.'