"use client";

import { useState } from "react";
import { Plus, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useBoard } from "@/hooks/use-board";

export function AddColumn() {
  const [isAdding, setIsAdding] = useState(false);
  const [title, setTitle] = useState("");
  const { createColumn } = useBoard();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    try {
      await createColumn(title.trim());
      setTitle("");
      setIsAdding(false);
    } catch (error) {
      console.error("Failed to create column:", error);
    }
  };

  const handleCancel = () => {
    setTitle("");
    setIsAdding(false);
  };

  if (!isAdding) {
    return (
      <div className="flex-shrink-0 w-80">
        <Button
          variant="outline"
          className="w-full h-full min-h-[100px] border-dashed"
          onClick={() => setIsAdding(true)}
        >
          <Plus className="h-4 w-4 mr-2" />
          Add column
        </Button>
      </div>
    );
  }

  return (
    <div className="flex-shrink-0 w-80 bg-muted/50 rounded-lg p-4">
      <form onSubmit={handleSubmit} className="space-y-2">
        <Input
          autoFocus
          placeholder="Enter column title..."
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              handleCancel();
            }
          }}
        />
        <div className="flex gap-2">
          <Button type="submit" size="sm" disabled={!title.trim()}>
            Add column
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={handleCancel}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </form>
    </div>
  );
}
