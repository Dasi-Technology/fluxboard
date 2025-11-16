import type { Board, Column, Card, Label } from "./types";

/**
 * SSE event types matching backend event names
 */
export type SSEEventType =
  | "board:updated"
  | "column:created"
  | "column:updated"
  | "column:deleted"
  | "column:reordered"
  | "card:created"
  | "card:updated"
  | "card:deleted"
  | "card:moved"
  | "card:reordered"
  | "board_label:created"
  | "board_label:updated"
  | "board_label:deleted"
  | "card_label:assigned"
  | "card_label:unassigned";

/**
 * SSE event data structures matching backend event payloads
 */
export interface SSEBoardUpdatedEvent {
  type: "board_updated";
  board: Board;
}

export interface SSEColumnCreatedEvent {
  type: "column_created";
  column: Column;
}

export interface SSEColumnUpdatedEvent {
  type: "column_updated";
  column: Column;
}

export interface SSEColumnDeletedEvent {
  type: "column_deleted";
  column_id: string;
}

export interface SSEColumnReorderedEvent {
  type: "column_reordered";
  column_id: string;
  new_position: number;
}

export interface SSECardCreatedEvent {
  type: "card_created";
  card: Card;
}

export interface SSECardUpdatedEvent {
  type: "card_updated";
  card: Card;
}

export interface SSECardDeletedEvent {
  type: "card_deleted";
  card_id: string;
}

export interface SSECardMovedEvent {
  type: "card_moved";
  card_id: string;
  from_column_id: string;
  to_column_id: string;
  new_position: number;
}

export interface SSECardReorderedEvent {
  type: "card_reordered";
  card_id: string;
  column_id: string;
  new_position: number;
}

export interface SSEBoardLabelCreatedEvent {
  type: "board_label_created";
  label: Label;
}

export interface SSEBoardLabelUpdatedEvent {
  type: "board_label_updated";
  label: Label;
}

export interface SSEBoardLabelDeletedEvent {
  type: "board_label_deleted";
  label_id: string;
}

export interface SSECardLabelAssignedEvent {
  type: "card_label_assigned";
  card_id: string;
  label: Label;
}

export interface SSECardLabelUnassignedEvent {
  type: "card_label_unassigned";
  card_id: string;
  label_id: string;
}

/**
 * Union type for all SSE events
 */
export type SSEEvent =
  | SSEBoardUpdatedEvent
  | SSEColumnCreatedEvent
  | SSEColumnUpdatedEvent
  | SSEColumnDeletedEvent
  | SSEColumnReorderedEvent
  | SSECardCreatedEvent
  | SSECardUpdatedEvent
  | SSECardDeletedEvent
  | SSECardMovedEvent
  | SSECardReorderedEvent
  | SSEBoardLabelCreatedEvent
  | SSEBoardLabelUpdatedEvent
  | SSEBoardLabelDeletedEvent
  | SSECardLabelAssignedEvent
  | SSECardLabelUnassignedEvent;

/**
 * Event handler type for SSE events
 */
export type SSEEventHandler = (event: SSEEvent) => void;

/**
 * Connection status for SSE client
 */
export type SSEConnectionStatus =
  | "connecting"
  | "connected"
  | "disconnected"
  | "error"
  | "reconnecting";

/**
 * Status change handler type
 */
export type SSEStatusHandler = (status: SSEConnectionStatus) => void;

/**
 * SSE Client class for managing Server-Sent Events connections
 */
export class SSEClient {
  private eventSource: EventSource | null = null;
  private shareToken: string;
  private eventHandlers: Set<SSEEventHandler> = new Set();
  private statusHandlers: Set<SSEStatusHandler> = new Set();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private reconnectTimeout: NodeJS.Timeout | null = null;
  private status: SSEConnectionStatus = "disconnected";

  constructor(shareToken: string) {
    this.shareToken = shareToken;
  }

  /**
   * Connect to the SSE endpoint
   */
  connect(): void {
    if (this.eventSource) {
      console.warn("[SSEClient] Already connected or connecting");
      return;
    }

    const apiUrl =
      process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api";
    const url = `${apiUrl}/sse/${this.shareToken}`;

    console.log("[SSEClient] Connecting to:", url);
    this.updateStatus("connecting");

    try {
      this.eventSource = new EventSource(url);

      // Handle connection open
      this.eventSource.onopen = () => {
        console.log("[SSEClient] Connection opened successfully");
        this.reconnectAttempts = 0;
        this.updateStatus("connected");
      };

      // Handle connection errors
      this.eventSource.onerror = (error) => {
        console.error("[SSEClient] Connection error:", error);

        // EventSource will automatically try to reconnect
        // We only need to handle cleanup and status updates
        if (this.eventSource?.readyState === EventSource.CLOSED) {
          console.log("[SSEClient] Connection closed");
          this.updateStatus("error");
          this.attemptReconnect();
        } else if (this.eventSource?.readyState === EventSource.CONNECTING) {
          console.log("[SSEClient] Reconnecting...");
          this.updateStatus("reconnecting");
        }
      };

      // Set up event listeners for each event type
      this.setupEventListeners();
    } catch (error) {
      console.error("[SSEClient] Failed to create EventSource:", error);
      this.updateStatus("error");
      this.attemptReconnect();
    }
  }

