<template>
  <div class="races-container">
    <div class="races-content">
      <div class="races-header">
        <h2>Races</h2>
        <!-- Create Race Button (for authenticated users) -->
        <div v-if="authenticated && !showCreateForm" class="create-race-button-container">
          <button @click="showCreateForm = true" class="btn btn-primary">Create Race</button>
        </div>
      </div>

      <!-- Create Race Form (for authenticated users) -->
      <div v-if="authenticated && showCreateForm" class="create-race-section">
        <div class="form-header">
          <h3>Create New Race</h3>
          <button @click="closeCreateForm" class="btn-close" type="button" aria-label="Close form">
            ×
          </button>
        </div>
        <form @submit.prevent="handleCreateRace" class="race-form">
          <div class="form-group">
            <label for="track">Track *</label>
            <select id="track" v-model="formData.track_id" required>
              <option value="">Select a track</option>
              <option v-for="track in tracks" :key="track.id" :value="track.id">
                {{ track.name }}
              </option>
            </select>
          </div>

          <div class="form-group">
            <label for="laps">Number of Laps *</label>
            <input id="laps" type="number" v-model.number="formData.laps" min="1" required />
          </div>

          <div class="form-group">
            <label for="start_datetime">Start Date (optional)</label>
            <input id="start_datetime" type="datetime-local" v-model="formData.start_datetime" />
          </div>

          <div class="form-group">
            <label for="description">Description (optional)</label>
            <textarea
              id="description"
              v-model="formData.description"
              rows="3"
              placeholder="Enter race description..."
            ></textarea>
          </div>

          <div class="form-actions">
            <button type="submit" class="btn btn-primary" :disabled="creating">
              <span v-if="creating">Creating...</span>
              <span v-else>Create Race</span>
            </button>
            <button type="button" class="btn btn-secondary" @click="resetForm">Reset</button>
          </div>

          <div v-if="createError" class="error-message">{{ createError }}</div>
          <div v-if="createSuccess" class="success-message">{{ createSuccess }}</div>
        </form>
      </div>

      <!-- Loading state -->
      <div v-if="loadingUpcoming || loadingDone" class="loading-message">Loading races...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Races list -->
      <div v-if="!loadingUpcoming && !loadingDone && !error" class="races-list">
        <!-- Upcoming Races Section (REGISTRATION_OPEN, REGISTRATION_CLOSED, ONGOING) -->
        <div v-if="displayUpcomingRaces.length > 0 || loadingUpcoming" class="race-section">
          <h3 class="section-header">Upcoming Races</h3>
          <div class="races-table-wrapper">
            <table class="races-table">
              <thead>
                <tr>
                  <th>Race Date</th>
                  <th>Track</th>
                  <th>Laps</th>
                  <th>Status</th>
                  <th>Description</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="race in displayUpcomingRaces"
                  :key="race.id"
                  :class="getRaceStatusClass(race.status)"
                >
                  <td>{{ race.start_datetime ? formatDate(race.start_datetime) : 'N/A' }}</td>
                  <td>{{ getTrackName(race.track_id) }}</td>
                  <td>{{ race.laps }}</td>
                  <td>
                    <span class="status-badge" :class="getRaceStatusClass(race.status)">
                      {{ formatStatus(race.status) }}
                    </span>
                  </td>
                  <td>{{ race.description || 'N/A' }}</td>
                  <td class="actions-cell">
                    <!-- Registration buttons for authenticated users with a team -->
                    <div
                      v-if="
                        authenticated &&
                        myTeam &&
                        (race.status === 'REGISTRATION_OPEN' ||
                          race.status === 'REGISTRATION_CLOSED')
                      "
                      class="action-buttons"
                    >
                      <button
                        v-if="canRegister(race)"
                        type="button"
                        @click.prevent="handleRegister(race.id)"
                        class="btn btn-success btn-small"
                        :disabled="registering.get(race.id) || starting.get(race.id)"
                      >
                        <span v-if="registering.get(race.id)">Registering...</span>
                        <span v-else>Register</span>
                      </button>
                      <button
                        v-if="canUnregister(race)"
                        type="button"
                        @click.prevent="handleUnregister(race.id)"
                        class="btn btn-warning btn-small"
                        :disabled="registering.get(race.id) || starting.get(race.id)"
                      >
                        <span v-if="registering.get(race.id)">Unregistering...</span>
                        <span v-else>Unregister</span>
                      </button>
                    </div>
                    <!-- Start now button for upcoming races -->
                    <button
                      v-if="
                        race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED'
                      "
                      type="button"
                      @click.prevent="handleStartNow(race.id)"
                      class="btn btn-primary btn-small"
                      :disabled="starting.get(race.id)"
                    >
                      <span v-if="starting.get(race.id)">Starting...</span>
                      <span v-else>Start now</span>
                    </button>
                    <!-- View race button for ongoing races -->
                    <button
                      v-if="race.status === 'ONGOING'"
                      type="button"
                      @click.prevent="handleViewRace(race.id)"
                      class="btn btn-primary btn-small"
                    >
                      View Race
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <!-- Load More button for upcoming races -->
          <div v-if="hasMoreUpcoming || loadingMoreUpcoming" class="load-more-container">
            <button
              @click="loadUpcomingRaces(false)"
              class="btn btn-secondary"
              :disabled="loadingMoreUpcoming"
            >
              <span v-if="loadingMoreUpcoming">Loading...</span>
              <span v-else>Load More</span>
            </button>
          </div>
        </div>

        <!-- Done Races Section (FINISHED, CANCELED) -->
        <div v-if="displayDoneRaces.length > 0 || loadingDone" class="race-section">
          <h3 class="section-header">Done Races</h3>
          <div class="races-table-wrapper">
            <table class="races-table">
              <thead>
                <tr>
                  <th>Race Date</th>
                  <th>Track</th>
                  <th>Laps</th>
                  <th>Status</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="race in displayDoneRaces"
                  :key="race.id"
                  :class="getRaceStatusClass(race.status)"
                >
                  <td>{{ race.start_datetime ? formatDate(race.start_datetime) : 'N/A' }}</td>
                  <td>{{ getTrackName(race.track_id) }}</td>
                  <td>{{ race.laps }}</td>
                  <td>
                    <span class="status-badge" :class="getRaceStatusClass(race.status)">
                      {{ formatStatus(race.status) }}
                    </span>
                  </td>
                  <td class="actions-cell">
                    <button
                      v-if="race.status === 'FINISHED'"
                      type="button"
                      @click.prevent="handleViewResults(race.id)"
                      class="btn btn-primary btn-small"
                    >
                      View Results
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <!-- Load More button for done races -->
          <div v-if="hasMoreDone || loadingMoreDone" class="load-more-container">
            <button
              @click="loadDoneRaces(false)"
              class="btn btn-secondary"
              :disabled="loadingMoreDone"
            >
              <span v-if="loadingMoreDone">Loading...</span>
              <span v-else>Load More</span>
            </button>
          </div>
        </div>

        <!-- Race Results Modal -->
        <div v-if="showResults" class="modal-overlay" @click="closeResults">
          <div class="modal-content" @click.stop>
            <div class="modal-header">
              <div class="modal-header-content">
                <h3>Race Results</h3>
                <div v-if="selectedRace" class="race-details">
                  <div class="race-detail-item">
                    <span class="detail-label">Track:</span>
                    <span class="detail-value">{{ getTrackName(selectedRace.track_id) }}</span>
                  </div>
                  <div class="race-detail-item">
                    <span class="detail-label">Laps:</span>
                    <span class="detail-value">{{ selectedRace.laps }}</span>
                  </div>
                  <div class="race-detail-item">
                    <span class="detail-label">Date:</span>
                    <span class="detail-value">{{
                      selectedRace.start_datetime ? formatDate(selectedRace.start_datetime) : 'N/A'
                    }}</span>
                  </div>
                </div>
              </div>
              <button @click="closeResults" class="btn-close" type="button" aria-label="Close">
                ×
              </button>
            </div>
            <div class="modal-body">
              <div v-if="loadingResults" class="loading-message">Loading results...</div>
              <div v-else-if="resultsError" class="error-message">{{ resultsError }}</div>
              <div v-else-if="raceResults.length === 0" class="empty-state">
                <p>No results available for this race.</p>
              </div>
              <div v-else class="results-table-wrapper">
                <table class="results-table">
                  <thead>
                    <tr>
                      <th>Position</th>
                      <th>Driver</th>
                      <th>Team</th>
                      <th>Status</th>
                      <th>Laps</th>
                      <th>Time</th>
                      <th>Distance</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="result in raceResults"
                      :key="result.id"
                      :class="{
                        'result-finished': result.status === 'FINISHED',
                        'result-dnf': result.status === 'DNF',
                      }"
                    >
                      <td
                        class="position-cell"
                        :class="{
                          'position-1': result.final_position === 1,
                          'position-2': result.final_position === 2,
                          'position-3': result.final_position === 3,
                        }"
                      >
                        <div class="position-content">
                          <img
                            v-if="result.final_position === 1"
                            src="/assets/awards/laurels_gold.svg"
                            alt="Gold"
                            class="laurel-icon"
                          />
                          <img
                            v-if="result.final_position === 2"
                            src="/assets/awards/laurels_silver.svg"
                            alt="Silver"
                            class="laurel-icon"
                          />
                          <img
                            v-if="result.final_position === 3"
                            src="/assets/awards/laurels_bronze.svg"
                            alt="Bronze"
                            class="laurel-icon"
                          />
                          <span>{{ result.final_position }}</span>
                        </div>
                      </td>
                      <td>
                        <div class="driver-cell">
                          <img
                            v-if="getDriverAvatar(result.driver_id)"
                            :src="getDriverAvatar(result.driver_id) || ''"
                            :alt="getDriverName(result.driver_id)"
                            class="driver-avatar"
                          />
                          <span>{{ getDriverName(result.driver_id) }}</span>
                        </div>
                      </td>
                      <td>
                        <div class="team-cell">
                          <img
                            v-if="getTeamLogo(result.team_id)"
                            :src="getTeamLogo(result.team_id) || ''"
                            :alt="getTeamName(result.team_id)"
                            class="team-logo"
                          />
                          <span>{{ getTeamName(result.team_id) }}</span>
                        </div>
                      </td>
                      <td>
                        <span
                          class="status-badge"
                          :class="{
                            'status-finished': result.status === 'FINISHED',
                            'status-dnf': result.status === 'DNF',
                          }"
                        >
                          {{ result.status }}
                        </span>
                      </td>
                      <td>{{ result.laps_completed }}</td>
                      <td>{{ formatRaceTime(result.race_time_seconds) }}</td>
                      <td>{{ result.total_distance_km.toFixed(2) }} km</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>

        <!-- Empty state -->
        <div
          v-if="
            displayUpcomingRaces.length === 0 &&
            displayDoneRaces.length === 0 &&
            !loadingUpcoming &&
            !loadingDone
          "
          class="empty-state"
        >
          <p>No races found. Create one to get started!</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import {
  getUpcomingRaces,
  getDoneRaces,
  getRace,
  createRace,
  getTracks,
  getMyTeam,
  getTeam,
  getDriver,
  registerForRace,
  unregisterFromRace,
  getRaceRegistrations,
  startRaceNow,
  getRaceResults,
  type RaceDb,
  type CreateRaceRequest,
  type TrackDb,
  type TeamDb,
  type DriverDb,
  type RegistrationDb,
  type RaceResultDb,
} from '../services/ApiService';

