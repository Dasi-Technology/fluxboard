"use client";

import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { LoginForm } from "./login-form";
import { RegisterForm } from "./register-form";

interface AuthDialogProps {
  isOpen: boolean;
  onClose: () => void;
  defaultTab?: "login" | "register";
}

export function AuthDialog({
  isOpen,
  onClose,
  defaultTab = "login",
}: AuthDialogProps) {
  const [activeTab, setActiveTab] = useState<"login" | "register">(defaultTab);

  const handleSuccess = () => {
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {activeTab === "login"
              ? "Log in to Fluxboard"
              : "Create an account"}
          </DialogTitle>
          <DialogDescription>
            {activeTab === "login"
              ? "Welcome back! Enter your credentials to continue."
              : "Sign up to save your boards and collaborate with others."}
          </DialogDescription>
        </DialogHeader>

        <div className="mt-4">
          {activeTab === "login" ? (
            <LoginForm
              onSuccess={handleSuccess}
              onSwitchToRegister={() => setActiveTab("register")}
            />
          ) : (
            <RegisterForm
              onSuccess={handleSuccess}
              onSwitchToLogin={() => setActiveTab("login")}
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
