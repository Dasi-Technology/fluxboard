# Fluxboard - Trello Clone Implementation Plan

## Project Overview

A real-time collaborative Kanban board application with instant synchronization across all connected users. The application features anonymous access via shareable links, eliminating the need for authentication.

### Tech Stack
- **Frontend**: Next.js 14+, React 18+, TypeScript, shadcn/ui, TailwindCSS
- **Backend**: Rust (Actix-web), WebSocket support
- **Database**: PostgreSQL (self-hosted)
- **Deployment**: Docker containers
- **Real-time**: WebSocket for bidirectional communication

### Core Features (v1)
- Kanban board with customizable column headers
- Real-time updates via WebSocket
- Anonymous access with shareable board links
- Cards with title, description, labels, and colors
- Drag-and-drop for cards (between columns and reordering)
- Drag-and-drop for columns (reordering)
- Dynamic column creation, deletion, and renaming
- Indefinite board persistence

---

## System Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph Client
        A[Next.js Frontend]
        B[React Components]
        C[WebSocket Client]
        D[State Management]
    end
    
    subgraph Backend
        E[Rust API Server]
        F[WebSocket Handler]
        G[Business Logic]
    end
    
    subgraph Database
        H[PostgreSQL]
    end
    
    A --> B
    B --> C
    B --> D
    C <-->|WSS| F
    D -->|REST API| E
    E --> G
    F --> G
    G --> H
```

### Communication Flow

```mermaid
sequenceDiagram
    participant U1 as User 1
    participant F1 as Frontend 1
    participant WS as WebSocket Server
    participant API as REST API
    participant DB as PostgreSQL
    participant F2 as Frontend 2
    participant U2 as User 2
    
    U1->>F1: Drag card to new column
    F1->>F1: Optimistic UI update
    F1->>WS: Send card_moved event
    WS->>API: Process update
    API->>DB: Update card position
    DB-->>API: Confirm
    API-->>WS: Broadcast to all clients
    WS-->>F2: card_moved event
    F2->>F2: Update UI
    F2->>U2: Display change
```

---

## Database Schema

### Tables

#### `boards`
```sql
CREATE TABLE boards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    share_token VARCHAR(64) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL DEFAULT 'Untitled Board',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_boards_share_token ON boards(share_token);
```

#### `columns`
```sql
CREATE TABLE columns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(board_id, position)
);

CREATE INDEX idx_columns_board_id ON columns(board_id);
CREATE INDEX idx_columns_position ON columns(board_id, position);
```

#### `cards`
```sql
CREATE TABLE cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    column_id UUID NOT NULL REFERENCES columns(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    position INTEGER NOT NULL,
    color VARCHAR(7), -- Hex color code
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(column_id, position)
);

CREATE INDEX idx_cards_column_id ON cards(column_id);
CREATE INDEX idx_cards_position ON cards(column_id, position);
```

#### `labels`
```sql
CREATE TABLE labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7) NOT NULL, -- Hex color code
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_labels_board_id ON labels(board_id);
```

#### `card_labels`
```sql
CREATE TABLE card_labels (
    card_id UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (card_id, label_id)
);

CREATE INDEX idx_card_labels_card_id ON card_labels(card_id);
CREATE INDEX idx_card_labels_label_id ON card_labels(label_id);
```

### Database Migrations Strategy
- Use `sqlx` for compile-time checked queries
- Migration files in `backend/migrations/`
- Apply migrations on application startup

---

## API Design

### REST Endpoints

#### Board Management
```
POST   /api/boards                    # Create new board
GET    /api/boards/:shareToken        # Get board by share token
PUT    /api/boards/:boardId           # Update board title
DELETE /api/boards/:boardId           # Delete board (optional)
```

#### Column Management
```
GET    /api/boards/:boardId/columns              # Get all columns
POST   /api/boards/:boardId/columns              # Create column
PUT    /api/columns/:columnId                    # Update column
DELETE /api/columns/:columnId                    # Delete column
PUT    /api/boards/:boardId/columns/reorder      # Reorder columns
```

#### Card Management
```
GET    /api/columns/:columnId/cards              # Get all cards in column
POST   /api/columns/:columnId/cards              # Create card
PUT    /api/cards/:cardId                        # Update card
DELETE /api/cards/:cardId                        # Delete card
PUT    /api/cards/:cardId/move                   # Move card to different column
PUT    /api/columns/:columnId/cards/reorder      # Reorder cards within column
```

#### Label Management
```
GET    /api/boards/:boardId/labels               # Get all labels
POST   /api/boards/:boardId/labels               # Create label
PUT    /api/labels/:labelId                      # Update label
DELETE /api/labels/:labelId                      # Delete label
POST   /api/cards/:cardId/labels/:labelId        # Add label to card
DELETE /api/cards/:cardId/labels/:labelId        # Remove label from card
```

### WebSocket Events

#### Client → Server Events
```typescript
{
  type: "subscribe",
  boardId: string
}

