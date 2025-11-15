//! Comprehensive benchmark suite for binary protocol message sizes and performance.
//!
//! This suite measures:
//! 1. Message sizes (exact byte counts)
//! 2. Encoding performance
//! 3. Decoding performance
//! 4. Throughput benchmarks
//! 5. Size comparison vs JSON
//! 6. Coordinate normalization performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use presence_service::protocol::{denormalize_coord, normalize_coord, BinaryMessage};
use serde_json::json;

// ============================================================================
// 1. Message Size Benchmarks
// ============================================================================

fn verify_message_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_sizes");

    // Cursor Update - Target: 7 bytes
    let cursor_update = BinaryMessage::CursorUpdate {
        board_id: 1234,
        x: normalize_coord(0.5),
        y: normalize_coord(0.75),
    };
    let cursor_bytes = cursor_update.encode();
    assert_eq!(
        cursor_bytes.len(),
        7,
        "CursorUpdate should be exactly 7 bytes"
    );

    group.bench_function("size_cursor_update", |b| {
        b.iter(|| {
            let bytes = black_box(&cursor_bytes);
            bytes.len()
        });
    });
    println!("✓ CursorUpdate: {} bytes", cursor_bytes.len());

    // Cursor Broadcast - Target: 8 bytes
    let cursor_broadcast = BinaryMessage::CursorBroadcast {
        user_id: 42,
        board_id: 1234,
        x: normalize_coord(0.3),
        y: normalize_coord(0.9),
    };
    let broadcast_bytes = cursor_broadcast.encode();
    assert_eq!(
        broadcast_bytes.len(),
        8,
        "CursorBroadcast should be exactly 8 bytes"
    );

    group.bench_function("size_cursor_broadcast", |b| {
        b.iter(|| {
            let bytes = black_box(&broadcast_bytes);
            bytes.len()
        });
    });
    println!("✓ CursorBroadcast: {} bytes", broadcast_bytes.len());

    // Join - Variable size (4-36 bytes)
    let join_short = BinaryMessage::Join {
        board_id: 1234,
        username: "".to_string(),
    };
    let join_short_bytes = join_short.encode();
    assert_eq!(
        join_short_bytes.len(),
        4,
        "Join with empty username should be 4 bytes"
    );
    println!("✓ Join (empty username): {} bytes", join_short_bytes.len());

    let join_long = BinaryMessage::Join {
        board_id: 1234,
        username: "a".repeat(32),
    };
    let join_long_bytes = join_long.encode();
    assert_eq!(
        join_long_bytes.len(),
        36,
        "Join with 32-char username should be 36 bytes"
    );
    println!("✓ Join (32-char username): {} bytes", join_long_bytes.len());

    // Leave - Target: 3 bytes
    let leave = BinaryMessage::Leave { board_id: 1234 };
    let leave_bytes = leave.encode();
    assert_eq!(leave_bytes.len(), 3, "Leave should be exactly 3 bytes");

    group.bench_function("size_leave", |b| {
        b.iter(|| {
            let bytes = black_box(&leave_bytes);
            bytes.len()
        });
    });
    println!("✓ Leave: {} bytes", leave_bytes.len());

    // User Joined - Variable size (7-40 bytes)
    let user_joined_short = BinaryMessage::UserJoined {
        user_id: 42,
        board_id: 1234,
        username: "".to_string(),
        color: [255, 0, 0],
    };
    let user_joined_short_bytes = user_joined_short.encode();
    assert_eq!(
        user_joined_short_bytes.len(),
        8,
        "UserJoined with empty username should be 8 bytes (includes 3-byte color)"
    );
    println!(
        "✓ UserJoined (empty username): {} bytes",
        user_joined_short_bytes.len()
    );

    let user_joined_long = BinaryMessage::UserJoined {
        user_id: 42,
        board_id: 1234,
        username: "a".repeat(32),
        color: [255, 0, 0],
    };
    let user_joined_long_bytes = user_joined_long.encode();
    assert_eq!(
        user_joined_long_bytes.len(),
        40,
        "UserJoined with 32-char username should be 40 bytes"
    );
    println!(
        "✓ UserJoined (32-char username): {} bytes",
        user_joined_long_bytes.len()
    );

    // User Left - Target: 4 bytes
    let user_left = BinaryMessage::UserLeft {
        user_id: 42,
        board_id: 1234,
    };
    let user_left_bytes = user_left.encode();
    assert_eq!(
        user_left_bytes.len(),
        4,
        "UserLeft should be exactly 4 bytes"
    );

    group.bench_function("size_user_left", |b| {
        b.iter(|| {
            let bytes = black_box(&user_left_bytes);
            bytes.len()
        });
    });
    println!("✓ UserLeft: {} bytes", user_left_bytes.len());

    // Presence Update - Target: 4 bytes
    let presence_update = BinaryMessage::PresenceUpdate {
        board_id: 1234,
        count: 5,
    };
    let presence_bytes = presence_update.encode();
    assert_eq!(
        presence_bytes.len(),
        4,
        "PresenceUpdate should be exactly 4 bytes"
    );

    group.bench_function("size_presence_update", |b| {
        b.iter(|| {
            let bytes = black_box(&presence_bytes);
            bytes.len()
        });
    });
    println!("✓ PresenceUpdate: {} bytes", presence_bytes.len());

    // Heartbeat - Target: 1 byte
    let heartbeat = BinaryMessage::Heartbeat;
    let heartbeat_bytes = heartbeat.encode();
    assert_eq!(
        heartbeat_bytes.len(),
        1,
        "Heartbeat should be exactly 1 byte"
    );

    group.bench_function("size_heartbeat", |b| {
        b.iter(|| {
            let bytes = black_box(&heartbeat_bytes);
            bytes.len()
        });
    });
    println!("✓ Heartbeat: {} bytes", heartbeat_bytes.len());

    group.finish();
}

