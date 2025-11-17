"use client";

import { useDroppable } from "@dnd-kit/core";
import {
  SortableContext,
  verticalListSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { MoreHorizontal, Pencil, Trash2, Filter } from "lucide-react";
import type { Column as ColumnType } from "@/lib/types";
import { Card } from "./card";
import { AddCard } from "./add-card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useUIStore } from "@/store/ui-store";
import { useBoard } from "@/hooks/use-board";
import { useBoardStore } from "@/store/board-store";

interface ColumnProps {
  column: ColumnType;
  isReadOnly?: boolean;
}

export function Column({ column, isReadOnly = false }: ColumnProps) {
  const { deleteColumn } = useBoard();
  const { openEditColumnDialog } = useUIStore();
  const getFilteredCards = useBoardStore((state) => state.getFilteredCards);
  const selectedLabelFilter = useBoardStore(
    (state) => state.selectedLabelFilter
  );

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

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.6 : 1,
  };

  // Get filtered cards instead of all cards
  const displayCards = getFilteredCards(column.id);
  const hasActiveFilter = selectedLabelFilter.length > 0;
  const filteredCount = hasActiveFilter ? displayCards.length : null;
  const totalCount = column.cards?.length || 0;
  const cardIds = displayCards.map((card) => card.id);

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
      ref={setSortableRef}
      style={style}
      className="flex-shrink-0 w-72 md:w-80 bg-muted/50 rounded-lg p-3 md:p-4 flex flex-col h-full"
    >
      <div
        {...attributes}
        {...listeners}
        className="flex items-center justify-between mb-2 md:mb-3 cursor-grab active:cursor-grabbing"
      >
        <div className="flex items-center gap-2 flex-1 mr-2">
          <h2 className="font-semibold text-base md:text-lg truncate">
            {column.title}
          </h2>
          {hasActiveFilter && filteredCount !== null ? (
            <Badge variant="secondary" className="text-xs">
              <Filter className="h-3 w-3 mr-1" />
              {filteredCount} / {totalCount}
            </Badge>
          ) : (
            <Badge variant="secondary" className="text-xs">
              {totalCount}
            </Badge>
          )}
        </div>
        {!isReadOnly && (
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
        )}
      </div>

      <div
        ref={setDroppableRef}
        className="flex-1 overflow-y-auto space-y-2 mb-2 min-h-[100px] transition-colors duration-200"
      >
        <SortableContext items={cardIds} strategy={verticalListSortingStrategy}>
          {displayCards.length > 0 ? (
            displayCards.map((card) => (
              <Card key={card.id} card={card} isReadOnly={isReadOnly} />
            ))
          ) : hasActiveFilter ? (
            <div className="text-center py-8 text-sm text-muted-foreground">
              <Filter className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No cards match the selected filters</p>
            </div>
          ) : null}
        </SortableContext>
        {!isReadOnly && (
          <div className="md:hidden">
            <AddCard columnId={column.id} />
          </div>
        )}
      </div>

      {!isReadOnly && (
        <div className="hidden md:block">
          <AddCard columnId={column.id} />
        </div>
      )}
    </div>
  );
}
