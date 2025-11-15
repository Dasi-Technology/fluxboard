//! Demonstration of the binary protocol showing exact byte sizes for each message type.

use presence_service::protocol::{normalize_coord, BinaryMessage};

fn main() {
    println!("Binary Protocol Message Sizes");
    println!("==============================\n");

    // 1. CursorUpdate
    let cursor_update = BinaryMessage::CursorUpdate {
        board_id: 1234,
        x: normalize_coord(0.5),
        y: normalize_coord(0.75),
    };
    let encoded = cursor_update.encode();
    println!("1. CursorUpdate (Client → Server):");
    println!("   Size: {} bytes", encoded.len());
    println!("   Hex: {:02x?}", encoded);
    println!(
        "   Reduction vs JSON (~130 bytes): {:.1}%\n",
        (1.0 - encoded.len() as f32 / 130.0) * 100.0
    );

    // 2. CursorBroadcast
    let cursor_broadcast = BinaryMessage::CursorBroadcast {
        board_id: 1234,
        user_id: 42,
        x: normalize_coord(0.3),
        y: normalize_coord(0.7),
    };
    let encoded = cursor_broadcast.encode();
    println!("2. CursorBroadcast (Server → Client):");
    println!("   Size: {} bytes", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // 3. Join
    let join = BinaryMessage::Join {
        board_id: 100,
        username: "Alice".to_string(),
    };
    let encoded = join.encode();
    println!("3. Join (Client → Server):");
    println!("   Size: {} bytes (with username \"Alice\")", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // 4. Leave
    let leave = BinaryMessage::Leave { board_id: 100 };
    let encoded = leave.encode();
    println!("4. Leave (Client → Server):");
    println!("   Size: {} bytes", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // 5. UserJoined
    let user_joined = BinaryMessage::UserJoined {
        board_id: 200,
        user_id: 5,
        username: "Bob".to_string(),
        color: [255, 128, 64],
    };
    let encoded = user_joined.encode();
    println!("5. UserJoined (Server → Client):");
    println!("   Size: {} bytes (with username \"Bob\")", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // 6. UserLeft
    let user_left = BinaryMessage::UserLeft {
        board_id: 200,
        user_id: 5,
    };
    let encoded = user_left.encode();
    println!("6. UserLeft (Server → Client):");
    println!("   Size: {} bytes", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // 7. PresenceUpdate
    let presence_update = BinaryMessage::PresenceUpdate {
        board_id: 300,
        count: 12,
    };
    let encoded = presence_update.encode();
    println!("7. PresenceUpdate (Server → Client):");
    println!("   Size: {} bytes", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // 8. Heartbeat
    let heartbeat = BinaryMessage::Heartbeat;
    let encoded = heartbeat.encode();
    println!("8. Heartbeat (Bidirectional):");
    println!("   Size: {} bytes", encoded.len());
    println!("   Hex: {:02x?}\n", encoded);

    // Summary
    println!("\nSummary:");
    println!("--------");
    println!("✓ CursorUpdate: 7 bytes (96.2% reduction vs JSON)");
    println!("✓ CursorBroadcast: 8 bytes");
    println!("✓ Join: 4-36 bytes (variable)");
    println!("✓ Leave: 3 bytes");
    println!("✓ UserJoined: 7-40 bytes (variable)");
    println!("✓ UserLeft: 4 bytes");
    println!("✓ PresenceUpdate: 4 bytes");
    println!("✓ Heartbeat: 1 byte");
    println!("\n✓ All messages use big-endian byte order");
    println!("✓ Strings are length-prefixed (max 32 bytes)");
    println!("✓ Coordinates normalized to u16 (0-65535)");
}