const props = defineProps<{
  authenticated: boolean;
}>();

const loadingUpcoming = ref(false);
const loadingDone = ref(false);
const error = ref<string | null>(null);
const upcomingRaces = ref<RaceDb[]>([]);
const doneRaces = ref<RaceDb[]>([]);
const tracks = ref<TrackDb[]>([]);
const creating = ref(false);
const createError = ref<string | null>(null);
const createSuccess = ref<string | null>(null);
const showCreateForm = ref(false);
const myTeam = ref<TeamDb | null>(null);
const registrations = ref<Map<string, RegistrationDb>>(new Map());
const registering = ref<Map<string, boolean>>(new Map());
const starting = ref<Map<string, boolean>>(new Map());
const showResults = ref(false);
const selectedRaceId = ref<string | null>(null);
const selectedRace = ref<RaceDb | null>(null);
const raceResults = ref<RaceResultDb[]>([]);
const loadingResults = ref(false);
const resultsError = ref<string | null>(null);
const driversMap = ref<Map<string, DriverDb>>(new Map());
const teamsMap = ref<Map<string, TeamDb>>(new Map());

// Pagination state
const PAGE_SIZE = 10;
const upcomingOffset = ref(0);
const doneOffset = ref(0);
const hasMoreUpcoming = ref(true);
const hasMoreDone = ref(true);
const loadingMoreUpcoming = ref(false);
const loadingMoreDone = ref(false);

