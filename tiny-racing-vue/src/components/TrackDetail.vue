<template>
  <div class="track-detail-container">
    <div class="track-detail-content">
      <!-- Loading state -->
      <div v-if="loadingTrack" class="loading-message">Loading track information...</div>

      <!-- Error state -->
      <div v-if="trackError" class="error-message">{{ trackError }}</div>

      <!-- Track information -->
      <div v-if="!loadingTrack && !trackError && track" class="track-info-section">
        <div class="track-header-section">
          <button @click="goBack" class="btn-back">← Back to Tracks</button>
          <h2 class="track-title">{{ track.name }}</h2>
          <div class="track-preview-large">
            <img
              :src="getTrackSvgPath(trackId)"
              :alt="track.name"
              class="track-svg-large"
              @error="handleImageError"
            />
          </div>
          <div v-if="track.description" class="track-description-large">
            {{ track.description }}
          </div>
          <div class="track-stats-large">
            <div class="stat-item-large">
              <span class="stat-label-large">Lap Length:</span>
              <span class="stat-value-large">{{ track.lap_length_km }} km</span>
            </div>
          </div>
        </div>

        <!-- Races section -->
        <div class="races-section">
          <h3 class="section-title">Races on this Track</h3>

          <!-- Loading races -->
          <div v-if="loadingRaces" class="loading-message">Loading races...</div>

          <!-- Error loading races -->
          <div v-if="racesError" class="error-message">{{ racesError }}</div>

          <!-- Races list -->
          <div v-if="!loadingRaces && !racesError" class="races-list">
            <!-- Upcoming Races Section -->
            <div v-if="upcomingRaces.length > 0" class="race-section">
              <h3 class="section-header">Upcoming Races</h3>
              <div class="races-table-wrapper">
                <table class="races-table">
                  <thead>
                    <tr>
                      <th class="mobile-visible">Race Date</th>
                      <th class="mobile-visible">Status</th>
                      <th class="mobile-hidden">Laps</th>
                      <th class="mobile-hidden">Description</th>
                      <th class="mobile-hidden">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="race in upcomingRaces"
                      :key="race.id"
                      :class="getRaceStatusClass(race.status)"
                      class="race-row"
                      @click="handleRaceRowClick(race, $event)"
                    >
                      <td class="mobile-visible">
                        {{ race.start_datetime ? formatDate(race.start_datetime) : 'N/A' }}
                      </td>
                      <td class="mobile-visible">
                        <span class="status-badge" :class="getRaceStatusClass(race.status)">
                          {{ formatStatus(race.status) }}
                        </span>
                      </td>
                      <td class="mobile-hidden">{{ race.laps }}</td>
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
                            race.status === 'REGISTRATION_OPEN' ||
                            race.status === 'REGISTRATION_CLOSED'
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
            </div>

            <!-- Past Races Section -->
            <div v-if="pastRaces.length > 0" class="race-section">
              <h3 class="section-header">Done Races</h3>
              <div class="races-table-wrapper">
                <table class="races-table">
                  <thead>
                    <tr>
                      <th class="mobile-visible">Race Date</th>
                      <th class="mobile-visible">Status</th>
                      <th class="mobile-hidden">Laps</th>
                      <th class="mobile-hidden">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="race in pastRaces"
                      :key="race.id"
                      :class="getRaceStatusClass(race.status)"
                      class="race-row"
                      @click="handleRaceRowClick(race, $event)"
                    >
                      <td class="mobile-visible">
                        {{ race.start_datetime ? formatDate(race.start_datetime) : 'N/A' }}
                      </td>
                      <td class="mobile-visible">
                        <span class="status-badge" :class="getRaceStatusClass(race.status)">
                          {{ formatStatus(race.status) }}
                        </span>
                      </td>
                      <td class="mobile-hidden">{{ race.laps }}</td>
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
            </div>

            <!-- Empty state -->
            <div v-if="upcomingRaces.length === 0 && pastRaces.length === 0" class="empty-state">
              <p>No races found for this track.</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Mobile Race Detail Popup -->
    <MobileRaceDetailPopup
      :visible="showMobileRaceDetail"
      :race="selectedMobileRace"
      :track-name="track?.name || ''"
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

    <!-- Race Results Modal -->
    <RaceResultsModal
      :visible="showResults"
      :race="selectedRace"
      :results="raceResults"
      :loading="loadingResults"
      :error="resultsError"
      :track-name="track?.name"
      :drivers="driversMap"
      :teams="teamsMap"
      @close="closeResults"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  getTracks,
  getUpcomingRaces,
  getDoneRaces,
  getRace,
  getMyTeam,
  getTeam,
  getDriver,
  registerForRace,
  unregisterFromRace,
  getRaceRegistrations,
  startRaceNow,
  getRaceResults,
  type TrackDb,
  type RaceDb,
  type TeamDb,
  type DriverDb,
  type RegistrationDb,
  type RaceResultDb,
} from '../services/ApiService';
import { isAuthenticated } from '../services/ApiService';
import RaceResultsModal from './RaceResultsModal.vue';
import MobileRaceDetailPopup from './MobileRaceDetailPopup.vue';

