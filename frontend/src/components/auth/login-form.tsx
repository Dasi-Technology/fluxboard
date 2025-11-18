"use client";

import { useState } from "react";
import { useAuthStore } from "@/store/auth-store";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface LoginFormProps {
  onSuccess?: () => void;
  onSwitchToRegister?: () => void;
}

export function LoginForm({ onSuccess, onSwitchToRegister }: LoginFormProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  const { login, error: storeError } = useAuthStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    // Basic validation
    if (!email || !password) {
      setLocalError("Please fill in all fields");
      return;
    }

    if (!email.includes("@")) {
      setLocalError("Please enter a valid email address");
      return;
    }

    setIsSubmitting(true);

    try {
      await login({ email, password });
      onSuccess?.();
    } catch (error) {
      // Error is already set in the store
      console.error("Login failed:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const error = localError || storeError;

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="email">Email</Label>
        <Input
          id="email"
          type="email"
          placeholder="your@email.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          disabled={isSubmitting}
          autoComplete="email"
          required
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="password">Password</Label>
        <Input
          id="password"
          type="password"
          placeholder="••••••••"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          disabled={isSubmitting}
          autoComplete="current-password"
          required
        />
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-800">
          {error}
        </div>
      )}

      <Button type="submit" className="w-full" disabled={isSubmitting}>
        {isSubmitting ? "Logging in..." : "Log In"}
      </Button>

      <div className="text-center text-sm text-slate-600">
        Don't have an account?{" "}
        <button
          type="button"
          onClick={onSwitchToRegister}
          className="font-medium text-blue-600 hover:text-blue-500"
          disabled={isSubmitting}
        >
          Sign up
        </button>
      </div>
    </form>
  );
}