  /**
   * Set up event listeners for all SSE event types
   */
  private setupEventListeners(): void {
    if (!this.eventSource) return;

    // Board events
    this.eventSource.addEventListener("board:updated", (e) => {
      this.handleEvent("board:updated", e);
    });

    // Column events
    this.eventSource.addEventListener("column:created", (e) => {
      this.handleEvent("column:created", e);
    });
    this.eventSource.addEventListener("column:updated", (e) => {
      this.handleEvent("column:updated", e);
    });
    this.eventSource.addEventListener("column:deleted", (e) => {
      this.handleEvent("column:deleted", e);
    });
    this.eventSource.addEventListener("column:reordered", (e) => {
      this.handleEvent("column:reordered", e);
    });

    // Card events
    this.eventSource.addEventListener("card:created", (e) => {
      this.handleEvent("card:created", e);
    });
    this.eventSource.addEventListener("card:updated", (e) => {
      this.handleEvent("card:updated", e);
    });
    this.eventSource.addEventListener("card:deleted", (e) => {
      this.handleEvent("card:deleted", e);
    });
    this.eventSource.addEventListener("card:moved", (e) => {
      this.handleEvent("card:moved", e);
    });
    this.eventSource.addEventListener("card:reordered", (e) => {
      this.handleEvent("card:reordered", e);
    });

    // Board label events
    this.eventSource.addEventListener("board_label:created", (e) => {
      this.handleEvent("board_label:created", e);
    });
    this.eventSource.addEventListener("board_label:updated", (e) => {
      this.handleEvent("board_label:updated", e);
    });
    this.eventSource.addEventListener("board_label:deleted", (e) => {
      this.handleEvent("board_label:deleted", e);
    });

    // Card label assignment events
    this.eventSource.addEventListener("card_label:assigned", (e) => {
      this.handleEvent("card_label:assigned", e);
    });
    this.eventSource.addEventListener("card_label:unassigned", (e) => {
      this.handleEvent("card_label:unassigned", e);
    });
  }

  /**
   * Handle incoming SSE events
   */
  private handleEvent(eventType: SSEEventType, event: MessageEvent): void {
    try {
      console.log(`[SSEClient] Received ${eventType} event:`, event.data);
      const data = JSON.parse(event.data) as SSEEvent;

      // Notify all event handlers
      this.eventHandlers.forEach((handler) => {
        try {
          handler(data);
        } catch (error) {
          console.error("[SSEClient] Error in event handler:", error);
        }
      });
    } catch (error) {
      console.error(`[SSEClient] Failed to parse ${eventType} event:`, error);
    }
  }

  /**
   * Subscribe to SSE events
   */
  subscribe(handler: SSEEventHandler): () => void {
    this.eventHandlers.add(handler);
    // Return unsubscribe function
    return () => {
      this.eventHandlers.delete(handler);
    };
  }

  /**
   * Subscribe to connection status changes
   */
  onStatusChange(handler: SSEStatusHandler): () => void {
    this.statusHandlers.add(handler);
    // Immediately call with current status
    handler(this.status);
    // Return unsubscribe function
    return () => {
      this.statusHandlers.delete(handler);
    };
  }

  /**
   * Update connection status and notify handlers
   */
  private updateStatus(newStatus: SSEConnectionStatus): void {
    if (this.status === newStatus) return;

    this.status = newStatus;
    console.log("[SSEClient] Status changed to:", newStatus);

    this.statusHandlers.forEach((handler) => {
      try {
        handler(newStatus);
      } catch (error) {
        console.error("[SSEClient] Error in status handler:", error);
      }
    });
  }

  /**
   * Attempt to reconnect with exponential backoff
   */
  private attemptReconnect(): void {
    // Clear existing EventSource
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    // Clear any existing reconnect timeout
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error("[SSEClient] Max reconnection attempts reached");
      this.updateStatus("error");
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    console.log(
      `[SSEClient] Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
    );
    this.updateStatus("reconnecting");

    this.reconnectTimeout = setTimeout(() => {
      this.connect();
    }, delay);
  }

  /**
   * Disconnect from the SSE endpoint
   */
  disconnect(): void {
    console.log("[SSEClient] Disconnecting");

    // Clear reconnect timeout
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    // Close EventSource
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    // Clear handlers
    this.eventHandlers.clear();
    this.statusHandlers.clear();

    // Reset state
    this.reconnectAttempts = 0;
    this.updateStatus("disconnected");
  }

  /**
   * Get current connection status
   */
  getStatus(): SSEConnectionStatus {
    return this.status;
  }

  /**
   * Check if currently connected
   */
  isConnected(): boolean {
    return (
      this.eventSource !== null &&
      this.eventSource.readyState === EventSource.OPEN &&
      this.status === "connected"
    );
  }
}

/**
 * Create an SSE client for a board
 */
export const createSSEClient = (shareToken: string): SSEClient => {
  return new SSEClient(shareToken);
};