const route = useRoute();
const router = useRouter();

const props = defineProps<{
  authenticated?: boolean;
}>();

const trackId = computed(() => route.params.trackId as string);

const track = ref<TrackDb | null>(null);
const loadingTrack = ref(true);
const trackError = ref<string | null>(null);

const upcomingRaces = ref<RaceDb[]>([]);
const pastRaces = ref<RaceDb[]>([]);
const loadingRaces = ref(true);
const racesError = ref<string | null>(null);

const myTeam = ref<TeamDb | null>(null);
const registrations = ref<Map<string, RegistrationDb>>(new Map());
const registering = ref<Map<string, boolean>>(new Map());
const starting = ref<Map<string, boolean>>(new Map());

const showResults = ref(false);
const selectedRace = ref<RaceDb | null>(null);
const raceResults = ref<RaceResultDb[]>([]);
const loadingResults = ref(false);
const resultsError = ref<string | null>(null);
const driversMap = ref<Map<string, DriverDb>>(new Map());
const teamsMap = ref<Map<string, TeamDb>>(new Map());

// Mobile race detail popup state
const showMobileRaceDetail = ref(false);
const selectedMobileRace = ref<RaceDb | null>(null);

function getTrackSvgPath(trackId: string): string {
  return `/assets/tracks/${trackId}/track.svg`;
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  img.style.display = 'none';
}

async function loadTrack() {
  loadingTrack.value = true;
  trackError.value = null;

  try {
    const tracks = await getTracks();
    // trackId from route is the track's track_id (e.g., "bahrain", "monaco")
    const foundTrack = tracks.find((t) => t.track_id === trackId.value || t.id === trackId.value);
    if (foundTrack) {
      track.value = foundTrack;
    } else {
      trackError.value = 'Track not found';
    }
  } catch (err) {
    trackError.value = err instanceof Error ? err.message : 'Failed to load track';
    console.error('Error loading track:', err);
  } finally {
    loadingTrack.value = false;
  }
}

