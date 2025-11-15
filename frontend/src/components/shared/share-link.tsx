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
      <Label htmlFor="share-link">Share Link</Label>
      <div className="flex gap-2">
        <Input
          id="share-link"
          value={shareUrl}
          readOnly
          className="flex-1"
          onClick={(e) => e.currentTarget.select()}
        />
        <Button
          onClick={handleCopy}
          variant={copied ? "default" : "outline"}
          className="min-w-[100px]"
        >
          {copied ? (
            <>
              <Check className="h-4 w-4 mr-2" />
              Copied!
            </>
          ) : (
            <>
              <Copy className="h-4 w-4 mr-2" />
              Copy
            </>
          )}
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">
        Anyone with this link can view and edit this board.
      </p>
    </div>
  );
}
