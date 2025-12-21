<template>
  <div class="teams-container">
    <div class="teams-content">
      <!-- <h2>My Team</h2> -->

      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading teams...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Create team form (shown when user has no team) -->
      <div v-if="!loading && !error && !team" class="create-team-section">
        <div class="create-team-form">
          <h3>Create Your First Team</h3>
          <p class="form-description">You don't have any teams yet. Create one to get started!</p>
          <form @submit.prevent="handleCreateTeam">
            <div class="form-group">
              <label for="teamName">Team Name:</label>
              <input
                id="teamName"
                v-model="formData.name"
                type="text"
                required
                :disabled="creating"
              />
            </div>
            <div class="form-group">
              <label for="teamLogo">Team Logo (JPG/PNG, max 1MB):</label>
              <input
                id="teamLogo"
                ref="logoFileInput"
                type="file"
                accept="image/jpeg,image/jpg,image/png"
                @change="handleLogoFileChange"
                :disabled="creating"
              />
              <div v-if="logoPreview" class="logo-preview">
                <img :src="logoPreview" alt="Logo preview" />
                <button
                  type="button"
                  @click="clearLogoPreview"
                  class="clear-logo-btn"
                  :disabled="creating"
                >
                  Remove
                </button>
              </div>
              <p v-if="logoError" class="logo-error">{{ logoError }}</p>
            </div>
            <div class="form-group">
              <label for="teamColor">Color:</label>
              <input
                id="teamColor"
                v-model="formData.color"
                type="color"
                required
                :disabled="creating"
              />
            </div>
            <div v-if="createError" class="error-message">{{ createError }}</div>
            <button type="submit" :disabled="creating">
              {{ creating ? 'Creating...' : 'Create Team' }}
            </button>
          </form>
        </div>
      </div>

      <!-- Team display (shown when user has a team) -->
      <div v-if="!loading && !error && team" class="team-display">
        <div class="team-layout">
          <div class="team-metadata-column">
            <div class="team-card">
              <div class="team-header">
                <div class="team-color" :style="{ backgroundColor: team.color }"></div>
                <div class="team-info">
                  <h3>#{{ team.number }} - {{ team.name }}</h3>
                  <p class="team-details">
                    <span>Pit Efficiency: {{ (team.pit_efficiency * 100).toFixed(1) }}%</span>
                    <span style="margin-left: 1rem">Cash: ${{ team.cash.toLocaleString() }}</span>
                  </p>
                </div>
              </div>
              <div v-if="team.logo" class="team-logo">
                <img :src="team.logo" :alt="team.name" />
              </div>
            </div>

            <!-- Upcoming Races Section -->
            <div class="upcoming-races-section">
              <h4 class="section-title-small">Upcoming Races</h4>
              <div v-if="loadingRegistrations" class="loading-small">Loading races...</div>
              <div v-else-if="errorRegistrations" class="error-small">{{ errorRegistrations }}</div>
              <div v-else-if="upcomingRaces.length === 0" class="no-races">
                <p>No upcoming races</p>
              </div>
              <div v-else class="races-list">
                <div
                  v-for="raceReg in upcomingRaces"
                  :key="raceReg.registration_id"
                  class="race-card-small"
                >
                  <div class="race-card-line-1">
                    <span class="race-track-name">{{ raceReg.track_name }}</span>
                    <span class="race-laps">{{ raceReg.laps }} laps</span>
                  </div>
                  <div class="race-card-line-2">
                    <span class="race-date">{{ formatRaceDate(raceReg.start_datetime) }}</span>
                    <button
                      @click="handleUnregisterFromRace(raceReg.race_id)"
                      class="btn-unregister"
                      :disabled="unregisteringRaceId === raceReg.race_id"
                    >
                      {{ unregisteringRaceId === raceReg.race_id ? '...' : 'Unregister' }}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Lineup Section -->
          <div class="lineup-column">
            <div v-if="!loadingLineup && !errorLineup" class="lineup-section">
              <!-- Active Section: Cars with Drivers -->
              <div class="active-section">
                <h3 class="section-title">Lineup</h3>
                <div class="cars-grid">
                  <div
                    v-for="car in sortedCars"
                    :key="car.id"
                    class="car-slot"
                    :class="{ 'has-driver': getDriverForCar(car) }"
                    @drop="handleDrop($event, car.id)"
                    @dragover.prevent
                    @dragenter.prevent
                  >
                    <div class="car-header">
                      <h5>Car #{{ car.number }}</h5>
                    </div>
                    <div class="car-stats">
                      <div class="stat-row">
                        <span class="stat-label">Handling:</span>
                        <span class="stat-value">{{ car.handling.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Acceleration:</span>
                        <span class="stat-value">{{ car.acceleration.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Top Speed:</span>
                        <span class="stat-value">{{ car.top_speed.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Reliability:</span>
                        <span class="stat-value">{{ car.reliability.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Fuel Consumption:</span>
                        <span class="stat-value">{{ car.fuel_consumption.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Tire Wear:</span>
                        <span class="stat-value">{{ car.tire_wear.toFixed(1) }}</span>
                      </div>
                    </div>
                    <div v-if="getDriverForCar(car)" class="driver-assigned">
                      <button
                        class="unseat-button"
                        @click="handleUnseat(getDriverForCar(car)!.id)"
                        :disabled="unseatingDriverId === getDriverForCar(car)?.id"
                        title="Unseat driver"
                      >
                        {{ unseatingDriverId === getDriverForCar(car)?.id ? '...' : '×' }}
                      </button>
                      <div class="driver-header">
                        <img
                          v-if="getDriverForCar(car)?.avatar_url"
                          :src="getDriverForCar(car)?.avatar_url"
                          :alt="`${getDriverForCar(car)?.first_name} ${getDriverForCar(car)?.last_name} avatar`"
                          class="driver-avatar"
                        />
                        <div class="driver-name">
                          <h4>
                            {{ getDriverForCar(car)?.first_name }}
                            {{ getDriverForCar(car)?.last_name }}
                          </h4>
                          <span class="driver-nationality">{{
                            getDriverForCar(car)?.nationality
                          }}</span>
                        </div>
                      </div>
                      <div class="driver-stats">
                        <div class="stat-row">
                          <span class="stat-label">Skill:</span>
                          <span class="stat-value">{{
                            getDriverForCar(car)?.skill_level.toFixed(1)
                          }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Stamina:</span>
                          <span class="stat-value">{{
                            getDriverForCar(car)?.stamina.toFixed(1)
                          }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Experience:</span>
                          <span class="stat-value">{{
                            getDriverForCar(car)?.experience.toFixed(1)
                          }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Consistency:</span>
                          <span class="stat-value">{{
                            getDriverForCar(car)?.consistency.toFixed(1)
                          }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Focus:</span>
                          <span class="stat-value">{{
                            getDriverForCar(car)?.focus.toFixed(1)
                          }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Weather:</span>
                          <span class="stat-value">{{
                            getDriverForCar(car)?.weather_tolerance.toFixed(1)
                          }}</span>
                        </div>
                      </div>
                    </div>
                    <div v-else class="empty-slot">
                      <p>Drop a driver here</p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Substitute Drivers Section -->
              <div class="substitute-section">
                <h4 class="section-title">Subs</h4>
                <div class="subs-grid">
                  <div
                    v-for="subDriver in substituteDrivers"
                    :key="subDriver.id"
                    class="sub-driver-card"
                    draggable="true"
                    @dragstart="handleDragStart($event, subDriver.id)"
                    @dragend="handleDragEnd"
                  >
                    <div class="driver-header">
                      <img
                        v-if="subDriver.avatar_url"
                        :src="subDriver.avatar_url"
                        :alt="`${subDriver.first_name} ${subDriver.last_name} avatar`"
                        class="driver-avatar"
                      />
                      <div class="driver-name">
                        <h4>{{ subDriver.first_name }} {{ subDriver.last_name }}</h4>
                        <span class="driver-nationality">{{ subDriver.nationality }}</span>
                      </div>
                    </div>
                    <div class="driver-stats">
                      <div class="stat-row">
                        <span class="stat-label">Skill:</span>
                        <span class="stat-value">{{ subDriver.skill_level.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Stamina:</span>
                        <span class="stat-value">{{ subDriver.stamina.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Experience:</span>
                        <span class="stat-value">{{ subDriver.experience.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Consistency:</span>
                        <span class="stat-value">{{ subDriver.consistency.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Focus:</span>
                        <span class="stat-value">{{ subDriver.focus.toFixed(1) }}</span>
                      </div>
                      <div class="stat-row">
                        <span class="stat-label">Weather:</span>
                        <span class="stat-value">{{ subDriver.weather_tolerance.toFixed(1) }}</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Loading lineup state -->
            <div v-if="loadingLineup" class="loading-message">Loading lineup...</div>

            <!-- Error lineup state -->
            <div v-if="errorLineup" class="error-message">{{ errorLineup }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue';
import {
  getMyTeam,
  createTeam,
  getTeamDrivers,
  getTeamCars,
  assignDriverToCar,
  getTeamRegistrations,
  unregisterFromRace,
  type TeamDb,
  type CreateTeamRequest,
  type DriverDb,
  type CarDb,
  type RegistrationWithRaceDetails,
} from '@/services/ApiService';

const props = defineProps<{
  authenticated: boolean;
}>();

const team = ref<TeamDb | null>(null);
const loading = ref(true);
const error = ref('');
const creating = ref(false);
const createError = ref('');

const drivers = ref<DriverDb[]>([]);
const cars = ref<CarDb[]>([]);
const loadingLineup = ref(false);
const errorLineup = ref('');
const draggedDriverId = ref<string | null>(null);
const unseatingDriverId = ref<string | null>(null);
const registeredRaces = ref<RegistrationWithRaceDetails[]>([]);
const loadingRegistrations = ref(false);
const errorRegistrations = ref('');
const unregisteringRaceId = ref<string | null>(null);

const formData = ref<CreateTeamRequest>({
  name: '',
  color: '#2d4059',
});

const logoFileInput = ref<HTMLInputElement | null>(null);
const logoFile = ref<File | null>(null);
const logoPreview = ref<string | null>(null);
const logoError = ref<string>('');

// Sort cars by number
const sortedCars = computed(() => {
  return [...cars.value].sort((a, b) => a.number - b.number);
});

// Get substitute drivers (those without car_id)
const substituteDrivers = computed(() => {
  return drivers.value.filter((driver) => driver.car_id === null);
});

// Filter races to only show non-started races (REGISTRATION_OPEN or REGISTRATION_CLOSED)
const upcomingRaces = computed(() => {
  const filtered = registeredRaces.value.filter(
    (race) =>
      race.race_status === 'REGISTRATION_OPEN' || race.race_status === 'REGISTRATION_CLOSED',
  );

  // Sort chronologically by start_datetime (earliest first)
  // Races without start_datetime go to the end
  const sorted = filtered.sort((a, b) => {
    if (!a.start_datetime && !b.start_datetime) return 0;
    if (!a.start_datetime) return 1; // a goes to end
    if (!b.start_datetime) return -1; // b goes to end
    return new Date(a.start_datetime).getTime() - new Date(b.start_datetime).getTime();
  });

  // Return only the next 3 upcoming races
  return sorted.slice(0, 3);
});

// Helper function to get driver for a car
function getDriverForCar(car: CarDb): DriverDb | undefined {
  return drivers.value.find((driver) => driver.car_id === car.id);
}

function handleDragStart(event: DragEvent, driverId: string) {
  draggedDriverId.value = driverId;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', driverId);
  }
}

function handleDragEnd() {
  draggedDriverId.value = null;
}

async function handleDrop(event: DragEvent, carId: string) {
  event.preventDefault();
  if (!draggedDriverId.value) return;

  const driverId = draggedDriverId.value;
  draggedDriverId.value = null;

  // Save scroll position
  const scrollPosition = window.scrollY || document.documentElement.scrollTop;

  try {
    await assignDriverToCar(driverId, carId);
    // Reload lineup to get updated data
    if (team.value) {
      await loadLineup(team.value.id);
    }
    // Restore scroll position after DOM updates
    await nextTick();
    window.scrollTo(0, scrollPosition);
  } catch (err) {
    errorLineup.value = err instanceof Error ? err.message : 'Failed to assign driver';
    console.error('Error assigning driver:', err);
    // Restore scroll position even on error
    window.scrollTo(0, scrollPosition);
  }
}

async function handleUnseat(driverId: string) {
  unseatingDriverId.value = driverId;

  // Save scroll position
  const scrollPosition = window.scrollY || document.documentElement.scrollTop;

  try {
    await assignDriverToCar(driverId, null);
    // Reload lineup to get updated data
    if (team.value) {
      await loadLineup(team.value.id);
    }
    // Restore scroll position after DOM updates
    await nextTick();
    window.scrollTo(0, scrollPosition);
  } catch (err) {
    errorLineup.value = err instanceof Error ? err.message : 'Failed to unseat driver';
    console.error('Error unseating driver:', err);
    // Restore scroll position even on error
    window.scrollTo(0, scrollPosition);
  } finally {
    unseatingDriverId.value = null;
  }
}

async function loadTeam() {
  loading.value = true;
  error.value = '';

  try {
    team.value = await getMyTeam();
    if (team.value) {
      await Promise.all([loadLineup(team.value.id), loadRegistrations(team.value.id)]);
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load team';
    console.error('Error loading team:', err);
  } finally {
    loading.value = false;
  }
}

async function loadRegistrations(teamId: string) {
  loadingRegistrations.value = true;
  errorRegistrations.value = '';

  try {
    registeredRaces.value = await getTeamRegistrations(teamId);
  } catch (err) {
    errorRegistrations.value = err instanceof Error ? err.message : 'Failed to load registrations';
    console.error('Error loading registrations:', err);
  } finally {
    loadingRegistrations.value = false;
  }
}

async function handleUnregisterFromRace(raceId: string) {
  unregisteringRaceId.value = raceId;

  try {
    await unregisterFromRace(raceId);
    // Reload registrations and team to update UI
    if (team.value) {
      await loadRegistrations(team.value.id);
      await loadTeam(); // Reload team to refresh data
    }
  } catch (err) {
    errorRegistrations.value =
      err instanceof Error ? err.message : 'Failed to unregister from race';
    console.error('Error unregistering from race:', err);
  } finally {
    unregisteringRaceId.value = null;
  }
}

function formatRaceDate(startDatetime: string | null): string {
  if (!startDatetime) return 'Date TBD';
  const date = new Date(startDatetime);
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });
}

async function loadLineup(teamId: string) {
  loadingLineup.value = true;
  errorLineup.value = '';

  try {
    const [teamDrivers, teamCars] = await Promise.all([
      getTeamDrivers(teamId),
      getTeamCars(teamId),
    ]);
    drivers.value = teamDrivers;
    cars.value = teamCars;
  } catch (err) {
    errorLineup.value = err instanceof Error ? err.message : 'Failed to load lineup';
    console.error('Error loading lineup:', err);
  } finally {
    loadingLineup.value = false;
  }
}

function handleLogoFileChange(event: Event) {
  logoError.value = '';
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];

  if (!file) {
    logoFile.value = null;
    logoPreview.value = null;
    return;
  }

  // Validate file type
  const validTypes = ['image/jpeg', 'image/jpg', 'image/png'];
  if (!validTypes.includes(file.type)) {
    logoError.value = 'Please select a JPG or PNG image file';
    target.value = '';
    logoFile.value = null;
    logoPreview.value = null;
    return;
  }

  // Validate file size (1MB = 1048576 bytes)
  const maxSize = 1048576;
  if (file.size > maxSize) {
    logoError.value = 'File size must be less than 1MB';
    target.value = '';
    logoFile.value = null;
    logoPreview.value = null;
    return;
  }

  logoFile.value = file;

  // Create preview
  const reader = new FileReader();
  reader.onload = (e) => {
    logoPreview.value = e.target?.result as string;
  };
  reader.readAsDataURL(file);
}

function clearLogoPreview() {
  if (logoFileInput.value) {
    logoFileInput.value.value = '';
  }
  logoFile.value = null;
  logoPreview.value = null;
  logoError.value = '';
}

async function handleCreateTeam() {
  createError.value = '';
  logoError.value = '';
  creating.value = true;

  try {
    const request: CreateTeamRequest = {
      ...formData.value,
    };

    await createTeam(request, logoFile.value);

    // Reset form
    formData.value = {
      name: '',
      color: '#2d4059',
    };
    clearLogoPreview();

    // Reload team
    await loadTeam();
  } catch (err) {
    createError.value = err instanceof Error ? err.message : 'Failed to create team';
    console.error('Error creating team:', err);
  } finally {
    creating.value = false;
  }
}

// Watch for authentication changes and reload team when user logs in
watch(
  () => props.authenticated,
  (newAuth) => {
    if (newAuth) {
      // User just logged in, reload team data
      loadTeam();
    } else {
      // User logged out, clear team data
      team.value = null;
      drivers.value = [];
      cars.value = [];
      registeredRaces.value = [];
      error.value = '';
      loading.value = false;
    }
  },
);

onMounted(() => {
  // Only load team if authenticated
  if (props.authenticated) {
    loadTeam();
  } else {
    loading.value = false;
  }
});
</script>

<style scoped>
.teams-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.teams-content {
  width: 100%;
}

h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 2rem;
  text-align: center;
}

h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin-bottom: 1rem;
}

.loading-message,
.error-message {
  text-align: center;
  padding: 1rem;
  margin-bottom: 1rem;
  border-radius: 4px;
}

.loading-message {
  color: #666;
  background-color: #f5f5f5;
}

.error-message {
  color: #d32f2f;
  background-color: #ffebee;
}

.create-team-section {
  display: flex;
  justify-content: center;
  margin-top: 2rem;
}

.create-team-form {
  background-color: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 500px;
}

.form-description {
  color: #666;
  margin-bottom: 1.5rem;
  text-align: center;
}

.form-group {
  margin-bottom: 1rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  color: #2d4059;
  font-weight: 500;
}

input[type='text'],
input[type='number'],
input[type='color'],
input[type='file'] {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
  box-sizing: border-box;
}

input[type='file'] {
  cursor: pointer;
}

input[type='color'] {
  height: 50px;
  cursor: pointer;
}

input:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

input:focus {
  outline: none;
  border-color: #2d4059;
}

button {
  width: 100%;
  padding: 0.75rem;
  background-color: #2d4059;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  margin-top: 1rem;
}

button:hover:not(:disabled) {
  background-color: #1e2a3a;
}

button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.team-display {
  width: 100%;
  margin-top: 2rem;
}

.team-layout {
  display: grid;
  grid-template-columns: 1fr 3fr;
  gap: 2rem;
  align-items: start;
}

.team-metadata-column {
  width: 100%;
}

.team-card {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  transition:
    transform 0.2s,
    box-shadow 0.2s;
  width: 100%;
  position: sticky;
  top: 2rem;
}

.lineup-column {
  width: 100%;
}

.team-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
}

.team-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.team-color {
  width: 40px;
  height: 40px;
  border-radius: 4px;
  flex-shrink: 0;
}

.team-info {
  flex: 1;
}

.team-info h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.2rem;
  color: #1a1a2e;
}

.team-details {
  color: #666;
  font-size: 0.9rem;
  margin: 0;
}

.team-logo {
  margin-top: 1rem;
  text-align: center;
}

.team-logo img {
  max-width: 100%;
  max-height: 100px;
  object-fit: contain;
}

.logo-preview {
  margin-top: 1rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.logo-preview img {
  max-width: 200px;
  max-height: 200px;
  object-fit: contain;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 0.5rem;
}

.clear-logo-btn {
  padding: 0.5rem 1rem;
  background-color: #f57c00;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.clear-logo-btn:hover:not(:disabled) {
  background-color: #e65100;
}

.clear-logo-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.logo-error {
  color: #d32f2f;
  font-size: 0.9rem;
  margin-top: 0.5rem;
}

.upcoming-races-section {
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 1px solid #eee;
}

.section-title-small {
  color: #2d4059;
  font-size: 1.1rem;
  margin-bottom: 1rem;
  margin-top: 0;
}

.loading-small,
.error-small {
  font-size: 0.9rem;
  padding: 0.5rem;
  text-align: center;
}

.loading-small {
  color: #666;
}

.error-small {
  color: #d32f2f;
}

.no-races {
  text-align: center;
  color: #999;
  font-size: 0.9rem;
  padding: 1rem;
}

.races-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.race-card-small {
  background-color: #f8f9fa;
  border-radius: 6px;
  padding: 0.75rem;
  border: 1px solid #e0e0e0;
}

.race-card-line-1 {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.race-track-name {
  font-weight: 500;
  color: #1a1a2e;
  font-size: 0.95rem;
}

.race-laps {
  color: #666;
  font-size: 0.85rem;
}

.race-card-line-2 {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.race-date {
  color: #666;
  font-size: 0.85rem;
}

.btn-unregister {
  padding: 0.35rem 0.75rem;
  background-color: #f57c00;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-unregister:hover:not(:disabled) {
  background-color: #e65100;
}

.btn-unregister:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.lineup-section {
  width: 100%;
}

.lineup-title {
  color: #1a1a2e;
  font-size: 1.8rem;
  margin-bottom: 1.5rem;
  text-align: center;
}

.section-title {
  color: #2d4059;
  font-size: 1.3rem;
  margin-bottom: 1rem;
  margin-top: 2rem;
}

.section-title:first-of-type {
  margin-top: 0;
}

.cars-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 1.5rem;
  margin-bottom: 1rem;
}

.car-slot {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  transition:
    transform 0.2s,
    box-shadow 0.2s,
    border-color 0.2s;
  border: 3px solid #e0e0e0;
  min-height: 200px;
}

.car-slot:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
  border-color: #4caf50;
}

.car-slot.has-driver {
  border-color: #4caf50;
}

.car-header h5 {
  margin: 0 0 1rem 0;
  color: #2d4059;
  font-size: 1.2rem;
}

.car-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  padding: 0.4rem 0;
  border-bottom: 1px solid #f0f0f0;
}

.stat-label {
  color: #666;
  font-size: 0.9rem;
}

.stat-value {
  color: #1a1a2e;
  font-weight: 500;
  font-size: 0.9rem;
}

.driver-assigned {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 2px solid #4caf50;
  position: relative;
}

.unseat-button {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 32px;
  height: 32px;
  padding: 0;
  background-color: #ff5722;
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 1.5rem;
  font-weight: bold;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color 0.2s,
    transform 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.unseat-button:hover:not(:disabled) {
  background-color: #e64a19;
  transform: scale(1.1);
}

.unseat-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  transform: none;
}

.empty-slot {
  margin-top: 1rem;
  padding: 2rem;
  text-align: center;
  color: #999;
  border: 2px dashed #ddd;
  border-radius: 4px;
}

.driver-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.driver-avatar {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e0e0e0;
}

.driver-name h4 {
  margin: 0 0 0.25rem 0;
  color: #1a1a2e;
  font-size: 1rem;
}

.driver-nationality {
  color: #666;
  font-size: 0.85rem;
}

.driver-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
}

.substitute-section {
  margin-top: 2rem;
}

.subs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
}

.sub-driver-card {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1rem;
  transition:
    transform 0.2s,
    box-shadow 0.2s;
  border-left: 4px solid #ff9800;
  cursor: move;
}

.sub-driver-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
}

.sub-driver-card:active {
  opacity: 0.7;
}

@media (max-width: 768px) {
  .team-layout {
    grid-template-columns: 1fr;
  }

  .team-card {
    position: static;
  }

  .cars-grid,
  .subs-grid {
    grid-template-columns: 1fr;
  }

  .car-stats,
  .driver-stats {
    grid-template-columns: 1fr;
  }
}
</style>
