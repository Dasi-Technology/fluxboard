"use client";

import { useState } from "react";
import { useAuthStore } from "@/store/auth-store";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface RegisterFormProps {
  onSuccess?: () => void;
  onSwitchToLogin?: () => void;
}

export function RegisterForm({
  onSuccess,
  onSwitchToLogin,
}: RegisterFormProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  const { register, error: storeError } = useAuthStore();

  const getPasswordStrength = (pwd: string): string => {
    if (pwd.length === 0) return "";
    if (pwd.length < 6) return "weak";
    if (pwd.length < 10) return "medium";
    return "strong";
  };

  const passwordStrength = getPasswordStrength(password);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    // Basic validation
    if (!email || !password || !confirmPassword) {
      setLocalError("Please fill in all required fields");
      return;
    }

    if (!email.includes("@")) {
      setLocalError("Please enter a valid email address");
      return;
    }

    if (password.length < 6) {
      setLocalError("Password must be at least 6 characters long");
      return;
    }

    if (password !== confirmPassword) {
      setLocalError("Passwords do not match");
      return;
    }

    setIsSubmitting(true);

    try {
      await register({
        email,
        password,
        display_name: displayName || undefined,
      });
      onSuccess?.();
    } catch (error) {
      // Error is already set in the store
      console.error("Registration failed:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const error = localError || storeError;

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="register-email">
          Email <span className="text-red-500">*</span>
        </Label>
        <Input
          id="register-email"
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
        <Label htmlFor="register-display-name">Display Name (optional)</Label>
        <Input
          id="register-display-name"
          type="text"
          placeholder="Your Name"
          value={displayName}
          onChange={(e) => setDisplayName(e.target.value)}
          disabled={isSubmitting}
          autoComplete="name"
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="register-password">
          Password <span className="text-red-500">*</span>
        </Label>
        <Input
          id="register-password"
          type="password"
          placeholder="••••••••"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          disabled={isSubmitting}
          autoComplete="new-password"
          required
        />
        {password && (
          <div className="flex gap-1 mt-2">
            <div
              className={`h-1 flex-1 rounded ${
                passwordStrength === "weak"
                  ? "bg-red-500"
                  : passwordStrength === "medium"
                  ? "bg-yellow-500"
                  : "bg-green-500"
              }`}
            />
            <div
              className={`h-1 flex-1 rounded ${
                passwordStrength === "medium" || passwordStrength === "strong"
                  ? passwordStrength === "medium"
                    ? "bg-yellow-500"
                    : "bg-green-500"
                  : "bg-gray-200"
              }`}
            />
            <div
              className={`h-1 flex-1 rounded ${
                passwordStrength === "strong" ? "bg-green-500" : "bg-gray-200"
              }`}
            />
          </div>
        )}
        {password && (
          <p className="text-xs text-slate-500 mt-1">
            Password strength:{" "}
            <span
              className={
                passwordStrength === "weak"
                  ? "text-red-600"
                  : passwordStrength === "medium"
                  ? "text-yellow-600"
                  : "text-green-600"
              }
            >
              {passwordStrength}
            </span>
          </p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="register-confirm-password">
          Confirm Password <span className="text-red-500">*</span>
        </Label>
        <Input
          id="register-confirm-password"
          type="password"
          placeholder="••••••••"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          disabled={isSubmitting}
          autoComplete="new-password"
          required
        />
      </div>

      {error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-800">
          {error}
        </div>
      )}

      <Button type="submit" className="w-full" disabled={isSubmitting}>
        {isSubmitting ? "Creating account..." : "Sign Up"}
      </Button>

      <div className="text-center text-sm text-slate-600">
        Already have an account?{" "}
        <button
          type="button"
          onClick={onSwitchToLogin}
          className="font-medium text-blue-600 hover:text-blue-500"
          disabled={isSubmitting}
        >
          Log in
        </button>
      </div>
    </form>
  );
}
