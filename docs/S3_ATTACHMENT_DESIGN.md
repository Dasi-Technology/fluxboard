# S3 Attachment System Design

## Overview

This document outlines the architecture and implementation plan for adding S3-based image attachments to Fluxboard cards. The system requires user authentication (email/password) to prevent abuse and supports image uploads up to 5MB.

## Goals

1. **User Authentication**: Implement email/password authentication stored in PostgreSQL
2. **S3 Integration**: Use AWS S3 for secure, scalable image storage
3. **Abuse Prevention**: Require authentication before allowing uploads
4. **Size Limits**: Enforce 5MB maximum file size
5. **Real-time Updates**: Broadcast attachment events via SSE
6. **Access Control**: Ensure users can only manage attachments on boards they have access to

## Architecture Overview

```mermaid
graph TB
    subgraph Frontend
        UI[User Interface]
        AuthForm[Login/Register Form]
        UploadWidget[Image Upload Widget]
        AttachmentDisplay[Attachment Display]
    end
    
    subgraph Backend
        AuthAPI[Auth API Endpoints]
        AttachmentAPI[Attachment API Endpoints]
        AuthMiddleware[Auth Middleware]
        S3Service[S3 Service]
        SSEManager[SSE Manager]
    end
    
    subgraph Storage
        PostgreSQL[(PostgreSQL)]
        S3[AWS S3 Bucket]
    end
    
    UI --> AuthForm
    UI --> UploadWidget
    UI --> AttachmentDisplay
    
    AuthForm --> AuthAPI
    UploadWidget --> AttachmentAPI
    AttachmentDisplay --> AttachmentAPI
    
    AuthAPI --> PostgreSQL
    AttachmentAPI --> AuthMiddleware
    AuthMiddleware --> PostgreSQL
    AttachmentAPI --> S3Service
    S3Service --> S3
    AttachmentAPI --> PostgreSQL
    AttachmentAPI --> SSEManager
```

## 1. User Authentication System

### Database Schema

#### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_users_email ON users(email);
```

#### Sessions Table (JWT Alternative)
```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    user_agent TEXT,
    ip_address INET
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
```

### Authentication Flow

1. **Registration**:
   - User provides email + password + optional display_name
   - Backend validates email format and password strength (min 8 chars)
   - Password hashed using `argon2` (Rust best practice)
   - User record created in database
   - Session token generated and returned

2. **Login**:
   - User provides email + password
   - Backend verifies credentials
   - Generate session token (random 32-byte token)
   - Store hashed token in `user_sessions` table
   - Return token to frontend
   - Frontend stores token in localStorage

3. **Authentication Middleware**:
   - Extract token from `Authorization: Bearer <token>` header
   - Hash token and verify against `user_sessions` table
   - Check expiration (7-day default)
   - Inject authenticated user into request context

4. **Logout**:
   - Delete session from database
   - Frontend clears localStorage

### Rust Dependencies

Add to `backend/Cargo.toml`:
```toml
# Authentication
argon2 = "0.5"
jsonwebtoken = "9.3"  # Optional: for JWT instead of sessions
base64 = "0.22"
```

## 2. Attachment System

### Database Schema

#### Attachments Table
```sql
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
    thumbnail_s3_key VARCHAR(512),  -- Optional: for image thumbnails
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_card_attachments_card_id ON card_attachments(card_id);
CREATE INDEX idx_card_attachments_uploaded_by ON card_attachments(uploaded_by);
CREATE INDEX idx_card_attachments_created_at ON card_attachments(created_at);
```

### S3 Configuration

#### Environment Variables
```env
# S3 Configuration
AWS_REGION=us-east-1
AWS_S3_BUCKET=fluxboard-attachments
AWS_ACCESS_KEY_ID=your_access_key_id
AWS_SECRET_ACCESS_KEY=your_secret_access_key
S3_UPLOAD_MAX_SIZE=5242880  # 5MB in bytes
S3_ALLOWED_TYPES=image/jpeg,image/png,image/gif,image/webp
```

#### Rust Dependencies

Add to `backend/Cargo.toml`:
```toml
# S3 Integration
aws-config = "1.5"
aws-sdk-s3 = "1.51"
tokio-util = { version = "0.7", features = ["codec"] }
bytes = "1.7"
mime = "0.3"
```

### S3 Service Implementation

Key features:
- **Pre-signed URLs**: Generate temporary upload URLs (15-minute expiration)
- **Multipart Upload**: Support for larger files (though we limit to 5MB)
- **Public Read Access**: Generate pre-signed download URLs (7-day expiration)
- **S3 Key Structure**: `attachments/{board_id}/{card_id}/{uuid}.{ext}`
- **Thumbnail Generation**: Optional - resize images to 200x200 thumbnails

### Upload Flow

1. **Frontend requests upload URL**:
   - `POST /api/cards/{card_id}/attachments/upload-url`
   - Body: `{ filename: string, content_type: string, file_size: number }`
   - Auth required (Bearer token)
   - Validates file size ≤ 5MB
   - Validates content type (images only)
   - Returns: `{ upload_url: string, attachment_id: uuid, s3_key: string }`

2. **Frontend uploads directly to S3**:
   - Use pre-signed URL (bypass backend for performance)
   - Show upload progress
   - Handle errors (network, S3 capacity, etc.)

3. **Frontend confirms upload**:
   - `POST /api/cards/{card_id}/attachments/{attachment_id}/confirm`
   - Backend verifies S3 object exists
   - Updates attachment record as confirmed
   - Broadcasts SSE event: `attachment:created`

4. **Cleanup**: 
   - Cron job to delete unconfirmed attachments after 1 hour
   - Delete S3 objects for orphaned attachments

### Download Flow

1. **Frontend requests download URL**:
   - `GET /api/attachments/{attachment_id}/download-url`
   - No auth required (board access implicitly grants access)
   - Returns pre-signed S3 URL (7-day expiration)

2. **Browser downloads from S3**:
   - Use pre-signed URL
   - Display inline or force download based on `Content-Disposition`

## 3. API Endpoints

### Authentication Endpoints

```rust
// POST /api/auth/register
{
  "email": "user@example.com",
  "password": "secure_password",
  "display_name": "John Doe"  // optional
}
// Response: { token: string, user: User, expires_at: string }

