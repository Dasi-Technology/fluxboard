import axios, { AxiosInstance } from "axios";
import type {
  Board,
  Column,
  Card,
  BoardLabel,
  SetLockStateRequest,
} from "./types";
import { getBoardPassword } from "./board-passwords";

// Create axios instance with base configuration
const api: AxiosInstance = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api",
  timeout: 10000,
  headers: {
    "Content-Type": "application/json",
  },
});

/**
 * Helper function to get headers with board password if available
 * @param shareToken - The board's share token
 * @returns Headers object with password if available
 */
function getHeadersWithPassword(shareToken?: string): Record<string, string> {
  const headers: Record<string, string> = {};

  if (shareToken) {
    const password = getBoardPassword(shareToken);
    if (password) {
      headers["X-Board-Password"] = password;
    }
  }

  return headers;
}

// Board API endpoints
export const createBoard = async (name: string): Promise<Board> => {
  const response = await api.post<Board>("/boards", { title: name });
  return response.data;
};

export const getBoard = async (shareToken: string): Promise<Board> => {
  const response = await api.get<Board>(`/boards/share/${shareToken}`);
  return response.data;
};

export const updateBoardName = async (
  shareToken: string,
  name: string
): Promise<Board> => {
  const response = await api.put<Board>(
    `/boards/share/${shareToken}`,
    { title: name },
    { headers: getHeadersWithPassword(shareToken) }
  );
  return response.data;
};

export const setBoardLockState = async (
  shareToken: string,
  password: string,
  isLocked: boolean
): Promise<Board> => {
  const response = await api.post<Board>(`/boards/share/${shareToken}/lock`, {
    password,
    is_locked: isLocked,
  } as SetLockStateRequest);
  return response.data;
};

// Column API endpoints
export const createColumn = async (
  boardId: string,
  title: string,
  position: number,
  shareToken?: string
): Promise<Column> => {
  const response = await api.post<Column>(
    `/boards/${boardId}/columns`,
    { title, position },
    { headers: getHeadersWithPassword(shareToken) }
  );
  return response.data;
};

export const updateColumn = async (
  columnId: string,
  updates: Partial<Pick<Column, "title" | "position">>,
  shareToken?: string
): Promise<Column> => {
  const response = await api.put<Column>(`/columns/${columnId}`, updates, {
    headers: getHeadersWithPassword(shareToken),
  });
  return response.data;
};

export const deleteColumn = async (
  columnId: string,
  shareToken?: string
): Promise<void> => {
  await api.delete(`/columns/${columnId}`, {
    headers: getHeadersWithPassword(shareToken),
  });
};

export const reorderColumns = async (
  boardId: string,
  columnPositions: Array<[string, number]>,
  shareToken?: string
): Promise<void> => {
  await api.patch(
    `/boards/${boardId}/columns/reorder`,
    { column_positions: columnPositions },
    { headers: getHeadersWithPassword(shareToken) }
  );
};

// Card API endpoints
export const createCard = async (
  columnId: string,
  title: string,
  position: number,
  shareToken?: string
): Promise<Card> => {
  const response = await api.post<Card>(
    `/columns/${columnId}/cards`,
    { title, position },
    { headers: getHeadersWithPassword(shareToken) }
  );
  return response.data;
};

export const reorderCards = async (
  columnId: string,
  cardPositions: Array<[string, number]>,
  shareToken?: string
): Promise<void> => {
  await api.patch(
    `/columns/${columnId}/cards/reorder`,
    { card_positions: cardPositions },
    { headers: getHeadersWithPassword(shareToken) }
  );
};

export const updateCard = async (
  cardId: string,
  updates: Partial<
    Pick<Card, "title" | "description" | "position" | "column_id">
  >,
  shareToken?: string
): Promise<Card> => {
  const response = await api.put<Card>(`/cards/${cardId}`, updates, {
    headers: getHeadersWithPassword(shareToken),
  });
  return response.data;
};

