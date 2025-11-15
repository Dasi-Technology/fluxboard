"use client";

import { use } from "react";

interface BoardPageProps {
  params: Promise<{
    shareToken: string;
  }>;
}

export default function BoardPage({ params }: BoardPageProps) {
  // Unwrap the params promise using React.use()
  const { shareToken } = use(params);

  return (
    <div className="min-h-screen bg-slate-100 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="mb-6">
          <h1 className="text-2xl font-bold text-slate-900">Board View</h1>
          <p className="text-sm text-slate-600 mt-1">
            Share Token: {shareToken}
          </p>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <p className="text-slate-600">
            Board components will be implemented here.
          </p>
        </div>
      </div>
    </div>
  );
}
