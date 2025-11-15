import type { Board, Column, Card, Label } from "./types";

// WebSocket message types
export type WSMessageType =
  | "board_updated"
  | "column_created"
  | "column_updated"
  | "column_deleted"
  | "card_created"
  | "card_updated"
  | "card_moved"
  | "card_deleted"
  | "label_created"
  | "label_updated"
  | "label_deleted";

export interface WSMessage {
  type: WSMessageType;
  data: Board | Column | Card | Label | { id: string };
}

export type WSMessageHandler = (message: WSMessage) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private shareToken: string;
  private handlers: Set<WSMessageHandler> = new Set();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  constructor(shareToken: string) {
    this.shareToken = shareToken;
  }

  /**
   * Connect to the WebSocket server
   */
  connect(): void {
    const wsUrl = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080/ws";
    const url = `${wsUrl}/${this.shareToken}`;

    console.log("[WebSocket] Attempting to connect to:", url);

    try {
      this.ws = new WebSocket(url);
      console.log(
        "[WebSocket] WebSocket object created, readyState:",
        this.ws.readyState
      );

      this.ws.onopen = () => {
        console.log("[WebSocket] Connection opened successfully!");
        this.reconnectAttempts = 0;
      };

      this.ws.onmessage = (event) => {
        try {
          console.log("[WebSocket] Raw message received:", event.data);
          const message: WSMessage = JSON.parse(event.data);
          console.log("[WebSocket] Parsed message:", message);
          this.handleMessage(message);
        } catch (error) {
          console.error("[WebSocket] Failed to parse message:", error);
        }
      };

      this.ws.onerror = (error) => {
        console.error("[WebSocket] Error occurred:", error);
      };

      this.ws.onclose = (event) => {
        console.log(
          "[WebSocket] Connection closed. Code:",
          event.code,
          "Reason:",
          event.reason
        );
        this.attemptReconnect();
      };
    } catch (error) {
      console.error("Failed to create WebSocket connection:", error);
      this.attemptReconnect();
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.handlers.clear();
  }

  /**
   * Subscribe to WebSocket messages
   */
  subscribe(handler: WSMessageHandler): () => void {
    this.handlers.add(handler);
    // Return unsubscribe function
    return () => {
      this.handlers.delete(handler);
    };
  }

  /**
   * Handle incoming WebSocket messages
   */
  private handleMessage(message: WSMessage): void {
    this.handlers.forEach((handler) => {
      try {
        handler(message);
      } catch (error) {
        console.error("Error in WebSocket message handler:", error);
      }
    });
  }

  /**
   * Attempt to reconnect to the WebSocket server
   */
  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error("Max reconnection attempts reached");
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    console.log(`Attempting to reconnect in ${delay}ms...`);
    setTimeout(() => {
      this.connect();
    }, delay);
  }

  /**
   * Check if WebSocket is connected
   */
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

/**
 * Create a WebSocket client for a board
 */
export const createWebSocketClient = (shareToken: string): WebSocketClient => {
  return new WebSocketClient(shareToken);
};