export const moveCard = async (
  cardId: string,
  columnId: string,
  position: number,
  shareToken?: string
): Promise<Card> => {
  const response = await api.patch<Card>(
    `/cards/${cardId}/move`,
    { column_id: columnId, position },
    { headers: getHeadersWithPassword(shareToken) }
  );
  return response.data;
};

export const deleteCard = async (
  cardId: string,
  shareToken?: string
): Promise<void> => {
  await api.delete(`/cards/${cardId}`, {
    headers: getHeadersWithPassword(shareToken),
  });
};

// Board Label API endpoints
export const getBoardLabels = async (
  boardId: string,
  shareToken?: string
): Promise<BoardLabel[]> => {
  const response = await api.get<BoardLabel[]>(`/boards/${boardId}/labels`, {
    headers: getHeadersWithPassword(shareToken),
  });
  return response.data;
};

export const createBoardLabel = async (
  boardId: string,
  name: string,
  color: string,
  shareToken?: string
): Promise<BoardLabel> => {
  const response = await api.post<BoardLabel>(
    `/boards/${boardId}/labels`,
    {
      name,
      color,
    },
    {
      headers: getHeadersWithPassword(shareToken),
    }
  );
  return response.data;
};

export const updateBoardLabel = async (
  labelId: string,
  updates: Partial<Pick<BoardLabel, "name" | "color">>,
  shareToken?: string
): Promise<BoardLabel> => {
  const response = await api.put<BoardLabel>(
    `/boards/labels/${labelId}`,
    updates,
    {
      headers: getHeadersWithPassword(shareToken),
    }
  );
  return response.data;
};

export const deleteBoardLabel = async (
  labelId: string,
  shareToken?: string
): Promise<void> => {
  await api.delete(`/boards/labels/${labelId}`, {
    headers: getHeadersWithPassword(shareToken),
  });
};

// Card Label Assignment endpoints
export const assignLabelToCard = async (
  cardId: string,
  labelId: string,
  shareToken?: string
): Promise<void> => {
  await api.post(`/cards/${cardId}/labels/${labelId}`, null, {
    headers: getHeadersWithPassword(shareToken),
  });
};

export const unassignLabelFromCard = async (
  cardId: string,
  labelId: string,
  shareToken?: string
): Promise<void> => {
  await api.delete(`/cards/${cardId}/labels/${labelId}`, {
    headers: getHeadersWithPassword(shareToken),
  });
};

// Legacy label endpoints (kept for backward compatibility during migration)
export const createLabel = async (
  cardId: string,
  name: string,
  color: string,
  shareToken?: string
): Promise<BoardLabel> => {
  const response = await api.post<BoardLabel>(
    `/cards/${cardId}/labels`,
    { name, color },
    { headers: getHeadersWithPassword(shareToken) }
  );
  return response.data;
};

export const updateLabel = async (
  labelId: string,
  updates: Partial<Pick<BoardLabel, "name" | "color">>,
  shareToken?: string
): Promise<BoardLabel> => {
  const response = await api.put<BoardLabel>(`/labels/${labelId}`, updates, {
    headers: getHeadersWithPassword(shareToken),
  });
  return response.data;
};

export const deleteLabel = async (
  labelId: string,
  shareToken?: string
): Promise<void> => {
  await api.delete(`/labels/${labelId}`, {
    headers: getHeadersWithPassword(shareToken),
  });
};

// AI generation endpoints
export type DescriptionFormat = "bullets" | "long";

export interface GenerateDescriptionRequest {
  title: string;
  context?: string;
  format: DescriptionFormat;
}

export interface GenerateDescriptionResponse {
  description: string;
}

export const generateDescription = async (
  request: GenerateDescriptionRequest
): Promise<GenerateDescriptionResponse> => {
  const response = await api.post<GenerateDescriptionResponse>(
    "/cards/ai/generate-description",
    request
  );
  return response.data;
};

export default api;
