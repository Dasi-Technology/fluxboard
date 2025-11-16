import { useEffect, useRef, useState } from "react";
import { SSEClient, SSEEvent, SSEConnectionStatus } from "@/lib/sse";
import { useBoardStore } from "@/store/board-store";
import type { Board, Column, Card, BoardLabel } from "@/lib/types";

export const useSSE = (shareToken: string | null) => {
  const sseClientRef = useRef<SSEClient | null>(null);
  const [connectionStatus, setConnectionStatus] =
    useState<SSEConnectionStatus>("disconnected");

  const {
    updateBoard,
    addColumn,
    updateColumn,
    deleteColumn,
    moveColumn,
    addCard,
    updateCard,
    deleteCard,
    moveCard,
    addBoardLabel,
    updateBoardLabel,
    deleteBoardLabel,
    assignLabelToCard,
    unassignLabelFromCard,
  } = useBoardStore();

  useEffect(() => {
    if (!shareToken) return;

    // Create SSE client
    const sseClient = new SSEClient(shareToken);
    sseClientRef.current = sseClient;

    // Handle incoming SSE events
    const handleEvent = (event: SSEEvent) => {
      console.log("[useSSE] Handling event:", event.type, event);

      switch (event.type) {
        case "board_updated": {
          const board = event.board as Board;
          updateBoard(board);
          break;
        }

        case "column_created": {
          const column = event.column as Column;
          addColumn(column);
          break;
        }

        case "column_updated": {
          const column = event.column as Column;
          updateColumn(column.id, column);
          break;
        }

        case "column_deleted": {
          const { column_id } = event;
          deleteColumn(column_id);
          break;
        }

        case "column_reordered": {
          const { column_id, new_position } = event;
          moveColumn(column_id, new_position);
          break;
        }

        case "card_created": {
          const card = event.card as Card;
          addCard(card);
          break;
        }

        case "card_updated": {
          const card = event.card as Card;
          console.log("[useSSE] Updating card:", card.id);
          updateCard(card.id, card);
          break;
        }

        case "card_deleted": {
          const { card_id } = event;
          deleteCard(card_id);
          break;
        }

        case "card_moved": {
          const { card_id, to_column_id, new_position } = event;
          console.log("[useSSE] Moving card:", {
            card_id,
            to_column_id,
            new_position,
          });
          // Move the card to the new column and position
          moveCard(card_id, to_column_id, new_position);
          break;
        }

        case "card_reordered": {
          const { card_id, column_id, new_position } = event;
          console.log("[useSSE] Reordering card:", {
            card_id,
            column_id,
            new_position,
          });
          // Move the card within the same column to the new position
          moveCard(card_id, column_id, new_position);
          break;
        }

        case "board_label_created": {
          const label = event.label as BoardLabel;
          addBoardLabel(label);
          break;
        }

        case "board_label_updated": {
          const label = event.label as BoardLabel;
          updateBoardLabel(label.id, label);
          break;
        }

        case "board_label_deleted": {
          const { label_id } = event;
          deleteBoardLabel(label_id);
          break;
        }

        case "card_label_assigned": {
          const { card_id, label } = event;
          const boardLabel = label as BoardLabel;
          assignLabelToCard(card_id, boardLabel.id);
          break;
        }

        case "card_label_unassigned": {
          const { card_id, label_id } = event;
          unassignLabelFromCard(card_id, label_id);
          break;
        }

        default:
          console.warn("[useSSE] Unknown event type:", event);
      }
    };

    // Handle connection status changes
    const handleStatusChange = (status: SSEConnectionStatus) => {
      console.log("[useSSE] Connection status changed to:", status);
      setConnectionStatus(status);
    };

    // Subscribe to events and status changes
    const unsubscribeEvents = sseClient.subscribe(handleEvent);
    const unsubscribeStatus = sseClient.onStatusChange(handleStatusChange);

    // Connect to SSE
    sseClient.connect();

    // Cleanup on unmount
    return () => {
      unsubscribeEvents();
      unsubscribeStatus();
      sseClient.disconnect();
      sseClientRef.current = null;
    };
  }, [
    shareToken,
    updateBoard,
    addColumn,
    updateColumn,
    deleteColumn,
    moveColumn,
    addCard,
    updateCard,
    deleteCard,
    moveCard,
    addBoardLabel,
    updateBoardLabel,
    deleteBoardLabel,
    assignLabelToCard,
    unassignLabelFromCard,
  ]);

  return {
    isConnected: sseClientRef.current?.isConnected() ?? false,
    connectionStatus,
  };
};
