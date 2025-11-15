"use client";

import { useEffect } from "react";
import { Board } from "@/components/board/board";
import { CreateBoardDialog } from "@/components/dialogs/create-board-dialog";
import { EditCardDialog } from "@/components/dialogs/edit-card-dialog";
import { EditColumnDialog } from "@/components/dialogs/edit-column-dialog";
import { ManageLabelsDialog } from "@/components/dialogs/manage-labels-dialog";
import { ShareLink } from "@/components/shared/share-link";
import { Toast } from "@/components/shared/toast";
import { useBoard } from "@/hooks/use-board";
import { useSSE } from "@/hooks/use-sse";
import { useBoardStore } from "@/store/board-store";

interface BoardPageProps {
  params: {
    shareToken: string;
  };
}

export default function BoardPage({ params }: BoardPageProps) {
  const { shareToken } = params;
  const { loadBoard } = useBoard();
  const { board, isLoading, error } = useBoardStore();

  // Establish SSE connection
  useSSE(shareToken);

  // Load board data on mount
  useEffect(() => {
    loadBoard(shareToken);
  }, [shareToken, loadBoard]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-slate-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-slate-900 mx-auto"></div>
          <p className="mt-4 text-slate-600">Loading board...</p>
        </div>
      </div>
    );
  }

  if (error || !board) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-slate-100">
        <div className="text-center max-w-md">
          <h1 className="text-2xl font-bold text-slate-900 mb-2">
            Board Not Found
          </h1>
          <p className="text-slate-600 mb-6">
            {error || "The board you're looking for doesn't exist."}
          </p>
          <a
            href="/"
            className="inline-block px-4 py-2 bg-slate-900 text-white rounded-lg hover:bg-slate-800"
          >
            Create New Board
          </a>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="flex flex-col h-screen bg-slate-100">
        {/* Header */}
        <header className="bg-white border-b border-slate-200 px-6 py-4">
          <div className="flex items-center justify-between max-w-[2000px] mx-auto">
            <div>
              <h1 className="text-2xl font-bold text-slate-900">
                {board.title}
              </h1>
              <p className="text-sm text-slate-600 mt-1">
                Real-time collaborative board
              </p>
            </div>
            <div className="w-96">
              <ShareLink shareToken={shareToken} />
            </div>
          </div>
        </header>

        {/* Board Content */}
        <div className="flex-1 overflow-hidden">
          <Board />
        </div>
      </div>

      {/* Dialogs */}
      <CreateBoardDialog />
      <EditCardDialog />
      <EditColumnDialog />
      <ManageLabelsDialog />
      <Toast />
    </>
  );
}