{
  type: "card_created",
  columnId: string,
  card: Card
}

{
  type: "card_updated",
  cardId: string,
  updates: Partial<Card>
}

{
  type: "card_moved",
  cardId: string,
  targetColumnId: string,
  targetPosition: number
}

{
  type: "card_deleted",
  cardId: string
}

{
  type: "column_created",
  boardId: string,
  column: Column
}

{
  type: "column_updated",
  columnId: string,
  updates: Partial<Column>
}

{
  type: "column_deleted",
  columnId: string
}

{
  type: "column_moved",
  columnId: string,
  targetPosition: number
}

{
  type: "label_added",
  cardId: string,
  labelId: string
}

{
  type: "label_removed",
  cardId: string,
  labelId: string
}
```

#### Server → Client Events
```typescript
{
  type: "board_state",
  board: Board,
  columns: Column[],
  cards: Card[],
  labels: Label[]
}

{
  type: "card_created",
  card: Card
}

{
  type: "card_updated",
  cardId: string,
  updates: Partial<Card>
}

{
  type: "card_moved",
  cardId: string,
  sourceColumnId: string,
  targetColumnId: string,
  targetPosition: number
}

{
  type: "card_deleted",
  cardId: string
}

{
  type: "column_created",
  column: Column
}

{
  type: "column_updated",
  columnId: string,
  updates: Partial<Column>
}

{
  type: "column_deleted",
  columnId: string
}

{
  type: "column_moved",
  columnId: string,
  targetPosition: number
}

{
  type: "label_added",
  cardId: string,
  label: Label
}

{
  type: "label_removed",
  cardId: string,
  labelId: string
}

