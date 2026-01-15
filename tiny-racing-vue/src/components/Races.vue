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
                  <th class="mobile-visible">Race Date</th>
                  <th class="mobile-visible">Track</th>
                  <th class="mobile-hidden">Laps</th>
                  <th class="mobile-hidden">Status</th>
                  <th class="mobile-hidden">Description</th>
                  <th class="mobile-hidden">Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="race in displayUpcomingRaces"
                  :key="race.id"
                  :class="getRaceStatusClass(race.status)"
                  class="race-row"
                  @click="handleRaceRowClick(race, $event)"
                >
                  <td class="mobile-visible">
                    {{ race.start_datetime ? formatDate(race.start_datetime) : 'N/A' }}
                  </td>
                  <td class="mobile-visible">
                    <router-link
                      :to="{ name: 'track', params: { trackId: getTrackId(race.track_id) } }"
                      class="track-link"
                      @click.stop
                    >
                      {{ getTrackName(race.track_id) }}
                    </router-link>
                  </td>
                  <td class="mobile-hidden">{{ race.laps }}</td>
                  <td class="mobile-hidden">
                    <span class="status-badge" :class="getRaceStatusClass(race.status)">
                      {{ formatStatus(race.status) }}
                    </span>
                  </td>
                  <td class="mobile-hidden">{{ race.description || 'N/A' }}</td>
                  <td class="actions-cell mobile-hidden">
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
                  <th class="mobile-visible">Race Date</th>
                  <th class="mobile-visible">Track</th>
                  <th class="mobile-hidden">Laps</th>
                  <th class="mobile-hidden">Status</th>
                  <th class="mobile-hidden">Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="race in displayDoneRaces"
                  :key="race.id"
                  :class="getRaceStatusClass(race.status)"
                  class="race-row"
                  @click="handleRaceRowClick(race, $event)"
                >
                  <td class="mobile-visible">
                    {{ race.start_datetime ? formatDate(race.start_datetime) : 'N/A' }}
                  </td>
                  <td class="mobile-visible">
                    <router-link
                      :to="{ name: 'track', params: { trackId: getTrackId(race.track_id) } }"
                      class="track-link"
                      @click.stop
                    >
                      {{ getTrackName(race.track_id) }}
                    </router-link>
                  </td>
                  <td class="mobile-hidden">{{ race.laps }}</td>
                  <td class="mobile-hidden">
                    <span class="status-badge" :class="getRaceStatusClass(race.status)">
                      {{ formatStatus(race.status) }}
                    </span>
                  </td>
                  <td class="actions-cell mobile-hidden">
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
        <RaceResultsModal
          :visible="showResults"
          :race="selectedRace"
          :results="raceResults"
          :loading="loadingResults"
          :error="resultsError"
          :track-name="selectedRace ? getTrackName(selectedRace.track_id) : undefined"
          :drivers="driversMap"
          :teams="teamsMap"
          @close="closeResults"
        />

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

    <!-- Mobile Race Detail Popup -->
    <MobileRaceDetailPopup
      :visible="showMobileRaceDetail"
      :race="selectedMobileRace"
      :track-name="selectedMobileRace ? getTrackName(selectedMobileRace.track_id) : ''"
      :authenticated="authenticated"
      :my-team="myTeam"
      :is-registered="isRegistered"
      :registering="registering"
      :starting="starting"
      @close="closeMobileRaceDetail"
      @register="handleRegister"
      @unregister="handleUnregister"
      @start-now="handleStartNow"
      @view-race="handleViewRace"
      @view-results="handleViewResults"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
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
import RaceResultsModal from './RaceResultsModal.vue';
import MobileRaceDetailPopup from './MobileRaceDetailPopup.vue';

const route = useRoute();
const router = useRouter();

const props = defineProps<{
  authenticated?: boolean;
  raceId?: string;
}>();

// Get raceId from route params if not provided as prop
const raceId = computed(() => {
  return props.raceId || (route.params.raceId as string) || null;
});

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

