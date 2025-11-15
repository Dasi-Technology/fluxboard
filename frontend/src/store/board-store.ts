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
      // Check if column already exists to prevent duplicates from SSE
      const existingColumn = state.board.columns?.find(
        (col) => col.id === column.id
      );
      if (existingColumn) {
        console.log(
          "[BoardStore] Column already exists, skipping duplicate:",
          column.id
        );
        return state;
      }
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
          // Check if card already exists to prevent duplicates from SSE
          const existingCard = col.cards?.find((c) => c.id === card.id);
          if (existingCard) {
            console.log(
              "[BoardStore] Card already exists, skipping duplicate:",
              card.id
            );
            return col;
          }
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

      // Find the card to get its current column_id
      let currentCard: Card | undefined;
      let currentColumnId: string | undefined;
      state.board.columns.forEach((col) => {
        const card = col.cards?.find((c) => c.id === cardId);
        if (card) {
          currentCard = card;
          currentColumnId = col.id;
        }
      });

      if (!currentCard || !currentColumnId) return state;

      // At this point, TypeScript knows currentCard is Card (not undefined)

      // Check if the card is being moved to a different column
      const newColumnId = updates.column_id;
      const isMovingColumn = newColumnId && newColumnId !== currentColumnId;

      if (isMovingColumn) {
        // Remove card from current column and add to new column
        let movedCard: Card | null = null;
        const columns = state.board.columns.map((col) => {
          if (col.id === currentColumnId) {
            // Remove from current column
            const card = col.cards?.find((c) => c.id === cardId);
            if (card) {
              movedCard = { ...card, ...updates };
            }
            return {
              ...col,
              cards: col.cards?.filter((c) => c.id !== cardId),
            };
          }
          return col;
        });

        if (!movedCard) return state;

        // Add to new column
        const updatedColumns = columns.map((col) => {
          if (col.id === newColumnId) {
            const cards = [...(col.cards || [])];
            // Insert at the specified position or at the end
            const position = updates.position ?? cards.length;
            cards.splice(position, 0, movedCard!);
            // Update positions
            return {
              ...col,
              cards: cards.map((card, index) => ({
                ...card,
                position: index,
              })),
            };
          }
          // Update positions in source column
          if (col.id === currentColumnId) {
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
      } else {
        // Check if position is changing within the same column
        const newPosition = updates.position;
        const currentPosition = currentCard.position;
        const isRepositioning =
          newPosition !== undefined && newPosition !== currentPosition;

        if (isRepositioning) {
          // Reorder cards within the same column
          const columns = state.board.columns.map((col) => {
            if (col.id === currentColumnId) {
              const cards = [...(col.cards || [])];
              // Remove the card from its current position
              const cardIndex = cards.findIndex((c) => c.id === cardId);
              if (cardIndex !== -1) {
                const [movedCard] = cards.splice(cardIndex, 1);
                // Insert at new position
                cards.splice(newPosition, 0, { ...movedCard, ...updates });
                // Update all positions
                return {
                  ...col,
                  cards: cards.map((card, index) => ({
                    ...card,
                    position: index,
                  })),
                };
              }
            }
            return col;
          });
          return {
            board: { ...state.board, columns },
          };
        } else {
          // Just update the card properties without repositioning
          const columns = state.board.columns.map((col) => ({
            ...col,
            cards: col.cards?.map((card) =>
              card.id === cardId ? { ...card, ...updates } : card
            ),
          }));
          return {
            board: { ...state.board, columns },
          };
        }
      }
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

      let labelAdded = false;
      const columns = state.board.columns.map((col) => {
        const hasCard = col.cards?.some((card) => card.id === cardId);
        if (!hasCard) return col;

        // Create new cards array with updated card
        const cards = col.cards?.map((card) => {
          if (card.id === cardId) {
            // Check if label already exists to prevent duplicates from SSE
            const existingLabel = card.labels?.find((l) => l.id === label.id);
            if (existingLabel) {
              console.log(
                "[BoardStore] Label already exists, skipping duplicate:",
                label.id
              );
              return card;
            }
            labelAdded = true;
            // Create new card with new labels array
            return { ...card, labels: [...(card.labels || []), label] };
          }
          return card;
        });

        // Only create new column object if we actually modified a card
        return labelAdded ? { ...col, cards } : col;
      });

      return labelAdded
        ? {
            board: { ...state.board, columns },
          }
        : state;
    }),

  updateLabel: (labelId, updates) =>
    set((state) => {
      if (!state.board?.columns) return state;

      let labelUpdated = false;
      const columns = state.board.columns.map((col) => {
        const hasLabel = col.cards?.some((card) =>
          card.labels?.some((label) => label.id === labelId)
        );
        if (!hasLabel) return col;

        // Create new cards array with updated labels
        const cards = col.cards?.map((card) => {
          const cardHasLabel = card.labels?.some(
            (label) => label.id === labelId
          );
          if (!cardHasLabel) return card;

          // Create new labels array
          const labels = card.labels?.map((label) => {
            if (label.id === labelId) {
              labelUpdated = true;
              return { ...label, ...updates };
            }
            return label;
          });

          // Create new card object
          return { ...card, labels };
        });

        // Create new column object
        return { ...col, cards };
      });

      return labelUpdated
        ? {
            board: { ...state.board, columns },
          }
        : state;
    }),

  deleteLabel: (labelId) =>
    set((state) => {
      if (!state.board?.columns) return state;

      let labelDeleted = false;
      const columns = state.board.columns.map((col) => {
        const hasLabel = col.cards?.some((card) =>
          card.labels?.some((label) => label.id === labelId)
        );
        if (!hasLabel) return col;

        // Create new cards array with filtered labels
        const cards = col.cards?.map((card) => {
          const cardHasLabel = card.labels?.some(
            (label) => label.id === labelId
          );
          if (!cardHasLabel) return card;

          // Create new labels array without the deleted label
          const labels = card.labels?.filter((label) => {
            if (label.id === labelId) {
              labelDeleted = true;
              return false;
            }
            return true;
          });

          // Create new card object
          return { ...card, labels };
        });

        // Create new column object
        return { ...col, cards };
      });

      return labelDeleted
        ? {
            board: { ...state.board, columns },
          }
        : state;
    }),
}));