{
  type: "error",
  message: string
}
```

---

## Backend Implementation (Rust)

### Project Structure
```
backend/
├── Cargo.toml
├── Dockerfile
├── .env.example
├── migrations/
│   ├── 001_create_boards.sql
│   ├── 002_create_columns.sql
│   ├── 003_create_cards.sql
│   ├── 004_create_labels.sql
│   └── 005_create_card_labels.sql
├── src/
│   ├── main.rs
│   ├── config.rs                 # Configuration management
│   ├── db.rs                     # Database connection pool
│   ├── error.rs                  # Error handling
│   ├── models/
│   │   ├── mod.rs
│   │   ├── board.rs
│   │   ├── column.rs
│   │   ├── card.rs
│   │   └── label.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── board.rs              # Board CRUD handlers
│   │   ├── column.rs             # Column CRUD handlers
│   │   ├── card.rs               # Card CRUD handlers
│   │   └── label.rs              # Label CRUD handlers
│   ├── websocket/
│   │   ├── mod.rs
│   │   ├── server.rs             # WebSocket server
│   │   ├── session.rs            # Client session
│   │   ├── messages.rs           # Message types
│   │   └── room.rs               # Room/board management
│   ├── services/
│   │   ├── mod.rs
│   │   ├── board_service.rs
│   │   ├── column_service.rs
│   │   ├── card_service.rs
│   │   └── label_service.rs
│   └── utils/
│       ├── mod.rs
│       └── token_generator.rs    # Share token generation
```

### Key Dependencies (Cargo.toml)
```toml
[dependencies]
actix-web = "4.4"
actix-ws = "0.2"
actix-rt = "2.9"
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"
env_logger = "0.11"
log = "0.4"
rand = "0.8"
```

### Implementation Steps

#### Step 1: Project Setup
1. Initialize Rust project with `cargo init --name fluxboard-backend`
2. Add dependencies to `Cargo.toml`
3. Create `.env.example` with database configuration
4. Set up database connection pool using `sqlx`
5. Create migration files for all tables

#### Step 2: Models & Database Layer
1. Define structs for `Board`, `Column`, `Card`, `Label` in `models/`
2. Implement `From<sqlx::Row>` traits for database mapping
3. Add validation logic (title length, color format, etc.)
4. Create database query functions using `sqlx::query!` macro

#### Step 3: Services Layer
1. Implement `BoardService` for board CRUD operations
2. Implement `ColumnService` with position management
3. Implement `CardService` with move and reorder logic
4. Implement `LabelService` for label management
5. Add transaction support for multi-table operations

#### Step 4: REST API Handlers
1. Create handler functions for each endpoint
2. Implement request validation and error handling
3. Add CORS configuration for frontend
4. Set up JSON serialization/deserialization
5. Add logging middleware

#### Step 5: WebSocket Implementation
1. Create `WebSocketServer` actor to manage connections
2. Implement `Room` actor for each board (manages subscribers)
3. Create `Session` actor for each client connection
4. Implement message routing and broadcasting
5. Add connection/disconnection handling
6. Implement heartbeat mechanism to detect disconnects

#### Step 6: Share Token Generation
1. Create secure random token generator (64 characters)
2. Ensure uniqueness by checking database
3. Add retry logic if collision occurs

#### Step 7: Testing
1. Write unit tests for services
2. Write integration tests for API endpoints
3. Test WebSocket message flow
4. Test concurrent operations and race conditions

---

## Frontend Implementation (Next.js + React)

### Project Structure
```
frontend/
├── package.json
├── tsconfig.json
├── next.config.js
├── tailwind.config.ts
├── Dockerfile
├── .env.example
├── public/
│   └── favicon.ico
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── page.tsx                    # Landing page (create board)
│   │   ├── board/
│   │   │   └── [shareToken]/
│   │   │       └── page.tsx            # Board view
│   │   └── globals.css
│   ├── components/
│   │   ├── ui/                         # shadcn/ui components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── input.tsx
│   │   │   ├── label.tsx
│   │   │   ├── popover.tsx
│   │   │   └── ...
│   │   ├── board/
│   │   │   ├── Board.tsx               # Main board container
│   │   │   ├── Column.tsx              # Column component
│   │   │   ├── Card.tsx                # Card component
│   │   │   ├── AddColumn.tsx           # Add column button
│   │   │   └── AddCard.tsx             # Add card button
│   │   ├── dialogs/
│   │   │   ├── CreateBoardDialog.tsx
│   │   │   ├── EditCardDialog.tsx
│   │   │   ├── EditColumnDialog.tsx
│   │   │   └── ManageLabelsDialog.tsx
│   │   └── shared/
│   │       ├── ShareLink.tsx           # Shareable link component
│   │       ├── ColorPicker.tsx
│   │       └── LabelBadge.tsx
│   ├── lib/
│   │   ├── api.ts                      # REST API client
│   │   ├── websocket.ts                # WebSocket client
│   │   ├── utils.ts                    # Utility functions
│   │   └── types.ts                    # TypeScript types
│   ├── hooks/
│   │   ├── useBoard.ts                 # Board state management
│   │   ├── useWebSocket.ts             # WebSocket hook
│   │   ├── useDragAndDrop.ts           # DnD logic
│   │   └── useOptimistic.ts            # Optimistic updates
│   └── store/
│       ├── boardStore.ts               # Zustand store for board state
│       └── uiStore.ts                  # UI state (dialogs, etc.)
```

### Key Dependencies (package.json)
```json
{
  "dependencies": {
    "next": "^14.0.4",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "typescript": "^5.3.3",
    "@dnd-kit/core": "^6.1.0",
    "@dnd-kit/sortable": "^8.0.0",
    "@dnd-kit/utilities": "^3.2.2",
    "zustand": "^4.4.7",
    "axios": "^1.6.2",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.0.0",
    "tailwind-merge": "^2.1.0",
    "lucide-react": "^0.295.0",
    "@radix-ui/react-dialog": "^1.0.5",
    "@radix-ui/react-popover": "^1.0.7",
    "@radix-ui/react-label": "^2.0.2",
    "@radix-ui/react-slot": "^1.0.2"
  },
  "devDependencies": {
    "@types/node": "^20.10.5",
    "@types/react": "^18.2.45",
    "@types/react-dom": "^18.2.18",
    "autoprefixer": "^10.4.16",
    "postcss": "^8.4.32",
    "tailwindcss": "^3.3.6"
  }
}
```

### TypeScript Types
```typescript
// lib/types.ts
export interface Board {
  id: string;
  shareToken: string;
  title: string;
  createdAt: string;
  updatedAt: string;
}

