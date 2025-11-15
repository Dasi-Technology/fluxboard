/**
 * WebSocket client for presence system.
 *
 * Manages WebSocket connection to the presence service, handles reconnection,
 * heartbeats, and message encoding/decoding.
 */

import {
  encodeCursorUpdate,
  encodeJoin,
  encodeLeave,
  encodeHeartbeat,
  decodeMessage,
  type BinaryMessage,
  ProtocolError,
} from "./protocol";

/**
 * Presence event types emitted by the WebSocket client
 */
export type PresenceEvent =
  | { type: "connected" }
  | { type: "disconnected" }
  | {
      type: "user_joined";
      userId: number;
      username: string;
      color: [number, number, number];
    }
  | { type: "user_left"; userId: number }
  | { type: "cursor_move"; userId: number; x: number; y: number }
  | { type: "presence_count"; count: number }
  | { type: "error"; error: Error };

/**
 * Event listener callback type
 */
export type EventListener = (event: PresenceEvent) => void;

/**
 * WebSocket client options
 */
export interface WebSocketClientOptions {
  /** Maximum number of reconnection attempts (default: 5) */
  maxReconnectAttempts?: number;
  /** Initial reconnection delay in ms (default: 1000) */
  reconnectDelay?: number;
  /** Heartbeat interval in ms (default: 30000) */
  heartbeatInterval?: number;
}

/**
 * WebSocket client for real-time presence communication.
 *
 * Features:
 * - Automatic reconnection with exponential backoff
 * - Periodic heartbeats to keep connection alive
 * - Event emitter pattern for presence updates
 * - Binary protocol message handling
 */
