"use client";

import { Button } from "@/components/ui/button";
import { CreateBoardDialog } from "@/components/dialogs/create-board-dialog";
import { Toast } from "@/components/shared/toast";
import { useUIStore } from "@/store/ui-store";

export default function Home() {
  const { openCreateBoardDialog } = useUIStore();

  return (
    <>
      <main className="flex min-h-screen flex-col items-center justify-center p-8 bg-gradient-to-br from-slate-50 to-slate-100">
        <div className="w-full max-w-md space-y-8 text-center">
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
      </main>

      <CreateBoardDialog />
      <Toast />
    </>
  );
}
