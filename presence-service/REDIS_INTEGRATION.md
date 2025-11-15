# Redis Integration for Multi-Instance WebSocket Presence

This document describes the Redis pub/sub integration that enables multiple presence-service instances to share presence information across a distributed deployment.

## Overview

Phase 2 adds Redis pub/sub to the presence service, allowing multiple instances to coordinate presence updates. This enables horizontal scaling while maintaining consistent presence information across all instances.

## Architecture

### Components

1. **RedisClient** (`src/redis/client.rs`)
   - Manages Redis connection with automatic pooling
   - Provides health check functionality
   - Handles connection failures gracefully

2. **RedisPubSub** (`src/redis/pubsub.rs`)
   - Implements pub/sub messaging layer
   - Encodes/decodes binary protocol messages
   - Provides message deduplication via instance IDs

3. **ConnectionManager** (`src/connection/manager.rs`)
   - Enhanced with Redis pub/sub support
   - Publishes presence events to Redis
   - Subscribes to Redis channels for cross-instance updates
   - Includes automatic reconnection logic

### Message Flow

```
Instance A                    Redis                    Instance B
    |                           |                           |
    | Publish UserJoined        |                           |
    |-------------------------->|                           |
    |                           |  Broadcast UserJoined    |
    |                           |------------------------->|
    |                           |                           |
    | Broadcast to local WS     |                           | Broadcast to local WS
    | clients                   |                           | clients
```

### Deduplication

Each service instance has a unique UUID. Messages include the instance ID to prevent echo:

```rust
struct RedisMessage {
    instance_id: String,  // UUID of sender
    payload: Vec<u8>,     // Encoded BinaryMessage
}
```

When receiving a message, instances check if `instance_id` matches their own and skip if it does.

## Redis Channels

- `presence:board:{board_id}` - Board-specific presence updates
- `presence:global` - Global announcements (currently subscribed by all instances)

## Configuration

### Environment Variables

```bash
REDIS_URL=redis://localhost:6379
WS_PORT=3001
LOG_LEVEL=info
```

### Example `.env` file

```env
REDIS_URL=redis://localhost:6379
WS_PORT=3001
LOG_LEVEL=debug
```

## Error Handling & Resilience

### Startup Behavior

- If Redis is unavailable at startup → service exits with error (required dependency)
- Connection failures are logged and returned as errors

### Runtime Behavior

- **Redis connection drops**: Automatic reconnection with exponential backoff
- **Publish fails**: Log warning but continue (graceful degradation - local broadcasting still works)
- **Subscribe fails**: Automatic resubscription with retry logic

### Reconnection Logic

The `subscribe_with_retry` method implements automatic reconnection:

```rust
async fn subscribe_with_retry(&self, channels: Vec<String>) {
    loop {
        match self.redis_pubsub.subscribe(channels.clone()).await {
            Ok(stream) => {
                // Process messages...
            }
            Err(e) => {
                error!("Subscribe failed: {}, retrying in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
```

## Testing

### Unit Tests

Run unit tests (some require Redis):

```bash
cd presence-service
cargo test
```

Ignored tests (require running Redis instance):

```bash
cargo test -- --ignored
```

### Manual Multi-Instance Testing

1. **Start Redis**:
   ```bash
   docker run -d -p 6379:6379 redis:alpine
   ```

2. **Start multiple service instances**:
   
   Terminal 1:
   ```bash
   cd presence-service
   WS_PORT=3001 cargo run
   ```
   
   Terminal 2:
   ```bash
   cd presence-service
   WS_PORT=3002 cargo run
   ```

3. **Connect WebSocket clients**:
   
   Client A → Instance 1 (port 3001)
   ```javascript
   const ws1 = new WebSocket('ws://localhost:3001');
   ```
   
   Client B → Instance 2 (port 3002)
   ```javascript
   const ws2 = new WebSocket('ws://localhost:3002');
   ```

4. **Test cross-instance presence**:
   - Client A joins board 1
   - Client B should receive `UserJoined` notification
   - Client A moves cursor
   - Client B should receive `CursorBroadcast` updates

### Verification Checklist

- ✅ Both instances connect to Redis successfully
- ✅ Messages published to Redis channels
- ✅ Subscription receives messages from other instances
- ✅ Message deduplication works (no echo)
- ✅ Local clients receive updates from remote instances
- ✅ Cursor movements synchronized across instances
- ✅ User join/leave events synchronized
- ✅ Automatic reconnection on Redis failure

### Debugging

Enable debug logging:

```bash
RUST_LOG=debug cargo run
```

Monitor Redis pub/sub:

```bash
redis-cli
> PSUBSCRIBE presence:*
```

## Performance Considerations

### Message Size

Redis messages are encoded as JSON with binary payloads:

- Heartbeat: ~50 bytes (JSON overhead)
- Cursor update: ~60-70 bytes
- User joined: ~80-120 bytes (depending on username length)

### Network Overhead

For each presence event:
1. Local WebSocket broadcast (existing)
2. Redis publish (new, ~1ms latency)
3. Redis subscription delivery to other instances

### Optimization Opportunities

1. **Selective channel subscription**: Currently all instances subscribe to global channel. Could optimize to only subscribe to active board channels.

2. **Batching**: Could batch cursor updates to reduce Redis traffic.

3. **Compression**: For very high-traffic scenarios, could compress payloads.

## Production Deployment

### Redis High Availability

For production, use Redis Sentinel or Redis Cluster:

```env
REDIS_URL=redis://sentinel-host:26379/mymaster
```

### Monitoring

Monitor these metrics:
- Redis connection status
- Pub/sub latency
- Message publish/subscribe rates
- Failed publish attempts

### Load Balancing

Deploy multiple instances behind a load balancer:

```
                     ┌──────────────┐
                     │ Load Balancer│
                     └──────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
         ┌────▼───┐    ┌────▼───┐   ┌────▼───┐
         │Instance│    │Instance│   │Instance│
         │   A    │    │   B    │   │   C    │
         └────┬───┘    └────┬───┘   └────┬───┘
              │             │             │
              └─────────────┼─────────────┘
                            │
                     ┌──────▼───────┐
                     │    Redis     │
                     └──────────────┘
```

WebSocket clients connect to any instance via load balancer, and all instances share presence state via Redis.

## Next Steps

1. **Dynamic channel subscription**: Subscribe/unsubscribe from board channels as boards become active/inactive
2. **Metrics**: Add Prometheus metrics for Redis operations
3. **Health checks**: Expose health endpoint that includes Redis connectivity
4. **Rate limiting**: Add rate limiting for cursor updates to reduce Redis traffic
5. **Compression**: Implement message compression for high-traffic scenarios

## Troubleshooting

### Redis connection fails

```
Error: Redis connection error: Connection refused
```

**Solution**: Ensure Redis is running and accessible:
```bash
docker run -d -p 6379:6379 redis:alpine
```

### Messages not synchronized across instances

1. Check Redis connectivity on both instances
2. Verify both instances use same `REDIS_URL`
3. Check instance IDs are different (logged at startup)
4. Monitor Redis channels: `redis-cli PSUBSCRIBE presence:*`

### High Redis latency

1. Check Redis server load
2. Consider using Redis Cluster for distribution
3. Enable message batching
4. Review cursor update frequency