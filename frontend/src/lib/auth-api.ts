import api from "./api";

/**
 * Authentication API types
 */

export interface User {
  id: string;
  email: string;
  display_name: string | null;
  created_at: string;
  last_login_at: string | null;
}

export interface RegisterRequest {
  email: string;
  password: string;
  display_name?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface LogoutRequest {
  refresh_token: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: User;
  access_token_expires_at: string;
  refresh_token_expires_at: string;
}

export interface LogoutResponse {
  message: string;
}

/**
 * Local storage keys for authentication
 */
const STORAGE_KEYS = {
  ACCESS_TOKEN: "fluxboard_access_token",
  REFRESH_TOKEN: "fluxboard_refresh_token",
  USER: "fluxboard_user",
  TOKEN_EXPIRY: "fluxboard_token_expiry",
  REFRESH_EXPIRY: "fluxboard_refresh_expiry",
};

/**
 * Token storage helpers
 */
export const storeTokens = (authResponse: AuthResponse): void => {
  if (typeof window === "undefined") return;

  localStorage.setItem(STORAGE_KEYS.ACCESS_TOKEN, authResponse.access_token);
  localStorage.setItem(STORAGE_KEYS.REFRESH_TOKEN, authResponse.refresh_token);
  localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(authResponse.user));
  localStorage.setItem(
    STORAGE_KEYS.TOKEN_EXPIRY,
    authResponse.access_token_expires_at
  );
  localStorage.setItem(
    STORAGE_KEYS.REFRESH_EXPIRY,
    authResponse.refresh_token_expires_at
  );
};

export const getAccessToken = (): string | null => {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(STORAGE_KEYS.ACCESS_TOKEN);
};

export const getRefreshToken = (): string | null => {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(STORAGE_KEYS.REFRESH_TOKEN);
};

export const getStoredUser = (): User | null => {
  if (typeof window === "undefined") return null;
  const userStr = localStorage.getItem(STORAGE_KEYS.USER);
  if (!userStr) return null;
  try {
    return JSON.parse(userStr);
  } catch {
    return null;
  }
};

export const clearTokens = (): void => {
  if (typeof window === "undefined") return;
  localStorage.removeItem(STORAGE_KEYS.ACCESS_TOKEN);
  localStorage.removeItem(STORAGE_KEYS.REFRESH_TOKEN);
  localStorage.removeItem(STORAGE_KEYS.USER);
  localStorage.removeItem(STORAGE_KEYS.TOKEN_EXPIRY);
  localStorage.removeItem(STORAGE_KEYS.REFRESH_EXPIRY);
};

export const isTokenExpired = (): boolean => {
  if (typeof window === "undefined") return true;
  const expiry = localStorage.getItem(STORAGE_KEYS.TOKEN_EXPIRY);
  if (!expiry) return true;
  return new Date(expiry).getTime() < Date.now();
};

/**
 * Authentication API endpoints
 */

export const register = async (
  data: RegisterRequest
): Promise<AuthResponse> => {
  const response = await api.post<AuthResponse>("/auth/register", data);
  storeTokens(response.data);
  return response.data;
};

export const login = async (data: LoginRequest): Promise<AuthResponse> => {
  const response = await api.post<AuthResponse>("/auth/login", data);
  storeTokens(response.data);
  return response.data;
};

export const refreshToken = async (): Promise<AuthResponse> => {
  const refresh = getRefreshToken();
  if (!refresh) {
    throw new Error("No refresh token available");
  }

  const response = await api.post<AuthResponse>("/auth/refresh", {
    refresh_token: refresh,
  });
  storeTokens(response.data);
  return response.data;
};

export const logout = async (): Promise<LogoutResponse> => {
  const refresh = getRefreshToken();
  if (!refresh) {
    clearTokens();
    return { message: "Logged out" };
  }

  try {
    const response = await api.post<LogoutResponse>("/auth/logout", {
      refresh_token: refresh,
    });
    clearTokens();
    return response.data;
  } catch (error) {
    // Clear tokens even if logout fails
    clearTokens();
    throw error;
  }
};

export const getCurrentUser = async (): Promise<User> => {
  const response = await api.get<User>("/auth/me");
  // Update stored user with latest data
  if (typeof window !== "undefined") {
    localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(response.data));
  }
  return response.data;
};