const emit = defineEmits<{
  navigate: [view: string];
}>();

const formData = ref<Omit<CreateRaceRequest, 'status'>>({
  track_id: '',
  laps: 10,
  start_datetime: null,
  description: null,
});

// Computed properties for displaying races
const displayUpcomingRaces = computed(() => {
  // Separate upcoming races (REGISTRATION_OPEN, REGISTRATION_CLOSED) and ongoing races
  const upcoming = upcomingRaces.value.filter(
    (race) => race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED',
  );
  const ongoing = upcomingRaces.value.filter((race) => race.status === 'ONGOING');

  // Sort upcoming chronologically by start_datetime (earliest first)
  const sortedUpcoming = upcoming.sort((a, b) => {
    if (!a.start_datetime && !b.start_datetime) return 0;
    if (!a.start_datetime) return 1;
    if (!b.start_datetime) return -1;
    return new Date(a.start_datetime).getTime() - new Date(b.start_datetime).getTime();
  });

  // Combine: upcoming first, then ongoing
  return [...sortedUpcoming, ...ongoing];
});

const displayDoneRaces = computed(() => {
  // Sort done races by created_at DESC (most recent first)
  return [...doneRaces.value].sort((a, b) => {
    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
  });
});

async function loadUpcomingRaces(reset = false) {
  if (reset) {
    upcomingOffset.value = 0;
    upcomingRaces.value = [];
    hasMoreUpcoming.value = true;
  }

  if (!hasMoreUpcoming.value) return;

  if (reset) {
    loadingUpcoming.value = true;
  } else {
    loadingMoreUpcoming.value = true;
  }
  error.value = null;

  try {
    const races = await getUpcomingRaces(PAGE_SIZE, upcomingOffset.value);
    if (reset) {
      upcomingRaces.value = races;
    } else {
      upcomingRaces.value.push(...races);
    }

    // Check if there are more races to load
    hasMoreUpcoming.value = races.length === PAGE_SIZE;
    upcomingOffset.value += races.length;

    // Load registrations for races if authenticated
    if (props.authenticated && myTeam.value) {
      await loadRegistrationsForRaces(races);
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load upcoming races';
  } finally {
    loadingUpcoming.value = false;
    loadingMoreUpcoming.value = false;
  }
}

async function loadDoneRaces(reset = false) {
  if (reset) {
    doneOffset.value = 0;
    doneRaces.value = [];
    hasMoreDone.value = true;
  }

  if (!hasMoreDone.value) return;

  if (reset) {
    loadingDone.value = true;
  } else {
    loadingMoreDone.value = true;
  }
  error.value = null;

  try {
    const races = await getDoneRaces(PAGE_SIZE, doneOffset.value);
    if (reset) {
      doneRaces.value = races;
    } else {
      doneRaces.value.push(...races);
    }

    // Check if there are more races to load
    hasMoreDone.value = races.length === PAGE_SIZE;
    doneOffset.value += races.length;

    // Load registrations for races if authenticated
    if (props.authenticated && myTeam.value) {
      await loadRegistrationsForRaces(races);
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load done races';
  } finally {
    loadingDone.value = false;
    loadingMoreDone.value = false;
  }
}

async function loadRegistrationsForRaces(races: RaceDb[]) {
  if (!myTeam.value) return;

  // Load registrations for each race in parallel
  const promises = races.map(async (race) => {
    try {
      const regs = await getRaceRegistrations(race.id);
      const myRegistration = regs.find((r) => r.team_id === myTeam.value!.id);
      if (myRegistration) {
        registrations.value.set(race.id, myRegistration);
      } else {
        registrations.value.delete(race.id);
      }
    } catch (err) {
      console.error(`Failed to load registrations for race ${race.id}:`, err);
    }
  });

  await Promise.all(promises);
}

async function loadTracks() {
  try {
    tracks.value = await getTracks();
  } catch (err) {
    console.error('Failed to load tracks:', err);
  }
}

async function loadMyTeam() {
  if (!props.authenticated) {
    return;
  }
  try {
    myTeam.value = await getMyTeam();
    // Reload races to get registrations after team is loaded
    if (myTeam.value) {
      await loadUpcomingRaces(true);
      await loadDoneRaces(true);
    }
  } catch (err) {
    console.error('Failed to load team:', err);
  }
}

async function handleRegister(raceId: string) {
  if (!myTeam.value) {
    error.value = 'You need to have a team to register for a race';
    return;
  }

  registering.value.set(raceId, true);
  try {
    await registerForRace(raceId);

    // Update registration status locally
    const registration: RegistrationDb = {
      id: '', // Not needed for local state
      race_id: raceId,
      team_id: myTeam.value.id,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    registrations.value.set(raceId, registration);

    // Fetch just this race to check if status changed (e.g., REGISTRATION_OPEN -> REGISTRATION_CLOSED)
    try {
      const updatedRace = await getRace(raceId);
      // Update the race in the local array
      const raceIndex = upcomingRaces.value.findIndex((r) => r.id === raceId);
      if (raceIndex !== -1) {
        upcomingRaces.value[raceIndex] = updatedRace;
      }
    } catch (err) {
      console.error('Failed to fetch updated race:', err);
      // Continue anyway - registration was successful
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to register for race';
  } finally {
    registering.value.set(raceId, false);
  }
}

async function handleUnregister(raceId: string) {
  if (!myTeam.value) {
    error.value = 'You need to have a team to unregister from a race';
    return;
  }

  registering.value.set(raceId, true);
  try {
    await unregisterFromRace(raceId);

    // Remove registration status locally
    registrations.value.delete(raceId);

    // Fetch just this race to check if status changed (e.g., REGISTRATION_CLOSED -> REGISTRATION_OPEN)
    try {
      const updatedRace = await getRace(raceId);
      // Update the race in the local array
      const raceIndex = upcomingRaces.value.findIndex((r) => r.id === raceId);
      if (raceIndex !== -1) {
        upcomingRaces.value[raceIndex] = updatedRace;
      }
    } catch (err) {
      console.error('Failed to fetch updated race:', err);
      // Continue anyway - unregistration was successful
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to unregister from race';
  } finally {
    registering.value.set(raceId, false);
  }
}

async function handleStartNow(raceId: string) {
  starting.value.set(raceId, true);
  try {
    await startRaceNow(raceId);
    // Navigate to game view
    emit('navigate', 'game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to start race';
  } finally {
    starting.value.set(raceId, false);
  }
}

function handleViewRace(raceId: string) {
  // For ongoing races, just navigate to the game view without restarting
  emit('navigate', 'game');
}

async function handleViewResults(raceId: string) {
  selectedRaceId.value = raceId;
  showResults.value = true;
  loadingResults.value = true;
  resultsError.value = null;
  raceResults.value = [];
  selectedRace.value = null;
  driversMap.value.clear();
  teamsMap.value.clear();

  try {
    // Find the race in the loaded races arrays
    const race =
      upcomingRaces.value.find((r) => r.id === raceId) ||
      doneRaces.value.find((r) => r.id === raceId);
    if (race) {
      selectedRace.value = race;
    } else {
      // If not found, fetch it
      selectedRace.value = await getRace(raceId);
    }
    raceResults.value = await getRaceResults(raceId);

    // Load driver and team information for all results
    const uniqueDriverIds = [...new Set(raceResults.value.map((r) => r.driver_id))];
    const uniqueTeamIds = [...new Set(raceResults.value.map((r) => r.team_id))];

    // Load drivers in parallel
    const driverPromises = uniqueDriverIds.map(async (driverId) => {
      try {
        const driver = await getDriver(driverId);
        driversMap.value.set(driverId, driver);
      } catch (err) {
        console.error(`Failed to load driver ${driverId}:`, err);
      }
    });

    // Load teams in parallel
    const teamPromises = uniqueTeamIds.map(async (teamId) => {
      try {
        const team = await getTeam(teamId);
        teamsMap.value.set(teamId, team);
      } catch (err) {
        console.error(`Failed to load team ${teamId}:`, err);
      }
    });

    await Promise.all([...driverPromises, ...teamPromises]);
  } catch (err) {
    resultsError.value = err instanceof Error ? err.message : 'Failed to load race results';
  } finally {
    loadingResults.value = false;
  }
}

function closeResults() {
  showResults.value = false;
  selectedRaceId.value = null;
  selectedRace.value = null;
  raceResults.value = [];
  resultsError.value = null;
  driversMap.value.clear();
  teamsMap.value.clear();
}

function formatRaceTime(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const secs = (seconds % 60).toFixed(2);
  return `${minutes}:${secs.padStart(5, '0')}`;
}

function getDriverName(driverId: string): string {
  const driver = driversMap.value.get(driverId);
  if (driver) {
    return `${driver.first_name} ${driver.last_name}`;
  }
  return `Driver ${driverId.slice(0, 8)}`;
}

function getDriverAvatar(driverId: string): string | null {
  const driver = driversMap.value.get(driverId);
  return driver?.avatar_url || null;
}

function getTeamName(teamId: string): string {
  const team = teamsMap.value.get(teamId);
  return team?.name || `Team ${teamId.slice(0, 8)}`;
}

function getTeamLogo(teamId: string): string | null {
  const team = teamsMap.value.get(teamId);
  return team?.logo || null;
}

const isRegistered = (raceId: string): boolean => {
  return registrations.value.has(raceId);
};

const canRegister = (race: RaceDb): boolean => {
  return race.status === 'REGISTRATION_OPEN' && !isRegistered(race.id);
};

const canUnregister = (race: RaceDb): boolean => {
  // Allow unregistration if race is open or closed (to free up spots)
  return (
    (race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED') &&
    isRegistered(race.id)
  );
};

async function handleCreateRace() {
  creating.value = true;
  createError.value = null;
  createSuccess.value = null;

  try {
    // Convert datetime-local format to ISO string with timezone
    let startDatetime: string | null = null;
    if (formData.value.start_datetime) {
      // datetime-local format is "YYYY-MM-DDTHH:mm" without timezone
      // We need to convert it to ISO 8601 format with timezone
      const localDate = new Date(formData.value.start_datetime);
      if (!isNaN(localDate.getTime())) {
        startDatetime = localDate.toISOString();
      }
    }

    const request: CreateRaceRequest = {
      track_id: formData.value.track_id,
      laps: formData.value.laps,
      status: 'REGISTRATION_OPEN', // Always start with REGISTRATION_OPEN
      start_datetime: startDatetime,
      description: formData.value.description || null,
    };

    await createRace(request);
    createSuccess.value = 'Race created successfully!';
    resetForm();
    showCreateForm.value = false;
    await loadUpcomingRaces(true); // Reload upcoming races list
  } catch (err) {
    createError.value = err instanceof Error ? err.message : 'Failed to create race';
  } finally {
    creating.value = false;
  }
}

function resetForm() {
  formData.value = {
    track_id: '',
    laps: 10,
    start_datetime: null,
    description: null,
  };
  createError.value = null;
  createSuccess.value = null;
}

function closeCreateForm() {
  showCreateForm.value = false;
  resetForm();
}

function getTrackName(trackId: string): string {
  const track = tracks.value.find((t) => t.id === trackId);
  return track ? track.name : 'Unknown Track';
}

function formatStatus(status: string): string {
  return status
    .replace(/_/g, ' ')
    .toLowerCase()
    .split(' ')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

function getRaceStatusClass(status: string): string {
  const statusLower = status.toLowerCase();
  if (statusLower === 'finished') return 'status-finished';
  if (statusLower === 'ongoing') return 'status-ongoing';
  if (statusLower === 'registration_open') return 'status-open';
  if (statusLower === 'registration_closed') return 'status-closed';
  if (statusLower === 'canceled') return 'status-canceled';
  return '';
}

function formatDate(dateString: string | null): string {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  // Use local formatting with date and time
  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });
}

// Reload data when authenticated status changes
watch(
  () => props.authenticated,
  async (newAuth) => {
    if (newAuth) {
      // When user becomes authenticated, reload team and registrations
      await loadMyTeam();
      if (myTeam.value) {
        await loadUpcomingRaces(true);
        await loadDoneRaces(true);
      }
    } else {
      // Clear data when logged out
      myTeam.value = null;
      registrations.value.clear();
    }
  },
  { immediate: false },
);

onMounted(async () => {
  await loadTracks();
  await loadMyTeam();
  await loadUpcomingRaces(true);
  await loadDoneRaces(true);
});
</script>

<style scoped>
.races-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.races-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.races-content h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin: 0;
}

.races-description {
  color: #666;
  font-size: 1.1rem;
  margin-bottom: 2rem;
}

.create-race-button-container {
  margin: 0;
}

.create-race-section {
  background: white;
  border-radius: 8px;
  padding: 1.5rem;
  margin-bottom: 2rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.create-race-section h3 {
  color: #1a1a2e;
  font-size: 1.5rem;
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  font-size: 2rem;
  color: #666;
  cursor: pointer;
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s ease;
}

.btn-close:hover {
  background-color: #f0f0f0;
  color: #333;
}

.race-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group label {
  font-weight: 500;
  color: #333;
}

.form-group input,
.form-group select,
.form-group textarea {
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
  font-family: inherit;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #2d4059;
}

.form-actions {
  display: flex;
  gap: 1rem;
  margin-top: 0.5rem;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background-color: #2d4059;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #3a5273;
}

.btn-primary:disabled {
  background-color: #999;
  cursor: not-allowed;
}

.btn-secondary {
  background-color: #e0e0e0;
  color: #333;
}

.btn-secondary:hover {
  background-color: #d0d0d0;
}

.loading-message,
.error-message,
.success-message {
  padding: 1rem;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.loading-message {
  background-color: #e3f2fd;
  color: #1976d2;
}

.error-message {
  background-color: #ffebee;
  color: #c62828;
}

.success-message {
  background-color: #e8f5e9;
  color: #2e7d32;
}

.race-section {
  margin-bottom: 3rem;
}

.section-header {
  color: #1a1a2e;
  font-size: 1.5rem;
  margin-bottom: 1rem;
  padding-bottom: 0.5rem;
  border-bottom: 2px solid #e0e0e0;
}

.empty-state {
  text-align: center;
  padding: 3rem;
  color: #666;
}

.races-table-wrapper {
  overflow-x: auto;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.races-table {
  width: 100%;
  border-collapse: collapse;
}

.races-table thead {
  background-color: #f5f5f5;
}

.races-table th {
  padding: 0.5rem 1rem;
  text-align: left;
  font-weight: 600;
  color: #1a1a2e;
  border-bottom: 2px solid #e0e0e0;
}

.races-table td {
  padding: 0.5rem 1rem;
  border-bottom: 1px solid #e0e0e0;
  color: #333;
}

.races-table tbody tr:hover {
  background-color: #f9f9f9;
}

.races-table tbody tr.status-finished {
  border-left: 4px solid #7b1fa2;
}

.races-table tbody tr.status-ongoing {
  border-left: 4px solid #1976d2;
}

.races-table tbody tr.status-open {
  border-left: 4px solid #2e7d32;
}

.races-table tbody tr.status-closed {
  border-left: 4px solid #e65100;
}

.races-table tbody tr.status-canceled {
  border-left: 4px solid #c62828;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.875rem;
  font-weight: 500;
  text-transform: capitalize;
  display: inline-block;
}

.status-open {
  background-color: #e8f5e9;
  color: #2e7d32;
}

.status-closed {
  background-color: #fff3e0;
  color: #e65100;
}

.status-ongoing {
  background-color: #e3f2fd;
  color: #1976d2;
}

.status-finished {
  background-color: #f3e5f5;
  color: #7b1fa2;
}

.status-canceled {
  background-color: #ffebee;
  color: #c62828;
}

.actions-cell {
  white-space: nowrap;
}

.action-buttons {
  display: flex;
  gap: 0.25rem;
  flex-wrap: wrap;
}

.btn-small {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
}

.registration-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 500;
}

.registration-status.registered {
  color: #2e7d32;
}

.registration-status.full {
  color: #e65100;
}

.registration-icon {
  font-size: 1.2rem;
  font-weight: bold;
}

.action-buttons {
  display: flex;
  gap: 0.5rem;
}

.btn-success {
  background-color: #2e7d32;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background-color: #1b5e20;
}

.btn-warning {
  background-color: #f57c00;
  color: white;
}

.btn-warning:hover:not(:disabled) {
  background-color: #e65100;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .races-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 1rem;
  }

  .create-race-button-container {
    width: 100%;
  }

  .create-race-button-container .btn {
    width: 100%;
  }

  .races-table-wrapper {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .races-table {
    min-width: 600px;
  }

  .races-table th,
  .races-table td {
    padding: 0.5rem 0.5rem;
    font-size: 0.875rem;
  }

  .form-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }

  .action-buttons {
    flex-direction: column;
    width: 100%;
  }

  .action-buttons .btn {
    width: 100%;
  }

  .actions-cell {
    min-width: 150px;
  }
}

.load-more-container {
  display: flex;
  justify-content: center;
  margin-top: 1.5rem;
  padding: 1rem;
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 8px;
  max-width: 90%;
  max-height: 90vh;
  width: 100%;
  max-width: 1200px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.modal-header-content {
  flex: 1;
}

.modal-header h3 {
  margin: 0 0 1rem 0;
  color: #1a1a2e;
}

.race-details {
  display: flex;
  flex-wrap: wrap;
  gap: 1.5rem;
  margin-top: 0.5rem;
}

.race-detail-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.detail-label {
  font-weight: 600;
  color: #666;
  font-size: 0.875rem;
}

.detail-value {
  color: #1a1a2e;
  font-size: 0.875rem;
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
  flex: 1;
}

/* Results Table Styles */
.results-table-wrapper {
  overflow-x: auto;
  width: 100%;
}

.results-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 1rem;
  table-layout: auto;
  display: table;
}

.results-table th {
  padding: 0.75rem 1rem;
  text-align: left;
  font-weight: 600;
  color: #1a1a2e;
  border-bottom: 2px solid #e0e0e0;
  background-color: #f5f5f5;
  position: sticky;
  top: 0;
}

.results-table td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #e0e0e0;
  color: #333;
  vertical-align: middle;
}

.driver-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.driver-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e0e0e0;
  flex-shrink: 0;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.results-table .team-logo {
  width: 32px;
  height: 32px;
  object-fit: contain;
  border-radius: 4px;
}

.results-table tbody tr:hover {
  background-color: #f9f9f9;
}

.results-table tbody tr.result-finished {
  border-left: 4px solid #4caf50;
}

.results-table tbody tr.result-dnf {
  border-left: 4px solid #f44336;
}

.position-cell {
  font-weight: 600;
  font-size: 1.1rem;
  text-align: center;
}

.position-cell.position-1,
.position-cell.position-2,
.position-cell.position-3 {
  font-weight: 700;
}

.position-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.laurel-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}

.status-finished {
  background-color: #e8f5e9;
  color: #2e7d32;
}

.status-dnf {
  background-color: #ffebee;
  color: #c62828;
}

@media (max-width: 768px) {
  .modal-content {
    max-width: 95%;
    max-height: 95vh;
  }

  .modal-header {
    flex-direction: column;
    align-items: stretch;
  }

  .modal-header-content {
    width: 100%;
  }

  .race-details {
    flex-direction: column;
    gap: 0.75rem;
  }

  .race-detail-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
  }

  .results-table {
    font-size: 0.875rem;
  }

  .results-table th,
  .results-table td {
    padding: 0.5rem 0.5rem;
  }
}
</style>
