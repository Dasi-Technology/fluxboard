"use client";

import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Pencil, Trash2 } from "lucide-react";
import type { Card as CardType } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useUIStore } from "@/store/ui-store";
import { useBoard } from "@/hooks/use-board";
import { MarkdownRenderer } from "@/components/shared/markdown-renderer";

interface CardProps {
  card: CardType;
  isReadOnly?: boolean;
}

export function Card({ card, isReadOnly = false }: CardProps) {
  const { deleteCard } = useBoard();
  const { openEditCardDialog } = useUIStore();

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: card.id,
    data: {
      type: "card",
      item: card,
    },
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.3 : 1,
    scale: isDragging ? 0.95 : 1,
  };

  const handleEdit = () => {
    openEditCardDialog(card.id);
  };

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm("Are you sure you want to delete this card?")) {
      await deleteCard(card.id);
    }
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...(!isReadOnly && attributes)}
      {...(!isReadOnly && listeners)}
      className={`group relative bg-white border rounded-lg p-2.5 md:p-3 shadow-sm hover:shadow-md transition-all duration-200 ${
        !isReadOnly ? "cursor-move" : "cursor-default"
      } ${isDragging ? "ring-2 ring-blue-400 ring-offset-2" : ""}`}
      onClick={!isReadOnly ? handleEdit : undefined}
    >
      <div className="flex items-start justify-between gap-1.5 md:gap-2">
        <h3 className="text-xs md:text-sm font-medium flex-1 break-words">
          {card.title}
        </h3>
        {!isReadOnly && (
          <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={handleEdit}
            >
              <Pencil className="h-3 w-3" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 text-destructive"
              onClick={handleDelete}
            >
              <Trash2 className="h-3 w-3" />
            </Button>
          </div>
        )}
      </div>

      {card.description && (
        <div className="text-xs text-muted-foreground mt-2">
          <MarkdownRenderer content={card.description} />
        </div>
      )}

      {card.labels && card.labels.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-2">
          {card.labels.map((label) => (
            <Badge
              key={label.id}
              variant="secondary"
              className="text-xs"
              style={{
                backgroundColor: label.color,
                color: getContrastColor(label.color),
              }}
            >
              {label.name}
            </Badge>
          ))}
        </div>
      )}
    </div>
  );
}

// Helper function to determine text color based on background
function getContrastColor(hexColor: string): string {
  // Convert hex to RGB
  const r = parseInt(hexColor.slice(1, 3), 16);
  const g = parseInt(hexColor.slice(3, 5), 16);
  const b = parseInt(hexColor.slice(5, 7), 16);

  // Calculate relative luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;

  // Return black or white based on luminance
  return luminance > 0.5 ? "#000000" : "#ffffff";
}
