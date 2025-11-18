"use client";

import { useState, useCallback, useRef } from "react";
import { Upload, X, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useAuthStore } from "@/store/auth-store";
import { useUIStore } from "@/store/ui-store";
import { useBoardStore } from "@/store/board-store";
import { AuthDialog } from "@/components/auth/auth-dialog";
import {
  requestUploadUrl,
  uploadToS3,
  confirmUpload,
} from "@/lib/attachment-api";

interface AttachmentUploadProps {
  cardId: string;
  shareToken?: string;
  maxAttachments?: number;
  currentAttachmentCount?: number;
}

const MAX_FILE_SIZE = 5 * 1024 * 1024; // 5MB
const ALLOWED_TYPES = ["image/jpeg", "image/png", "image/gif", "image/webp"];

export function AttachmentUpload({
  cardId,
  shareToken,
  maxAttachments = 10,
  currentAttachmentCount = 0,
}: AttachmentUploadProps) {
  const { isAuthenticated } = useAuthStore();
  const { showToast } = useUIStore();
  const { addAttachment } = useBoardStore();

  const [isDragging, setIsDragging] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [showAuthDialog, setShowAuthDialog] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);

  const validateFile = useCallback(
    (file: File): string | null => {
      if (!ALLOWED_TYPES.includes(file.type)) {
        return "Only image files (JPEG, PNG, GIF, WebP) are allowed";
      }
      if (file.size > MAX_FILE_SIZE) {
        return "File size must be under 5MB";
      }
      if (currentAttachmentCount >= maxAttachments) {
        return `Maximum ${maxAttachments} attachments per card`;
      }
      return null;
    },
    [currentAttachmentCount, maxAttachments]
  );

  const handleFileSelect = useCallback(
    (file: File) => {
      const error = validateFile(file);
      if (error) {
        showToast(error, "error");
        return;
      }

      setSelectedFile(file);

      // Create preview for images
      const reader = new FileReader();
      reader.onload = (e) => {
        setPreviewUrl(e.target?.result as string);
      };
      reader.readAsDataURL(file);
    },
    [validateFile, showToast]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);

      if (!isAuthenticated) {
        showToast("Please login to upload attachments", "error");
        return;
      }

      const files = Array.from(e.dataTransfer.files);
      if (files.length > 0) {
        handleFileSelect(files[0]);
      }
    },
    [isAuthenticated, handleFileSelect, showToast]
  );

  const handleFileInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (files && files.length > 0) {
        handleFileSelect(files[0]);
      }
    },
    [handleFileSelect]
  );

  const handleUpload = async () => {
    if (!selectedFile || !isAuthenticated) return;

    setIsUploading(true);
    setUploadProgress(0);

    try {
      // Step 1: Request upload URL
      const { upload_url, attachment_id } = await requestUploadUrl(
        cardId,
        selectedFile.name,
        selectedFile.type,
        selectedFile.size,
        shareToken
      );

      // Step 2: Upload to S3
      await uploadToS3(upload_url, selectedFile, (progress) => {
        setUploadProgress(progress);
      });

      // Step 3: Confirm upload
      const attachment = await confirmUpload(cardId, attachment_id, shareToken);

      // Add to store (SSE will also update, but this is optimistic)
      addAttachment(cardId, attachment);

      showToast("Attachment uploaded successfully", "success");

      // Clear selection
      setSelectedFile(null);
      setPreviewUrl(null);
      setUploadProgress(0);
    } catch (error: any) {
      console.error("Upload failed:", error);
      const message =
        error.response?.status === 401
          ? "Please login to upload attachments"
          : error.response?.status === 403
          ? "Board password required"
          : error.message || "Upload failed. Please try again.";
      showToast(message, "error");
    } finally {
      setIsUploading(false);
    }
  };

  const handleCancel = () => {
    setSelectedFile(null);
    setPreviewUrl(null);
    setUploadProgress(0);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  if (!isAuthenticated) {
    return (
      <>
        <div className="border-2 border-dashed rounded-lg p-3 text-center">
          <Upload className="h-6 w-6 mx-auto mb-1.5 text-muted-foreground" />
          <p className="text-xs text-muted-foreground mb-2">
            Please login to upload attachments
          </p>
          <Button onClick={() => setShowAuthDialog(true)} size="sm">
            Login
          </Button>
        </div>
        <AuthDialog
          isOpen={showAuthDialog}
          onClose={() => setShowAuthDialog(false)}
        />
      </>
    );
  }

  if (currentAttachmentCount >= maxAttachments) {
    return (
      <div className="border-2 border-dashed rounded-lg p-3 text-center">
        <p className="text-xs text-muted-foreground">
          Maximum {maxAttachments} attachments reached
        </p>
      </div>
    );
  }

  if (selectedFile && previewUrl) {
    return (
      <div className="border rounded-lg p-2 space-y-2">
        <div className="flex items-center gap-2">
          {/* Small thumbnail */}
          <div className="relative w-12 h-12 bg-muted rounded overflow-hidden flex-shrink-0">
            <img
              src={previewUrl}
              alt="Preview"
              className="w-full h-full object-cover"
            />
          </div>

          {/* File info */}
          <div className="flex-1 min-w-0">
            <p className="text-xs font-medium truncate">{selectedFile.name}</p>
            <p className="text-xs text-muted-foreground">
              {(selectedFile.size / 1024).toFixed(1)} KB
            </p>
          </div>

          {/* Cancel button */}
          <Button
            variant="ghost"
            size="icon"
            onClick={handleCancel}
            disabled={isUploading}
            className="flex-shrink-0 h-7 w-7"
          >
            <X className="h-3 w-3" />
          </Button>
        </div>

        {/* Progress bar */}
        {isUploading && (
          <div className="space-y-1">
            <div className="w-full bg-muted rounded-full h-1">
              <div
                className="bg-primary h-1 rounded-full transition-all"
                style={{ width: `${uploadProgress}%` }}
              />
            </div>
            <p className="text-xs text-center text-muted-foreground">
              {uploadProgress}%
            </p>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <Button
            onClick={handleUpload}
            disabled={isUploading}
            className="flex-1"
            size="sm"
          >
            {isUploading ? (
              <>
                <Loader2 className="h-3 w-3 mr-1.5 animate-spin" />
                Uploading...
              </>
            ) : (
              "Upload"
            )}
          </Button>
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={isUploading}
            size="sm"
          >
            Cancel
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`border-2 border-dashed rounded-lg p-3 text-center transition-colors ${
        isDragging
          ? "border-primary bg-primary/5"
          : "border-muted-foreground/25 hover:border-muted-foreground/50"
      }`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <Upload className="h-6 w-6 mx-auto mb-1.5 text-muted-foreground" />
      <p className="text-xs text-muted-foreground mb-0.5">
        Drag and drop or click to select
      </p>
      <p className="text-xs text-muted-foreground mb-2">
        JPEG, PNG, GIF, WebP • Max 5MB
      </p>
      <input
        ref={fileInputRef}
        type="file"
        accept={ALLOWED_TYPES.join(",")}
        onChange={handleFileInputChange}
        className="hidden"
      />
      <Button
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          fileInputRef.current?.click();
        }}
        variant="secondary"
        size="sm"
        type="button"
      >
        Select File
      </Button>
    </div>
  );
}