export interface Column {
  id: string;
  boardId: string;
  title: string;
  position: number;
  createdAt: string;
  updatedAt: string;
}

export interface Card {
  id: string;
  columnId: string;
  title: string;
  description: string | null;
  position: number;
  color: string | null;
  createdAt: string;
  updatedAt: string;
  labels: Label[];
}

export interface Label {
  id: string;
  boardId: string;
  name: string;
  color: string;
  createdAt: string;
}

export interface BoardState {
  board: Board | null;
  columns: Column[];
  cards: Card[];
  labels: Label[];
}

export type WebSocketEvent =
  | { type: 'card_created'; card: Card }
  | { type: 'card_updated'; cardId: string; updates: Partial<Card> }
  | { type: 'card_moved'; cardId: string; sourceColumnId: string; targetColumnId: string; targetPosition: number }
  | { type: 'card_deleted'; cardId: string }
  | { type: 'column_created'; column: Column }
  | { type: 'column_updated'; columnId: string; updates: Partial<Column> }
  | { type: 'column_deleted'; columnId: string }
  | { type: 'column_moved'; columnId: string; targetPosition: number }
  | { type: 'label_added'; cardId: string; label: Label }
  | { type: 'label_removed'; cardId: string; labelId: string }
  | { type: 'board_state'; board: Board; columns: Column[]; cards: Card[]; labels: Label[] }
  | { type: 'error'; message: string };
```

### Implementation Steps

#### Step 1: Project Setup
1. Create Next.js app with TypeScript: `npx create-next-app@latest`
2. Install dependencies (dnd-kit, zustand, axios, shadcn/ui)
3. Initialize shadcn/ui: `npx shadcn-ui@latest init`
4. Configure Tailwind CSS
5. Set up environment variables

#### Step 2: UI Components (shadcn/ui)
1. Install required shadcn components:
   - `npx shadcn-ui@latest add button`
   - `npx shadcn-ui@latest add card`
   - `npx shadcn-ui@latest add dialog`
   - `npx shadcn-ui@latest add input`
   - `npx shadcn-ui@latest add label`
   - `npx shadcn-ui@latest add popover`
2. Create custom `ColorPicker` component
3. Create `LabelBadge` component

#### Step 3: State Management
1. Set up Zustand store for board state
2. Create actions for CRUD operations
3. Implement optimistic update logic
4. Add error rollback mechanism

#### Step 4: API Client
1. Create axios instance with base URL configuration
2. Implement API functions for all endpoints
3. Add error handling and retry logic
4. Create type-safe request/response interfaces

#### Step 5: WebSocket Integration
1. Create WebSocket client class
2. Implement connection management (connect, disconnect, reconnect)
3. Add event listeners and handlers
4. Implement heartbeat/ping mechanism
5. Create `useWebSocket` hook for React integration

#### Step 6: Drag and Drop
1. Set up @dnd-kit providers and contexts
2. Implement draggable card components
3. Implement droppable column components
4. Add drag overlay for visual feedback
5. Handle drop events and update state
6. Implement column reordering
7. Add animations and transitions

#### Step 7: Board Components
1. Create `Board` container component
2. Implement `Column` component with add/edit/delete
3. Implement `Card` component with inline editing
4. Create `AddColumn` button component
5. Create `AddCard` button component
6. Add loading states and skeletons

#### Step 8: Dialogs and Modals
1. Create `CreateBoardDialog` for landing page
2. Implement `EditCardDialog` with full editor
3. Create `EditColumnDialog` for renaming
4. Build `ManageLabelsDialog` for label CRUD
5. Add form validation

#### Step 9: Landing Page
1. Create hero section with "Create Board" CTA
2. Add share link display after creation
3. Implement copy-to-clipboard functionality
4. Add board redirection

#### Step 10: Board Page
1. Fetch board data on page load
2. Establish WebSocket connection
3. Render columns and cards
4. Handle real-time updates
5. Implement error states (board not found)

#### Step 11: Optimistic Updates
1. Implement optimistic card creation
2. Add optimistic card movement
3. Handle optimistic column operations
4. Add rollback on WebSocket error

#### Step 12: Polish and UX
1. Add loading spinners and skeletons
2. Implement toast notifications
3. Add keyboard shortcuts (ESC to close dialogs, etc.)
4. Improve mobile responsiveness
5. Add smooth animations

---

## Real-Time Synchronization Strategy

### Architecture Overview

The real-time system uses WebSocket for bidirectional communication with the following characteristics:

1. **Optimistic Updates**: Client applies changes immediately for perceived instant response
2. **Server Authority**: Server validates and broadcasts canonical state
3. **Conflict Resolution**: Last-write-wins with server timestamp

### Connection Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Disconnected
    Disconnected --> Connecting: User opens board
    Connecting --> Connected: WS handshake success
    Connecting --> Disconnected: Connection failed
    Connected --> Subscribed: Subscribe to board
    Subscribed --> Disconnected: Connection lost
    Disconnected --> Connecting: Auto-reconnect
    Subscribed --> [*]: User leaves board
```

