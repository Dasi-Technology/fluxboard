import { create } from "zustand";
import type { Board, Column, Card, Label } from "@/lib/types";

interface BoardStore {
  board: Board | null;
  isLoading: boolean;
  error: string | null;

  // State setters
  setBoard: (board: Board) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;

  // Board operations
  updateBoard: (updates: Partial<Board>) => void;

  // Column operations
  addColumn: (column: Column) => void;
  updateColumn: (columnId: string, updates: Partial<Column>) => void;
  deleteColumn: (columnId: string) => void;
  moveColumn: (columnId: string, newPosition: number) => void;

  // Card operations
  addCard: (card: Card) => void;
  updateCard: (cardId: string, updates: Partial<Card>) => void;
  deleteCard: (cardId: string) => void;
  moveCard: (cardId: string, newColumnId: string, newPosition: number) => void;

  // Label operations
  addLabel: (cardId: string, label: Label) => void;
  updateLabel: (labelId: string, updates: Partial<Label>) => void;
  deleteLabel: (labelId: string) => void;
}

export const useBoardStore = create<BoardStore>((set) => ({
  board: null,
  isLoading: false,
  error: null,

  setBoard: (board) => set({ board, error: null }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error, isLoading: false }),
  reset: () => set({ board: null, isLoading: false, error: null }),

  updateBoard: (updates) =>
    set((state) => ({
      board: state.board ? { ...state.board, ...updates } : null,
    })),

  addColumn: (column) =>
    set((state) => {
      if (!state.board) return state;
      const columns = [...(state.board.columns || []), column];
      return {
        board: { ...state.board, columns },
      };
    }),

  updateColumn: (columnId, updates) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) =>
        col.id === columnId ? { ...col, ...updates } : col
      );
      return {
        board: { ...state.board, columns },
      };
    }),

  deleteColumn: (columnId) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.filter((col) => col.id !== columnId);
      return {
        board: { ...state.board, columns },
      };
    }),

  moveColumn: (columnId, newPosition) =>
    set((state) => {
      if (!state.board?.columns) return state;

      const columns = [...state.board.columns];
      const columnIndex = columns.findIndex((col) => col.id === columnId);
      if (columnIndex === -1) return state;

      const [movedColumn] = columns.splice(columnIndex, 1);
      columns.splice(newPosition, 0, movedColumn);

      // Update positions
      const updatedColumns = columns.map((col, index) => ({
        ...col,
        position: index,
      }));

      return {
        board: { ...state.board, columns: updatedColumns },
      };
    }),

  addCard: (card) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) => {
        if (col.id === card.column_id) {
          return {
            ...col,
            cards: [...(col.cards || []), card],
          };
        }
        return col;
      });
      return {
        board: { ...state.board, columns },
      };
    }),

  updateCard: (cardId, updates) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) => ({
        ...col,
        cards: col.cards?.map((card) =>
          card.id === cardId ? { ...card, ...updates } : card
        ),
      }));
      return {
        board: { ...state.board, columns },
      };
    }),

  deleteCard: (cardId) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) => ({
        ...col,
        cards: col.cards?.filter((card) => card.id !== cardId),
      }));
      return {
        board: { ...state.board, columns },
      };
    }),

  moveCard: (cardId, newColumnId, newPosition) =>
    set((state) => {
      if (!state.board?.columns) return state;

      let movedCard: Card | null = null;
      let sourceColumnId = "";

      // Find and remove card from source column
      const columns = state.board.columns.map((col) => {
        const card = col.cards?.find((c) => c.id === cardId);
        if (card) {
          movedCard = card;
          sourceColumnId = col.id;
          return {
            ...col,
            cards: col.cards?.filter((c) => c.id !== cardId),
          };
        }
        return col;
      });

      if (!movedCard) return state;

      // Add card to target column
      const updatedColumns = columns.map((col) => {
        if (col.id === newColumnId) {
          const cards = [...(col.cards || [])];
          cards.splice(newPosition, 0, {
            ...movedCard!,
            column_id: newColumnId,
            position: newPosition,
          });
          // Update positions
          return {
            ...col,
            cards: cards.map((card, index) => ({
              ...card,
              position: index,
            })),
          };
        }
        // Update positions in source column if different from target
        if (col.id === sourceColumnId && col.id !== newColumnId) {
          return {
            ...col,
            cards: col.cards?.map((card, index) => ({
              ...card,
              position: index,
            })),
          };
        }
        return col;
      });

      return {
        board: { ...state.board, columns: updatedColumns },
      };
    }),

  addLabel: (cardId, label) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) => ({
        ...col,
        cards: col.cards?.map((card) =>
          card.id === cardId
            ? { ...card, labels: [...(card.labels || []), label] }
            : card
        ),
      }));
      return {
        board: { ...state.board, columns },
      };
    }),

  updateLabel: (labelId, updates) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) => ({
        ...col,
        cards: col.cards?.map((card) => ({
          ...card,
          labels: card.labels?.map((label) =>
            label.id === labelId ? { ...label, ...updates } : label
          ),
        })),
      }));
      return {
        board: { ...state.board, columns },
      };
    }),

  deleteLabel: (labelId) =>
    set((state) => {
      if (!state.board?.columns) return state;
      const columns = state.board.columns.map((col) => ({
        ...col,
        cards: col.cards?.map((card) => ({
          ...card,
          labels: card.labels?.filter((label) => label.id !== labelId),
        })),
      }));
      return {
        board: { ...state.board, columns },
      };
    }),
}));
