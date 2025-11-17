# Fluxboard

A real-time collaborative Kanban board application with live presence and updates. Built with a modern microservices architecture optimized for performance and scalability.

## Features

- **Real-time Collaboration**: Live updates via Server-Sent Events (SSE) and WebSocket presence
- **Live Cursors**: See collaborators' cursors moving in real-time
- **Drag & Drop**: Intuitive card and column management with @dnd-kit
- **Board Sharing**: Share boards via unique tokens with optional password protection
- **Label System**: Organize cards with board-level labels and many-to-many relationships
- **AI-Powered**: Generate card descriptions using Google Gemini API
- **Optimistic Updates**: Instant UI feedback with automatic error recovery
- **Binary Protocol**: Ultra-efficient WebSocket presence (96% size reduction vs JSON)
- **Multi-Instance Support**: Redis pub/sub for horizontal scalability

## Tech Stack

### Backend (Rust)
- **Framework**: Actix-Web 4.9
- **Database**: PostgreSQL with SQLx (compile-time query validation)
- **Real-time**: Server-Sent Events (SSE) for board updates
- **AI**: Google Gemini API integration
- **Runtime**: Tokio async runtime

### Frontend (Next.js)
- **Framework**: Next.js 14.2 with App Router
- **Language**: TypeScript 5.3
- **State Management**: Zustand 4.4
- **UI Components**: Radix UI + Tailwind CSS
- **Drag & Drop**: @dnd-kit
- **HTTP Client**: Axios

### Presence Service (Rust)
- **Protocol**: Binary WebSocket (custom 5-6 byte messages)
- **Pub/Sub**: Redis for multi-instance coordination
- **WebSocket**: tokio-tungstenite
- **Runtime**: Tokio async runtime

## Architecture

Fluxboard uses a microservices architecture with three distinct services:

```
┌──────────────┐
│   Frontend   │ (Next.js - Port 3000)
│              │
│  ┌────────┐  │
│  │ Zustand│  │
│  │ Store  │  │
│  └────────┘  │
└──────┬───┬───┘
       │   │
       │   └─────────────┐
       │                 │
   REST/SSE          WebSocket
       │                 │
       │                 │
┌──────▼──────┐   ┌──────▼────────┐
│   Backend   │   │   Presence    │
│  (Port 8080)│   │  (Port 3001)  │
│             │   │               │
│  ┌────────┐ │   │  ┌─────────┐  │
│  │ SSE    │ │   │  │  Redis  │  │
│  │ Manager│ │   │  │ Pub/Sub │  │
│  └────────┘ │   │  └─────────┘  │
└──────┬──────┘   └───────────────┘
       │
  PostgreSQL
```

- **Backend**: Handles persistent data (boards, cards, columns) via PostgreSQL and broadcasts updates via SSE
- **Presence Service**: Manages ephemeral presence data (live cursors, user tracking) via binary WebSocket
- **Frontend**: React application with optimistic updates and real-time synchronization

## Getting Started

### Prerequisites

- **Rust**: 1.70+ (edition 2024 for backend, 2021 for presence)
- **Node.js**: 18+ with pnpm
- **PostgreSQL**: 16+
- **Redis**: 7+
- **Docker**: Optional, for containerized deployment

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/fluxboard.git
cd fluxboard
```

2. **Setup Backend**
```bash
cd backend
cp .env.example .env
# Edit .env and configure:
# - DATABASE_URL=postgresql://user:password@localhost/fluxboard
# - GEMINI_API_KEY=your_gemini_api_key
cargo build --release
```

3. **Setup Database**
```bash
# Create PostgreSQL database
createdb fluxboard

# Migrations run automatically on backend startup
```

4. **Setup Presence Service**
```bash
cd presence-service
cp .env.example .env
# Edit .env and configure:
# - REDIS_URL=redis://localhost:6379
cargo build --release
```

5. **Setup Frontend**
```bash
cd frontend
pnpm install
```

### Running the Application

**Using Docker (Recommended for PostgreSQL + Redis)**
```bash
# Start PostgreSQL
cd backend
docker-compose up -d

# Start Redis
docker run -d -p 6379:6379 redis:latest
```

**Start All Services**

Terminal 1 - Backend:
```bash
cd backend
cargo run
# Server starts on http://localhost:8080
```

Terminal 2 - Presence Service:
```bash
cd presence-service
cargo run
# WebSocket server starts on ws://localhost:3001
```

Terminal 3 - Frontend:
```bash
cd frontend
pnpm dev
# Application available at http://localhost:3000
```

## Development

### Backend Development
```bash
cd backend
cargo run                    # Development mode
cargo build --release        # Production build
cargo test                   # Run tests
cargo clippy                 # Linting
```

### Frontend Development
```bash
cd frontend
pnpm dev                    # Development server
pnpm build                  # Production build
pnpm start                  # Production server
pnpm lint                   # ESLint
```

### Presence Service Development
```bash
cd presence-service
cargo run                   # Development mode
cargo test                  # Run tests
cargo bench                 # Run benchmarks
```

## Project Structure

```
fluxboard/
├── backend/                 # Rust backend service
│   ├── src/
│   │   ├── handlers/       # HTTP route handlers
│   │   ├── services/       # Business logic layer
│   │   ├── models/         # Database models & events
│   │   ├── sse/            # SSE connection manager
│   │   └── main.rs         # Entry point
│   ├── migrations/         # SQLx database migrations
│   └── Cargo.toml
│
├── frontend/               # Next.js frontend
│   ├── src/
│   │   ├── app/           # Next.js app router pages
│   │   ├── components/    # React components
│   │   ├── store/         # Zustand state stores
│   │   ├── hooks/         # Custom React hooks
│   │   └── lib/           # Utilities (API, SSE, WebSocket)
│   └── package.json
│
├── presence-service/       # Rust WebSocket presence service
│   ├── src/
│   │   ├── connection/    # Connection & room management
│   │   ├── protocol/      # Binary protocol definitions
│   │   ├── redis/         # Redis pub/sub integration
│   │   └── main.rs        # Entry point
│   └── Cargo.toml
│
└── docs/                  # Design documentation
    ├── WEBSOCKET_PRESENCE_DESIGN.md
    ├── SSE_MIGRATION_DESIGN.md
    └── BOARD_LABELS_DESIGN.md