### Event Flow

1. **User Action** → Client updates UI optimistically
2. **Client** → Sends event via WebSocket
3. **Server** → Validates and persists to database
4. **Server** → Broadcasts event to all subscribed clients (including sender)
5. **Clients** → Update UI with authoritative state

### Handling Edge Cases

#### Concurrent Edits
- Use version numbers or timestamps
- Server resolves conflicts with last-write-wins
- Client receives authoritative update and reconciles

#### Connection Loss
- Client buffers pending operations
- On reconnect, request full board state
- Merge buffered operations with server state
- Show reconnection status to user

#### Stale Data
- Server sends full `board_state` on subscription
- Periodic sync every 30 seconds for long sessions
- Client timestamp checks for out-of-order messages

---

## Deployment Strategy

### Docker Setup

#### Backend Dockerfile
```dockerfile
# backend/Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fluxboard-backend /app/
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080
CMD ["/app/fluxboard-backend"]
```

#### Frontend Dockerfile
```dockerfile
# frontend/Dockerfile
FROM node:20-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:20-alpine AS runner

WORKDIR /app
ENV NODE_ENV=production

COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

EXPOSE 3000
CMD ["node", "server.js"]
```

#### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'

services:
  backend:
    build: ./backend
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://fluxboard:${DB_PASSWORD}@db:5432/fluxboard
      - RUST_LOG=info
    depends_on:
      - db
    restart: unless-stopped

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:8080
      - NEXT_PUBLIC_WS_URL=ws://backend:8080/ws
    depends_on:
      - backend
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=fluxboard
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=fluxboard
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    restart: unless-stopped

volumes:
  postgres_data:
```

### Environment Variables

#### Backend (.env)
```
DATABASE_URL=postgresql://fluxboard:your_secure_password@db:5432/fluxboard
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

#### Frontend (.env.local)
```
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
```

### Deployment Steps

1. **Local Development**
   ```bash
   # Backend
   cd backend
   cargo run
   
   # Frontend
   cd frontend
   npm run dev
   ```

2. **Docker Deployment**
   ```bash
   docker-compose up -d
   ```

3. **Database Setup**
   - PostgreSQL will be automatically set up via Docker Compose
   - Set DB_PASSWORD environment variable before running docker-compose
   - Migrations will run automatically on backend startup
   - Data persists in the postgres_data Docker volume

4. **Production Considerations**
   - Set up reverse proxy (Nginx/Caddy) for HTTPS
   - Configure environment-specific URLs
   - Set up monitoring and logging
   - Implement database backups
   - Add rate limiting on API endpoints

---

## Testing Strategy

### Backend Testing
```rust
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_create_board() {
        // Test board creation
    }

    #[actix_rt::test]
    async fn test_move_card() {
        // Test card movement logic
    }
}
```

### Frontend Testing
- Unit tests with Jest/Vitest
- Component tests with React Testing Library
- E2E tests with Playwright (optional for v1)

### Integration Testing
- Test WebSocket message flow
- Verify database transactions
- Test concurrent operations

---

## Security Considerations

1. **Input Validation**
   - Sanitize all user inputs
   - Validate color hex codes
   - Limit string lengths (title, description)
   - Prevent SQL injection via parameterized queries