// POST /api/auth/login
{
  "email": "user@example.com",
  "password": "secure_password"
}
// Response: { token: string, user: User, expires_at: string }

// POST /api/auth/logout
// Headers: Authorization: Bearer <token>
// Response: 204 No Content

// GET /api/auth/me
// Headers: Authorization: Bearer <token>
// Response: { user: User }

// POST /api/auth/refresh
// Headers: Authorization: Bearer <token>
// Response: { token: string, expires_at: string }
```

### Attachment Endpoints

```rust
// Request upload URL
// POST /api/cards/{card_id}/attachments/upload-url
// Headers: Authorization: Bearer <token>, X-Board-Password: <password>
{
  "filename": "screenshot.png",
  "content_type": "image/png",
  "file_size": 2048576
}
// Response: { upload_url: string, attachment_id: uuid, s3_key: string }

// Confirm upload completion
// POST /api/cards/{card_id}/attachments/{attachment_id}/confirm
// Headers: Authorization: Bearer <token>, X-Board-Password: <password>
// Response: { attachment: Attachment }

// List card attachments
// GET /api/cards/{card_id}/attachments
// Headers: X-Board-Password: <password> (optional)
// Response: [Attachment]

// Get download URL
// GET /api/attachments/{attachment_id}/download-url
// Headers: X-Board-Password: <password> (optional)
// Response: { download_url: string, expires_at: string }

// Delete attachment
// DELETE /api/attachments/{attachment_id}
// Headers: Authorization: Bearer <token>, X-Board-Password: <password>
// Response: 204 No Content
```

## 4. SSE Events

Add new event types to [`backend/src/sse/events.rs`](backend/src/sse/events.rs):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    // ... existing events ...
    
    // Attachment events
    AttachmentCreated {
        attachment: CardAttachment,
    },
    AttachmentDeleted {
        attachment_id: Uuid,
        card_id: Uuid,
    },
}
```

Update [`frontend/src/store/board-store.ts`](frontend/src/store/board-store.ts) to handle these events.

## 5. Frontend Integration

### Components

1. **Login/Register Dialog** (`frontend/src/components/dialogs/auth-dialog.tsx`)
   - Toggle between login and register forms
   - Email validation
   - Password strength indicator
   - Error handling

2. **Attachment Upload Widget** (`frontend/src/components/card/attachment-upload.tsx`)
   - Drag-and-drop zone
   - File picker
   - Image preview before upload
   - Progress bar
   - Size/type validation
   - Maximum 5 attachments per card (configurable)

3. **Attachment List** (`frontend/src/components/card/attachment-list.tsx`)
   - Thumbnail grid view
   - Click to view full size (modal)
   - Delete button (only for uploader or board creator)
   - Download button

