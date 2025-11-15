import { useState } from "react";
import {
  DndContext,
  DragEndEvent,
  DragOverEvent,
  DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { arrayMove } from "@dnd-kit/sortable";
import type { Card, Column } from "@/lib/types";

interface UseDragAndDropProps {
  onCardMove: (
    cardId: string,
    newColumnId: string,
    newPosition: number
  ) => void;
  onColumnMove: (columnId: string, newPosition: number) => void;
}

export const useDragAndDrop = ({
  onCardMove,
  onColumnMove,
}: UseDragAndDropProps) => {
  const [activeId, setActiveId] = useState<string | null>(null);
  const [activeType, setActiveType] = useState<"card" | "column" | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // Require 8px of movement before drag starts
      },
    })
  );

  const handleDragStart = (event: DragStartEvent) => {
    const { active } = event;
    setActiveId(active.id as string);

    // Determine type from data
    const type = active.data.current?.type as "card" | "column" | undefined;
    setActiveType(type || null);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (!over || active.id === over.id) {
      setActiveId(null);
      setActiveType(null);
      return;
    }

    const activeType = active.data.current?.type;
    const overType = over.data.current?.type;

    // Handle card dragging
    if (activeType === "card") {
      const activeCard = active.data.current?.item as Card;
      const overItem = over.data.current?.item;
      const overColumnId = over.data.current?.columnId as string | undefined;

      // Dragging over another card
      if (overType === "card" && overItem) {
        const overCard = overItem as Card;
        onCardMove(activeCard.id, overCard.column_id, overCard.position);
      }
      // Dragging over column container
      else if (overType === "column-container" && overColumnId) {
        onCardMove(activeCard.id, overColumnId, 0);
      }
    }

    // Handle column dragging
    if (activeType === "column" && overType === "column") {
      const activeColumn = active.data.current?.item as Column;
      const overColumn = over.data.current?.item as Column;
      onColumnMove(activeColumn.id, overColumn.position);
    }

    setActiveId(null);
    setActiveType(null);
  };

  const handleDragCancel = () => {
    setActiveId(null);
    setActiveType(null);
  };

  return {
    sensors,
    activeId,
    activeType,
    handleDragStart,
    handleDragEnd,
    handleDragCancel,
  };
};
