"use client";

import { useState, useCallback, useEffect } from "react";
import {
  Image as ImageIcon,
  X,
  Download,
  Loader2,
  Maximize2,
} from "lucide-react";
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

interface AttachmentThumbnailProps {
  cardId: string;
  attachments: CardAttachment[];
  shareToken?: string;
}

export function AttachmentThumbnail({
  cardId,
  attachments,
  shareToken,
}: AttachmentThumbnailProps) {
  const { isAuthenticated, user } = useAuthStore();
  const { showToast } = useUIStore();
  const { removeAttachment } = useBoardStore();

  const [selectedAttachment, setSelectedAttachment] =
    useState<CardAttachment | null>(null);
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [loadingDownload, setLoadingDownload] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [thumbnailUrls, setThumbnailUrls] = useState<Record<string, string>>(
    {}
  );
  const [loadingThumbnails, setLoadingThumbnails] = useState<Set<string>>(
    new Set()
  );

  // Load thumbnail images on mount
  useEffect(() => {
    if (!attachments || attachments.length === 0) return;

    const loadThumbnails = async () => {
      const visibleAttachments = attachments.slice(0, 3);

      for (const attachment of visibleAttachments) {
        if (thumbnailUrls[attachment.id]) continue;

        setLoadingThumbnails((prev) => new Set(prev).add(attachment.id));

        try {
          const { download_url } = await getDownloadUrl(
            attachment.id,
            shareToken
          );
          setThumbnailUrls((prev) => ({
            ...prev,
            [attachment.id]: download_url,
          }));
        } catch (error) {
          console.error("Failed to load thumbnail:", error);
        } finally {
          setLoadingThumbnails((prev) => {
            const newSet = new Set(prev);
            newSet.delete(attachment.id);
            return newSet;
          });
        }
      }
    };

    loadThumbnails();
  }, [attachments, shareToken]);

  const handleViewImage = useCallback(
    async (attachment: CardAttachment, e: React.MouseEvent) => {
      e.stopPropagation(); // Prevent card edit dialog from opening
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
    async (attachment: CardAttachment, e?: React.MouseEvent) => {
      if (e) e.stopPropagation();
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
    async (attachment: CardAttachment, e?: React.MouseEvent) => {
      if (e) e.stopPropagation();

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

  // Show first 3 attachments as thumbnails
  const visibleAttachments = attachments.slice(0, 3);
  const remainingCount = Math.max(0, attachments.length - 3);

  return (
    <>
      <div className="flex flex-wrap gap-1 mt-2">
        {visibleAttachments.map((attachment) => {
          const thumbnailUrl = thumbnailUrls[attachment.id];
          const isLoadingThumbnail = loadingThumbnails.has(attachment.id);

          return (
            <button
              key={attachment.id}
              onClick={(e) => handleViewImage(attachment, e)}
              className="group relative w-16 h-16 rounded border border-slate-200 overflow-hidden bg-slate-50 hover:border-slate-400 transition-colors flex items-center justify-center"
              disabled={loadingDownload === attachment.id || isLoadingThumbnail}
              title={attachment.original_filename}
            >
              {isLoadingThumbnail || loadingDownload === attachment.id ? (
                <Loader2 className="h-4 w-4 animate-spin text-slate-400" />
              ) : thumbnailUrl ? (
                <>
                  <img
                    src={thumbnailUrl}
                    alt={attachment.original_filename}
                    className="w-full h-full object-cover"
                  />
                  <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors flex items-center justify-center">
                    <Maximize2 className="h-4 w-4 text-white opacity-0 group-hover:opacity-100 transition-opacity drop-shadow-lg" />
                  </div>
                </>
              ) : (
                <>
                  <ImageIcon className="h-6 w-6 text-slate-400" />
                  <div className="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors flex items-center justify-center">
                    <Maximize2 className="h-3 w-3 text-white opacity-0 group-hover:opacity-100 transition-opacity" />
                  </div>
                </>
              )}
            </button>
          );
        })}
        {remainingCount > 0 && (
          <div className="w-16 h-16 rounded border border-slate-200 bg-slate-50 flex items-center justify-center text-xs text-slate-600 font-medium">
            +{remainingCount}
          </div>
        )}
      </div>

      {/* Full-screen image modal */}
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
              <div className="flex items-center justify-center h-96 bg-slate-100 rounded-md">
                <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
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
                    onClick={(e) =>
                      selectedAttachment && handleDelete(selectedAttachment, e)
                    }
                    disabled={deletingId === selectedAttachment?.id}
                  >
                    {deletingId === selectedAttachment?.id ? (
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    ) : (
                      <X className="h-4 w-4 mr-2" />
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

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
