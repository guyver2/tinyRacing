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

// Get player ID from JWT token
export function getPlayerId(): string | null {
  const token = getToken();
  if (!token || isTokenExpired()) {
    return null;
  }

  try {
    // Decode JWT token (base64url decode the payload)
    const parts = token.split('.');
    if (parts.length !== 3) {
      return null;
    }

    // Decode the payload (second part)
    const payload = parts[1];
    // Add padding if needed
    const paddedPayload = payload + '='.repeat((4 - (payload.length % 4)) % 4);
    const decoded = atob(paddedPayload.replace(/-/g, '+').replace(/_/g, '/'));
    const claims = JSON.parse(decoded);
    
    return claims.sub || null;
  } catch (err) {
    console.error('Error decoding token:', err);
    return null;
  }
}

// Team interfaces
export interface TeamDb {
  id: string;
  number: number;
  name: string;
  logo: string;
  color: string;
  pit_efficiency: number;
  cash: number;
  player_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTeamRequest {
  number?: number;
  name: string;
  logo: string;
  color: string;
  pit_efficiency?: number;
  player_id?: string | null;
}

export interface ApiResponse<T> {
  status: string;
  message?: string;
  data?: T;
}

// Get teams (optionally filtered by player_id)
export async function getTeams(playerId?: string): Promise<TeamDb[]> {
  const url = playerId ? `/teams?player_id=${playerId}` : '/teams';
  const response = await apiRequest(url);
  
  if (!response.ok) {
    throw new Error(`Failed to fetch teams: ${response.statusText}`);
  }

  const data: ApiResponse<TeamDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }
  
  throw new Error(data.message || 'Failed to fetch teams');
}

// Get the current player's team
export async function getMyTeam(): Promise<TeamDb | null> {
  const response = await apiRequest('/teams/my');
  
  if (!response.ok) {
    if (response.status === 401) {
      throw new Error('You must be logged in to view your team');
    }
    throw new Error(`Failed to fetch team: ${response.statusText}`);
  }

  const data: ApiResponse<TeamDb | null> = await response.json();
  if (data.status === 'success') {
    return data.data || null;
  }
  
  throw new Error(data.message || 'Failed to fetch team');
}

// Create a new team
export async function createTeam(request: CreateTeamRequest): Promise<TeamDb> {
  const response = await apiRequest('/teams', {
    method: 'POST',
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to create team: ${response.statusText}`);
  }

  const data: ApiResponse<TeamDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to create team');
}

// Driver interfaces
export interface DriverDb {
  id: string;
  first_name: string;
  last_name: string;
  date_of_birth: string;
  nationality: string;
  gender: string;
  skill_level: number;
  stamina: number;
  weather_tolerance: number;
  experience: number;
  consistency: number;
  focus: number;
  team_id: string | null;
  car_id: string | null;
  created_at: string;
  updated_at: string;
}

// Car interfaces
export interface CarDb {
  id: string;
  number: number;
  team_id: string | null;
  handling: number;
  acceleration: number;
  top_speed: number;
  reliability: number;
  fuel_consumption: number;
  tire_wear: number;
  base_performance: number;
  created_at: string;
  updated_at: string;
}

// Get unassigned drivers (for market)
export async function getUnassignedDrivers(): Promise<DriverDb[]> {
  const response = await apiRequest('/drivers/unassigned');
  
  if (!response.ok) {
    throw new Error(`Failed to fetch unassigned drivers: ${response.statusText}`);
  }

  const data: ApiResponse<DriverDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }
  
  throw new Error(data.message || 'Failed to fetch unassigned drivers');
}

// Get unassigned cars (for market)
export async function getUnassignedCars(): Promise<CarDb[]> {
  const response = await apiRequest('/cars/unassigned');
  
  if (!response.ok) {
    throw new Error(`Failed to fetch unassigned cars: ${response.statusText}`);
  }

  const data: ApiResponse<CarDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }
  
  throw new Error(data.message || 'Failed to fetch unassigned cars');
}
