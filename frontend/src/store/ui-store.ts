import { create } from "zustand";

interface UIStore {
  // Dialog states
  isCreateBoardDialogOpen: boolean;
  isEditCardDialogOpen: boolean;
  isEditColumnDialogOpen: boolean;
  isManageLabelsDialogOpen: boolean;

  // Selected items
  selectedCardId: string | null;
  selectedColumnId: string | null;

  // Toast notifications
  toast: {
    message: string;
    type: "success" | "error" | "info";
  } | null;

  // Loading states
  isCreatingBoard: boolean;
  isCreatingColumn: boolean;
  isCreatingCard: boolean;

  // Actions
  openCreateBoardDialog: () => void;
  closeCreateBoardDialog: () => void;

  openEditCardDialog: (cardId: string) => void;
  closeEditCardDialog: () => void;

  openEditColumnDialog: (columnId: string) => void;
  closeEditColumnDialog: () => void;

  openManageLabelsDialog: (cardId: string) => void;
  closeManageLabelsDialog: () => void;

  showToast: (message: string, type: "success" | "error" | "info") => void;
  hideToast: () => void;

  setCreatingBoard: (isCreating: boolean) => void;
  setCreatingColumn: (isCreating: boolean) => void;
  setCreatingCard: (isCreating: boolean) => void;

  reset: () => void;
}

export const useUIStore = create<UIStore>((set) => ({
  // Initial state
  isCreateBoardDialogOpen: false,
  isEditCardDialogOpen: false,
  isEditColumnDialogOpen: false,
  isManageLabelsDialogOpen: false,
  selectedCardId: null,
  selectedColumnId: null,
  toast: null,
  isCreatingBoard: false,
  isCreatingColumn: false,
  isCreatingCard: false,

  // Dialog actions
  openCreateBoardDialog: () => set({ isCreateBoardDialogOpen: true }),
  closeCreateBoardDialog: () => set({ isCreateBoardDialogOpen: false }),

  openEditCardDialog: (cardId) =>
    set({ isEditCardDialogOpen: true, selectedCardId: cardId }),
  closeEditCardDialog: () =>
    set({ isEditCardDialogOpen: false, selectedCardId: null }),

  openEditColumnDialog: (columnId) =>
    set({ isEditColumnDialogOpen: true, selectedColumnId: columnId }),
  closeEditColumnDialog: () =>
    set({ isEditColumnDialogOpen: false, selectedColumnId: null }),

  openManageLabelsDialog: (cardId) =>
    set({ isManageLabelsDialogOpen: true, selectedCardId: cardId }),
  closeManageLabelsDialog: () => set({ isManageLabelsDialogOpen: false }),

  // Toast actions
  showToast: (message, type) => set({ toast: { message, type } }),
  hideToast: () => set({ toast: null }),

  // Loading actions
  setCreatingBoard: (isCreating) => set({ isCreatingBoard: isCreating }),
  setCreatingColumn: (isCreating) => set({ isCreatingColumn: isCreating }),
  setCreatingCard: (isCreating) => set({ isCreatingCard: isCreating }),

  // Reset
  reset: () =>
    set({
      isCreateBoardDialogOpen: false,
      isEditCardDialogOpen: false,
      isEditColumnDialogOpen: false,
      isManageLabelsDialogOpen: false,
      selectedCardId: null,
      selectedColumnId: null,
      toast: null,
      isCreatingBoard: false,
      isCreatingColumn: false,
      isCreatingCard: false,
    }),
}));
