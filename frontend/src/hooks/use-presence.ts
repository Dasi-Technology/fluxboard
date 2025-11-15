/**
 * React hook for managing presence in a board.
 *
 * Handles WebSocket connection, user tracking, cursor updates,
 * and automatic cleanup.
 */

import { useEffect, useState, useCallback, useRef } from "react";
import { WebSocketClient, type PresenceEvent } from "@/lib/websocket-client";

/**
 * User presence data
 */
export interface User {
  userId: number;
  username: string;
  color: [number, number, number];
  cursor: { x: number; y: number } | null;
}

/**
 * Hook options
 */
export interface UsePresenceOptions {
  /** Board ID to join */
  boardId: number;
  /** Username for this user */
  username: string;
  /** Whether presence is enabled (default: true) */
  enabled?: boolean;
  /** Cursor update throttle in milliseconds (default: 50) */
  throttleMs?: number;
}

/**
 * Hook return value
 */
export interface UsePresenceReturn {
  /** Map of user ID to user data */
  users: Map<number, User>;
  /** Total number of users present */
  presenceCount: number;
  /** Whether currently connected to presence service */
  isConnected: boolean;
  /** Update cursor position (throttled) */
  updateCursor: (x: number, y: number) => void;
  /** Leave the board */
  leave: () => void;
}

/**
 * Throttle helper function
 */
function throttle<T extends (...args: any[]) => void>(
  func: T,
  delay: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;
  let timeoutId: NodeJS.Timeout | null = null;

  return (...args: Parameters<T>) => {
    const now = Date.now();
    const timeSinceLastCall = now - lastCall;

    if (timeSinceLastCall >= delay) {
      lastCall = now;
      func(...args);
    } else {
      // Schedule a call at the end of the throttle period
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      timeoutId = setTimeout(() => {
        lastCall = Date.now();
        func(...args);
      }, delay - timeSinceLastCall);
    }
  };
}

/**
 * Hook for managing presence in a board.
 *
 * Features:
 * - Automatic connection and join on mount
 * - Real-time user tracking
 * - Throttled cursor updates
 * - Automatic cleanup on unmount
 *
 * @param options - Hook options
 * @returns Presence state and methods
 */
export function usePresence(options: UsePresenceOptions): UsePresenceReturn {
  const { boardId, username, enabled = true, throttleMs = 50 } = options;

  const [users, setUsers] = useState<Map<number, User>>(new Map());
  const [presenceCount, setPresenceCount] = useState(0);
  const [isConnected, setIsConnected] = useState(false);

  const clientRef = useRef<WebSocketClient | null>(null);
  const hasJoinedRef = useRef(false);
  const throttledUpdateRef = useRef<((x: number, y: number) => void) | null>(
    null
  );

  // Initialize WebSocket client and connect
  useEffect(() => {
    if (!enabled) {
      return;
    }

    const wsUrl = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001";
    const client = new WebSocketClient(wsUrl);
    clientRef.current = client;

    // Set up event handlers
    const handleConnected = () => {
      console.log("[usePresence] Connected to presence service");
      setIsConnected(true);

      // Join board on connection
      if (!hasJoinedRef.current) {
        client.sendJoin(boardId, username);
        hasJoinedRef.current = true;
      }
    };

    const handleDisconnected = () => {
      console.log("[usePresence] Disconnected from presence service");
      setIsConnected(false);
    };

    const handleUserJoined = (event: PresenceEvent) => {
      if (event.type === "user_joined") {
        console.log("[usePresence] User joined:", event.username);
        setUsers((prev) => {
          const next = new Map(prev);
          next.set(event.userId, {
            userId: event.userId,
            username: event.username,
            color: event.color,
            cursor: null,
          });
          return next;
        });
      }
    };

    const handleUserLeft = (event: PresenceEvent) => {
      if (event.type === "user_left") {
        console.log("[usePresence] User left:", event.userId);
        setUsers((prev) => {
          const next = new Map(prev);
          next.delete(event.userId);
          return next;
        });
      }
    };

    const handleCursorMove = (event: PresenceEvent) => {
      if (event.type === "cursor_move") {
        setUsers((prev) => {
          const user = prev.get(event.userId);
          if (!user) {
            return prev;
          }

          const next = new Map(prev);
          next.set(event.userId, {
            ...user,
            cursor: { x: event.x, y: event.y },
          });
          return next;
        });
      }
    };

    const handlePresenceCount = (event: PresenceEvent) => {
      if (event.type === "presence_count") {
        console.log("[usePresence] Presence count:", event.count);
        setPresenceCount(event.count);
      }
    };

    const handleError = (event: PresenceEvent) => {
      if (event.type === "error") {
        console.error("[usePresence] Error:", event.error);
      }
    };

    // Register event handlers
    client.on("connected", handleConnected);
    client.on("disconnected", handleDisconnected);
    client.on("user_joined", handleUserJoined);
    client.on("user_left", handleUserLeft);
    client.on("cursor_move", handleCursorMove);
    client.on("presence_count", handlePresenceCount);
    client.on("error", handleError);

    // Connect
    client.connect().catch((error) => {
      console.error("[usePresence] Failed to connect:", error);
    });

    // Cleanup
    return () => {
      if (hasJoinedRef.current) {
        client.sendLeave(boardId);
        hasJoinedRef.current = false;
      }

      client.disconnect();
      clientRef.current = null;
    };
  }, [enabled, boardId, username]);

  // Create throttled cursor update function
  useEffect(() => {
    if (!clientRef.current) {
      return;
    }

    const client = clientRef.current;
    const throttledUpdate = throttle((x: number, y: number) => {
      if (client.isConnected()) {
        client.sendCursorUpdate(boardId, x, y);
      }
    }, throttleMs);

    throttledUpdateRef.current = throttledUpdate;
  }, [boardId, throttleMs]);

  // Update cursor position (throttled)
  const updateCursor = useCallback((x: number, y: number) => {
    if (throttledUpdateRef.current) {
      throttledUpdateRef.current(x, y);
    }
  }, []);

  // Leave board manually
  const leave = useCallback(() => {
    if (clientRef.current && hasJoinedRef.current) {
      clientRef.current.sendLeave(boardId);
      hasJoinedRef.current = false;
    }
  }, [boardId]);

  return {
    users,
    presenceCount,
    isConnected,
    updateCursor,
    leave,
  };
}