async function loadRaces() {
  loadingRaces.value = true;
  racesError.value = null;

  try {
    // Load all races (upcoming and done)
    const [upcoming, done] = await Promise.all([
      getUpcomingRaces(1000, 0), // Load a large number to get all
      getDoneRaces(1000, 0),
    ]);

    // Filter races by track_id
    // The race.track_id is a UUID that references track.id (not track.track_id)
    // So we need to match race.track_id with track.id
    if (!track.value) {
      // If track not loaded yet, can't filter properly
      upcomingRaces.value = [];
      pastRaces.value = [];
      return;
    }

    const targetTrackUuid = track.value.id;
    const filteredUpcoming = upcoming.filter((race) => race.track_id === targetTrackUuid);
    const filteredDone = done.filter((race) => race.track_id === targetTrackUuid);

    // Sort races by date/time (most recent first)
    // Use start_datetime if available, otherwise use created_at
    const sortRacesByDate = (races: RaceDb[]) => {
      return [...races].sort((a, b) => {
        const dateA = a.start_datetime
          ? new Date(a.start_datetime).getTime()
          : new Date(a.created_at).getTime();
        const dateB = b.start_datetime
          ? new Date(b.start_datetime).getTime()
          : new Date(b.created_at).getTime();
        return dateB - dateA; // DESC order (most recent first)
      });
    };

    upcomingRaces.value = sortRacesByDate(filteredUpcoming);
    pastRaces.value = sortRacesByDate(filteredDone);

    // Load registrations if authenticated
    if (props.authenticated && myTeam.value) {
      await loadRegistrationsForRaces([...filteredUpcoming, ...filteredDone]);
    }
  } catch (err) {
    racesError.value = err instanceof Error ? err.message : 'Failed to load races';
    console.error('Error loading races:', err);
  } finally {
    loadingRaces.value = false;
  }
}

