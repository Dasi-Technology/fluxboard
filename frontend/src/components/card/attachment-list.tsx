"use client";

import { useState, useCallback } from "react";
import { Download, Trash2, Loader2, Image as ImageIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useAuthStore } from "@/store/auth-store";
import { useUIStore } from "@/store/ui-store";
import { useBoardStore } from "@/store/board-store";
import type { CardAttachment } from "@/lib/types";
import { getDownloadUrl, deleteAttachment } from "@/lib/attachment-api";

interface AttachmentListProps {
  cardId: string;
  attachments: CardAttachment[];
  shareToken?: string;
  uploadedBy?: string; // Current user's ID for permission checks
}

export function AttachmentList({
  cardId,
  attachments,
  shareToken,
  uploadedBy,
}: AttachmentListProps) {
  const { isAuthenticated, user } = useAuthStore();
  const { showToast } = useUIStore();
  const { board, removeAttachment } = useBoardStore();

  const [selectedAttachment, setSelectedAttachment] =
    useState<CardAttachment | null>(null);
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [loadingDownload, setLoadingDownload] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const handleViewImage = useCallback(
    async (attachment: CardAttachment) => {
      setSelectedAttachment(attachment);
      setLoadingDownload(attachment.id);

      try {
        const { download_url } = await getDownloadUrl(
          attachment.id,
          shareToken
        );
        setDownloadUrl(download_url);
      } catch (error: any) {
        console.error("Failed to get download URL:", error);
        showToast("Failed to load image", "error");
        setSelectedAttachment(null);
      } finally {
        setLoadingDownload(null);
      }
    },
    [shareToken, showToast]
  );

  const handleDownload = useCallback(
    async (attachment: CardAttachment) => {
      setLoadingDownload(attachment.id);

      try {
        const { download_url } = await getDownloadUrl(
          attachment.id,
          shareToken
        );

        // Open download URL in new window
        const link = document.createElement("a");
        link.href = download_url;
        link.download = attachment.original_filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);

        showToast("Download started", "success");
      } catch (error: any) {
        console.error("Failed to download:", error);
        showToast("Failed to download attachment", "error");
      } finally {
        setLoadingDownload(null);
      }
    },
    [shareToken, showToast]
  );

  const handleDelete = useCallback(
    async (attachment: CardAttachment) => {
      if (!isAuthenticated) {
        showToast("Please login to delete attachments", "error");
        return;
      }

      // Check permissions: user must be uploader
      const canDelete = user?.id === attachment.uploaded_by;

      if (!canDelete) {
        showToast(
          "You don't have permission to delete this attachment",
          "error"
        );
        return;
      }

      if (!confirm("Are you sure you want to delete this attachment?")) {
        return;
      }

      setDeletingId(attachment.id);

      try {
        await deleteAttachment(attachment.id, shareToken);
        removeAttachment(cardId, attachment.id);
        showToast("Attachment deleted", "success");

        // Close modal if deleting the currently viewed attachment
        if (selectedAttachment?.id === attachment.id) {
          setSelectedAttachment(null);
          setDownloadUrl(null);
        }
      } catch (error: any) {
        console.error("Failed to delete:", error);
        const message =
          error.response?.status === 403
            ? "You don't have permission to delete this attachment"
            : "Failed to delete attachment";
        showToast(message, "error");
      } finally {
        setDeletingId(null);
      }
    },
    [
      isAuthenticated,
      user,
      board,
      cardId,
      shareToken,
      selectedAttachment,
      removeAttachment,
      showToast,
    ]
  );

  const handleCloseModal = () => {
    setSelectedAttachment(null);
    setDownloadUrl(null);
  };

  if (!attachments || attachments.length === 0) {
    return null;
  }

  return (
    <>
      <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
        {attachments.map((attachment) => {
          const canDelete =
            isAuthenticated && user?.id === attachment.uploaded_by;

          return (
            <div
              key={attachment.id}
              className="group relative border rounded-lg overflow-hidden bg-muted hover:border-primary transition-colors"
            >
              {/* Thumbnail */}
              <button
                onClick={() => handleViewImage(attachment)}
                className="w-full aspect-video bg-muted flex items-center justify-center relative overflow-hidden"
                disabled={loadingDownload === attachment.id}
              >
                {loadingDownload === attachment.id ? (
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                ) : (
                  <ImageIcon className="h-6 w-6 text-muted-foreground" />
                )}
                <div className="absolute inset-0 bg-gradient-to-t from-black/50 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
              </button>

              {/* Info */}
              <div className="p-2 space-y-1">
                <p
                  className="text-xs font-medium truncate"
                  title={attachment.original_filename}
                >
                  {attachment.original_filename}
                </p>
                <p className="text-xs text-muted-foreground">
                  {formatFileSize(attachment.file_size)}
                </p>
              </div>

              {/* Actions */}
              <div className="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <Button
                  size="icon"
                  variant="secondary"
                  className="h-7 w-7"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDownload(attachment);
                  }}
                  disabled={loadingDownload === attachment.id}
                  title="Download"
                >
                  {loadingDownload === attachment.id ? (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  ) : (
                    <Download className="h-3 w-3" />
                  )}
                </Button>
                {canDelete && (
                  <Button
                    size="icon"
                    variant="destructive"
                    className="h-7 w-7"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(attachment);
                    }}
                    disabled={deletingId === attachment.id}
                    title="Delete"
                  >
                    {deletingId === attachment.id ? (
                      <Loader2 className="h-3 w-3 animate-spin" />
                    ) : (
                      <Trash2 className="h-3 w-3" />
                    )}
                  </Button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {/* Full-size image modal */}
      <Dialog open={!!selectedAttachment} onOpenChange={handleCloseModal}>
        <DialogContent className="max-w-5xl max-h-[90vh]">
          <DialogHeader>
            <DialogTitle className="truncate pr-8">
              {selectedAttachment?.original_filename}
            </DialogTitle>
          </DialogHeader>
          <div
            className="relative w-full"
            style={{ maxHeight: "calc(90vh - 200px)" }}
          >
            {downloadUrl ? (
              <img
                src={downloadUrl}
                alt={selectedAttachment?.original_filename}
                className="w-full h-full object-contain rounded-md"
                style={{ maxHeight: "calc(90vh - 200px)" }}
              />
            ) : (
              <div className="flex items-center justify-center h-96 bg-muted rounded-md">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            )}
          </div>
          <div className="flex justify-between items-center gap-4 pt-4 border-t">
            <div className="text-sm text-slate-600">
              {selectedAttachment && (
                <>
                  {selectedAttachment.content_type} •{" "}
                  {formatFileSize(selectedAttachment.file_size)}
                </>
              )}
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() =>
                  selectedAttachment && handleDownload(selectedAttachment)
                }
                disabled={
                  !downloadUrl || loadingDownload === selectedAttachment?.id
                }
              >
                {loadingDownload === selectedAttachment?.id ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <Download className="h-4 w-4 mr-2" />
                )}
                Download
              </Button>
              {selectedAttachment &&
                isAuthenticated &&
                user?.id === selectedAttachment.uploaded_by && (
                  <Button
                    variant="destructive"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(selectedAttachment);
                    }}
                    disabled={deletingId === selectedAttachment?.id}
                  >
                    {deletingId === selectedAttachment?.id ? (
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    ) : (
                      <Trash2 className="h-4 w-4 mr-2" />
                    )}
                    Delete
                  </Button>
                )}
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
