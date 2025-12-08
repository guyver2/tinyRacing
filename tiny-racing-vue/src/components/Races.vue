<template>
  <div class="races-container">
    <div class="races-content">
      <h2>Races</h2>
      <p class="races-description">View and create racing events</p>

      <!-- Create Race Form (for authenticated users) -->
      <div v-if="authenticated" class="create-race-section">
        <h3>Create New Race</h3>
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
            <input
              id="laps"
              type="number"
              v-model.number="formData.laps"
              min="1"
              required
            />
          </div>

          <div class="form-group">
            <label for="start_datetime">Start Date (optional)</label>
            <input
              id="start_datetime"
              type="datetime-local"
              v-model="formData.start_datetime"
            />
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
            <button type="button" class="btn btn-secondary" @click="resetForm">
              Reset
            </button>
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
        <h3>All Races</h3>
        <div v-if="races.length === 0" class="empty-state">
          <p>No races found. Create one to get started!</p>
        </div>
        <div v-else class="races-grid">
          <div
            v-for="race in races"
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getRaces, createRace, getTracks, type RaceDb, type CreateRaceRequest, type TrackDb } from '../services/ApiService';

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

const formData = ref<Omit<CreateRaceRequest, 'status'>>({
  track_id: '',
  laps: 10,
  start_datetime: null,
  description: null,
});

async function loadRaces() {
  loading.value = true;
  error.value = null;
  try {
    races.value = await getRaces();
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

function getTrackName(trackId: string): string {
  const track = tracks.value.find(t => t.id === trackId);
  return track ? track.name : 'Unknown Track';
}

function formatStatus(status: string): string {
  return status.replace(/_/g, ' ').toLowerCase()
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
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
    hour12: true
  });
}

onMounted(async () => {
  await loadTracks();
  await loadRaces();
});
</script>

<style scoped>
.races-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.races-content h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.races-description {
  color: #666;
  font-size: 1.1rem;
  margin-bottom: 2rem;
}

.create-race-section {
  background: white;
  border-radius: 8px;
  padding: 1.5rem;
  margin-bottom: 2rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.create-race-section h3 {
  color: #1a1a2e;
  font-size: 1.5rem;
  margin-bottom: 1rem;
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

.races-list h3 {
  color: #1a1a2e;
  font-size: 1.5rem;
  margin-bottom: 1rem;
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
  transition: transform 0.2s ease, box-shadow 0.2s ease;
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

@media (max-width: 768px) {
  .races-grid {
    grid-template-columns: 1fr;
  }

  .form-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }
}
</style>

