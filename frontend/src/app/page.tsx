"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CreateBoardDialog } from "@/components/dialogs/create-board-dialog";
import { Toast } from "@/components/shared/toast";
import { useUIStore } from "@/store/ui-store";
import {
  getRecentBoardsLimited,
  searchRecentBoards,
  type RecentBoard,
} from "@/lib/recent-boards";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Search } from "lucide-react";

export default function Home() {
  const { openCreateBoardDialog } = useUIStore();
  const [recentBoards, setRecentBoards] = useState<RecentBoard[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    setRecentBoards(getRecentBoardsLimited());
  }, []);

  // Handle search
  useEffect(() => {
    if (searchQuery.trim()) {
      setRecentBoards(searchRecentBoards(searchQuery));
    } else {
      setRecentBoards(getRecentBoardsLimited());
    }
  }, [searchQuery]);

  const formatRelativeTime = (visitedAt: string) => {
    const now = Date.now();
    const visitedTime = new Date(visitedAt).getTime();
    const diff = now - visitedTime;
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days} day${days > 1 ? "s" : ""} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? "s" : ""} ago`;
    if (minutes > 0) return `${minutes} minute${minutes > 1 ? "s" : ""} ago`;
    return "Just now";
  };

  return (
    <>
      <main className="flex min-h-screen flex-col items-center justify-center p-8 bg-gradient-to-br from-slate-50 to-slate-100">
        <div className="w-full max-w-5xl space-y-12">
          {/* Hero Section */}
          <div className="w-full max-w-md mx-auto space-y-8 text-center">
            <div>
              <h1 className="text-5xl font-bold tracking-tight text-slate-900">
                Fluxboard
              </h1>
              <p className="mt-3 text-lg text-slate-600">
                Real-time collaborative Kanban board
              </p>
            </div>

            <div className="mt-10">
              <Button
                size="lg"
                onClick={openCreateBoardDialog}
                className="w-full sm:w-auto px-8 py-6 text-lg"
              >
                Create New Board
              </Button>
            </div>

            <div className="pt-8 border-t border-slate-200">
              <p className="text-sm text-slate-500">
                Collaborate in real-time • No sign-up required • Share instantly
              </p>
            </div>
          </div>

          {/* Recent Boards Section */}
          {mounted && (
            <div className="w-full space-y-6">
              <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                <h2 className="text-2xl font-semibold text-slate-900">
                  {searchQuery.trim() ? "Search Results" : "Recent Boards"}
                </h2>
                <div className="relative w-full sm:w-96">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-slate-400" />
                  <Input
                    type="text"
                    placeholder="Search boards by name or token..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-10 bg-white"
                  />
                </div>
              </div>

              {recentBoards.length > 0 ? (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {recentBoards.map((board) => (
                    <Link
                      key={board.shareToken}
                      href={`/board/${board.shareToken}`}
                      className="transition-transform hover:scale-105"
                    >
                      <Card className="h-full cursor-pointer hover:shadow-lg transition-shadow bg-white">
                        <CardHeader>
                          <CardTitle className="truncate text-xl">
                            {board.name}
                          </CardTitle>
                          <CardDescription>
                            Last visited {formatRelativeTime(board.visitedAt)}
                          </CardDescription>
                        </CardHeader>
                        <CardContent>
                          <p className="text-xs text-slate-500 font-mono truncate">
                            {board.shareToken}
                          </p>
                        </CardContent>
                      </Card>
                    </Link>
                  ))}
                </div>
              ) : (
                <div className="text-center py-12">
                  <p className="text-slate-600">
                    {searchQuery.trim()
                      ? `No boards found matching "${searchQuery}"`
                      : "No recent boards yet. Create your first board to get started!"}
                  </p>
                </div>
              )}
            </div>
          )}
        </div>
      </main>

      <CreateBoardDialog />
      <Toast />
    </>
  );
}
