/**
 * Username prompt dialog for new users.
 *
 * Prompts users to enter their username on first visit and stores it in localStorage.
 */

import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

export interface UsernamePromptProps {
  open: boolean;
  onSubmit: (username: string) => void;
}

const USERNAME_MIN_LENGTH = 1;
const USERNAME_MAX_LENGTH = 32;

/**
 * Username prompt dialog component.
 *
 * Features:
 * - Validates username length (1-32 characters)
 * - Stores username in localStorage
 * - Cannot be dismissed without entering a username
 */
export function UsernamePrompt({ open, onSubmit }: UsernamePromptProps) {
  const [username, setUsername] = useState("");
  const [error, setError] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const trimmed = username.trim();

    // Validate username
    if (trimmed.length < USERNAME_MIN_LENGTH) {
      setError("Username is required");
      return;
    }

    if (trimmed.length > USERNAME_MAX_LENGTH) {
      setError(`Username must be ${USERNAME_MAX_LENGTH} characters or less`);
      return;
    }

    // Clear error and submit
    setError("");
    onSubmit(trimmed);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setUsername(e.target.value);
    setError(""); // Clear error on input
  };

  return (
    <Dialog open={open} onOpenChange={() => {}}>
      <DialogContent
        className="sm:max-w-[425px]"
        onPointerDownOutside={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>Enter Your Name</DialogTitle>
          <DialogDescription>
            Choose a display name to identify yourself to other users on this
            board.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="username">Display Name</Label>
              <Input
                id="username"
                placeholder="Enter your name..."
                value={username}
                onChange={handleChange}
                maxLength={USERNAME_MAX_LENGTH}
                autoFocus
                className={error ? "border-red-500" : ""}
              />
              {error && <p className="text-sm text-red-500">{error}</p>}
              <p className="text-xs text-gray-500">
                {username.length}/{USERNAME_MAX_LENGTH} characters
              </p>
            </div>
          </div>

          <DialogFooter>
            <Button type="submit" disabled={username.trim().length === 0}>
              Continue
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Hook for managing username from localStorage.
 *
 * @returns Username management utilities
 */
export function useUsername() {
  const STORAGE_KEY = "fluxboard_username";

  const [username, setUsernameState] = useState<string | null>(() => {
    if (typeof window !== "undefined") {
      return localStorage.getItem(STORAGE_KEY);
    }
    return null;
  });

  const setUsername = (newUsername: string) => {
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, newUsername);
      setUsernameState(newUsername);
    }
  };

  const clearUsername = () => {
    if (typeof window !== "undefined") {
      localStorage.removeItem(STORAGE_KEY);
      setUsernameState(null);
    }
  };

  return {
    username,
    setUsername,
    clearUsername,
    hasUsername: username !== null && username.length > 0,
  };
}
