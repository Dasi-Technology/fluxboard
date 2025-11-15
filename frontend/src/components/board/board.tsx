"use client";

import { DndContext, DragOverlay } from "@dnd-kit/core";
import {
  SortableContext,
  horizontalListSortingStrategy,
} from "@dnd-kit/sortable";
import { Column } from "./column";
import { AddColumn } from "./add-column";
import { Card } from "./card";
import { useBoardStore } from "@/store/board-store";
import { useBoard } from "@/hooks/use-board";
import { useDragAndDrop } from "@/hooks/use-drag-and-drop";
import type { Card as CardType } from "@/lib/types";

export function Board() {
  const { board } = useBoardStore();
  const { moveCard, reorderColumn } = useBoard();

  const {
    sensors,
    activeId,
    activeType,
    handleDragStart,
    handleDragEnd,
    handleDragCancel,
  } = useDragAndDrop({
    onCardMove: moveCard,
    onColumnMove: reorderColumn,
  });

  const columnIds = board?.columns?.map((col) => col.id) || [];

  // Find active card for drag overlay
  const activeCard =
    activeType === "card"
      ? board?.columns
          ?.flatMap((col) => col.cards || [])
          .find((card) => card.id === activeId)
      : undefined;

  if (!board) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-muted-foreground">No board data available</p>
      </div>
    );
  }

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <div className="flex-1 overflow-x-auto overflow-y-hidden">
        <div className="flex gap-4 p-6 h-full">
          <SortableContext
            items={columnIds}
            strategy={horizontalListSortingStrategy}
          >
            {board.columns?.map((column) => (
              <Column key={column.id} column={column} />
            ))}
          </SortableContext>
          <AddColumn />
        </div>
      </div>

      <DragOverlay>
        {activeType === "card" && activeCard ? (
          <div className="rotate-3 opacity-80">
            <Card card={activeCard} />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