// Mobile race detail popup state
const showMobileRaceDetail = ref(false);
const selectedMobileRace = ref<RaceDb | null>(null);

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
  // Backend already sorts by start_datetime DESC, but we sort here too for consistency
  // Sort by start_datetime (or created_at as fallback) DESC (most recent first)
  return [...doneRaces.value].sort((a, b) => {
    const dateA = a.start_datetime
      ? new Date(a.start_datetime).getTime()
      : new Date(a.created_at).getTime();
    const dateB = b.start_datetime
      ? new Date(b.start_datetime).getTime()
      : new Date(b.created_at).getTime();
    return dateB - dateA; // DESC order (newest first)
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

async function handleStartNow(raceIdParam: string) {
  starting.value.set(raceIdParam, true);
  try {
    await startRaceNow(raceIdParam);
    // Navigate to game view
    router.push({ name: 'game' });
    emit('navigate', 'game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to start race';
  } finally {
    starting.value.set(raceIdParam, false);
  }
}

function handleViewRace(raceIdParam: string) {
  // For ongoing races, just navigate to the game view without restarting
  router.push({ name: 'game' });
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

function handleRaceRowClick(race: RaceDb, event: MouseEvent) {
  // Only open popup on mobile (screen width <= 768px)
  if (window.innerWidth <= 768) {
    openMobileRaceDetail(race);
  }
}

function openMobileRaceDetail(race: RaceDb) {
  selectedMobileRace.value = race;
  showMobileRaceDetail.value = true;
}

function closeMobileRaceDetail() {
  showMobileRaceDetail.value = false;
  selectedMobileRace.value = null;
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

function getTrackId(trackId: string): string {
  const track = tracks.value.find((t) => t.id === trackId);
  return track ? track.track_id : trackId;
}

function formatStatus(status: string): string {
  return status.replace(/_/g, ' ').toLowerCase();
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

// Watch for route changes (raceId changes)
watch(
  () => raceId.value,
  async (newRaceId) => {
    if (newRaceId) {
      // If viewing a specific race, show its results
      await handleViewResults(newRaceId);
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await loadTracks();
  await loadMyTeam();
  await loadUpcomingRaces(true);
  await loadDoneRaces(true);

  // If raceId is provided, show results for that race
  if (raceId.value) {
    await handleViewResults(raceId.value);
  }
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
  display: inline-block;
}

/* Smaller status badge in results table */
.results-table .status-badge {
  font-size: 0.7rem;
  padding: 0.15rem 0.5rem;
}

@media (max-width: 768px) {
  .results-table .status-badge {
    font-size: 0.65rem;
    padding: 0.15rem 0.4rem;
  }
}

.status-open {
  /* background-color: #e8f5e9; */
  color: #2e7d32;
}

.status-closed {
  /* background-color: #fff3e0; */
  color: #e65100;
}

.status-ongoing {
  /* background-color: #e3f2fd; */
  color: #1976d2;
}

.status-finished {
  /* background-color: #f3e5f5; */
  color: #7b1fa2;
}

.status-canceled {
  /* background-color: #ffebee; */
  color: #c62828;
}

.actions-cell {
  white-space: nowrap;
}

.track-link {
  color: #2d4059;
  text-decoration: none;
  font-weight: 500;
  background-color: transparent !important;
  transition: color 0.2s ease;
}

.track-link:hover {
  color: #1a1a2e;
  text-decoration: none;
  background-color: transparent !important;
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
  .races-container {
    padding: 1rem;
  }

  .races-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 1rem;
  }

  .races-content h2 {
    font-size: 1.5rem;
  }

  .create-race-button-container {
    width: 100%;
  }

  .create-race-button-container .btn {
    width: 100%;
  }

  .create-race-section {
    padding: 1rem;
  }

  .create-race-section h3 {
    font-size: 1.25rem;
  }

  .races-table-wrapper {
    overflow-x: visible;
    width: 100%;
  }

  .races-table {
    width: 100%;
    table-layout: fixed;
  }

  .races-table th.mobile-hidden,
  .races-table td.mobile-hidden {
    display: none;
  }

  .races-table th.mobile-visible,
  .races-table td.mobile-visible {
    display: table-cell;
  }

  /* Set column widths for mobile - Both sections now have 2 columns */
  .races-table th.mobile-visible:nth-child(1),
  .races-table td.mobile-visible:nth-child(1) {
    width: 40%;
  }

  .races-table th.mobile-visible:nth-child(2),
  .races-table td.mobile-visible:nth-child(2) {
    width: 60%;
  }

  .race-row {
    cursor: pointer;
  }

  .race-row:active {
    background-color: #e8e8e8;
  }

  .races-table th,
  .races-table td {
    padding: 0.75rem 0.4rem;
    font-size: 0.8rem;
    word-wrap: break-word;
    overflow-wrap: break-word;
  }

  .races-table .track-link {
    word-break: break-word;
  }

  .section-header {
    font-size: 1.25rem;
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
    gap: 0.5rem;
  }

  .action-buttons .btn {
    width: 100%;
  }

  .actions-cell {
    min-width: 150px;
  }
}

@media (max-width: 480px) {
  .races-container {
    padding: 0.75rem;
  }

  .races-content h2 {
    font-size: 1.25rem;
  }

  .races-table {
    width: 100%;
    table-layout: fixed;
    font-size: 0.75rem;
  }

  .races-table th,
  .races-table td {
    padding: 0.6rem 0.3rem;
    font-size: 0.75rem;
  }

  /* Both race sections - 2 columns */
  .races-table th.mobile-visible:nth-child(1),
  .races-table td.mobile-visible:nth-child(1) {
    width: 40%;
  }

  .races-table th.mobile-visible:nth-child(2),
  .races-table td.mobile-visible:nth-child(2) {
    width: 60%;
  }

  .section-header {
    font-size: 1.1rem;
  }
}

.load-more-container {
  display: flex;
  justify-content: center;
  margin-top: 1.5rem;
  padding: 1rem;
}
</style>
