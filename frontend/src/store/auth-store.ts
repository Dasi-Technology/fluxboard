import { create } from "zustand";
import type { User } from "@/lib/auth-api";
import {
  login as apiLogin,
  register as apiRegister,
  logout as apiLogout,
  refreshToken as apiRefreshToken,
  getCurrentUser as apiGetCurrentUser,
  getAccessToken,
  getRefreshToken,
  getStoredUser,
  clearTokens,
  isTokenExpired,
  type LoginRequest,
  type RegisterRequest,
} from "@/lib/auth-api";

interface AuthStore {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  login: (credentials: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  refreshAccessToken: () => Promise<void>;
  checkAuth: () => Promise<void>;
  setUser: (user: User | null) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

// Token refresh interval (14 minutes - refresh before 15 min expiry)
const REFRESH_INTERVAL = 14 * 60 * 1000;
let refreshIntervalId: NodeJS.Timeout | null = null;

export const useAuthStore = create<AuthStore>((set, get) => ({
  user: null,
  accessToken: null,
  refreshToken: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,

  login: async (credentials: LoginRequest) => {
    set({ isLoading: true, error: null });
    try {
      const response = await apiLogin(credentials);
      set({
        user: response.user,
        accessToken: response.access_token,
        refreshToken: response.refresh_token,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      // Start token refresh interval
      startTokenRefresh();
    } catch (error: any) {
      const errorMessage =
        error.response?.data?.message ||
        error.message ||
        "Login failed. Please check your credentials.";
      set({
        user: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: false,
        isLoading: false,
        error: errorMessage,
      });
      throw error;
    }
  },

  register: async (data: RegisterRequest) => {
    set({ isLoading: true, error: null });
    try {
      const response = await apiRegister(data);
      set({
        user: response.user,
        accessToken: response.access_token,
        refreshToken: response.refresh_token,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      // Start token refresh interval
      startTokenRefresh();
    } catch (error: any) {
      const errorMessage =
        error.response?.data?.message ||
        error.message ||
        "Registration failed. Please try again.";
      set({
        user: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: false,
        isLoading: false,
        error: errorMessage,
      });
      throw error;
    }
  },

  logout: async () => {
    set({ isLoading: true, error: null });
    try {
      await apiLogout();
    } catch (error) {
      // Ignore logout errors, still clear local state
      console.error("Logout error:", error);
    } finally {
      set({
        user: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
      });

      // Stop token refresh interval
      stopTokenRefresh();
    }
  },

  refreshAccessToken: async () => {
    try {
      const response = await apiRefreshToken();
      set({
        user: response.user,
        accessToken: response.access_token,
        refreshToken: response.refresh_token,
        isAuthenticated: true,
        error: null,
      });
    } catch (error: any) {
      console.error("Token refresh failed:", error);
      // If refresh fails, log out the user
      set({
        user: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: false,
        error: "Session expired. Please log in again.",
      });
      clearTokens();
      stopTokenRefresh();
      throw error;
    }
  },

  checkAuth: async () => {
    set({ isLoading: true });

    // Check if we have tokens in localStorage
    const storedAccessToken = getAccessToken();
    const storedRefreshToken = getRefreshToken();
    const storedUser = getStoredUser();

    if (!storedAccessToken || !storedRefreshToken || !storedUser) {
      set({
        user: null,
        accessToken: null,
        refreshToken: null,
        isAuthenticated: false,
        isLoading: false,
      });
      return;
    }

    // Check if access token is expired
    if (isTokenExpired()) {
      // Try to refresh the token
      try {
        await get().refreshAccessToken();
        set({ isLoading: false });
        startTokenRefresh();
        return;
      } catch {
        set({ isLoading: false });
        return;
      }
    }

    // Token is valid, restore session
    set({
      user: storedUser,
      accessToken: storedAccessToken,
      refreshToken: storedRefreshToken,
      isAuthenticated: true,
      isLoading: false,
    });

    // Verify token with backend
    try {
      const user = await apiGetCurrentUser();
      set({ user });
      startTokenRefresh();
    } catch {
      // Token is invalid, try to refresh
      try {
        await get().refreshAccessToken();
        startTokenRefresh();
      } catch {
        // Refresh failed, clear session
        set({
          user: null,
          accessToken: null,
          refreshToken: null,
          isAuthenticated: false,
        });
        clearTokens();
      }
    } finally {
      set({ isLoading: false });
    }
  },

  setUser: (user: User | null) => {
    set({ user });
  },

  setError: (error: string | null) => {
    set({ error });
  },

  clearError: () => {
    set({ error: null });
  },
}));

/**
 * Start automatic token refresh
 */
function startTokenRefresh() {
  // Clear any existing interval
  stopTokenRefresh();

  // Set up new interval
  refreshIntervalId = setInterval(async () => {
    const store = useAuthStore.getState();
    if (store.isAuthenticated) {
      try {
        await store.refreshAccessToken();
      } catch (error) {
        console.error("Auto token refresh failed:", error);
      }
    }
  }, REFRESH_INTERVAL);
}

/**
 * Stop automatic token refresh
 */
function stopTokenRefresh() {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
    refreshIntervalId = null;
  }
}
