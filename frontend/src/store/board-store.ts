import { create } from "zustand";
import type {
  Board,
  Column,
  Card,
  BoardLabel,
  CardAttachment,
} from "@/lib/types";

interface BoardStore {
  board: Board | null;
  isLoading: boolean;
  error: string | null;
  selectedLabelFilter: string[]; // Array of label IDs to filter by

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

  // Board Label operations (new)
  addBoardLabel: (label: BoardLabel) => void;
  updateBoardLabel: (labelId: string, updates: Partial<BoardLabel>) => void;
  deleteBoardLabel: (labelId: string) => void;

  // Card Label operations (legacy - kept for compatibility)
  addLabel: (cardId: string, label: BoardLabel) => void;
  updateLabel: (labelId: string, updates: Partial<BoardLabel>) => void;
  deleteLabel: (labelId: string) => void;

  // Card-Label assignment operations (new)
  assignLabelToCard: (cardId: string, labelId: string) => void;
  unassignLabelFromCard: (cardId: string, labelId: string) => void;

  // Attachment operations
  addAttachment: (cardId: string, attachment: CardAttachment) => void;
  removeAttachment: (cardId: string, attachmentId: string) => void;

  // Filter operations
  setLabelFilter: (labelIds: string[]) => void;
  toggleLabelFilter: (labelId: string) => void;
  clearLabelFilter: () => void;
  getFilteredCards: (columnId: string) => Card[];
}

export const useBoardStore = create<BoardStore>((set, get) => ({
  board: null,
  isLoading: false,
  error: null,
  selectedLabelFilter: [],

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

  // Board label operations
  addBoardLabel: (label) =>
    set((state) => {
      if (!state.board) return state;
      // Check if label already exists
      const existingLabel = state.board.labels?.find((l) => l.id === label.id);
      if (existingLabel) {
        console.log(
          "[BoardStore] Board label already exists, skipping duplicate:",
          label.id
        );
        return state;
      }
      const labels = [...(state.board.labels || []), label];
      return {
        board: { ...state.board, labels },
      };
    }),

  updateBoardLabel: (labelId, updates) =>
    set((state) => {
      if (!state.board?.labels) return state;
      const labels = state.board.labels.map((label) =>
        label.id === labelId ? { ...label, ...updates } : label
      );
      return {
        board: { ...state.board, labels },
      };
    }),

  deleteBoardLabel: (labelId) =>
    set((state) => {
      if (!state.board) return state;

      // Remove label from board labels
      const labels = state.board.labels?.filter(
        (label) => label.id !== labelId
      );

      // Remove label from all cards that have it
      const columns = state.board.columns?.map((col) => ({
        ...col,
        cards: col.cards?.map((card) => ({
          ...card,
          labels: card.labels?.filter((label) => label.id !== labelId),
        })),
      }));

      return {
        board: { ...state.board, labels, columns },
      };
    }),

  // Card-label assignment operations
  assignLabelToCard: (cardId, labelId) =>
    set((state) => {
      if (!state.board?.columns) return state;

      // Find the label from board labels
      const label = state.board.labels?.find((l) => l.id === labelId);
      if (!label) {
        console.warn("[BoardStore] Label not found:", labelId);
        return state;
      }

      let labelAssigned = false;
      const columns = state.board.columns.map((col) => {
        const hasCard = col.cards?.some((card) => card.id === cardId);
        if (!hasCard) return col;

        const cards = col.cards?.map((card) => {
          if (card.id === cardId) {
            // Check if label already assigned
            const existingLabel = card.labels?.find((l) => l.id === labelId);
            if (existingLabel) {
              console.log(
                "[BoardStore] Label already assigned to card, skipping:",
                labelId
              );
              return card;
            }
            labelAssigned = true;
            return { ...card, labels: [...(card.labels || []), label] };
          }
          return card;
        });

        return labelAssigned ? { ...col, cards } : col;
      });

      return labelAssigned
        ? {
            board: { ...state.board, columns },
          }
        : state;
    }),

  unassignLabelFromCard: (cardId, labelId) =>
    set((state) => {
      if (!state.board?.columns) return state;

      let labelUnassigned = false;
      const columns = state.board.columns.map((col) => {
        const hasCard = col.cards?.some((card) => card.id === cardId);
        if (!hasCard) return col;

        const cards = col.cards?.map((card) => {
          if (card.id === cardId) {
            const labels = card.labels?.filter((label) => {
              if (label.id === labelId) {
                labelUnassigned = true;
                return false;
              }
              return true;
            });
            return { ...card, labels };
          }
          return card;
        });

        return { ...col, cards };
      });

      return labelUnassigned
        ? {
            board: { ...state.board, columns },
          }
        : state;
    }),

  // Legacy label operations (for backward compatibility)
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

  // Attachment operations
  addAttachment: (cardId, attachment) =>
    set((state) => {
      if (!state.board?.columns) return state;

      const columns = state.board.columns.map((col) => {
        const hasCard = col.cards?.some((card) => card.id === cardId);
        if (!hasCard) return col;

        const cards = col.cards?.map((card) => {
          if (card.id === cardId) {
            // Check if attachment already exists to prevent duplicates from SSE
            const existingAttachment = card.attachments?.find(
              (a) => a.id === attachment.id
            );
            if (existingAttachment) {
              console.log(
                "[BoardStore] Attachment already exists, skipping duplicate:",
                attachment.id
              );
              return card;
            }
            return {
              ...card,
              attachments: [...(card.attachments || []), attachment],
            };
          }
          return card;
        });

        return { ...col, cards };
      });

      return {
        board: { ...state.board, columns },
      };
    }),

  removeAttachment: (cardId, attachmentId) =>
    set((state) => {
      if (!state.board?.columns) return state;

      const columns = state.board.columns.map((col) => {
        const hasCard = col.cards?.some((card) => card.id === cardId);
        if (!hasCard) return col;

        const cards = col.cards?.map((card) => {
          if (card.id === cardId) {
            return {
              ...card,
              attachments: card.attachments?.filter(
                (a) => a.id !== attachmentId
              ),
            };
          }
          return card;
        });

        return { ...col, cards };
      });

      return {
        board: { ...state.board, columns },
      };
    }),

  // Filter operations
  setLabelFilter: (labelIds: string[]) => {
    set({ selectedLabelFilter: labelIds });
  },

  toggleLabelFilter: (labelId: string) => {
    set((state) => {
      const currentFilters = state.selectedLabelFilter;
      const isSelected = currentFilters.includes(labelId);

      return {
        selectedLabelFilter: isSelected
          ? currentFilters.filter((id) => id !== labelId)
          : [...currentFilters, labelId],
      };
    });
  },

  clearLabelFilter: () => {
    set({ selectedLabelFilter: [] });
  },

  getFilteredCards: (columnId: string) => {
    const state = get();
    const column = state.board?.columns?.find((col) => col.id === columnId);

    if (!column) return [];

    const { selectedLabelFilter } = state;

    // If no filters are active, return all cards
    if (selectedLabelFilter.length === 0) {
      return column.cards || [];
    }

    // Filter cards that have at least one of the selected labels
    return (column.cards || []).filter((card) => {
      if (!card.labels || card.labels.length === 0) {
        return false;
      }
      return card.labels.some((label) =>
        selectedLabelFilter.includes(label.id)
      );
    });
  },
}));
