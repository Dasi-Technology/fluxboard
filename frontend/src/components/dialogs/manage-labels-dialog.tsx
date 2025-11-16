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
import { Plus, Trash2, Edit2, Check, X } from "lucide-react";
import { useBoardStore } from "@/store/board-store";
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
  const { board } = useBoardStore();
  const { isManageLabelsDialogOpen, closeManageLabelsDialog } = useUIStore();
  const { createBoardLabel, updateBoardLabel, deleteBoardLabel } = useBoard();

  const [name, setName] = useState("");
  const [selectedColor, setSelectedColor] = useState(PRESET_COLORS[0]);
  const [isCreating, setIsCreating] = useState(false);

  const [editingLabelId, setEditingLabelId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editColor, setEditColor] = useState("");

  const boardLabels = board?.labels || [];

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    setIsCreating(true);
    try {
      await createBoardLabel(name.trim(), selectedColor);
      setName("");
      setSelectedColor(PRESET_COLORS[0]);
    } catch (error) {
      console.error("Failed to create label:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const startEdit = (
    labelId: string,
    currentName: string,
    currentColor: string
  ) => {
    setEditingLabelId(labelId);
    setEditName(currentName);
    setEditColor(currentColor);
  };

  const cancelEdit = () => {
    setEditingLabelId(null);
    setEditName("");
    setEditColor("");
  };

  const saveEdit = async (labelId: string) => {
    if (!editName.trim()) return;

    try {
      await updateBoardLabel(labelId, {
        name: editName.trim(),
        color: editColor,
      });
      setEditingLabelId(null);
      setEditName("");
      setEditColor("");
    } catch (error) {
      console.error("Failed to update label:", error);
    }
  };

  const handleDelete = async (labelId: string) => {
    if (!confirm("Delete this label? It will be removed from all cards.")) {
      return;
    }

    try {
      await deleteBoardLabel(labelId);
    } catch (error) {
      console.error("Failed to delete label:", error);
    }
  };

  const handleClose = () => {
    if (!isCreating) {
      closeManageLabelsDialog();
      setName("");
      setSelectedColor(PRESET_COLORS[0]);
      cancelEdit();
    }
  };

  return (
    <Dialog open={isManageLabelsDialogOpen} onOpenChange={handleClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Manage Board Labels</DialogTitle>
          <DialogDescription>
            Create and manage labels for this board. Labels can be assigned to
            any card.
          </DialogDescription>
        </DialogHeader>

        {/* Create new label form */}
        <form onSubmit={handleCreate} className="space-y-3 border-b pb-4">
          <h3 className="text-sm font-medium">Create New Label</h3>
          <div>
            <Label htmlFor="label-name">Label Name</Label>
            <Input
              id="label-name"
              placeholder="e.g., High Priority, Bug, Feature"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isCreating}
            />
          </div>

          <div>
            <Label>Color</Label>
            <div className="grid grid-cols-8 gap-2 mt-2">
              {PRESET_COLORS.map((color) => (
                <button
                  key={color}
                  type="button"
                  className={`h-10 w-10 rounded-md border-2 transition-all ${
                    selectedColor === color
                      ? "border-foreground scale-110 shadow-lg"
                      : "border-transparent hover:scale-105"
                  }`}
                  style={{ backgroundColor: color }}
                  onClick={() => setSelectedColor(color)}
                  disabled={isCreating}
                  title={color}
                />
              ))}
            </div>
          </div>

          <Button
            type="submit"
            disabled={!name.trim() || isCreating}
            className="w-full"
          >
            <Plus className="h-4 w-4 mr-2" />
            {isCreating ? "Creating..." : "Create Label"}
          </Button>
        </form>

        {/* Existing labels list */}
        <div className="space-y-3">
          <h3 className="text-sm font-medium">
            Board Labels ({boardLabels.length})
          </h3>

          {boardLabels.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-8">
              No labels yet. Create your first label above!
            </p>
          ) : (
            <div className="space-y-2">
              {boardLabels.map((label) => (
                <div
                  key={label.id}
                  className="flex items-center gap-3 p-3 border rounded-lg hover:bg-muted/50 transition-colors"
                >
                  {editingLabelId === label.id ? (
                    <>
                      {/* Editing mode */}
                      <Input
                        value={editName}
                        onChange={(e) => setEditName(e.target.value)}
                        className="flex-1"
                        autoFocus
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            saveEdit(label.id);
                          } else if (e.key === "Escape") {
                            cancelEdit();
                          }
                        }}
                      />
                      <div className="flex gap-1">
                        {PRESET_COLORS.slice(0, 6).map((color) => (
                          <button
                            key={color}
                            type="button"
                            className={`h-6 w-6 rounded border-2 transition-all ${
                              editColor === color
                                ? "border-foreground scale-110"
                                : "border-transparent"
                            }`}
                            style={{ backgroundColor: color }}
                            onClick={() => setEditColor(color)}
                          />
                        ))}
                      </div>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => saveEdit(label.id)}
                      >
                        <Check className="h-4 w-4" />
                      </Button>
                      <Button size="sm" variant="ghost" onClick={cancelEdit}>
                        <X className="h-4 w-4" />
                      </Button>
                    </>
                  ) : (
                    <>
                      {/* Display mode */}
                      <div
                        className="h-8 w-8 rounded-md flex-shrink-0"
                        style={{ backgroundColor: label.color }}
                      />
                      <span className="flex-1 font-medium">{label.name}</span>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() =>
                          startEdit(label.id, label.name, label.color)
                        }
                      >
                        <Edit2 className="h-4 w-4" />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => handleDelete(label.id)}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="flex justify-end pt-4 border-t">
          <Button variant="outline" onClick={handleClose}>
            Done
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
