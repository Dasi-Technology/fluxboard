"use client";

import { useState, useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Plus, X } from "lucide-react";
import { useUIStore } from "@/store/ui-store";
import { useBoardStore } from "@/store/board-store";
import { useBoard } from "@/hooks/use-board";
import type { Card } from "@/lib/types";

export function EditCardDialog() {
  const {
    isEditCardDialogOpen,
    selectedCardId,
    closeEditCardDialog,
    openManageLabelsDialog,
  } = useUIStore();
  const { board } = useBoardStore();
  const { updateCard, deleteLabel } = useBoard();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  // Find the card
  let card: Card | undefined;
  board?.columns?.forEach((col) => {
    const foundCard = col.cards?.find((c) => c.id === selectedCardId);
    if (foundCard) card = foundCard;
  });

  useEffect(() => {
    if (card) {
      setTitle(card.title);
      setDescription(card.description || "");
    }
  }, [card]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim() || !selectedCardId) return;

    setIsLoading(true);
    try {
      await updateCard(selectedCardId, {
        title: title.trim(),
        description: description.trim() || null,
      });
      closeEditCardDialog();
    } catch (error) {
      console.error("Failed to update card:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRemoveLabel = async (labelId: string) => {
    try {
      await deleteLabel(labelId);
    } catch (error) {
      console.error("Failed to remove label:", error);
    }
  };

  const handleManageLabels = () => {
    if (selectedCardId) {
      openManageLabelsDialog(selectedCardId);
    }
  };

  const handleClose = () => {
    if (!isLoading) {
      closeEditCardDialog();
      setTitle("");
      setDescription("");
    }
  };

  return (
    <Dialog open={isEditCardDialogOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-2xl">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Edit Card</DialogTitle>
            <DialogDescription>
              Make changes to your card details.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div>
              <Label htmlFor="card-title">Title</Label>
              <Input
                id="card-title"
                placeholder="Card title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                disabled={isLoading}
                autoFocus
              />
            </div>

            <div>
              <Label htmlFor="card-description">Description</Label>
              <Textarea
                id="card-description"
                placeholder="Add a more detailed description..."
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                disabled={isLoading}
                rows={4}
              />
            </div>

            <div>
              <div className="flex items-center justify-between mb-2">
                <Label>Labels</Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={handleManageLabels}
                >
                  <Plus className="h-4 w-4 mr-2" />
                  Manage Labels
                </Button>
              </div>
              {card?.labels && card.labels.length > 0 ? (
                <div className="flex flex-wrap gap-2">
                  {card.labels.map((label) => (
                    <Badge
                      key={label.id}
                      variant="secondary"
                      className="text-sm pl-3 pr-1"
                      style={{
                        backgroundColor: label.color,
                        color: getContrastColor(label.color),
                      }}
                    >
                      {label.name}
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="h-4 w-4 ml-1 hover:bg-transparent"
                        onClick={() => handleRemoveLabel(label.id)}
                      >
                        <X className="h-3 w-3" />
                      </Button>
                    </Badge>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">
                  No labels yet. Click "Manage Labels" to add some.
                </p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={handleClose}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!title.trim() || isLoading}>
              {isLoading ? "Saving..." : "Save Changes"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

// Helper function to determine text color based on background
function getContrastColor(hexColor: string): string {
  const r = parseInt(hexColor.slice(1, 3), 16);
  const g = parseInt(hexColor.slice(3, 5), 16);
  const b = parseInt(hexColor.slice(5, 7), 16);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.5 ? "#000000" : "#ffffff";
}
