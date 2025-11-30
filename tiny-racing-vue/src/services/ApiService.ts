const API_URL = 'http://localhost:3000';

// Token storage key
const TOKEN_KEY = 'jwt_token';
const TOKEN_EXPIRY_KEY = 'jwt_token_expires_at';

// Get stored token
export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

// Store token
export function setToken(token: string, expiresAt: string): void {
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(TOKEN_EXPIRY_KEY, expiresAt);
}

// Remove token
export function removeToken(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(TOKEN_EXPIRY_KEY);
}

// Check if token is expired
export function isTokenExpired(): boolean {
  const expiryStr = localStorage.getItem(TOKEN_EXPIRY_KEY);
  if (!expiryStr) return true;

  const expiry = new Date(expiryStr);
  return expiry < new Date();
}

// Check if user is authenticated
export function isAuthenticated(): boolean {
  const token = getToken();
  return token !== null && !isTokenExpired();
}

// Make authenticated API request
export async function apiRequest(endpoint: string, options: RequestInit = {}): Promise<Response> {
  const token = getToken();

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {}),
  };

  // Add Authorization header if token exists
  if (token && !isTokenExpired()) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_URL}${endpoint}`, {
    ...options,
    headers: headers as HeadersInit,
  });

  // If unauthorized, clear token
  if (response.status === 401) {
    removeToken();
  }

  return response;
}

// Login function
export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  status: string;
  message?: string;
  data?: {
    token: string;
    expires_at: string;
  };
}

export async function login(username: string, password: string): Promise<LoginResponse> {
  const response = await fetch(`${API_URL}/auth/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ username, password }),
  });

  const data: LoginResponse = await response.json();

  if (data.status === 'success' && data.data) {
    setToken(data.data.token, data.data.expires_at);
  }

  return data;
}

// Register function
export interface RegisterRequest {
  username: string;
  email?: string;
  password: string;
}

export interface RegisterResponse {
  status: string;
  message?: string;
  data?: {
    id: string;
    username: string;
  };
}

export async function register(
  username: string,
  email: string | undefined,
  password: string,
): Promise<RegisterResponse> {
  const response = await fetch(`${API_URL}/auth/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ username, email, password }),
  });

  const data: RegisterResponse = await response.json();
  return data;
}

// Logout function
export async function logout(): Promise<void> {
  const token = getToken();

  // If we have a token, try to delete it from the server
  if (token && !isTokenExpired()) {
    try {
      await apiRequest('/auth/logout', {
        method: 'POST',
      });
    } catch (err) {
      // Even if the server call fails, remove the token locally
      console.error('Logout error:', err);
    }
  }

  // Always remove token from local storage
  removeToken();
}