// ============================================================================
// 2. Encoding Performance Benchmarks
// ============================================================================

fn encode_cursor_update(c: &mut Criterion) {
    c.bench_function("encode_cursor_update", |b| {
        let msg = BinaryMessage::CursorUpdate {
            board_id: 1234,
            x: normalize_coord(0.5),
            y: normalize_coord(0.75),
        };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_cursor_broadcast(c: &mut Criterion) {
    c.bench_function("encode_cursor_broadcast", |b| {
        let msg = BinaryMessage::CursorBroadcast {
            user_id: 42,
            board_id: 1234,
            x: normalize_coord(0.3),
            y: normalize_coord(0.9),
        };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_join(c: &mut Criterion) {
    c.bench_function("encode_join", |b| {
        let msg = BinaryMessage::Join {
            board_id: 1234,
            username: "Alice".to_string(),
        };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_leave(c: &mut Criterion) {
    c.bench_function("encode_leave", |b| {
        let msg = BinaryMessage::Leave { board_id: 1234 };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_user_joined(c: &mut Criterion) {
    c.bench_function("encode_user_joined", |b| {
        let msg = BinaryMessage::UserJoined {
            user_id: 42,
            board_id: 1234,
            username: "Bob".to_string(),
            color: [255, 128, 64],
        };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_user_left(c: &mut Criterion) {
    c.bench_function("encode_user_left", |b| {
        let msg = BinaryMessage::UserLeft {
            user_id: 42,
            board_id: 1234,
        };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_presence_update(c: &mut Criterion) {
    c.bench_function("encode_presence_update", |b| {
        let msg = BinaryMessage::PresenceUpdate {
            board_id: 1234,
            count: 5,
        };
        b.iter(|| black_box(msg.encode()));
    });
}

fn encode_heartbeat(c: &mut Criterion) {
    c.bench_function("encode_heartbeat", |b| {
        let msg = BinaryMessage::Heartbeat;
        b.iter(|| black_box(msg.encode()));
    });
}

// ============================================================================
// 3. Decoding Performance Benchmarks
// ============================================================================

fn decode_cursor_update(c: &mut Criterion) {
    let msg = BinaryMessage::CursorUpdate {
        board_id: 1234,
        x: normalize_coord(0.5),
        y: normalize_coord(0.75),
    };
    let bytes = msg.encode();

    c.bench_function("decode_cursor_update", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_cursor_broadcast(c: &mut Criterion) {
    let msg = BinaryMessage::CursorBroadcast {
        user_id: 42,
        board_id: 1234,
        x: normalize_coord(0.3),
        y: normalize_coord(0.9),
    };
    let bytes = msg.encode();

    c.bench_function("decode_cursor_broadcast", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_join(c: &mut Criterion) {
    let msg = BinaryMessage::Join {
        board_id: 1234,
        username: "Alice".to_string(),
    };
    let bytes = msg.encode();

    c.bench_function("decode_join", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_leave(c: &mut Criterion) {
    let msg = BinaryMessage::Leave { board_id: 1234 };
    let bytes = msg.encode();

    c.bench_function("decode_leave", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_user_joined(c: &mut Criterion) {
    let msg = BinaryMessage::UserJoined {
        user_id: 42,
        board_id: 1234,
        username: "Bob".to_string(),
        color: [255, 128, 64],
    };
    let bytes = msg.encode();

    c.bench_function("decode_user_joined", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_user_left(c: &mut Criterion) {
    let msg = BinaryMessage::UserLeft {
        user_id: 42,
        board_id: 1234,
    };
    let bytes = msg.encode();

    c.bench_function("decode_user_left", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_presence_update(c: &mut Criterion) {
    let msg = BinaryMessage::PresenceUpdate {
        board_id: 1234,
        count: 5,
    };
    let bytes = msg.encode();

    c.bench_function("decode_presence_update", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

fn decode_heartbeat(c: &mut Criterion) {
    let msg = BinaryMessage::Heartbeat;
    let bytes = msg.encode();

    c.bench_function("decode_heartbeat", |b| {
        b.iter(|| {
            let result = BinaryMessage::decode(black_box(&bytes));
            black_box(result)
        });
    });
}

// ============================================================================
// 4. Throughput Benchmarks
// ============================================================================

fn throughput_cursor_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Elements(10_000));

    group.bench_function("10k_cursor_updates", |b| {
        b.iter(|| {
            for i in 0..10_000 {
                let msg = BinaryMessage::CursorUpdate {
                    board_id: 1234,
                    x: normalize_coord((i % 1000) as f32 / 1000.0),
                    y: normalize_coord((i % 500) as f32 / 500.0),
                };
                let bytes = msg.encode();
                let _decoded = BinaryMessage::decode(&bytes).unwrap();
                black_box(_decoded);
            }
        });
    });

    group.finish();
}

fn throughput_mixed_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_mixed");
    group.throughput(Throughput::Elements(1_000));

    group.bench_function("mixed_message_stream", |b| {
        b.iter(|| {
            // Simulate realistic message mix:
            // 80% cursor updates, 10% presence updates, 10% other
            for i in 0..1_000 {
                let msg = match i % 10 {
                    0..=7 => {
                        // 80% cursor updates
                        BinaryMessage::CursorUpdate {
                            board_id: 1234,
                            x: normalize_coord((i % 100) as f32 / 100.0),
                            y: normalize_coord((i % 50) as f32 / 50.0),
                        }
                    }
                    8 => {
                        // 10% presence updates
                        BinaryMessage::PresenceUpdate {
                            board_id: 1234,
                            count: (i % 20) as u8,
                        }
                    }
                    _ => {
                        // 10% other (heartbeat, user left, etc.)
                        if i % 2 == 0 {
                            BinaryMessage::Heartbeat
                        } else {
                            BinaryMessage::UserLeft {
                                user_id: (i % 100) as u8,
                                board_id: 1234,
                            }
                        }
                    }
                };

                let bytes = msg.encode();
                let _decoded = BinaryMessage::decode(&bytes).unwrap();
                black_box(_decoded);
            }
        });
    });

    group.finish();
}

// ============================================================================
// 5. Size Comparison vs JSON
// ============================================================================

fn size_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("size_comparison");

    // Binary cursor update
    let binary_msg = BinaryMessage::CursorUpdate {
        board_id: 1234,
        x: normalize_coord(0.5),
        y: normalize_coord(0.75),
    };
    let binary_bytes = binary_msg.encode();

    // JSON equivalent
    let json_msg = json!({
        "type": "cursor_update",
        "board_id": 1234,
        "x": 0.5,
        "y": 0.75
    });
    let json_bytes = serde_json::to_vec(&json_msg).unwrap();

    let reduction = (1.0 - (binary_bytes.len() as f64 / json_bytes.len() as f64)) * 100.0;

    println!("\n=== Size Comparison: Binary vs JSON ===");
    println!("Binary size: {} bytes", binary_bytes.len());
    println!("JSON size: {} bytes", json_bytes.len());
    println!("Size reduction: {:.1}%", reduction);

    assert!(
        reduction >= 87.0,
        "Size reduction should be at least 87%, got {:.1}%",
        reduction
    );

    group.bench_function("encode_binary_cursor", |b| {
        b.iter(|| {
            let msg = BinaryMessage::CursorUpdate {
                board_id: 1234,
                x: normalize_coord(0.5),
                y: normalize_coord(0.75),
            };
            black_box(msg.encode())
        });
    });

    group.bench_function("encode_json_cursor", |b| {
        b.iter(|| {
            let msg = json!({
                "type": "cursor_update",
                "board_id": 1234,
                "x": 0.5,
                "y": 0.75
            });
            black_box(serde_json::to_vec(&msg).unwrap())
        });
    });

    // Compare more message types
    let binary_presence = BinaryMessage::PresenceUpdate {
        board_id: 1234,
        count: 5,
    };
    let binary_presence_bytes = binary_presence.encode();

    let json_presence = json!({
        "type": "presence_update",
        "board_id": 1234,
        "count": 5
    });
    let json_presence_bytes = serde_json::to_vec(&json_presence).unwrap();

    let presence_reduction =
        (1.0 - (binary_presence_bytes.len() as f64 / json_presence_bytes.len() as f64)) * 100.0;

    println!("\nPresence Update:");
    println!("Binary size: {} bytes", binary_presence_bytes.len());
    println!("JSON size: {} bytes", json_presence_bytes.len());
    println!("Size reduction: {:.1}%", presence_reduction);

    group.finish();
}

// ============================================================================
// 6. Coordinate Normalization Benchmarks
// ============================================================================

fn benchmark_normalize_coord(c: &mut Criterion) {
    c.bench_function("normalize_coord", |b| {
        let mut x = 0.0;
        b.iter(|| {
            x = (x + 0.001) % 1.0;
            black_box(normalize_coord(x))
        });
    });
}

fn benchmark_denormalize_coord(c: &mut Criterion) {
    c.bench_function("denormalize_coord", |b| {
        let mut coord: u16 = 0;
        b.iter(|| {
            coord = (coord + 10) % 65535;
            black_box(denormalize_coord(coord))
        });
    });
}

fn benchmark_coord_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordinate_precision");

    // Test roundtrip accuracy
    let test_values = [0.0, 0.25, 0.5, 0.75, 1.0, 0.123456, 0.987654];

    for &val in &test_values {
        let normalized = normalize_coord(val);
        let denormalized = denormalize_coord(normalized);
        let error = (val - denormalized).abs();

        println!(
            "Coordinate roundtrip: {} -> {} -> {} (error: {:.6})",
            val, normalized, denormalized, error
        );

        // Verify precision is within acceptable range (< 0.0001)
        assert!(error < 0.0001, "Roundtrip error too large: {}", error);
    }

    group.bench_function("coord_roundtrip", |b| {
        let mut x = 0.0;
        b.iter(|| {
            x = (x + 0.001) % 1.0;
            let normalized = normalize_coord(black_box(x));
            black_box(denormalize_coord(normalized))
        });
    });

    group.finish();
}

// ============================================================================
// Additional Performance Benchmarks
// ============================================================================

fn benchmark_encoding_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_by_username_length");

    for length in [0, 8, 16, 32] {
        let username = "a".repeat(length);

        group.bench_with_input(
            BenchmarkId::from_parameter(length),
            &username,
            |b, username| {
                let msg = BinaryMessage::Join {
                    board_id: 1234,
                    username: username.clone(),
                };
                b.iter(|| black_box(msg.encode()));
            },
        );
    }

    group.finish();
}

fn benchmark_decode_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_performance");

    // Pre-encode messages for decoding benchmarks
    let messages = vec![
        (
            "cursor_update",
            BinaryMessage::CursorUpdate {
                board_id: 1234,
                x: normalize_coord(0.5),
                y: normalize_coord(0.75),
            },
        ),
        ("heartbeat", BinaryMessage::Heartbeat),
        (
            "presence_update",
            BinaryMessage::PresenceUpdate {
                board_id: 1234,
                count: 5,
            },
        ),
        (
            "user_left",
            BinaryMessage::UserLeft {
                user_id: 42,
                board_id: 1234,
            },
        ),
    ];

    for (name, msg) in messages {
        let bytes = msg.encode();
        group.bench_with_input(BenchmarkId::new("decode", name), &bytes, |b, bytes| {
            b.iter(|| black_box(BinaryMessage::decode(bytes)));
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(size_benches, verify_message_sizes, size_comparison);

criterion_group!(
    encoding_benches,
    encode_cursor_update,
    encode_cursor_broadcast,
    encode_join,
    encode_leave,
    encode_user_joined,
    encode_user_left,
    encode_presence_update,
    encode_heartbeat,
    benchmark_encoding_by_size
);

criterion_group!(
    decoding_benches,
    decode_cursor_update,
    decode_cursor_broadcast,
    decode_join,
    decode_leave,
    decode_user_joined,
    decode_user_left,
    decode_presence_update,
    decode_heartbeat,
    benchmark_decode_performance
);

criterion_group!(
    throughput_benches,
    throughput_cursor_updates,
    throughput_mixed_messages
);

criterion_group!(
    coordinate_benches,
    benchmark_normalize_coord,
    benchmark_denormalize_coord,
    benchmark_coord_roundtrip
);

criterion_main!(
    size_benches,
    encoding_benches,
    decoding_benches,
    throughput_benches,
    coordinate_benches
);