4. **Auth Provider** (`frontend/src/providers/auth-provider.tsx`)
   - Context for auth state
   - Auto-refresh tokens before expiration
   - Logout on 401 errors
   - Persist session across page reloads

### State Management

Update [`frontend/src/store/board-store.ts`](frontend/src/store/board-store.ts):

```typescript
interface CardAttachment {
  id: string;
  card_id: string;
  uploaded_by: string;
  filename: string;
  original_filename: string;
  content_type: string;
  file_size: number;
  s3_key: string;
  created_at: string;
  updated_at: string;
}

interface Card {
  // ... existing fields ...
  attachments?: CardAttachment[];
}
```

Create new store for auth ([`frontend/src/store/auth-store.ts`](frontend/src/store/auth-store.ts)):

```typescript
interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, displayName?: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
}
```

## 6. Security Considerations

### Authentication Security
- **Password Hashing**: Use Argon2 with appropriate cost parameters
- **Session Tokens**: Generate cryptographically secure random tokens (32 bytes)
- **Token Expiration**: 7-day default, configurable
- **HTTPS Only**: Enforce HTTPS in production (tokens in headers)
- **Rate Limiting**: Limit login attempts (5 per 15 minutes per IP)
- **Email Verification**: Optional - send verification email on registration

### Upload Security
- **File Type Validation**: Server-side MIME type verification
- **File Size Validation**: Reject files > 5MB
- **Malware Scanning**: Optional - integrate ClamAV or similar
- **Content Scanning**: Validate image headers to prevent exploits
- **S3 Bucket Policy**: Private bucket with pre-signed URL access only

### Access Control
- **Board Password**: Required for protected boards (existing system)
- **Attachment Ownership**: Store uploader ID, allow deletion by uploader or board creator
- **Pre-signed URL Expiration**: Short-lived URLs (15 min upload, 7 day download)
- **CORS Configuration**: Restrict S3 bucket CORS to frontend domain

### Data Privacy
- **PII Handling**: Email addresses are PII - ensure GDPR compliance
- **User Deletion**: Cascade delete user sessions and set attachments.uploaded_by to NULL
- **Attachment Cleanup**: Delete S3 objects when attachments are deleted
- **Audit Logging**: Log authentication events and upload events

## 7. Implementation Phases

### Phase 1: User Authentication (Week 1)
- [x] Database migrations for users and sessions
- [x] User model and service layer
- [x] Authentication handlers (register, login, logout)
- [x] Auth middleware
- [x] Frontend auth store and context
- [x] Login/Register dialog components
- [x] Session persistence in localStorage
- [x] Protected route wrapper

### Phase 2: S3 Integration (Week 2)
- [x] S3 service configuration
- [x] Pre-signed URL generation
- [x] Upload/download flow implementation
- [x] Database migrations for attachments
- [x] Attachment model and service layer
- [x] Backend API endpoints
- [x] Error handling and validation

### Phase 3: Frontend Upload UI (Week 3)
- [x] Attachment upload widget
- [x] Drag-and-drop support
- [x] Upload progress tracking
- [x] Image preview
- [x] File validation
- [x] Integration with edit card dialog

### Phase 4: Attachment Display (Week 4)
- [x] Attachment list component
- [x] Thumbnail grid view
- [x] Full-size image modal
- [x] Download functionality
- [x] Delete functionality
- [x] SSE event handling

### Phase 5: Testing & Optimization (Week 5)
- [x] Unit tests for auth system
- [x] Integration tests for upload flow
- [x] Load testing S3 operations
- [x] Security audit
- [x] Performance optimization
- [x] Documentation updates

## 8. Testing Strategy

### Backend Tests
- **Auth Tests**: Registration, login, logout, session validation
- **Upload Tests**: URL generation, file validation, S3 upload
- **Access Control**: Verify board password protection
- **Error Cases**: Invalid tokens, expired sessions, file too large

### Frontend Tests
- **Auth Flow**: Login, register, logout, session refresh
- **Upload Flow**: File selection, validation, progress, confirmation
- **UI Components**: Render tests for dialogs and widgets
- **Integration**: E2E tests for complete upload workflow

### Security Tests
- **Penetration Testing**: Test for auth bypass, SQL injection, XSS
- **File Upload Exploits**: Test malicious file uploads
- **Token Security**: Test token theft, replay attacks
- **Access Control**: Test unauthorized access attempts

