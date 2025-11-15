"use client";

import { DndContext, DragOverlay } from "@dnd-kit/core";
import {
  SortableContext,
  horizontalListSortingStrategy,
} from "@dnd-kit/sortable";
import { useRef, useCallback, useState, useEffect } from "react";
import { Column } from "./column";
import { AddColumn } from "./add-column";
import { Card } from "./card";
import { useBoardStore } from "@/store/board-store";
import { useBoard } from "@/hooks/use-board";
import { useDragAndDrop } from "@/hooks/use-drag-and-drop";
import { usePresence } from "@/hooks/use-presence";
import { Cursor } from "@/components/presence/cursor";
import { ActiveUsers } from "@/components/presence/active-users";
import {
  UsernamePrompt,
  useUsername,
} from "@/components/presence/username-prompt";

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

  // Track if currently dragging
  const [isDragging, setIsDragging] = useState(false);

  // Update dragging state
  useEffect(() => {
    setIsDragging(activeId !== null);
  }, [activeId]);

  const columnIds = board?.columns?.map((col) => col.id) || [];

  // Find active card for drag overlay
  const activeCard =
    activeType === "card"
      ? board?.columns
          ?.flatMap((col) => col.cards || [])
          .find((card) => card.id === activeId)
      : undefined;

  // Username management
  const { username, setUsername, hasUsername } = useUsername();
  const [showUsernamePrompt, setShowUsernamePrompt] = useState(false);

  // Board container ref for cursor positioning
  const boardRef = useRef<HTMLDivElement>(null);

  // Convert board ID (string) to number for presence system
  const getBoardIdNumber = (boardId: string): number => {
    // Simple hash function to convert string to number
    let hash = 0;
    for (let i = 0; i < boardId.length; i++) {
      const char = boardId.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash) % 65535; // Keep within u16 range
  };

  // Presence integration
  const { users, presenceCount, isConnected, updateCursor } = usePresence({
    boardId: board?.id ? getBoardIdNumber(board.id) : 0,
    username: username || "Anonymous",
    enabled: !!board && hasUsername,
    throttleMs: 50,
  });

  // Update cursor position from mouse event
  const updateCursorFromEvent = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (!boardRef.current || !isConnected) return;

      const rect = boardRef.current.getBoundingClientRect();
      const x = (e.clientX - rect.left) / rect.width;
      const y = (e.clientY - rect.top) / rect.height;

      updateCursor(x, y);
    },
    [isConnected, updateCursor]
  );

  // Handle mouse move on board
  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      updateCursorFromEvent(e);
    },
    [updateCursorFromEvent]
  );

  // Handle click on board
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      updateCursorFromEvent(e);
    },
    [updateCursorFromEvent]
  );

  // Handle mouse down (for drag tracking)
  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      updateCursorFromEvent(e);
    },
    [updateCursorFromEvent]
  );

  // Handle username submission
  const handleUsernameSubmit = useCallback(
    (newUsername: string) => {
      setUsername(newUsername);
      setShowUsernamePrompt(false);
    },
    [setUsername]
  );

  // Track cursor during drag operations globally
  useEffect(() => {
    if (!isDragging || !boardRef.current || !isConnected) return;

    const handleGlobalMouseMove = (e: MouseEvent) => {
      if (!boardRef.current) return;

      const rect = boardRef.current.getBoundingClientRect();
      const x = (e.clientX - rect.left) / rect.width;
      const y = (e.clientY - rect.top) / rect.height;

      updateCursor(x, y);
    };

    // Add global mouse move listener during drag
    document.addEventListener("mousemove", handleGlobalMouseMove);

    return () => {
      document.removeEventListener("mousemove", handleGlobalMouseMove);
    };
  }, [isDragging, isConnected, updateCursor]);

  // Show username prompt if no username set
  if (!hasUsername) {
    return <UsernamePrompt open={true} onSubmit={handleUsernameSubmit} />;
  }

  if (!board) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-muted-foreground">No board data available</p>
      </div>
    );
  }

  return (
    <>
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onDragCancel={handleDragCancel}
      >
        <div
          ref={boardRef}
          className="flex-1 overflow-x-auto overflow-y-hidden relative h-full"
          onMouseMove={handleMouseMove}
          onClick={handleClick}
          onMouseDown={handleMouseDown}
        >
          <div className="flex gap-3 md:gap-4 p-3 md:p-6 h-full min-h-0">
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

          {/* Presence overlay - cursors */}
          <div className="absolute inset-0 pointer-events-none">
            {Array.from(users.values()).map((user) =>
              user.cursor ? (
                <Cursor
                  key={user.userId}
                  userId={user.userId}
                  username={user.username}
                  color={user.color}
                  x={user.cursor.x}
                  y={user.cursor.y}
                  containerRef={boardRef}
                />
              ) : null
            )}
          </div>
        </div>

        <DragOverlay>
          {activeType === "card" && activeCard ? (
            <div className="rotate-6 scale-105 cursor-grabbing shadow-2xl">
              <Card card={activeCard} />
            </div>
          ) : null}
        </DragOverlay>
      </DndContext>

      {/* Active users list */}
      <ActiveUsers users={users} presenceCount={presenceCount} />

      {/* Username prompt dialog */}
      <UsernamePrompt
        open={showUsernamePrompt}
        onSubmit={handleUsernameSubmit}
      />
    </>
  );
}
