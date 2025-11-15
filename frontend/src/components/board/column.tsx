"use client";

import { useDroppable } from "@dnd-kit/core";
import {
  SortableContext,
  verticalListSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { MoreHorizontal, Pencil, Trash2 } from "lucide-react";
import type { Column as ColumnType } from "@/lib/types";
import { Card } from "./card";
import { AddCard } from "./add-card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useUIStore } from "@/store/ui-store";
import { useBoard } from "@/hooks/use-board";

interface ColumnProps {
  column: ColumnType;
}

export function Column({ column }: ColumnProps) {
  const { deleteColumn } = useBoard();
  const { openEditColumnDialog } = useUIStore();

  const {
    attributes,
    listeners,
    setNodeRef: setSortableRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: column.id,
    data: {
      type: "column",
      item: column,
    },
  });

  const { setNodeRef: setDroppableRef } = useDroppable({
    id: `column-container-${column.id}`,
    data: {
      type: "column-container",
      columnId: column.id,
    },
  });

  const setNodeRef = (node: HTMLElement | null) => {
    setSortableRef(node);
    setDroppableRef(node);
  };

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const cardIds = column.cards?.map((card) => card.id) || [];

  const handleEdit = () => {
    openEditColumnDialog(column.id);
  };

  const handleDelete = async () => {
    if (
      confirm(
        "Are you sure you want to delete this column? All cards will be deleted."
      )
    ) {
      await deleteColumn(column.id);
    }
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex-shrink-0 w-80 bg-muted/50 rounded-lg p-4 flex flex-col max-h-full"
    >
      <div
        {...attributes}
        {...listeners}
        className="flex items-center justify-between mb-3 cursor-grab active:cursor-grabbing"
      >
        <h2 className="font-semibold text-lg">{column.title}</h2>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={handleEdit}>
              <Pencil className="h-4 w-4 mr-2" />
              Rename
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={handleDelete}
              className="text-destructive"
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Delete
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="flex-1 overflow-y-auto space-y-2 mb-2 min-h-[100px]">
        <SortableContext items={cardIds} strategy={verticalListSortingStrategy}>
          {column.cards?.map((card) => (
            <Card key={card.id} card={card} />
          ))}
        </SortableContext>
      </div>

      <AddCard columnId={column.id} />
    </div>
  );
}