## 9. Configuration Example

### Backend `.env`
```env
# Existing configuration...
DATABASE_URL=postgresql://postgres:password@localhost:5432/fluxboard
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
GEMINI_API_KEY=...

# New: Authentication
AUTH_SESSION_EXPIRY_DAYS=7
AUTH_PASSWORD_MIN_LENGTH=8
AUTH_RATE_LIMIT_ATTEMPTS=5
AUTH_RATE_LIMIT_WINDOW_MINUTES=15

# New: S3 Configuration
AWS_REGION=us-east-1
AWS_S3_BUCKET=fluxboard-attachments
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=...
S3_UPLOAD_MAX_SIZE=5242880
S3_ALLOWED_TYPES=image/jpeg,image/png,image/gif,image/webp
S3_UPLOAD_URL_EXPIRY_MINUTES=15
S3_DOWNLOAD_URL_EXPIRY_DAYS=7
```

### Frontend `.env.local`
```env
NEXT_PUBLIC_API_URL=http://localhost:8080/api
NEXT_PUBLIC_WS_URL=ws://localhost:3001
NEXT_PUBLIC_MAX_ATTACHMENT_SIZE=5242880
NEXT_PUBLIC_ALLOWED_IMAGE_TYPES=image/jpeg,image/png,image/gif,image/webp
```

## 10. Migration Strategy

### Backward Compatibility
- Authentication is **optional** initially - anonymous users can still use boards
- Add "Login to upload attachments" prompt in UI
- Existing functionality remains unchanged
- No breaking changes to existing API endpoints

### Gradual Rollout
1. **Phase 1**: Deploy auth system, allow registration
2. **Phase 2**: Deploy attachment system with "beta" flag
3. **Phase 3**: Enable for all users after testing
4. **Phase 4**: Optional - require auth for all board modifications

### Database Migrations
- Run migrations automatically on backend startup (existing pattern)
- Ensure migrations are idempotent
- Test migration rollback procedures

## 11. Cost Estimation

### AWS S3 Costs (assuming 1000 active users)
- **Storage**: $0.023/GB/month
  - Average 10 attachments per user @ 2MB = 20GB
  - Monthly cost: ~$0.46
- **PUT Requests**: $0.005 per 1,000 requests
  - 10,000 uploads/month = $0.05
- **GET Requests**: $0.0004 per 1,000 requests
  - 50,000 downloads/month = $0.02
- **Data Transfer**: $0.09/GB (first 10TB)
  - 100GB/month = $9.00

**Total Monthly Cost**: ~$10/month for 1,000 active users

### Database Storage
- Users table: ~500 bytes/user = 0.5MB for 1,000 users
- Sessions table: ~300 bytes/session = 0.3MB for 1,000 sessions
- Attachments table: ~400 bytes/attachment = 4MB for 10,000 attachments

**Total DB Impact**: Negligible (~5MB additional storage)

## 12. Future Enhancements

### Short-term
- **Image Compression**: Auto-compress images on upload
- **Thumbnail Generation**: Generate 200x200 thumbnails server-side
- **Multiple File Upload**: Upload multiple files simultaneously
- **Attachment Comments**: Allow users to comment on attachments

### Medium-term
- **Video Support**: Extend to video attachments (larger size limits)
- **PDF Support**: Allow PDF attachments
- **Attachment Search**: Full-text search across attachment metadata
- **User Profiles**: Avatar uploads, bio, etc.

### Long-term
- **CDN Integration**: CloudFront or CloudFlare for faster downloads
- **Image Editing**: Basic cropping/resizing in-browser
- **Collaborative Annotations**: Allow users to annotate images
- **Version History**: Track attachment versions/revisions

## 13. Documentation Requirements

### Developer Documentation
- API endpoint documentation (OpenAPI/Swagger)
- Database schema documentation
- S3 bucket setup guide
- Deployment guide with AWS credentials

### User Documentation
- User registration guide
- Upload attachment tutorial
- Security best practices
- Privacy policy updates

---

## Summary

This design provides a comprehensive, secure, and scalable approach to adding S3-based image attachments to Fluxboard. The phased implementation allows for iterative development and testing, while the authentication system prevents abuse and enables future user-centric features.

**Key Benefits**:
- ✅ Secure authentication with industry-standard practices
- ✅ Scalable S3 storage with cost-effective pricing
- ✅ Real-time updates via existing SSE infrastructure
- ✅ Minimal impact on existing codebase
- ✅ Clear migration path and backward compatibility