"use client";

import { useEffect } from "react";
import { X, AlertCircle, CheckCircle, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useUIStore } from "@/store/ui-store";

export function Toast() {
  const { toast, hideToast } = useUIStore();

  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => {
        hideToast();
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [toast, hideToast]);

  if (!toast) return null;

  const icons = {
    success: <CheckCircle className="h-5 w-5 text-green-600" />,
    error: <AlertCircle className="h-5 w-5 text-red-600" />,
    info: <Info className="h-5 w-5 text-blue-600" />,
  };

  const bgColors = {
    success: "bg-green-50 border-green-200",
    error: "bg-red-50 border-red-200",
    info: "bg-blue-50 border-blue-200",
  };

  return (
    <div className="fixed bottom-4 right-4 z-50 animate-in slide-in-from-bottom-5">
      <div
        className={`flex items-center gap-3 p-4 rounded-lg border shadow-lg max-w-md ${
          bgColors[toast.type]
        }`}
      >
        {icons[toast.type]}
        <p className="flex-1 text-sm font-medium">{toast.message}</p>
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6"
          onClick={hideToast}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
