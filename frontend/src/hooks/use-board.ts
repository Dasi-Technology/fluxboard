import { useCallback } from "react";
import { useBoardStore } from "@/store/board-store";
import { useUIStore } from "@/store/ui-store";
import * as api from "@/lib/api";
import type { Card, Column, Label } from "@/lib/types";

export const useBoard = () => {
  const {
    board,
    isLoading,
    error,
    setBoard,
    setLoading,
    setError,
    updateBoard,
    addColumn,
    updateColumn,
    deleteColumn,
    moveColumn,
    addCard,
    updateCard,
    deleteCard,
    moveCard,
    addLabel,
    updateLabel,
    deleteLabel,
  } = useBoardStore();

  const { showToast } = useUIStore();

  // Load board by share token
  const loadBoard = useCallback(
    async (shareToken: string) => {
      setLoading(true);
      setError(null);
      try {
        const boardData = await api.getBoard(shareToken);
        setBoard(boardData);
        return boardData;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to load board";
        setError(message);
        showToast(message, "error");
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [setBoard, setLoading, setError, showToast]
  );

  // Create a new board
  const createBoard = useCallback(
    async (name: string) => {
      try {
        const newBoard = await api.createBoard(name);
        setBoard(newBoard);
        showToast("Board created successfully", "success");
        return newBoard;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to create board";
        showToast(message, "error");
        throw err;
      }
    },
    [setBoard, showToast]
  );

  // Update board name
  const updateBoardName = useCallback(
    async (shareToken: string, name: string) => {
      if (!board) return;
      const previousBoard = { ...board };
      try {
        updateBoard({ title: name });
        await api.updateBoardName(shareToken, name);
        showToast("Board name updated", "success");
      } catch (err) {
        setBoard(previousBoard);
        const message =
          err instanceof Error ? err.message : "Failed to update board";
        showToast(message, "error");
        throw err;
      }
    },
    [board, updateBoard, setBoard, showToast]
  );

  // Create column
  const createColumn = useCallback(
    async (title: string) => {
      if (!board) return;
      try {
        const position = board.columns?.length || 0;
        const newColumn = await api.createColumn(board.id, title, position);
        addColumn(newColumn);
        showToast("Column created", "success");
        return newColumn;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to create column";
        showToast(message, "error");
        throw err;
      }
    },
    [board, addColumn, showToast]
  );

  // Update column
  const handleUpdateColumn = useCallback(
    async (columnId: string, updates: Partial<Column>) => {
      const column = board?.columns?.find((c) => c.id === columnId);
      if (!column) return;
      try {
        updateColumn(columnId, updates);
        await api.updateColumn(columnId, updates);
        showToast("Column updated", "success");
      } catch (err) {
        updateColumn(columnId, column);
        const message =
          err instanceof Error ? err.message : "Failed to update column";
        showToast(message, "error");
        throw err;
      }
    },
    [board, updateColumn, showToast]
  );

  // Delete column
  const handleDeleteColumn = useCallback(
    async (columnId: string) => {
      const column = board?.columns?.find((c) => c.id === columnId);
      if (!column) return;
      try {
        deleteColumn(columnId);
        await api.deleteColumn(columnId);
        showToast("Column deleted", "success");
      } catch (err) {
        addColumn(column);
        const message =
          err instanceof Error ? err.message : "Failed to delete column";
        showToast(message, "error");
        throw err;
      }
    },
    [board, deleteColumn, addColumn, showToast]
  );

  // Reorder column
  const reorderColumn = useCallback(
    async (columnId: string, newPosition: number) => {
      const oldColumns = board?.columns ? [...board.columns] : [];
      try {
        moveColumn(columnId, newPosition);
        await api.updateColumn(columnId, { position: newPosition });
      } catch (err) {
        if (board) {
          setBoard({ ...board, columns: oldColumns });
        }
        const message =
          err instanceof Error ? err.message : "Failed to reorder column";
        showToast(message, "error");
        throw err;
      }
    },
    [board, moveColumn, setBoard, showToast]
  );

  // Create card
  const createCard = useCallback(
    async (columnId: string, title: string) => {
      const column = board?.columns?.find((c) => c.id === columnId);
      if (!column) return;
      try {
        const position = column.cards?.length || 0;
        const newCard = await api.createCard(columnId, title, position);
        addCard(newCard);
        showToast("Card created", "success");
        return newCard;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to create card";
        showToast(message, "error");
        throw err;
      }
    },
    [board, addCard, showToast]
  );

  // Update card
  const handleUpdateCard = useCallback(
    async (cardId: string, updates: Partial<Card>) => {
      let originalCard: Card | undefined;
      board?.columns?.forEach((col) => {
        const card = col.cards?.find((c) => c.id === cardId);
        if (card) originalCard = card;
      });
      if (!originalCard) return;

      try {
        updateCard(cardId, updates);
        await api.updateCard(cardId, updates);
        showToast("Card updated", "success");
      } catch (err) {
        updateCard(cardId, originalCard);
        const message =
          err instanceof Error ? err.message : "Failed to update card";
        showToast(message, "error");
        throw err;
      }
    },
    [board, updateCard, showToast]
  );

  // Delete card
  const handleDeleteCard = useCallback(
    async (cardId: string) => {
      let originalCard: Card | undefined;
      board?.columns?.forEach((col) => {
        const card = col.cards?.find((c) => c.id === cardId);
        if (card) originalCard = card;
      });
      if (!originalCard) return;

      try {
        deleteCard(cardId);
        await api.deleteCard(cardId);
        showToast("Card deleted", "success");
      } catch (err) {
        addCard(originalCard);
        const message =
          err instanceof Error ? err.message : "Failed to delete card";
        showToast(message, "error");
        throw err;
      }
    },
    [board, deleteCard, addCard, showToast]
  );

  // Move card
  const handleMoveCard = useCallback(
    async (cardId: string, newColumnId: string, newPosition: number) => {
      const oldColumns = board?.columns ? [...board.columns] : [];
      try {
        moveCard(cardId, newColumnId, newPosition);
        await api.updateCard(cardId, {
          column_id: newColumnId,
          position: newPosition,
        });
      } catch (err) {
        if (board) {
          setBoard({ ...board, columns: oldColumns });
        }
        const message =
          err instanceof Error ? err.message : "Failed to move card";
        showToast(message, "error");
        throw err;
      }
    },
    [board, moveCard, setBoard, showToast]
  );

  // Create label
  const createLabel = useCallback(
    async (cardId: string, name: string, color: string) => {
      try {
        const newLabel = await api.createLabel(cardId, name, color);
        addLabel(cardId, newLabel);
        showToast("Label created", "success");
        return newLabel;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to create label";
        showToast(message, "error");
        throw err;
      }
    },
    [addLabel, showToast]
  );

  // Update label
  const handleUpdateLabel = useCallback(
    async (labelId: string, updates: Partial<Label>) => {
      let originalLabel: Label | undefined;
      board?.columns?.forEach((col) => {
        col.cards?.forEach((card) => {
          const label = card.labels?.find((l) => l.id === labelId);
          if (label) originalLabel = label;
        });
      });
      if (!originalLabel) return;

      try {
        updateLabel(labelId, updates);
        await api.updateLabel(labelId, updates);
        showToast("Label updated", "success");
      } catch (err) {
        updateLabel(labelId, originalLabel);
        const message =
          err instanceof Error ? err.message : "Failed to update label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, updateLabel, showToast]
  );

  // Delete label
  const handleDeleteLabel = useCallback(
    async (labelId: string) => {
      let originalLabel: Label | undefined;
      let originalCardId: string | undefined;
      board?.columns?.forEach((col) => {
        col.cards?.forEach((card) => {
          const label = card.labels?.find((l) => l.id === labelId);
          if (label) {
            originalLabel = label;
            originalCardId = card.id;
          }
        });
      });
      if (!originalLabel || !originalCardId) return;

      try {
        deleteLabel(labelId);
        await api.deleteLabel(labelId);
        showToast("Label deleted", "success");
      } catch (err) {
        addLabel(originalCardId, originalLabel);
        const message =
          err instanceof Error ? err.message : "Failed to delete label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, deleteLabel, addLabel, showToast]
  );

  return {
    board,
    isLoading,
    error,
    loadBoard,
    createBoard,
    updateBoardName,
    createColumn,
    updateColumn: handleUpdateColumn,
    deleteColumn: handleDeleteColumn,
    reorderColumn,
    createCard,
    updateCard: handleUpdateCard,
    deleteCard: handleDeleteCard,
    moveCard: handleMoveCard,
    createLabel,
    updateLabel: handleUpdateLabel,
    deleteLabel: handleDeleteLabel,
  };
};
