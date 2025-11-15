/**
 * Binary protocol for WebSocket presence communication.
 *
 * This module implements a highly efficient binary protocol that achieves 5-8 byte
 * message sizes for cursor updates (96% reduction vs JSON). All multi-byte integers
 * use big-endian byte order to match the Rust backend implementation.
 */

// Message type constants (must match Rust implementation)
export const MSG_CURSOR_UPDATE = 0x01;
export const MSG_CURSOR_BROADCAST = 0x02;
export const MSG_JOIN = 0x03;
export const MSG_LEAVE = 0x04;
export const MSG_USER_JOINED = 0x05;
export const MSG_USER_LEFT = 0x06;
export const MSG_PRESENCE_UPDATE = 0x07;
export const MSG_HEARTBEAT = 0x08;

// Protocol constants
export const MAX_USERNAME_LENGTH = 32;

/**
 * Binary message types
 */
export type BinaryMessage =
  | { type: "cursor_update"; boardId: number; x: number; y: number }
  | {
      type: "cursor_broadcast";
      boardId: number;
      userId: number;
      x: number;
      y: number;
    }
  | { type: "join"; boardId: number; username: string }
  | { type: "leave"; boardId: number }
  | {
      type: "user_joined";
      boardId: number;
      userId: number;
      username: string;
      color: [number, number, number];
    }
  | { type: "user_left"; boardId: number; userId: number }
  | { type: "presence_update"; boardId: number; count: number }
  | { type: "heartbeat" };

/**
 * Protocol errors
 */
export class ProtocolError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ProtocolError";
  }
}

/**
 * Normalize a floating-point coordinate (0.0-1.0) to a 16-bit unsigned integer (0-65535).
 *
 * This allows us to represent fractional coordinates with high precision
 * while using only 2 bytes per coordinate.
 *
 * @param coord - A floating-point coordinate in the range [0.0, 1.0]
 * @returns A 16-bit unsigned integer in the range [0, 65535]
 */
export function normalizeCoord(coord: number): number {
  // Clamp to [0.0, 1.0] range to prevent overflow
  const clamped = Math.max(0, Math.min(1, coord));
  return Math.floor(clamped * 65535);
}

/**
 * Denormalize a 16-bit unsigned integer (0-65535) to a floating-point coordinate (0.0-1.0).
 *
 * This is the inverse operation of normalizeCoord.
 *
 * @param coord - A 16-bit unsigned integer in the range [0, 65535]
 * @returns A floating-point coordinate in the range [0.0, 1.0]
 */
export function denormalizeCoord(coord: number): number {
  return coord / 65535;
}

/**
 * Encode a cursor update message.
 *
 * Layout (7 bytes):
 * - byte 0: message type (0x01)
 * - bytes 1-2: board_id (u16, big-endian)
 * - bytes 3-4: x coordinate (u16, big-endian, normalized 0-65535)
 * - bytes 5-6: y coordinate (u16, big-endian, normalized 0-65535)
 *
 * @param boardId - The board ID (0-65535)
 * @param x - X coordinate (0.0-1.0)
 * @param y - Y coordinate (0.0-1.0)
 * @returns Encoded message as Uint8Array
 */
export function encodeCursorUpdate(
  boardId: number,
  x: number,
  y: number
): Uint8Array {
  const buffer = new ArrayBuffer(7);
  const view = new DataView(buffer);

  view.setUint8(0, MSG_CURSOR_UPDATE);
  view.setUint16(1, boardId, false); // false = big-endian
  view.setUint16(3, normalizeCoord(x), false);
  view.setUint16(5, normalizeCoord(y), false);

  return new Uint8Array(buffer);
}

/**
 * Encode a join message.
 *
 * Layout (4-36 bytes):
 * - byte 0: message type (0x03)
 * - bytes 1-2: board_id (u16, big-endian)
 * - byte 3: username length (u8)
 * - bytes 4+: username UTF-8 bytes (max 32 bytes)
 *
 * @param boardId - The board ID (0-65535)
 * @param username - Username string (max 32 bytes UTF-8)
 * @returns Encoded message as Uint8Array
 */
export function encodeJoin(boardId: number, username: string): Uint8Array {
  // Encode username to UTF-8
  const encoder = new TextEncoder();
  const usernameBytes = encoder.encode(username);

  // Validate username length
  if (usernameBytes.length > MAX_USERNAME_LENGTH) {
    throw new ProtocolError(
      `Username too long: ${usernameBytes.length} bytes (max ${MAX_USERNAME_LENGTH})`
    );
  }

  const buffer = new ArrayBuffer(4 + usernameBytes.length);
  const view = new DataView(buffer);

  view.setUint8(0, MSG_JOIN);
  view.setUint16(1, boardId, false);
  view.setUint8(3, usernameBytes.length);

  // Copy username bytes
  const uint8Array = new Uint8Array(buffer);
  uint8Array.set(usernameBytes, 4);

  return uint8Array;
}

/**
 * Encode a leave message.
 *
 * Layout (3 bytes):
 * - byte 0: message type (0x04)
 * - bytes 1-2: board_id (u16, big-endian)
 *
 * @param boardId - The board ID (0-65535)
 * @returns Encoded message as Uint8Array
 */
