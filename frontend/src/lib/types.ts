/**
 * Core type definitions for the Fluxboard application
 */

export interface Board {
  id: string;
  title: string;
  share_token: string;
  password: string;
  is_locked: boolean;
  created_at: string;
  updated_at: string;
  columns?: Column[];
  labels?: BoardLabel[];
}

export interface Column {
  id: string;
  board_id: string;
  title: string;
  position: number;
  created_at: string;
  updated_at: string;
  cards?: Card[];
}

export interface Card {
  id: string;
  column_id: string;
  title: string;
  description?: string | null;
  position: number;
  created_at: string;
  updated_at: string;
  labels?: BoardLabel[];
  attachments?: CardAttachment[];
}

export interface CardAttachment {
  id: string;
  card_id: string;
  uploaded_by: string;
  filename: string;
  original_filename: string;
  content_type: string;
  file_size: number;
  s3_key: string;
  is_confirmed: boolean;
  created_at: string;
  updated_at: string;
}

export interface BoardLabel {
  id: string;
  board_id: string;
  name: string;
  color: string;
  created_at: string;
  updated_at: string;
}

// Keep Label as alias for backward compatibility during transition
export type Label = BoardLabel;

/**
 * API Request/Response types
 */

export interface CreateBoardRequest {
  title: string;
}

export interface UpdateBoardRequest {
  title?: string;
}

export interface SetLockStateRequest {
  password: string;
  is_locked: boolean;
}

export interface CreateColumnRequest {
  board_id: string;
  title: string;
  position: number;
}

export interface UpdateColumnRequest {
  title?: string;
  position?: number;
}

export interface CreateCardRequest {
  column_id: string;
  title: string;
  position: number;
}

export interface UpdateCardRequest {
  title?: string;
  description?: string | null;
  position?: number;
  column_id?: string;
}

export interface CreateLabelRequest {
  card_id: string;
  name: string;
  color: string;
}

export interface UpdateLabelRequest {
  name?: string;
  color?: string;
}

/**
 * Drag and Drop types
 */

export interface DragStartEvent {
  active: {
    id: string;
    data: {
      current?: {
        type: "card" | "column";
        item: Card | Column;
      };
    };
  };
}

export interface DragEndEvent {
  active: {
    id: string;
    data: {
      current?: {
        type: "card" | "column";
        item: Card | Column;
      };
    };
  };
  over: {
    id: string;
    data: {
      current?: {
        type: "card" | "column" | "column-container";
        item?: Card | Column;
        columnId?: string;
      };
    };
  } | null;
}

/**
 * State management types
 */

export interface BoardState {
  board: Board | null;
  isLoading: boolean;
  error: string | null;
  setBoard: (board: Board) => void;
  updateBoard: (updates: Partial<Board>) => void;
  addColumn: (column: Column) => void;
  updateColumn: (columnId: string, updates: Partial<Column>) => void;
  deleteColumn: (columnId: string) => void;
  addCard: (card: Card) => void;
  updateCard: (cardId: string, updates: Partial<Card>) => void;
  deleteCard: (cardId: string) => void;
  moveCard: (cardId: string, newColumnId: string, newPosition: number) => void;
  moveColumn: (columnId: string, newPosition: number) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}
