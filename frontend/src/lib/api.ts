import axios, { AxiosInstance } from "axios";
import type { Board, Column, Card, Label } from "./types";

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

export const updateCard = async (
  cardId: string,
  updates: Partial<
    Pick<Card, "title" | "description" | "position" | "column_id">
  >
): Promise<Card> => {
  const response = await api.put<Card>(`/cards/${cardId}`, updates);
  return response.data;
};

export const deleteCard = async (cardId: string): Promise<void> => {
  await api.delete(`/cards/${cardId}`);
};

// Label API endpoints
export const createLabel = async (
  cardId: string,
  name: string,
  color: string
): Promise<Label> => {
  const response = await api.post<Label>(`/cards/${cardId}/labels`, {
    name,
    color,
  });
  return response.data;
};

export const updateLabel = async (
  labelId: string,
  updates: Partial<Pick<Label, "name" | "color">>
): Promise<Label> => {
  const response = await api.put<Label>(`/labels/${labelId}`, updates);
  return response.data;
};

export const deleteLabel = async (labelId: string): Promise<void> => {
  await api.delete(`/labels/${labelId}`);
};

export default api;
