"use client";

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Plus, X } from "lucide-react";
import { useUIStore } from "@/store/ui-store";
import { useBoard } from "@/hooks/use-board";

const PRESET_COLORS = [
  "#ef4444", // red
  "#f97316", // orange
  "#f59e0b", // amber
  "#eab308", // yellow
  "#84cc16", // lime
  "#22c55e", // green
  "#10b981", // emerald
  "#14b8a6", // teal
  "#06b6d4", // cyan
  "#0ea5e9", // sky
  "#3b82f6", // blue
  "#6366f1", // indigo
  "#8b5cf6", // violet
  "#a855f7", // purple
  "#d946ef", // fuchsia
  "#ec4899", // pink
];

export function ManageLabelsDialog() {
  const { isManageLabelsDialogOpen, selectedCardId, closeManageLabelsDialog } =
    useUIStore();
  const { createLabel } = useBoard();
  const [name, setName] = useState("");
  const [selectedColor, setSelectedColor] = useState(PRESET_COLORS[0]);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !selectedCardId) return;

    setIsLoading(true);
    try {
      await createLabel(selectedCardId, name.trim(), selectedColor);
      setName("");
      setSelectedColor(PRESET_COLORS[0]);
    } catch (error) {
      console.error("Failed to create label:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    if (!isLoading) {
      closeManageLabelsDialog();
      setName("");
      setSelectedColor(PRESET_COLORS[0]);
    }
  };

  return (
    <Dialog open={isManageLabelsDialogOpen} onOpenChange={handleClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Labels</DialogTitle>
          <DialogDescription>
            Create labels to organize your cards.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Label htmlFor="label-name">Label Name</Label>
            <Input
              id="label-name"
              placeholder="e.g., High Priority"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isLoading}
              autoFocus
            />
          </div>

          <div>
            <Label>Color</Label>
            <div className="grid grid-cols-8 gap-2 mt-2">
              {PRESET_COLORS.map((color) => (
                <button
                  key={color}
                  type="button"
                  className={`h-8 w-8 rounded-md border-2 transition-all ${
                    selectedColor === color
                      ? "border-foreground scale-110"
                      : "border-transparent hover:scale-105"
                  }`}
                  style={{ backgroundColor: color }}
                  onClick={() => setSelectedColor(color)}
                  disabled={isLoading}
                />
              ))}
            </div>
          </div>

          <div className="flex gap-2">
            <Button type="submit" disabled={!name.trim() || isLoading}>
              {isLoading ? "Adding..." : "Add Label"}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={handleClose}
              disabled={isLoading}
            >
              Done
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