export function encodeLeave(boardId: number): Uint8Array {
  const buffer = new ArrayBuffer(3);
  const view = new DataView(buffer);

  view.setUint8(0, MSG_LEAVE);
  view.setUint16(1, boardId, false);

  return new Uint8Array(buffer);
}

/**
 * Encode a heartbeat message.
 *
 * Layout (1 byte):
 * - byte 0: message type (0x08)
 *
 * @returns Encoded message as Uint8Array
 */
export function encodeHeartbeat(): Uint8Array {
  return new Uint8Array([MSG_HEARTBEAT]);
}

/**
 * Decode a binary message from a byte array.
 *
 * @param data - The byte array to decode
 * @returns Decoded message object
 * @throws ProtocolError if decoding fails
 */
export function decodeMessage(data: Uint8Array): BinaryMessage {
  if (data.length === 0) {
    throw new ProtocolError("Buffer underflow: empty message");
  }

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const msgType = view.getUint8(0);

  switch (msgType) {
    case MSG_CURSOR_UPDATE: {
      if (data.length !== 7) {
        throw new ProtocolError(
          `Invalid length for cursor_update: expected 7, got ${data.length}`
        );
      }

      return {
        type: "cursor_update",
        boardId: view.getUint16(1, false),
        x: denormalizeCoord(view.getUint16(3, false)),
        y: denormalizeCoord(view.getUint16(5, false)),
      };
    }

    case MSG_CURSOR_BROADCAST: {
      if (data.length !== 8) {
        throw new ProtocolError(
          `Invalid length for cursor_broadcast: expected 8, got ${data.length}`
        );
      }

      return {
        type: "cursor_broadcast",
        boardId: view.getUint16(1, false),
        userId: view.getUint8(3),
        x: denormalizeCoord(view.getUint16(4, false)),
        y: denormalizeCoord(view.getUint16(6, false)),
      };
    }

    case MSG_JOIN: {
      if (data.length < 4) {
        throw new ProtocolError(
          `Invalid length for join: expected at least 4, got ${data.length}`
        );
      }

      const boardId = view.getUint16(1, false);
      const usernameLength = view.getUint8(3);

      if (usernameLength > MAX_USERNAME_LENGTH) {
        throw new ProtocolError(
          `Username too long: ${usernameLength} bytes (max ${MAX_USERNAME_LENGTH})`
        );
      }

      if (data.length !== 4 + usernameLength) {
        throw new ProtocolError(
          `Invalid length for join: expected ${4 + usernameLength}, got ${
            data.length
          }`
        );
      }

      const usernameBytes = data.slice(4, 4 + usernameLength);
      const decoder = new TextDecoder();
      const username = decoder.decode(usernameBytes);

      return {
        type: "join",
        boardId,
        username,
      };
    }

    case MSG_LEAVE: {
      if (data.length !== 3) {
        throw new ProtocolError(
          `Invalid length for leave: expected 3, got ${data.length}`
        );
      }

      return {
        type: "leave",
        boardId: view.getUint16(1, false),
      };
    }

    case MSG_USER_JOINED: {
      if (data.length < 8) {
        throw new ProtocolError(
          `Invalid length for user_joined: expected at least 8, got ${data.length}`
        );
      }

      const boardId = view.getUint16(1, false);
      const userId = view.getUint8(3);
      const usernameLength = view.getUint8(4);

      if (usernameLength > MAX_USERNAME_LENGTH) {
        throw new ProtocolError(
          `Username too long: ${usernameLength} bytes (max ${MAX_USERNAME_LENGTH})`
        );
      }

      if (data.length !== 8 + usernameLength) {
        throw new ProtocolError(
          `Invalid length for user_joined: expected ${
            8 + usernameLength
          }, got ${data.length}`
        );
      }

      const usernameBytes = data.slice(5, 5 + usernameLength);
      const decoder = new TextDecoder();
      const username = decoder.decode(usernameBytes);

      const colorOffset = 5 + usernameLength;
      const color: [number, number, number] = [
        view.getUint8(colorOffset),
        view.getUint8(colorOffset + 1),
        view.getUint8(colorOffset + 2),
      ];

      return {
        type: "user_joined",
        boardId,
        userId,
        username,
        color,
      };
    }

    case MSG_USER_LEFT: {
      if (data.length !== 4) {
        throw new ProtocolError(
          `Invalid length for user_left: expected 4, got ${data.length}`
        );
      }

      return {
        type: "user_left",
        boardId: view.getUint16(1, false),
        userId: view.getUint8(3),
      };
    }

    case MSG_PRESENCE_UPDATE: {
      if (data.length !== 4) {
        throw new ProtocolError(
          `Invalid length for presence_update: expected 4, got ${data.length}`
        );
      }

      return {
        type: "presence_update",
        boardId: view.getUint16(1, false),
        count: view.getUint8(3),
      };
    }

    case MSG_HEARTBEAT: {
      if (data.length !== 1) {
        throw new ProtocolError(
          `Invalid length for heartbeat: expected 1, got ${data.length}`
        );
      }

      return {
        type: "heartbeat",
      };
    }

    default:
      throw new ProtocolError(
        `Unknown message type: 0x${msgType.toString(16)}`
      );
  }
}
