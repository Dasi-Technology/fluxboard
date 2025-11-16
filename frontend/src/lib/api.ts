import axios, { AxiosInstance } from "axios";
import type { Board, Column, Card, BoardLabel } from "./types";

// Create axios instance with base configuration
const api: AxiosInstance = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api",
  timeout: 10000,
  headers: {
    "Content-Type": "application/json",
  },
});

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
  const response = await api.put<Board>(`/boards/share/${shareToken}`, {
    title: name,
  });
  return response.data;
};

// Column API endpoints
export const createColumn = async (
  boardId: string,
  title: string,
  position: number
): Promise<Column> => {
  const response = await api.post<Column>(`/boards/${boardId}/columns`, {
    title,
    position,
  });
  return response.data;
};

export const updateColumn = async (
  columnId: string,
  updates: Partial<Pick<Column, "title" | "position">>
): Promise<Column> => {
  const response = await api.put<Column>(`/columns/${columnId}`, updates);
  return response.data;
};

export const deleteColumn = async (columnId: string): Promise<void> => {
  await api.delete(`/columns/${columnId}`);
};

export const reorderColumns = async (
  boardId: string,
  columnPositions: Array<[string, number]>
): Promise<void> => {
  await api.patch(`/boards/${boardId}/columns/reorder`, {
    column_positions: columnPositions,
  });
};

// Card API endpoints
export const createCard = async (
  columnId: string,
  title: string,
  position: number
): Promise<Card> => {
  const response = await api.post<Card>(`/columns/${columnId}/cards`, {
    title,
    position,
  });
  return response.data;
};

export const reorderCards = async (
  columnId: string,
  cardPositions: Array<[string, number]>
): Promise<void> => {
  await api.patch(`/columns/${columnId}/cards/reorder`, {
    card_positions: cardPositions,
  });
};

export const updateCard = async (
  cardId: string,
  updates: Partial<
    Pick<Card, "title" | "description" | "position" | "column_id">
  >
): Promise<Card> => {
  const response = await api.put<Card>(`/cards/${cardId}`, updates);
  return response.data;
};

export const moveCard = async (
  cardId: string,
  columnId: string,
  position: number
): Promise<Card> => {
  const response = await api.patch<Card>(`/cards/${cardId}/move`, {
    column_id: columnId,
    position,
  });
  return response.data;
};

export const deleteCard = async (cardId: string): Promise<void> => {
  await api.delete(`/cards/${cardId}`);
};

// Board Label API endpoints
export const getBoardLabels = async (
  boardId: string
): Promise<BoardLabel[]> => {
  const response = await api.get<BoardLabel[]>(`/boards/${boardId}/labels`);
  return response.data;
};

export const createBoardLabel = async (
  boardId: string,
  name: string,
  color: string
): Promise<BoardLabel> => {
  const response = await api.post<BoardLabel>(`/boards/${boardId}/labels`, {
    name,
    color,
  });
  return response.data;
};

export const updateBoardLabel = async (
  labelId: string,
  updates: Partial<Pick<BoardLabel, "name" | "color">>
): Promise<BoardLabel> => {
  const response = await api.put<BoardLabel>(
    `/boards/labels/${labelId}`,
    updates
  );
  return response.data;
};

export const deleteBoardLabel = async (labelId: string): Promise<void> => {
  await api.delete(`/boards/labels/${labelId}`);
};

// Card Label Assignment endpoints
export const assignLabelToCard = async (
  cardId: string,
  labelId: string
): Promise<void> => {
  await api.post(`/cards/${cardId}/labels/${labelId}`);
};

export const unassignLabelFromCard = async (
  cardId: string,
  labelId: string
): Promise<void> => {
  await api.delete(`/cards/${cardId}/labels/${labelId}`);
};

// Legacy label endpoints (kept for backward compatibility during migration)
export const createLabel = async (
  cardId: string,
  name: string,
  color: string
): Promise<BoardLabel> => {
  const response = await api.post<BoardLabel>(`/cards/${cardId}/labels`, {
    name,
    color,
  });
  return response.data;
};

export const updateLabel = async (
  labelId: string,
  updates: Partial<Pick<BoardLabel, "name" | "color">>
): Promise<BoardLabel> => {
  const response = await api.put<BoardLabel>(`/labels/${labelId}`, updates);
  return response.data;
};

export const deleteLabel = async (labelId: string): Promise<void> => {
  await api.delete(`/labels/${labelId}`);
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