```

## API Overview

### REST Endpoints

**Boards**
- `POST /api/boards` - Create new board
- `GET /api/boards/:shareToken` - Get board by share token
- `PUT /api/boards/:shareToken` - Update board
- `DELETE /api/boards/:shareToken` - Delete board
- `GET /api/boards` - List user's boards

**Columns**
- `POST /api/boards/:shareToken/columns` - Create column
- `PUT /api/columns/:id` - Update column
- `DELETE /api/columns/:id` - Delete column
- `POST /api/columns/reorder` - Reorder columns

**Cards**
- `POST /api/columns/:columnId/cards` - Create card
- `PUT /api/cards/:id` - Update card
- `DELETE /api/cards/:id` - Delete card
- `POST /api/cards/move` - Move card between columns
- `POST /api/cards/reorder` - Reorder cards
- `POST /api/cards/ai/generate-description` - Generate AI description

**Labels**
- `POST /api/boards/:shareToken/labels` - Create board label
- `PUT /api/labels/:id` - Update label
- `DELETE /api/labels/:id` - Delete label
- `POST /api/cards/:cardId/labels/:labelId` - Assign label to card
- `DELETE /api/cards/:cardId/labels/:labelId` - Unassign label from card

### Real-time Events

**SSE Events** (14 event types)
- Board: `board:updated`
- Column: `column:created`, `column:updated`, `column:deleted`, `column:reordered`
- Card: `card:created`, `card:updated`, `card:deleted`, `card:moved`, `card:reordered`
- Label: `label:created`, `label:updated`, `label:deleted`, `label:assigned`, `label:unassigned`

**WebSocket Messages** (Binary Protocol)
- `CursorUpdate` - 7 bytes: Cursor position updates (60fps capable)
- `Join` - 4-36 bytes: User joins board
- `Leave` - 3 bytes: User leaves board
- `Heartbeat` - 2 bytes: Keep-alive ping
- `Pong` - 2 bytes: Heartbeat response

## Environment Variables

### Backend (.env)
```env
DATABASE_URL=postgresql://user:password@localhost/fluxboard
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
CORS_ORIGIN=http://localhost:3000
GEMINI_API_KEY=your_gemini_api_key
RUST_LOG=info
```

### Presence Service (.env)
```env
REDIS_URL=redis://localhost:6379
WS_PORT=3001
LOG_LEVEL=info
```

### Frontend (.env.local)
```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:3001
```

## Key Features Deep Dive

### Binary WebSocket Protocol
The presence service uses a custom binary protocol for ultra-efficient communication:
- Coordinates normalized to u16 (0-65535) for 96% size reduction vs JSON
- Capable of handling 60fps cursor updates
- Messages as small as 2 bytes (heartbeat) to 7 bytes (cursor update)

### SSE Connection Manager
In-memory connection tracking with:
- Per-board subscriber management
- 100-event buffer per client
- Automatic cleanup of disconnected clients
- Broadcast to all board subscribers

### State Management
Frontend uses Zustand with:
- Immutable updates for predictable state changes
- Client-side event deduplication
- Optimistic updates with automatic rollback
- Centralized board state in `board-store.ts`

### Database Schema
PostgreSQL schema with:
- Cascade deletes for data integrity
- Share tokens (8-char alphanumeric) for board access
- Password-based board locking
- Position-based ordering for drag & drop
- Many-to-many card-label relationships

## Performance

- **Binary Protocol**: 96% message size reduction compared to JSON
- **Connection Pooling**: SQLx connection pooling for database efficiency
- **Indexed Queries**: Strategic indexes on share_token, board_id, positions
- **Coordinate Normalization**: u16 instead of f32 for network efficiency
- **Client Deduplication**: Prevent duplicate SSE event processing
- **Exponential Backoff**: SSE/WebSocket reconnection with max 5 attempts

## Security Considerations

- Board passwords currently stored in plain text (should be hashed for production)
- Share token uniqueness enforced by database unique constraint
- CORS configuration allows localhost:3000 by default
- Custom `X-Board-Password` header for password-protected operations

## Documentation

For detailed design documentation, see:
- `CLAUDE.md` - Development guide for AI assistants
- `WEBSOCKET_PRESENCE_DESIGN.md` - Presence service architecture
- `SSE_MIGRATION_DESIGN.md` - SSE implementation design
- `BOARD_LABELS_DESIGN.md` - Label system architecture
- `FEATURE_ROADMAP.md` - Planned features
- `AI_FEATURES_GUIDE.md` - AI integration guide

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- Rust code passes `cargo clippy` and `cargo test`
- Frontend code passes `pnpm lint`
- Database migrations are included for schema changes
- SSE events are documented for new features

## License

MIT

## Acknowledgments

- Built with Rust, Next.js, and PostgreSQL
- Powered by Actix-Web and Tokio for async runtime
- UI components from Radix UI
- Drag & drop by @dnd-kit
- AI features by Google Gemini

---

**Note**: This project is used internally for task coordination. Feel free to use it as a reference or starting point for your own collaborative board application.
