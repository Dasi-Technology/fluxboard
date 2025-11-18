"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/store/auth-store";

interface ProtectedRouteProps {
  children: React.ReactNode;
  fallbackPath?: string;
}

export function ProtectedRoute({
  children,
  fallbackPath = "/",
}: ProtectedRouteProps) {
  const router = useRouter();
  const { isAuthenticated, isLoading, checkAuth } = useAuthStore();

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push(fallbackPath);
    }
  }, [isAuthenticated, isLoading, router, fallbackPath]);

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-slate-200 border-t-slate-900" />
          <p className="text-sm text-slate-600">Loading...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return null;
  }

  return <>{children}</>;
}
