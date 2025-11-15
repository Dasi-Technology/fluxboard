"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { createBoard } from "@/lib/api";

export default function Home() {
  const [boardName, setBoardName] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();

  const handleCreateBoard = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!boardName.trim()) return;

    setIsLoading(true);
    try {
      const board = await createBoard(boardName.trim());
      // Navigate to the newly created board using its share token
      router.push(`/board/${board.share_token}`);
    } catch (error) {
      console.error("Failed to create board:", error);
      alert("Failed to create board. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24 bg-gradient-to-br from-slate-50 to-slate-100">
      <div className="w-full max-w-md space-y-8">
        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight text-slate-900">
            Fluxboard
          </h1>
          <p className="mt-2 text-slate-600">
            Real-time collaborative Kanban board
          </p>
        </div>

        <form onSubmit={handleCreateBoard} className="mt-8 space-y-6">
          <div>
            <label htmlFor="board-name" className="sr-only">
              Board name
            </label>
            <input
              id="board-name"
              name="board-name"
              type="text"
              required
              value={boardName}
              onChange={(e) => setBoardName(e.target.value)}
              className="relative block w-full rounded-lg border-0 px-4 py-3 text-slate-900 ring-1 ring-inset ring-slate-300 placeholder:text-slate-400 focus:z-10 focus:ring-2 focus:ring-inset focus:ring-slate-600"
              placeholder="Enter board name"
              disabled={isLoading}
            />
          </div>

          <button
            type="submit"
            disabled={isLoading || !boardName.trim()}
            className="group relative flex w-full justify-center rounded-lg bg-slate-900 px-4 py-3 text-sm font-semibold text-white hover:bg-slate-800 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-900 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isLoading ? "Creating..." : "Create New Board"}
          </button>
        </form>
      </div>
    </main>
  );
}