2. **Rate Limiting**
   - Limit board creation per IP
   - Limit WebSocket message frequency
   - Prevent spam card creation

3. **Share Token Security**
   - Use cryptographically secure random generation
   - 64-character alphanumeric tokens
   - Prevent enumeration attacks

4. **WebSocket Security**
   - Validate board access on connection
   - Sanitize all incoming messages
   - Implement message size limits
   - Add connection throttling

5. **Database Security**
   - Use prepared statements (sqlx handles this)
   - Set strong database passwords
   - Regular backups
   - Connection pooling limits

---

## Performance Optimization

1. **Database**
   - Index frequently queried columns
   - Use connection pooling
   - Batch operations where possible
   - Consider caching for read-heavy operations

2. **WebSocket**
   - Implement message batching
   - Compress large payloads
   - Use binary protocol for efficiency (optional)

3. **Frontend**
   - Virtualize long lists of cards
   - Debounce drag events
   - Lazy load card descriptions
   - Optimize re-renders with React.memo

4. **Network**
   - Enable gzip compression
   - Use CDN for static assets
   - Implement WebSocket reconnection backoff

---

## Future Enhancements (Post-v1)

1. **Features**
   - Card comments and activity log
   - File attachments
   - Due dates and reminders
   - Card assignments (even without auth)
   - Search and filter
   - Archive boards/cards
   - Board templates
   - Keyboard shortcuts

2. **Technical**
   - Offline support with service workers
   - Operational transform for conflict resolution
   - Redis for WebSocket scaling
   - GraphQL API option
   - Mobile apps (React Native)

3. **UX**
   - Dark mode
   - Custom themes
   - Board backgrounds
   - Card covers
   - Emoji support
   - Undo/redo

---

## Implementation Timeline Estimate

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| **Phase 1: Setup** | Project initialization, dependencies, database schema | 2-3 days |
| **Phase 2: Backend Core** | Models, services, REST API | 5-7 days |
| **Phase 3: WebSocket** | WebSocket server, room management, message handling | 4-5 days |
| **Phase 4: Frontend Core** | Components, state management, API client | 5-7 days |
| **Phase 5: Real-time Integration** | WebSocket client, optimistic updates | 3-4 days |
| **Phase 6: Drag & Drop** | DnD implementation, animations | 3-4 days |
| **Phase 7: Polish** | Error handling, loading states, UX improvements | 3-4 days |
| **Phase 8: Testing** | Unit, integration, E2E tests | 4-5 days |
| **Phase 9: Deployment** | Docker setup, documentation, deployment | 2-3 days |
| **Total** | | **31-42 days** |

*Note: Timeline assumes a single full-time developer. Can be parallelized with multiple developers.*

---

## Getting Started

### Prerequisites
- Rust 1.75+
- Node.js 20+
- PostgreSQL 16+ (or use Docker Compose to run it automatically)
- Docker & Docker Compose (for containerized deployment)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd fluxboard
   ```

2. **Set up Environment Variables**
   - Create a `.env` file in the root directory
   - Set `DB_PASSWORD=your_secure_password`
   - This will be used by both the backend and the PostgreSQL container

3. **Run with Docker Compose**
   ```bash
   # Set your database password
   export DB_PASSWORD=your_secure_password
   
   # Start all services
   docker-compose up -d
   
   # View logs
   docker-compose logs -f
   ```

4. **Run Locally (Development)**
   ```bash
   # Start PostgreSQL
   docker-compose up -d db
   
   # Backend
   cd backend
   cp .env.example .env
   # Update DATABASE_URL in .env to: postgresql://fluxboard:your_secure_password@localhost:5432/fluxboard
   cargo run
   
   # Frontend (in another terminal)
   cd frontend
   cp .env.example .env.local
   # Update API URLs in .env.local
   npm install
   npm run dev
   ```

5. **Access Application**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8080
   - PostgreSQL: localhost:5432 (user: fluxboard, db: fluxboard)

---

## Conclusion

This implementation plan provides a comprehensive roadmap for building a real-time Trello clone with Rust and Next.js. The architecture emphasizes:

- **Simplicity**: No authentication complexity
- **Real-time**: WebSocket-based instant updates
- **Scalability**: Modular architecture ready for growth
- **Developer Experience**: Type-safe, modern tooling

Follow the phases sequentially, and you'll have a fully functional collaborative Kanban board application.