async function loadRegistrationsForRaces(races: RaceDb[]) {
  if (!myTeam.value) return;

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

async function loadMyTeam() {
  if (!props.authenticated) {
    return;
  }
  try {
    myTeam.value = await getMyTeam();
    if (myTeam.value) {
      await loadRaces();
    }
  } catch (err) {
    console.error('Failed to load team:', err);
  }
}

async function handleRegister(raceId: string) {
  if (!myTeam.value) {
    racesError.value = 'You need to have a team to register for a race';
    return;
  }

  registering.value.set(raceId, true);
  try {
    await registerForRace(raceId);

    const registration: RegistrationDb = {
      id: '',
      race_id: raceId,
      team_id: myTeam.value.id,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    registrations.value.set(raceId, registration);

    try {
      const updatedRace = await getRace(raceId);
      const raceIndex = upcomingRaces.value.findIndex((r) => r.id === raceId);
      if (raceIndex !== -1) {
        upcomingRaces.value[raceIndex] = updatedRace;
      }
    } catch (err) {
      console.error('Failed to fetch updated race:', err);
    }
  } catch (err) {
    racesError.value = err instanceof Error ? err.message : 'Failed to register for race';
  } finally {
    registering.value.set(raceId, false);
  }
}

async function handleUnregister(raceId: string) {
  if (!myTeam.value) {
    racesError.value = 'You need to have a team to unregister from a race';
    return;
  }

  registering.value.set(raceId, true);
  try {
    await unregisterFromRace(raceId);
    registrations.value.delete(raceId);

    try {
      const updatedRace = await getRace(raceId);
      const raceIndex = upcomingRaces.value.findIndex((r) => r.id === raceId);
      if (raceIndex !== -1) {
        upcomingRaces.value[raceIndex] = updatedRace;
      }
    } catch (err) {
      console.error('Failed to fetch updated race:', err);
    }
  } catch (err) {
    racesError.value = err instanceof Error ? err.message : 'Failed to unregister from race';
  } finally {
    registering.value.set(raceId, false);
  }
}

async function handleStartNow(raceIdParam: string) {
  starting.value.set(raceIdParam, true);
  try {
    await startRaceNow(raceIdParam);
    router.push({ name: 'game' });
  } catch (err) {
    racesError.value = err instanceof Error ? err.message : 'Failed to start race';
  } finally {
    starting.value.set(raceIdParam, false);
  }
}

function handleViewRace(raceIdParam: string) {
  router.push({ name: 'game' });
}

async function handleViewResults(raceId: string) {
  selectedRace.value = null;
  showResults.value = true;
  loadingResults.value = true;
  resultsError.value = null;
  raceResults.value = [];
  driversMap.value.clear();
  teamsMap.value.clear();

  try {
    const race =
      upcomingRaces.value.find((r) => r.id === raceId) ||
      pastRaces.value.find((r) => r.id === raceId);
    if (race) {
      selectedRace.value = race;
    } else {
      selectedRace.value = await getRace(raceId);
    }
    raceResults.value = await getRaceResults(raceId);

    const uniqueDriverIds = [...new Set(raceResults.value.map((r) => r.driver_id))];
    const uniqueTeamIds = [...new Set(raceResults.value.map((r) => r.team_id))];

    const driverPromises = uniqueDriverIds.map(async (driverId) => {
      try {
        const driver = await getDriver(driverId);
        driversMap.value.set(driverId, driver);
      } catch (err) {
        console.error(`Failed to load driver ${driverId}:`, err);
      }
    });

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
  return (
    (race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED') &&
    isRegistered(race.id)
  );
};

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
  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });
}

function goBack() {
  router.push({ name: 'tracks' });
}

watch(
  () => props.authenticated,
  async (newAuth) => {
    if (newAuth) {
      await loadMyTeam();
    } else {
      myTeam.value = null;
      registrations.value.clear();
    }
  },
  { immediate: false },
);

watch(
  () => trackId.value,
  async () => {
    await loadTrack();
    if (track.value) {
      await loadRaces();
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await loadTrack();
  if (track.value) {
    await loadMyTeam();
    if (!myTeam.value) {
      await loadRaces();
    }
  }
});
</script>

<style scoped>
.track-detail-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.track-detail-content {
  width: 100%;
}

.btn-back {
  background: none;
  border: none;
  color: #2d4059;
  font-size: 1rem;
  cursor: pointer;
  padding: 0.5rem 0;
  margin-bottom: 1rem;
  transition: color 0.2s ease;
}

.btn-back:hover {
  color: #1a1a2e;
  text-decoration: underline;
}

.track-title {
  color: #1a1a2e;
  font-size: 2.5rem;
  margin: 0 0 2rem 0;
  text-align: center;
}

.track-header-section {
  background: white;
  border-radius: 8px;
  padding: 2rem;
  margin-bottom: 2rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.track-preview-large {
  width: 100%;
  max-width: 600px;
  height: 300px;
  margin: 2rem auto;
  background: #f9f9f9;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  border-radius: 8px;
}

.track-svg-large {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
}

.track-description-large {
  color: #666;
  font-size: 1.1rem;
  line-height: 1.8;
  text-align: center;
  margin-bottom: 2rem;
}

.track-stats-large {
  display: flex;
  justify-content: center;
  gap: 3rem;
  padding-top: 2rem;
  border-top: 1px solid #e0e0e0;
}

.stat-item-large {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.stat-label-large {
  color: #666;
  font-size: 0.9rem;
  font-weight: 500;
}

.stat-value-large {
  color: #1a1a2e;
  font-size: 1.5rem;
  font-weight: 600;
}

.races-section {
  margin-top: 3rem;
}

.section-title {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 2rem;
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

.loading-message,
.error-message {
  text-align: center;
  padding: 1rem;
  margin-bottom: 1rem;
  border-radius: 4px;
}

.loading-message {
  background-color: #e3f2fd;
  color: #1976d2;
}

.error-message {
  background-color: #ffebee;
  color: #c62828;
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

.action-buttons {
  display: flex;
  gap: 0.25rem;
  flex-wrap: wrap;
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

.btn-small {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
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
  .track-detail-container {
    padding: 1rem;
  }

  .track-title {
    font-size: 1.75rem;
  }

  .track-stats-large {
    flex-direction: column;
    gap: 1.5rem;
  }

  .track-preview-large {
    height: 200px;
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

  /* Set column widths for mobile - 2 columns: Date and Status */
  .races-table th.mobile-visible:nth-child(1),
  .races-table td.mobile-visible:nth-child(1) {
    width: 60%;
  }

  .races-table th.mobile-visible:nth-child(2),
  .races-table td.mobile-visible:nth-child(2) {
    width: 40%;
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

  .btn {
    width: 100%;
  }
}
</style>