export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private listeners: Map<string, Set<EventListener>> = new Map();
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number;
  private reconnectDelay: number;
  private reconnectTimeout: NodeJS.Timeout | null = null;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private heartbeatIntervalMs: number;
  private isConnecting: boolean = false;
  private isDisconnecting: boolean = false;

  constructor(url: string, options: WebSocketClientOptions = {}) {
    this.url = url;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 5;
    this.reconnectDelay = options.reconnectDelay ?? 1000;
    this.heartbeatIntervalMs = options.heartbeatInterval ?? 30000;
  }

  /**
   * Connect to the WebSocket server.
   *
   * @returns Promise that resolves when connected
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        console.warn("[WebSocketClient] Already connected");
        resolve();
        return;
      }

      if (this.isConnecting) {
        console.warn("[WebSocketClient] Connection already in progress");
        reject(new Error("Connection already in progress"));
        return;
      }

      this.isConnecting = true;
      this.isDisconnecting = false;

      console.log("[WebSocketClient] Connecting to:", this.url);

      try {
        this.ws = new WebSocket(this.url);
        this.ws.binaryType = "arraybuffer";

        // Connection opened
        this.ws.onopen = () => {
          console.log("[WebSocketClient] Connection opened");
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.startHeartbeat();
          this.emit({ type: "connected" });
          resolve();
        };

        // Connection closed
        this.ws.onclose = (event) => {
          console.log(
            "[WebSocketClient] Connection closed:",
            event.code,
            event.reason
          );
          this.stopHeartbeat();

          if (!this.isDisconnecting) {
            this.emit({ type: "disconnected" });
            this.reconnect();
          }
        };

        // Connection error
        this.ws.onerror = (event) => {
          console.error("[WebSocketClient] WebSocket error:", event);
          this.isConnecting = false;

          const error = new Error("WebSocket connection error");
          this.emit({ type: "error", error });

          if (this.ws?.readyState === WebSocket.CONNECTING) {
            reject(error);
          }
        };

        // Message received
        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };
      } catch (error) {
        this.isConnecting = false;
        console.error("[WebSocketClient] Failed to create WebSocket:", error);
        const err =
          error instanceof Error
            ? error
            : new Error("Failed to create WebSocket");
        this.emit({ type: "error", error: err });
        reject(err);
      }
    });
  }

  /**
   * Disconnect from the WebSocket server.
   */
  disconnect(): void {
    console.log("[WebSocketClient] Disconnecting");
    this.isDisconnecting = true;

    // Clear reconnect timeout
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    // Stop heartbeat
    this.stopHeartbeat();

    // Close WebSocket
    if (this.ws) {
      this.ws.close(1000, "Client disconnect");
      this.ws = null;
    }

    // Reset state
    this.reconnectAttempts = 0;
    this.isConnecting = false;

    this.emit({ type: "disconnected" });
  }

  /**
   * Send a cursor update message.
   *
   * @param boardId - The board ID
   * @param x - X coordinate (0.0-1.0)
   * @param y - Y coordinate (0.0-1.0)
   */
  sendCursorUpdate(boardId: number, x: number, y: number): void {
    const message = encodeCursorUpdate(boardId, x, y);
    this.send(message);
  }

  /**
   * Send a join message.
   *
   * @param boardId - The board ID
   * @param username - Username string
   */
  sendJoin(boardId: number, username: string): void {
    const message = encodeJoin(boardId, username);
    this.send(message);
  }

  /**
   * Send a leave message.
   *
   * @param boardId - The board ID
   */
  sendLeave(boardId: number): void {
    const message = encodeLeave(boardId);
    this.send(message);
  }

  /**
   * Add an event listener.
   *
   * @param event - Event type (use '*' for all events)
   * @param callback - Event handler callback
   */
  on(event: string, callback: EventListener): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  /**
   * Remove an event listener.
   *
   * @param event - Event type
   * @param callback - Event handler callback to remove
   */
  off(event: string, callback: EventListener): void {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.delete(callback);
      if (callbacks.size === 0) {
        this.listeners.delete(event);
      }
    }
  }

  /**
   * Check if currently connected.
   */
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  /**
   * Send a binary message over the WebSocket.
   *
   * @param data - Binary data to send
   */
  private send(data: Uint8Array): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn("[WebSocketClient] Cannot send message: not connected");
      return;
    }

    try {
      this.ws.send(data);
    } catch (error) {
      console.error("[WebSocketClient] Failed to send message:", error);
      const err =
        error instanceof Error ? error : new Error("Failed to send message");
      this.emit({ type: "error", error: err });
    }
  }

  /**
   * Handle incoming binary messages.
   *
   * @param data - ArrayBuffer from WebSocket message event
   */
  private handleMessage(data: ArrayBuffer): void {
    try {
      const uint8Array = new Uint8Array(data);
      const message = decodeMessage(uint8Array);

      // Convert binary message to presence event
      switch (message.type) {
        case "cursor_broadcast":
          this.emit({
            type: "cursor_move",
            userId: message.userId,
            x: message.x,
            y: message.y,
          });
          break;

        case "user_joined":
          this.emit({
            type: "user_joined",
            userId: message.userId,
            username: message.username,
            color: message.color,
          });
          break;

        case "user_left":
          this.emit({
            type: "user_left",
            userId: message.userId,
          });
          break;

        case "presence_update":
          this.emit({
            type: "presence_count",
            count: message.count,
          });
          break;

        case "heartbeat":
          // Heartbeat received, no action needed
          break;

        default:
          console.warn(
            "[WebSocketClient] Received unexpected message type:",
            message.type
          );
      }
    } catch (error) {
      if (error instanceof ProtocolError) {
        console.error("[WebSocketClient] Protocol error:", error.message);
      } else {
        console.error("[WebSocketClient] Failed to handle message:", error);
      }
      const err =
        error instanceof Error ? error : new Error("Failed to handle message");
      this.emit({ type: "error", error: err });
    }
  }

  /**
   * Emit an event to all listeners.
   *
   * @param event - Presence event to emit
   */
  private emit(event: PresenceEvent): void {
    // Call specific event listeners
    const callbacks = this.listeners.get(event.type);
    if (callbacks) {
      callbacks.forEach((callback) => {
        try {
          callback(event);
        } catch (error) {
          console.error("[WebSocketClient] Error in event handler:", error);
        }
      });
    }

    // Call wildcard listeners
    const wildcardCallbacks = this.listeners.get("*");
    if (wildcardCallbacks) {
      wildcardCallbacks.forEach((callback) => {
        try {
          callback(event);
        } catch (error) {
          console.error(
            "[WebSocketClient] Error in wildcard event handler:",
            error
          );
        }
      });
    }
  }

  /**
   * Attempt to reconnect with exponential backoff.
   */
  private reconnect(): void {
    if (this.isDisconnecting) {
      return;
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error("[WebSocketClient] Max reconnection attempts reached");
      const error = new Error("Max reconnection attempts reached");
      this.emit({ type: "error", error });
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    console.log(
      `[WebSocketClient] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
    );

    this.reconnectTimeout = setTimeout(() => {
      this.connect().catch((error) => {
        console.error("[WebSocketClient] Reconnection failed:", error);
      });
    }, delay);
  }

  /**
   * Start sending periodic heartbeats.
   */
  private startHeartbeat(): void {
    this.stopHeartbeat();

    this.heartbeatInterval = setInterval(() => {
      if (this.isConnected()) {
        const heartbeat = encodeHeartbeat();
        this.send(heartbeat);
      }
    }, this.heartbeatIntervalMs);
  }

  /**
   * Stop sending periodic heartbeats.
   */
  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }
}

/**
 * Create a WebSocket client for presence.
 *
 * @param url - WebSocket server URL
 * @param options - Client options
 * @returns WebSocket client instance
 */
export function createWebSocketClient(
  url: string,
  options?: WebSocketClientOptions
): WebSocketClient {
  return new WebSocketClient(url, options);
}
