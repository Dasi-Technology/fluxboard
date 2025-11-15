"use client";

import { useState } from "react";
import { Copy, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface ShareLinkProps {
  shareToken: string;
}

export function ShareLink({ shareToken }: ShareLinkProps) {
  const [copied, setCopied] = useState(false);
  const shareUrl = `${window.location.origin}/board/${shareToken}`;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy:", error);
    }
  };

  return (
    <div className="space-y-2">
      <Label htmlFor="share-link" className="hidden md:block">
        Share Link
      </Label>
      <div className="flex gap-2">
        <Input
          id="share-link"
          value={shareUrl}
          readOnly
          className="flex-1 text-xs md:text-sm"
          onClick={(e) => e.currentTarget.select()}
        />
        <Button
          onClick={handleCopy}
          variant={copied ? "default" : "outline"}
          className="min-w-[80px] md:min-w-[100px] text-xs md:text-sm"
        >
          {copied ? (
            <>
              <Check className="h-3 w-3 md:h-4 md:w-4 mr-1 md:mr-2" />
              <span className="hidden sm:inline">Copied!</span>
              <span className="sm:hidden">✓</span>
            </>
          ) : (
            <>
              <Copy className="h-3 w-3 md:h-4 md:w-4 mr-1 md:mr-2" />
              <span className="hidden sm:inline">Copy</span>
              <span className="sm:hidden">📋</span>
            </>
          )}
        </Button>
      </div>
      <p className="text-xs text-muted-foreground hidden md:block">
        Anyone with this link can view and edit this board.
      </p>
    </div>
  );
}
