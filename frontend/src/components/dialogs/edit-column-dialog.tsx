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
import { useUIStore } from "@/store/ui-store";
import { useBoardStore } from "@/store/board-store";
import { useBoard } from "@/hooks/use-board";

export function EditColumnDialog() {
  const { isEditColumnDialogOpen, selectedColumnId, closeEditColumnDialog } =
    useUIStore();
  const { board } = useBoardStore();
  const { updateColumn } = useBoard();
  const [title, setTitle] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const column = board?.columns?.find((col) => col.id === selectedColumnId);

  useEffect(() => {
    if (column) {
      setTitle(column.title);
    }
  }, [column]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim() || !selectedColumnId) return;

    setIsLoading(true);
    try {
      await updateColumn(selectedColumnId, { title: title.trim() });
      closeEditColumnDialog();
    } catch (error) {
      console.error("Failed to update column:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    if (!isLoading) {
      closeEditColumnDialog();
      setTitle("");
    }
  };

  return (
    <Dialog open={isEditColumnDialogOpen} onOpenChange={handleClose}>
      <DialogContent>
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Rename Column</DialogTitle>
            <DialogDescription>
              Change the name of this column.
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label htmlFor="column-title">Column Title</Label>
            <Input
              id="column-title"
              placeholder="e.g., To Do"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              disabled={isLoading}
              autoFocus
            />
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
              {isLoading ? "Saving..." : "Save"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
