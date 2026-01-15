const getApiUrl = () => {
  const apiUrl = import.meta.env.VITE_API_URL;

  // If VITE_API_URL is explicitly set, use it
  if (apiUrl) {
    // If it's a relative URL, use it directly (nginx will proxy it)
    if (apiUrl.startsWith('/')) {
      return apiUrl;
    }
    return apiUrl;
  }

  // If no explicit URL is set, use the current page's hostname with port 3000
  // This allows it to work both locally (localhost) and on the network (192.168.x.x)
  const protocol = window.location.protocol;
  const hostname = window.location.hostname;
  return `${protocol}//${hostname}:3000`;
};

// Get API URL dynamically (not cached at module load time)
const getAPI_URL = () => getApiUrl();

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

  const response = await fetch(`${getAPI_URL()}${endpoint}`, {
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
  const response = await fetch(`${getAPI_URL()}/auth/login`, {
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
  const response = await fetch(`${getAPI_URL()}/auth/register`, {
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
  logo?: string; // Optional, can be provided as file upload
  color: string;
  pit_efficiency?: number;
  player_id?: string | null;
}

export interface ApiResponse<T> {
  status: string;
  message?: string;
  data?: T;
}

// Pagination parameters interface
export interface PaginationParams {
  limit?: number;
  offset?: number;
}

// Helper function to build URL with pagination and optional additional parameters
function buildPaginatedUrl(
  basePath: string,
  pagination?: PaginationParams,
  additionalParams?: Record<string, string>,
): string {
  const params = new URLSearchParams();

  // Add pagination parameters
  if (pagination?.limit !== undefined) {
    params.append('limit', pagination.limit.toString());
  }
  if (pagination?.offset !== undefined) {
    params.append('offset', pagination.offset.toString());
  }

  // Add additional parameters
  if (additionalParams) {
    for (const [key, value] of Object.entries(additionalParams)) {
      if (value) {
        params.append(key, value);
      }
    }
  }

  const queryString = params.toString();
  return queryString ? `${basePath}?${queryString}` : basePath;
}

// Get teams (optionally filtered by player_id)
export async function getTeams(
  playerId?: string,
  limit?: number,
  offset?: number,
): Promise<TeamDb[]> {
  const url = buildPaginatedUrl(
    '/teams',
    { limit, offset },
    playerId ? { player_id: playerId } : undefined,
  );
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

// Get a single team by ID
export async function getTeam(teamId: string): Promise<TeamDb> {
  const response = await apiRequest(`/teams/${teamId}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch team: ${response.statusText}`);
  }

  const data: ApiResponse<TeamDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch team');
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
export async function createTeam(
  request: CreateTeamRequest,
  logoFile?: File | null,
): Promise<TeamDb> {
  // Create FormData for multipart form submission
  const formData = new FormData();
  formData.append('name', request.name);
  formData.append('color', request.color);

  if (request.number !== undefined) {
    formData.append('number', request.number.toString());
  }

  if (request.pit_efficiency !== undefined) {
    formData.append('pit_efficiency', request.pit_efficiency.toString());
  }

  // Add logo file if provided
  if (logoFile) {
    formData.append('logo', logoFile);
  }

  const token = getToken();
  const headers: HeadersInit = {};
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(`${getAPI_URL()}/teams`, {
    method: 'POST',
    headers,
    body: formData,
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
  total_exp: number;
  spent_exp: number;
  team_id: string | null;
  car_id: string | null;
  created_at: string;
  updated_at: string;
  avatar_url: string;
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

// Get a single driver by ID
export async function getDriver(driverId: string): Promise<DriverDb> {
  const response = await apiRequest(`/drivers/${driverId}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch driver: ${response.statusText}`);
  }

  const data: ApiResponse<DriverDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch driver');
}

// Driver race result interface
export interface DriverRaceResultDb {
  race_result_id: string;
  race_id: string;
  track_id: string; // Track identifier (e.g., "bahrain", "monaco") used for routing
  track_name: string;
  race_date: string | null; // ISO date string or null
  final_position: number;
}

// Get race results for a driver
export async function getDriverRaceResults(
  driverId: string,
  limit?: number,
  offset?: number,
): Promise<DriverRaceResultDb[]> {
  const url = buildPaginatedUrl(`/drivers/${driverId}/race-results`, { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch driver race results: ${response.statusText}`);
  }

  const data: ApiResponse<DriverRaceResultDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch driver race results');
}

// Get unassigned drivers (for market)
export async function getUnassignedDrivers(limit?: number, offset?: number): Promise<DriverDb[]> {
  const url = buildPaginatedUrl('/drivers/unassigned', { limit, offset });
  const response = await apiRequest(url);

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
export async function getUnassignedCars(limit?: number, offset?: number): Promise<CarDb[]> {
  const url = buildPaginatedUrl('/cars/unassigned', { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch unassigned cars: ${response.statusText}`);
  }

  const data: ApiResponse<CarDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch unassigned cars');
}

// Buy a driver
export async function buyDriver(driverId: string): Promise<TeamDb> {
  const response = await apiRequest(`/drivers/${driverId}/buy`, {
    method: 'POST',
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to buy driver: ${response.statusText}`);
  }

  const data: ApiResponse<TeamDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to buy driver');
}

// Buy a car
export async function buyCar(carId: string): Promise<TeamDb> {
  const response = await apiRequest(`/cars/${carId}/buy`, {
    method: 'POST',
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to buy car: ${response.statusText}`);
  }

  const data: ApiResponse<TeamDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to buy car');
}

// Get drivers for a team
export async function getTeamDrivers(
  teamId: string,
  limit?: number,
  offset?: number,
): Promise<DriverDb[]> {
  const url = buildPaginatedUrl(`/teams/${teamId}/drivers`, { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch team drivers: ${response.statusText}`);
  }

  const data: ApiResponse<DriverDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch team drivers');
}

// Get cars for a team
export async function getTeamCars(
  teamId: string,
  limit?: number,
  offset?: number,
): Promise<CarDb[]> {
  const url = buildPaginatedUrl(`/teams/${teamId}/cars`, { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch team cars: ${response.statusText}`);
  }

  const data: ApiResponse<CarDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch team cars');
}

// Assign/unassign driver to car
export async function assignDriverToCar(driverId: string, carId: string | null): Promise<DriverDb> {
  const response = await apiRequest(`/drivers/${driverId}/assign-car`, {
    method: 'POST',
    body: JSON.stringify({ car_id: carId }),
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to assign driver to car: ${response.statusText}`);
  }

  const data: ApiResponse<DriverDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to assign driver to car');
}

// Level up a driver by spending experience points
export async function levelUpDriver(driverId: string, stat: string): Promise<DriverDb> {
  const response = await apiRequest(`/drivers/${driverId}/level-up`, {
    method: 'POST',
    body: JSON.stringify({ stat }),
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to level up driver: ${response.statusText}`);
  }

  const data: ApiResponse<DriverDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to level up driver');
}

// Get a single car by ID
export async function getCar(carId: string): Promise<CarDb> {
  const response = await apiRequest(`/cars/${carId}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch car: ${response.statusText}`);
  }

  const data: ApiResponse<CarDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch car');
}

// Improve a car stat by spending team cash
export interface ImproveCarResponse {
  car: CarDb;
  team: TeamDb;
}

export async function improveCar(carId: string, stat: string): Promise<ImproveCarResponse> {
  const response = await apiRequest(`/cars/${carId}/improve`, {
    method: 'POST',
    body: JSON.stringify({ stat }),
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to improve car: ${response.statusText}`);
  }

  const data: ApiResponse<ImproveCarResponse> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to improve car');
}

// Track interfaces
export interface TrackDb {
  id: string;
  track_id: string;
  name: string;
  description: string | null;
  laps: number;
  lap_length_km: number;
  created_at: string;
  updated_at: string;
}

// Get all tracks
export async function getTracks(limit?: number, offset?: number): Promise<TrackDb[]> {
  const url = buildPaginatedUrl('/tracks', { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch tracks: ${response.statusText}`);
  }

  const data: ApiResponse<TrackDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch tracks');
}

// Race interfaces
export interface RaceDb {
  id: string;
  track_id: string;
  laps: number;
  status: string;
  start_datetime: string | null;
  creator_id: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateRaceRequest {
  track_id: string;
  laps: number;
  status?: string;
  start_datetime?: string | null;
  description?: string | null;
}

// Get all races (optionally filtered by status)
export async function getRaces(
  limit?: number,
  offset?: number,
  status?: 'upcoming' | 'done',
): Promise<RaceDb[]> {
  const params: Record<string, string> = {};
  if (limit !== undefined) {
    params.limit = limit.toString();
  }
  if (offset !== undefined) {
    params.offset = offset.toString();
  }
  if (status) {
    params.status = status;
  }

  const queryString = new URLSearchParams(params).toString();
  const url = queryString ? `/races?${queryString}` : '/races';
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch races: ${response.statusText}`);
  }

  const data: ApiResponse<RaceDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch races');
}

// Get upcoming races (REGISTRATION_OPEN, REGISTRATION_CLOSED, ONGOING)
export async function getUpcomingRaces(limit?: number, offset?: number): Promise<RaceDb[]> {
  return getRaces(limit, offset, 'upcoming');
}

// Get done races (FINISHED, CANCELED)
export async function getDoneRaces(limit?: number, offset?: number): Promise<RaceDb[]> {
  return getRaces(limit, offset, 'done');
}

// Get a single race by ID
export async function getRace(raceId: string): Promise<RaceDb> {
  const response = await apiRequest(`/races/${raceId}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch race: ${response.statusText}`);
  }

  const data: ApiResponse<RaceDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch race');
}

// Race Result interface
export interface RaceResultDb {
  id: string;
  race_id: string;
  car_id: string;
  driver_id: string;
  team_id: string;
  car_number: number;
  final_position: number;
  race_time_seconds: number;
  status: 'FINISHED' | 'DNF';
  laps_completed: number;
  total_distance_km: number;
  created_at: string;
  updated_at: string;
}

// Get race results for a race
export async function getRaceResults(raceId: string): Promise<RaceResultDb[]> {
  const response = await apiRequest(`/races/${raceId}/results`);

  if (!response.ok) {
    throw new Error(`Failed to fetch race results: ${response.statusText}`);
  }

  const data: ApiResponse<RaceResultDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch race results');
}

// Create a new race
export async function createRace(request: CreateRaceRequest): Promise<RaceDb> {
  const response = await apiRequest('/races', {
    method: 'POST',
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to create race: ${response.statusText}`);
  }

  const data: ApiResponse<RaceDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to create race');
}

// Registration interfaces
export interface RegistrationDb {
  id: string;
  race_id: string;
  team_id: string;
  created_at: string;
  updated_at: string;
}

// Register team for a race
export async function registerForRace(raceId: string): Promise<RegistrationDb> {
  const response = await apiRequest(`/races/${raceId}/register`, {
    method: 'POST',
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to register for race: ${response.statusText}`);
  }

  const data: ApiResponse<RegistrationDb> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to register for race');
}

// Unregister team from a race
export async function unregisterFromRace(raceId: string): Promise<void> {
  const response = await apiRequest(`/races/${raceId}/register`, {
    method: 'DELETE',
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to unregister from race: ${response.statusText}`);
  }

  const data: ApiResponse<null> = await response.json();
  if (data.status !== 'success') {
    throw new Error(data.message || 'Failed to unregister from race');
  }
}

// Start a race now (loads from DB and starts it)
export async function startRaceNow(raceId: string): Promise<void> {
  const response = await apiRequest(`/races/${raceId}/start-now`, {
    method: 'POST',
  });

  if (!response.ok) {
    const errorData: ApiResponse<null> = await response.json();
    throw new Error(errorData.message || `Failed to start race: ${response.statusText}`);
  }

  const data: ApiResponse<null> = await response.json();
  if (data.status !== 'success') {
    throw new Error(data.message || 'Failed to start race');
  }
}

// Get registrations for a race
export async function getRaceRegistrations(
  raceId: string,
  limit?: number,
  offset?: number,
): Promise<RegistrationDb[]> {
  const url = buildPaginatedUrl(`/races/${raceId}/registrations`, { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch registrations: ${response.statusText}`);
  }

  const data: ApiResponse<RegistrationDb[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch registrations');
}

// Registration with race details interface
export interface RegistrationWithRaceDetails {
  registration_id: string;
  race_id: string;
  team_id: string;
  track_name: string;
  track_id: string;
  laps: number;
  race_status: string;
  start_datetime: string | null;
  description: string | null;
  registration_created_at: string;
}

// Get registrations for a team (with race details)
export async function getTeamRegistrations(
  teamId: string,
  limit?: number,
  offset?: number,
): Promise<RegistrationWithRaceDetails[]> {
  const url = buildPaginatedUrl(`/teams/${teamId}/registrations`, { limit, offset });
  const response = await apiRequest(url);

  if (!response.ok) {
    throw new Error(`Failed to fetch team registrations: ${response.statusText}`);
  }

  const data: ApiResponse<RegistrationWithRaceDetails[]> = await response.json();
  if (data.status === 'success' && data.data) {
    return data.data;
  }

  throw new Error(data.message || 'Failed to fetch team registrations');
}
