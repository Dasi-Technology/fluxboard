import axios from "axios";
import api from "./api";
import type { CardAttachment } from "./types";
import { getBoardPassword } from "./board-passwords";

/**
 * Request upload URL for a file
 */
export async function requestUploadUrl(
  cardId: string,
  filename: string,
  contentType: string,
  fileSize: number,
  shareToken?: string
): Promise<{ upload_url: string; attachment_id: string; s3_key: string }> {
  const headers: Record<string, string> = {};
  if (shareToken) {
    const password = getBoardPassword(shareToken);
    if (password) {
      headers["X-Board-Password"] = password;
    }
  }

  const response = await api.post(
    `/cards/${cardId}/attachments/upload-url`,
    {
      filename,
      content_type: contentType,
      file_size: fileSize,
    },
    { headers }
  );
  return response.data;
}

/**
 * Upload file directly to S3 with progress tracking
 */
export async function uploadToS3(
  uploadUrl: string,
  file: File,
  onProgress?: (progress: number) => void
): Promise<void> {
  console.log("[S3 Upload] Starting upload to S3");
  console.log("[S3 Upload] File type:", file.type);
  console.log("[S3 Upload] File size:", file.size);
  console.log(
    "[S3 Upload] Upload URL (first 100 chars):",
    uploadUrl.substring(0, 100)
  );

  try {
    await axios.put(uploadUrl, file, {
      headers: {
        "Content-Type": file.type,
      },
      onUploadProgress: (progressEvent) => {
        if (onProgress && progressEvent.total) {
          const percentCompleted = Math.round(
            (progressEvent.loaded * 100) / progressEvent.total
          );
          onProgress(percentCompleted);
        }
      },
    });
    console.log("[S3 Upload] Upload successful");
  } catch (error) {
    console.error("[S3 Upload] Upload failed:", error);
    if (axios.isAxiosError(error)) {
      console.error("[S3 Upload] Response status:", error.response?.status);
      console.error("[S3 Upload] Response data:", error.response?.data);
      console.error("[S3 Upload] Request headers:", error.config?.headers);
    }
    throw error;
  }
}

/**
 * Confirm upload completion
 */
export async function confirmUpload(
  cardId: string,
  attachmentId: string,
  shareToken?: string
): Promise<CardAttachment> {
  console.log("[Confirm Upload] Starting confirmation for:", {
    cardId,
    attachmentId,
    shareToken,
  });

  const headers: Record<string, string> = {};
  if (shareToken) {
    const password = getBoardPassword(shareToken);
    if (password) {
      headers["X-Board-Password"] = password;
      console.log("[Confirm Upload] Using board password");
    }
  }

  console.log("[Confirm Upload] Sending confirmation request to backend");
  const response = await api.post(
    `/cards/${cardId}/attachments/${attachmentId}/confirm`,
    null,
    { headers }
  );
  console.log("[Confirm Upload] Confirmation successful:", response.data);
  return response.data;
}

/**
 * List attachments for a card
 */
export async function getCardAttachments(
  cardId: string,
  shareToken?: string
): Promise<CardAttachment[]> {
  const headers: Record<string, string> = {};
  if (shareToken) {
    const password = getBoardPassword(shareToken);
    if (password) {
      headers["X-Board-Password"] = password;
    }
  }

  const response = await api.get<CardAttachment[]>(
    `/cards/${cardId}/attachments`,
    { headers }
  );
  return response.data;
}

/**
 * Get download URL for an attachment
 */
export async function getDownloadUrl(
  attachmentId: string,
  shareToken?: string
): Promise<{ download_url: string; expires_at: string }> {
  const headers: Record<string, string> = {};
  if (shareToken) {
    const password = getBoardPassword(shareToken);
    if (password) {
      headers["X-Board-Password"] = password;
    }
  }

  const response = await api.get(`/attachments/${attachmentId}/download-url`, {
    headers,
  });
  return response.data;
}

/**
 * Delete an attachment
 */
export async function deleteAttachment(
  attachmentId: string,
  shareToken?: string
): Promise<void> {
  const headers: Record<string, string> = {};
  if (shareToken) {
    const password = getBoardPassword(shareToken);
    if (password) {
      headers["X-Board-Password"] = password;
    }
  }

  await api.delete(`/attachments/${attachmentId}`, { headers });
}
