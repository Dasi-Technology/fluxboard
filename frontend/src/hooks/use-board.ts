import { useCallback } from "react";
import { useBoardStore } from "@/store/board-store";
import { useUIStore } from "@/store/ui-store";
import * as api from "@/lib/api";
import { validateBoardOperation } from "@/lib/board-access";
import type { Card, Column, BoardLabel } from "@/lib/types";

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
    addBoardLabel,
    updateBoardLabel,
    deleteBoardLabel,
    assignLabelToCard,
    unassignLabelFromCard,
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
        validateBoardOperation(board.share_token, board.is_locked);
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
        validateBoardOperation(board.share_token, board.is_locked);
        const position = board.columns?.length || 0;
        const newColumn = await api.createColumn(
          board.id,
          title,
          position,
          board.share_token
        );
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
      if (!column || !board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        updateColumn(columnId, updates);
        await api.updateColumn(columnId, updates, board.share_token);
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
      if (!column || !board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        deleteColumn(columnId);
        await api.deleteColumn(columnId, board.share_token);
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
      if (!board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);

        // Optimistic update
        moveColumn(columnId, newPosition);

        // Calculate new positions for all columns
        const currentColumns = [...oldColumns];
        const draggedColumnIndex = currentColumns.findIndex(
          (c) => c.id === columnId
        );
        if (draggedColumnIndex === -1) return;

        const [draggedColumn] = currentColumns.splice(draggedColumnIndex, 1);
        currentColumns.splice(newPosition, 0, draggedColumn);

        const columnPositions: Array<[string, number]> = currentColumns.map(
          (col, index) => [col.id, index]
        );

        // Call the new reorder API endpoint
        await api.reorderColumns(board.id, columnPositions, board.share_token);
      } catch (err) {
        // Rollback on error
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
      if (!column || !board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        const position = column.cards?.length || 0;
        const newCard = await api.createCard(
          columnId,
          title,
          position,
          board.share_token
        );
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
      if (!originalCard || !board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);
        updateCard(cardId, updates);
        await api.updateCard(cardId, updates, board.share_token);
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
      if (!originalCard || !board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);
        deleteCard(cardId);
        await api.deleteCard(cardId, board.share_token);
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
      if (!board) return;

      // Find the card's current column
      let currentCard: Card | undefined;
      let currentColumnId: string | undefined;

      board?.columns?.forEach((col) => {
        const card = col.cards?.find((c) => c.id === cardId);
        if (card) {
          currentCard = card;
          currentColumnId = col.id;
        }
      });

      if (!currentCard || !currentColumnId) return;

      const isWithinSameColumn = currentColumnId === newColumnId;

      try {
        validateBoardOperation(board.share_token, board.is_locked);

        // Optimistic update
        moveCard(cardId, newColumnId, newPosition);

        if (isWithinSameColumn) {
          // For same-column moves, use the reorder endpoint with all card positions
          const columnCards =
            board?.columns?.find((col) => col.id === newColumnId)?.cards || [];

          // Calculate new positions for all cards after the move
          const updatedCards = [...columnCards];
          const cardIndex = updatedCards.findIndex((c) => c.id === cardId);

          if (cardIndex !== -1) {
            // Remove card from current position
            const [movedCard] = updatedCards.splice(cardIndex, 1);
            // Insert at new position
            updatedCards.splice(newPosition, 0, movedCard);

            // Create position array with sequential positions
            const cardPositions: Array<[string, number]> = updatedCards.map(
              (c, index) => [c.id, index]
            );

            await api.reorderCards(
              newColumnId,
              cardPositions,
              board.share_token
            );
          }
        } else {
          // For cross-column moves, use the move_card endpoint
          await api.moveCard(
            cardId,
            newColumnId,
            newPosition,
            board.share_token
          );
        }
      } catch (err) {
        // Rollback on error
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
      if (!board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        const newLabel = await api.createLabel(
          cardId,
          name,
          color,
          board.share_token
        );
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
    [board, addLabel, showToast]
  );

  // Update label
  const handleUpdateLabel = useCallback(
    async (labelId: string, updates: Partial<BoardLabel>) => {
      let originalLabel: BoardLabel | undefined;
      board?.columns?.forEach((col) => {
        col.cards?.forEach((card) => {
          const label = card.labels?.find((l) => l.id === labelId);
          if (label) originalLabel = label;
        });
      });
      if (!originalLabel || !board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);
        updateLabel(labelId, updates);
        await api.updateLabel(labelId, updates, board.share_token);
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
      let originalLabel: BoardLabel | undefined;
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
      if (!originalLabel || !originalCardId || !board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);
        deleteLabel(labelId);
        await api.deleteLabel(labelId, board.share_token);
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

  // Board Label operations (new)
  const createBoardLabel = useCallback(
    async (name: string, color: string) => {
      if (!board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        const newLabel = await api.createBoardLabel(
          board.id,
          name,
          color,
          board.share_token
        );
        addBoardLabel(newLabel);
        showToast("Label created", "success");
        return newLabel;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to create label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, addBoardLabel, showToast]
  );

  const handleUpdateBoardLabel = useCallback(
    async (labelId: string, updates: Partial<BoardLabel>) => {
      const originalLabel = board?.labels?.find((l) => l.id === labelId);
      if (!originalLabel || !board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);
        updateBoardLabel(labelId, updates);
        await api.updateBoardLabel(labelId, updates, board.share_token);
        showToast("Label updated", "success");
      } catch (err) {
        updateBoardLabel(labelId, originalLabel);
        const message =
          err instanceof Error ? err.message : "Failed to update label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, updateBoardLabel, showToast]
  );

  const handleDeleteBoardLabel = useCallback(
    async (labelId: string) => {
      const originalLabel = board?.labels?.find((l) => l.id === labelId);
      if (!originalLabel || !board) return;

      try {
        validateBoardOperation(board.share_token, board.is_locked);
        deleteBoardLabel(labelId);
        await api.deleteBoardLabel(labelId, board.share_token);
        showToast("Label deleted", "success");
      } catch (err) {
        addBoardLabel(originalLabel);
        const message =
          err instanceof Error ? err.message : "Failed to delete label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, deleteBoardLabel, addBoardLabel, showToast]
  );

  const handleAssignLabelToCard = useCallback(
    async (cardId: string, labelId: string) => {
      if (!board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        assignLabelToCard(cardId, labelId);
        await api.assignLabelToCard(cardId, labelId, board.share_token);
        showToast("Label assigned", "success");
      } catch (err) {
        unassignLabelFromCard(cardId, labelId);
        const message =
          err instanceof Error ? err.message : "Failed to assign label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, assignLabelToCard, unassignLabelFromCard, showToast]
  );

  const handleUnassignLabelFromCard = useCallback(
    async (cardId: string, labelId: string) => {
      if (!board) return;
      try {
        validateBoardOperation(board.share_token, board.is_locked);
        unassignLabelFromCard(cardId, labelId);
        await api.unassignLabelFromCard(cardId, labelId, board.share_token);
        showToast("Label unassigned", "success");
      } catch (err) {
        assignLabelToCard(cardId, labelId);
        const message =
          err instanceof Error ? err.message : "Failed to unassign label";
        showToast(message, "error");
        throw err;
      }
    },
    [board, unassignLabelFromCard, assignLabelToCard, showToast]
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
    // Board label operations
    createBoardLabel,
    updateBoardLabel: handleUpdateBoardLabel,
    deleteBoardLabel: handleDeleteBoardLabel,
    assignLabelToCard: handleAssignLabelToCard,
    unassignLabelFromCard: handleUnassignLabelFromCard,
    // Legacy label operations
    createLabel,
    updateLabel: handleUpdateLabel,
    deleteLabel: handleDeleteLabel,
  };
};
