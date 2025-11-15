import { useEffect, useRef } from "react";
import { WebSocketClient, WSMessage } from "@/lib/websocket";
import { useBoardStore } from "@/store/board-store";
import type { Board, Column, Card, Label } from "@/lib/types";

export const useWebSocket = (shareToken: string | null) => {
  const wsClientRef = useRef<WebSocketClient | null>(null);

  const {
    updateBoard,
    addColumn,
    updateColumn,
    deleteColumn,
    addCard,
    updateCard,
    deleteCard,
    moveCard,
    addLabel,
    updateLabel,
    deleteLabel,
  } = useBoardStore();

  useEffect(() => {
    if (!shareToken) return;

    // Create WebSocket client
    const wsClient = new WebSocketClient(shareToken);
    wsClientRef.current = wsClient;

    // Handle incoming messages
    const handleMessage = (message: WSMessage) => {
      switch (message.type) {
        case "board_updated": {
          const board = message.data as Board;
          updateBoard(board);
          break;
        }

        case "column_created": {
          const column = message.data as Column;
          addColumn(column);
          break;
        }

        case "column_updated": {
          const column = message.data as Column;
          updateColumn(column.id, column);
          break;
        }

        case "column_deleted": {
          const { id } = message.data as { id: string };
          deleteColumn(id);
          break;
        }

        case "card_created": {
          const card = message.data as Card;
          addCard(card);
          break;
        }

        case "card_updated": {
          const card = message.data as Card;
          updateCard(card.id, card);
          break;
        }

        case "card_deleted": {
          const { id } = message.data as { id: string };
          deleteCard(id);
          break;
        }

        case "label_created": {
          const label = message.data as Label;
          addLabel(label.card_id, label);
          break;
        }

        case "label_updated": {
          const label = message.data as Label;
          updateLabel(label.id, label);
          break;
        }

        case "label_deleted": {
          const { id } = message.data as { id: string };
          deleteLabel(id);
          break;
        }

        default:
          console.warn("Unknown WebSocket message type:", message.type);
      }
    };

    // Subscribe to messages
    const unsubscribe = wsClient.subscribe(handleMessage);

    // Connect to WebSocket
    wsClient.connect();

    // Cleanup on unmount
    return () => {
      unsubscribe();
      wsClient.disconnect();
      wsClientRef.current = null;
    };
  }, [
    shareToken,
    updateBoard,
    addColumn,
    updateColumn,
    deleteColumn,
    addCard,
    updateCard,
    deleteCard,
    moveCard,
    addLabel,
    updateLabel,
    deleteLabel,
  ]);

  return {
    isConnected: wsClientRef.current?.isConnected() ?? false,
  };
};
