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
      <div v-if="loading" class="loading-message">Loading races...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Races list -->
      <div v-if="!loading && !error" class="races-list">
        <!-- Upcoming Races Section (REGISTRATION_OPEN, REGISTRATION_CLOSED) -->
        <div v-if="upcomingRaces.length > 0" class="race-section">
          <h3 class="section-header">Upcoming Races</h3>
          <div class="races-grid">
            <div
              v-for="race in upcomingRaces"
              :key="race.id"
              class="race-card"
              :class="getRaceStatusClass(race.status)"
            >
              <div class="race-header">
                <h4>{{ getTrackName(race.track_id) }}</h4>
                <span class="status-badge" :class="getRaceStatusClass(race.status)">
                  {{ formatStatus(race.status) }}
                </span>
              </div>
              <div class="race-details">
                <div class="detail-item">
                  <span class="detail-label">Laps:</span>
                  <span class="detail-value">{{ race.laps }}</span>
                </div>
                <div v-if="race.start_datetime" class="detail-item">
                  <span class="detail-label">Start Date:</span>
                  <span class="detail-value">{{ formatDate(race.start_datetime) }}</span>
                </div>
                <div v-if="race.description" class="detail-item">
                  <span class="detail-label">Description:</span>
                  <span class="detail-value">{{ race.description }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Created:</span>
                  <span class="detail-value">{{ formatDate(race.created_at) }}</span>
                </div>
              </div>
              <!-- Registration section for authenticated users with a team -->
              <div v-if="authenticated && myTeam" class="race-actions">
                <div v-if="isRegistered(race.id)" class="registration-status registered">
                  <span class="registration-icon">✓</span>
                  <span>Registered</span>
                </div>
                <!-- <div v-else-if="race.status === 'REGISTRATION_CLOSED'" class="registration-status full">
                <span class="registration-icon">✗</span>
                <span>Registration Closed</span>
              </div> -->
                <div class="action-buttons">
                  <button
                    v-if="canRegister(race)"
                    @click="handleRegister(race.id)"
                    class="btn btn-success"
                    :disabled="registering.get(race.id) || starting.get(race.id)"
                  >
                    <span v-if="registering.get(race.id)">Registering...</span>
                    <span v-else>Register</span>
                  </button>
                  <button
                    v-if="canUnregister(race)"
                    @click="handleUnregister(race.id)"
                    class="btn btn-warning"
                    :disabled="registering.get(race.id) || starting.get(race.id)"
                  >
                    <span v-if="registering.get(race.id)">Unregistering...</span>
                    <span v-else>Unregister</span>
                  </button>
                </div>
              </div>
              <!-- Start now button for all races -->
              <div class="race-start-action">
                <button
                  @click="handleStartNow(race.id)"
                  class="btn btn-primary"
                  :disabled="starting.get(race.id)"
                >
                  <span v-if="starting.get(race.id)">Starting...</span>
                  <span v-else>Start now</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Ongoing Races Section -->
        <div v-if="ongoingRaces.length > 0" class="race-section">
          <h3 class="section-header">Ongoing Races</h3>
          <div class="races-grid">
            <div
              v-for="race in ongoingRaces"
              :key="race.id"
              class="race-card"
              :class="getRaceStatusClass(race.status)"
            >
              <div class="race-header">
                <h4>{{ getTrackName(race.track_id) }}</h4>
                <span class="status-badge" :class="getRaceStatusClass(race.status)">
                  {{ formatStatus(race.status) }}
                </span>
              </div>
              <div class="race-details">
                <div class="detail-item">
                  <span class="detail-label">Laps:</span>
                  <span class="detail-value">{{ race.laps }}</span>
                </div>
                <div v-if="race.start_datetime" class="detail-item">
                  <span class="detail-label">Start Date:</span>
                  <span class="detail-value">{{ formatDate(race.start_datetime) }}</span>
                </div>
                <div v-if="race.description" class="detail-item">
                  <span class="detail-label">Description:</span>
                  <span class="detail-value">{{ race.description }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Created:</span>
                  <span class="detail-value">{{ formatDate(race.created_at) }}</span>
                </div>
              </div>
              <!-- View race button for ongoing races -->
              <div class="race-start-action">
                <button @click="handleViewRace(race.id)" class="btn btn-primary">View Race</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Finished Races Section -->
        <div v-if="finishedRaces.length > 0" class="race-section">
          <h3 class="section-header">Finished Races</h3>
          <div class="races-grid">
            <div
              v-for="race in finishedRaces"
              :key="race.id"
              class="race-card"
              :class="getRaceStatusClass(race.status)"
            >
              <div class="race-header">
                <h4>{{ getTrackName(race.track_id) }}</h4>
                <span class="status-badge" :class="getRaceStatusClass(race.status)">
                  {{ formatStatus(race.status) }}
                </span>
              </div>
              <div class="race-details">
                <div class="detail-item">
                  <span class="detail-label">Laps:</span>
                  <span class="detail-value">{{ race.laps }}</span>
                </div>
                <div v-if="race.start_datetime" class="detail-item">
                  <span class="detail-label">Start Date:</span>
                  <span class="detail-value">{{ formatDate(race.start_datetime) }}</span>
                </div>
                <div v-if="race.description" class="detail-item">
                  <span class="detail-label">Description:</span>
                  <span class="detail-value">{{ race.description }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Created:</span>
                  <span class="detail-value">{{ formatDate(race.created_at) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Canceled Races Section -->
        <div v-if="canceledRaces.length > 0" class="race-section">
          <h3 class="section-header">Canceled Races</h3>
          <div class="races-grid">
            <div
              v-for="race in canceledRaces"
              :key="race.id"
              class="race-card"
              :class="getRaceStatusClass(race.status)"
            >
              <div class="race-header">
                <h4>{{ getTrackName(race.track_id) }}</h4>
                <span class="status-badge" :class="getRaceStatusClass(race.status)">
                  {{ formatStatus(race.status) }}
                </span>
              </div>
              <div class="race-details">
                <div class="detail-item">
                  <span class="detail-label">Laps:</span>
                  <span class="detail-value">{{ race.laps }}</span>
                </div>
                <div v-if="race.start_datetime" class="detail-item">
                  <span class="detail-label">Start Date:</span>
                  <span class="detail-value">{{ formatDate(race.start_datetime) }}</span>
                </div>
                <div v-if="race.description" class="detail-item">
                  <span class="detail-label">Description:</span>
                  <span class="detail-value">{{ race.description }}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Created:</span>
                  <span class="detail-value">{{ formatDate(race.created_at) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Empty state -->
        <div v-if="races.length === 0" class="empty-state">
          <p>No races found. Create one to get started!</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import {
  getRaces,
  createRace,
  getTracks,
  getMyTeam,
  registerForRace,
  unregisterFromRace,
  getRaceRegistrations,
  startRaceNow,
  type RaceDb,
  type CreateRaceRequest,
  type TrackDb,
  type TeamDb,
  type RegistrationDb,
} from '../services/ApiService';

const props = defineProps<{
  authenticated: boolean;
}>();

const loading = ref(false);
const error = ref<string | null>(null);
const races = ref<RaceDb[]>([]);
const tracks = ref<TrackDb[]>([]);
const creating = ref(false);
const createError = ref<string | null>(null);
const createSuccess = ref<string | null>(null);
const showCreateForm = ref(false);
const myTeam = ref<TeamDb | null>(null);
const registrations = ref<Map<string, RegistrationDb>>(new Map());
const registering = ref<Map<string, boolean>>(new Map());
const starting = ref<Map<string, boolean>>(new Map());

const emit = defineEmits<{
  navigate: [view: string];
}>();

const formData = ref<Omit<CreateRaceRequest, 'status'>>({
  track_id: '',
  laps: 10,
  start_datetime: null,
  description: null,
});

// Filter races by status
const upcomingRaces = computed(() => {
  return races.value.filter(
    (race) => race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED',
  );
});

const ongoingRaces = computed(() => {
  return races.value.filter((race) => race.status === 'ONGOING');
});

const finishedRaces = computed(() => {
  return races.value.filter((race) => race.status === 'FINISHED');
});

const canceledRaces = computed(() => {
  return races.value.filter((race) => race.status === 'CANCELED');
});

async function loadRaces() {
  loading.value = true;
  error.value = null;
  try {
    races.value = await getRaces();
    // Load registrations for all races if authenticated
    if (props.authenticated && myTeam.value) {
      await loadRegistrationsForAllRaces();
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load races';
  } finally {
    loading.value = false;
  }
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
    // Load registrations after team is loaded
    if (myTeam.value) {
      await loadRegistrationsForAllRaces();
    }
  } catch (err) {
    console.error('Failed to load team:', err);
  }
}

async function loadRegistrationsForAllRaces() {
  if (!myTeam.value) return;

  // Load registrations for each race in parallel
  const promises = races.value.map(async (race) => {
    try {
      const regs = await getRaceRegistrations(race.id);
      // Check if our team is registered
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

async function handleRegister(raceId: string) {
  if (!myTeam.value) {
    error.value = 'You need to have a team to register for a race';
    return;
  }

  registering.value.set(raceId, true);
  try {
    await registerForRace(raceId);
    // Reload races to get updated status (might have changed to REGISTRATION_CLOSED)
    await loadRaces();
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
    // Reload races to get updated status (might have changed back to REGISTRATION_OPEN)
    await loadRaces();
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
    await loadRaces(); // Reload races list
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
        await loadRaces();
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
  await loadRaces();
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

.races-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.race-card {
  background: white;
  border-radius: 8px;
  padding: 1.5rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
}

.race-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.race-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.race-header h4 {
  color: #1a1a2e;
  font-size: 1.25rem;
  margin: 0;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.875rem;
  font-weight: 500;
  text-transform: capitalize;
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

.race-card.status-finished {
  border-left: 4px solid #7b1fa2;
}

.race-card.status-ongoing {
  border-left: 4px solid #1976d2;
}

.race-card.status-open {
  border-left: 4px solid #2e7d32;
}

.race-card.status-closed {
  border-left: 4px solid #e65100;
}

.race-card.status-canceled {
  border-left: 4px solid #c62828;
}

.race-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.detail-item {
  display: flex;
  gap: 0.5rem;
}

.detail-label {
  font-weight: 500;
  color: #666;
}

.detail-value {
  color: #333;
}

.race-actions {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #eee;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.race-start-action {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #eee;
  display: flex;
  justify-content: center;
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

  .races-grid {
    grid-template-columns: 1fr;
  }

  .form-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }

  .race-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .action-buttons {
    flex-direction: column;
  }

  .action-buttons .btn {
    width: 100%;
  }
}
</style>
