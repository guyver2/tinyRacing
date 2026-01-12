<template>
  <div class="driver-detail-container">
    <div class="driver-detail-content">
      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading driver...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Driver display -->
      <div v-if="!loading && !error && driver" class="driver-display">
        <div class="driver-header-section">
          <div class="driver-avatar-large">
            <img
              v-if="driver.avatar_url"
              :src="driver.avatar_url"
              :alt="`${driver.first_name} ${driver.last_name} avatar`"
            />
            <div v-else class="avatar-placeholder">
              {{ driver.first_name.charAt(0) }}{{ driver.last_name.charAt(0) }}
            </div>
          </div>
          <div class="driver-info">
            <h2>{{ driver.first_name }} {{ driver.last_name }}</h2>
            <div class="driver-meta">
              <span class="nationality">
                <img
                  v-if="getCountryCode(driver.nationality)"
                  :src="getFlagUrl(getCountryCode(driver.nationality)!)"
                  :alt="driver.nationality"
                  :title="driver.nationality"
                  class="country-flag"
                />
                <span v-else class="country-flag-fallback" :title="driver.nationality">🏁</span>
                {{ driver.nationality }}
              </span>
              <span class="gender">{{ driver.gender }}</span>
              <span class="dob"
                >Born: {{ formatDate(driver.date_of_birth) }} ({{
                  calculateAge(driver.date_of_birth)
                }})</span
              >
            </div>
          </div>
        </div>

        <div class="driver-stats-section">
          <div class="stats-layout">
            <div class="stats-column stats-values">
              <h3>Driver Statistics</h3>
              <div class="stats-grid">
                <div class="stat-item">
                  <span class="stat-label">Skill Level:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="canLevelUpDriver && canLevelUpStat('skill_level')"
                      @click="levelUp('skill_level')"
                      class="level-up-btn"
                      :disabled="levelingUp"
                    >
                      Level Up
                    </button>
                    <span v-else-if="driver.skill_level >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ driver.skill_level.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Stamina:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="canLevelUpDriver && canLevelUpStat('stamina')"
                      @click="levelUp('stamina')"
                      class="level-up-btn"
                      :disabled="levelingUp"
                    >
                      Level Up
                    </button>
                    <span v-else-if="driver.stamina >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ driver.stamina.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Experience:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="canLevelUpDriver && canLevelUpStat('experience')"
                      @click="levelUp('experience')"
                      class="level-up-btn"
                      :disabled="levelingUp"
                    >
                      Level Up
                    </button>
                    <span v-else-if="driver.experience >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ driver.experience.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Consistency:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="canLevelUpDriver && canLevelUpStat('consistency')"
                      @click="levelUp('consistency')"
                      class="level-up-btn"
                      :disabled="levelingUp"
                    >
                      Level Up
                    </button>
                    <span v-else-if="driver.consistency >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ driver.consistency.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Focus:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="canLevelUpDriver && canLevelUpStat('focus')"
                      @click="levelUp('focus')"
                      class="level-up-btn"
                      :disabled="levelingUp"
                    >
                      Level Up
                    </button>
                    <span v-else-if="driver.focus >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ driver.focus.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Weather Tolerance:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="canLevelUpDriver && canLevelUpStat('weather_tolerance')"
                      @click="levelUp('weather_tolerance')"
                      class="level-up-btn"
                      :disabled="levelingUp"
                    >
                      Level Up
                    </button>
                    <span v-else-if="driver.weather_tolerance >= 1.0" class="max-stat-badge"
                      >MAX</span
                    >
                    <span class="stat-value">{{ driver.weather_tolerance.toFixed(1) }}</span>
                  </div>
                </div>
              </div>
            </div>
            <div class="stats-column stats-chart">
              <DriverStatsRadarChart
                :skill-level="driver.skill_level"
                :stamina="driver.stamina"
                :experience="driver.experience"
                :consistency="driver.consistency"
                :focus="driver.focus"
                :weather-tolerance="driver.weather_tolerance"
              />
            </div>
          </div>
        </div>

        <div
          v-if="driver.total_exp !== undefined && driver.spent_exp !== undefined"
          class="experience-section"
        >
          <h3>Experience & Leveling</h3>
          <div class="experience-info">
            <div class="experience-item">
              <span class="experience-label">Total Experience:</span>
              <span class="experience-value">{{ driver.total_exp }}</span>
            </div>
            <div class="experience-item">
              <span class="experience-label">Spent Experience:</span>
              <span class="experience-value">{{ driver.spent_exp }}</span>
            </div>
            <div class="experience-item">
              <span class="experience-label">Available Experience:</span>
              <span class="experience-value available">{{ availableExp }}</span>
            </div>
            <div class="experience-item">
              <span class="experience-label">Available Levels:</span>
              <span class="experience-value available">{{ availableLevels }}</span>
            </div>
          </div>
        </div>

        <div v-if="team" class="team-section">
          <h3>Team</h3>
          <div class="team-info">
            <router-link :to="{ name: 'team', params: { teamId: team.id } }" class="team-link">
              <img v-if="team.logo" :src="team.logo" :alt="team.name" class="team-logo" />
              <div class="team-color" :style="{ backgroundColor: team.color }"></div>
              <span class="team-name">{{ team.name }}</span>
            </router-link>
          </div>
        </div>

        <div class="race-results-section">
          <h3>Race History</h3>
          <div v-if="raceResultsLoading" class="race-results-loading">Loading race results...</div>
          <div v-else-if="raceResultsError" class="race-results-error">{{ raceResultsError }}</div>
          <div v-else-if="raceResults.length === 0" class="race-results-empty">
            No race results found for this driver.
          </div>
          <div v-else>
            <table class="race-results-table">
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Track</th>
                  <th>Position</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="result in raceResults" :key="result.race_result_id">
                  <td>{{ formatRaceDate(result.race_date) }}</td>
                  <td>
                    <router-link
                      :to="{ name: 'track', params: { trackId: result.track_id } }"
                      class="track-link"
                      :title="`View track ${result.track_name}`"
                    >
                      {{ result.track_name }}
                    </router-link>
                  </td>
                  <td class="position-cell">{{ result.final_position }}</td>
                </tr>
              </tbody>
            </table>
            <div class="pagination-controls">
              <button
                @click="goToPage(currentPage - 1)"
                :disabled="!hasPreviousPage"
                class="pagination-btn"
              >
                Previous
              </button>
              <span class="pagination-info">Page {{ currentPage }}</span>
              <button
                @click="goToPage(currentPage + 1)"
                :disabled="!hasNextPage"
                class="pagination-btn"
              >
                Next
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useRoute } from 'vue-router';
import {
  getDriver,
  getTeam,
  getDriverRaceResults,
  levelUpDriver,
  isAuthenticated,
  getPlayerId,
  type DriverDb,
  type TeamDb,
  type DriverRaceResultDb,
} from '@/services/ApiService';
import DriverStatsRadarChart from './DriverStatsRadarChart.vue';
import { getCountryCode, getFlagUrl } from '@/utils/countryFlags';

const route = useRoute();
const props = defineProps<{
  driverId?: string;
}>();

// Get driverId from route params if not provided as prop
const driverId = computed(() => {
  return props.driverId || (route.params.driverId as string) || null;
});

const driver = ref<DriverDb | null>(null);
const team = ref<TeamDb | null>(null);
const loading = ref(true);
const error = ref('');

// Race results state
const raceResults = ref<DriverRaceResultDb[]>([]);
const raceResultsLoading = ref(false);
const raceResultsError = ref('');
const pageSize = 10;
const currentPage = ref(1);
const totalRaceResults = ref(0);

// Level up state
const levelingUp = ref(false);

// Computed properties for experience
const availableExp = computed(() => {
  if (
    !driver.value ||
    driver.value.total_exp === undefined ||
    driver.value.spent_exp === undefined
  ) {
    return 0;
  }
  return driver.value.total_exp - driver.value.spent_exp;
});

const availableLevels = computed(() => {
  return Math.floor(availableExp.value / 100);
});

const canLevelUp = computed(() => {
  return availableExp.value >= 100;
});

// Check if user can level up this driver (must be authenticated and own the team)
const canLevelUpDriver = computed(() => {
  // Must be authenticated
  if (!isAuthenticated()) {
    return false;
  }

  // Must have a team loaded
  if (!team.value) {
    return false;
  }

  // Must own the team
  const currentPlayerId = getPlayerId();
  if (!currentPlayerId || team.value.player_id !== currentPlayerId) {
    return false;
  }

  // Must have enough experience
  return canLevelUp.value;
});

function canLevelUpStat(statName: string): boolean {
  if (!driver.value) return false;

  const currentValue = (() => {
    switch (statName) {
      case 'skill_level':
        return driver.value!.skill_level;
      case 'stamina':
        return driver.value!.stamina;
      case 'weather_tolerance':
        return driver.value!.weather_tolerance;
      case 'experience':
        return driver.value!.experience;
      case 'consistency':
        return driver.value!.consistency;
      case 'focus':
        return driver.value!.focus;
      default:
        return 0;
    }
  })();

  // Can level up if stat is less than 1.0 (even if adding 0.1 would reach exactly 1.0)
  // The backend will cap it at 1.0 using LEAST()
  return currentValue < 1.0;
}

async function loadDriver() {
  if (!driverId.value) {
    error.value = 'No driver ID provided';
    loading.value = false;
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    driver.value = await getDriver(driverId.value);

    // Load team if driver has a team
    if (driver.value.team_id) {
      try {
        team.value = await getTeam(driver.value.team_id);
      } catch (err) {
        console.error('Failed to load team:', err);
        // Don't fail the whole component if team load fails
      }
    }

    // Load race results
    await loadRaceResults();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load driver';
    console.error('Error loading driver:', err);
  } finally {
    loading.value = false;
  }
}

async function loadRaceResults() {
  if (!driverId.value) {
    return;
  }

  raceResultsLoading.value = true;
  raceResultsError.value = '';

  try {
    const offset = (currentPage.value - 1) * pageSize;
    const results = await getDriverRaceResults(driverId.value, pageSize, offset);
    raceResults.value = results;

    // Estimate total count (if we got a full page, there might be more)
    // For a proper implementation, we'd need a count endpoint, but for now
    // we'll assume there are more if we got a full page
    if (results.length === pageSize) {
      totalRaceResults.value = currentPage.value * pageSize + 1; // Indicates more available
    } else {
      totalRaceResults.value = (currentPage.value - 1) * pageSize + results.length;
    }
  } catch (err) {
    raceResultsError.value = err instanceof Error ? err.message : 'Failed to load race results';
    console.error('Error loading race results:', err);
  } finally {
    raceResultsLoading.value = false;
  }
}

function goToPage(page: number) {
  if (page < 1) return;
  currentPage.value = page;
  loadRaceResults();
}

function formatRaceDate(dateString: string | null): string {
  if (!dateString) return 'TBD';
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

const hasNextPage = computed(() => {
  return raceResults.value.length === pageSize;
});

const hasPreviousPage = computed(() => {
  return currentPage.value > 1;
});

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

function calculateAge(dateOfBirth: string): string {
  const birthDate = new Date(dateOfBirth);
  const today = new Date();
  let age = today.getFullYear() - birthDate.getFullYear();
  const monthDiff = today.getMonth() - birthDate.getMonth();

  // Adjust age if birthday hasn't occurred this year yet
  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthDate.getDate())) {
    age--;
  }

  return `${age}`;
}

// Watch for route changes (driverId changes)
watch(
  () => driverId.value,
  () => {
    currentPage.value = 1; // Reset to first page
    loadDriver();
  },
);

async function levelUp(stat: string) {
  if (!driverId.value || !canLevelUpDriver.value || levelingUp.value) {
    return;
  }

  levelingUp.value = true;
  try {
    const updatedDriver = await levelUpDriver(driverId.value, stat);
    driver.value = updatedDriver;
    // Reload race results to ensure data is fresh
    await loadRaceResults();
  } catch (err) {
    alert(err instanceof Error ? err.message : 'Failed to level up driver');
    console.error('Error leveling up driver:', err);
  } finally {
    levelingUp.value = false;
  }
}

onMounted(() => {
  loadDriver();
});
</script>

<style scoped>
.driver-detail-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.driver-detail-content {
  width: 100%;
}

.loading-message,
.error-message {
  text-align: center;
  padding: 2rem;
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

.driver-display {
  width: 100%;
}

.driver-header-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
  margin-bottom: 2rem;
  display: flex;
  align-items: center;
  gap: 2rem;
}

.driver-avatar-large {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  overflow: hidden;
  border: 4px solid #e0e0e0;
  flex-shrink: 0;
}

.driver-avatar-large img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #2d4059;
  color: white;
  font-size: 3rem;
  font-weight: bold;
}

.driver-info {
  flex: 1;
}

.driver-info h2 {
  margin: 0 0 1rem 0;
  color: #1a1a2e;
  font-size: 2rem;
}

.driver-meta {
  display: flex;
  gap: 1.5rem;
  flex-wrap: wrap;
  color: #666;
  font-size: 1rem;
}

.nationality {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.country-flag {
  width: 20px;
  height: 15px;
  object-fit: cover;
  border-radius: 2px;
  flex-shrink: 0;
}

.country-flag-fallback {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.driver-stats-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
  margin-bottom: 2rem;
}

.driver-stats-section h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin-bottom: 1rem;
  margin-top: 0;
}

.stats-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
  align-items: start;
}

.stats-column {
  display: flex;
  flex-direction: column;
}

.stats-values {
  min-width: 0;
}

.stats-chart {
  display: flex;
  justify-content: center;
  align-items: center;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.25rem;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 0.125rem 0;
}

.stat-label {
  color: #666;
  font-weight: 500;
  font-size: 0.875rem;
}

.stat-value {
  color: #1a1a2e;
  font-weight: 600;
  font-size: 0.875rem;
}

.team-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
}

.team-section h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin-bottom: 1rem;
}

.team-link {
  display: flex;
  align-items: center;
  gap: 1rem;
  text-decoration: none;
  color: #1a1a2e;
  padding: 1rem;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.team-link:hover {
  background-color: #f5f5f5;
}

.team-logo {
  width: 40px;
  height: 40px;
  object-fit: cover;
  object-position: center;
  border-radius: 4px;
  flex-shrink: 0;
}

.team-color {
  width: 40px;
  height: 40px;
  border-radius: 4px;
  flex-shrink: 0;
}

.team-name {
  font-size: 1.2rem;
  font-weight: 500;
}

.race-results-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
}

.race-results-section h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin-bottom: 1rem;
  margin-top: 0;
}

.race-results-loading,
.race-results-error,
.race-results-empty {
  text-align: center;
  padding: 2rem;
  color: #666;
}

.race-results-error {
  color: #d32f2f;
}

.race-results-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 1rem;
}

.race-results-table thead {
  background-color: #f5f5f5;
}

.race-results-table th {
  padding: 0.75rem;
  text-align: left;
  font-weight: 600;
  color: #2d4059;
  border-bottom: 2px solid #e0e0e0;
}

.race-results-table td {
  padding: 0.75rem;
  border-bottom: 1px solid #e0e0e0;
  color: #1a1a2e;
}

.race-results-table tbody tr:hover {
  background-color: #f9f9f9;
}

.position-cell {
  font-weight: 600;
  font-size: 1.1rem;
  color: #2d4059;
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

.pagination-controls {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  margin-top: 1.5rem;
  padding-top: 1rem;
  border-top: 1px solid #e0e0e0;
}

.pagination-btn {
  padding: 0.5rem 1rem;
  background-color: #2d4059;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.2s;
}

.pagination-btn:hover:not(:disabled) {
  background-color: #1a1a2e;
}

.pagination-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  opacity: 0.6;
}

.pagination-info {
  color: #666;
  font-size: 0.9rem;
}

.experience-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
  margin-bottom: 2rem;
}

.experience-section h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin-bottom: 1rem;
  margin-top: 0;
}

.experience-info {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 1rem;
}

.experience-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.experience-label {
  color: #666;
  font-size: 0.875rem;
  font-weight: 500;
}

.experience-value {
  color: #1a1a2e;
  font-weight: 600;
  font-size: 1.25rem;
}

.experience-value.available {
  color: #2d4059;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.125rem 0;
  gap: 0.5rem;
}

.stat-right-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  justify-content: flex-end;
}

.level-up-btn {
  padding: 0.25rem 0.75rem;
  background-color: #2d4059;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75rem;
  font-weight: 600;
  transition: background-color 0.2s;
  flex-shrink: 0;
}

.level-up-btn:hover:not(:disabled) {
  background-color: #1a1a2e;
}

.level-up-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  opacity: 0.6;
}

.max-stat-badge {
  padding: 0.25rem 0.75rem;
  background: linear-gradient(135deg, #ffd700 0%, #ffed4e 100%);
  color: #8b6914;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  flex-shrink: 0;
  box-shadow: 0 2px 4px rgba(255, 215, 0, 0.3);
}

@media (max-width: 768px) {
  .driver-header-section {
    flex-direction: column;
    text-align: center;
  }

  .driver-meta {
    justify-content: center;
  }

  .stats-layout {
    grid-template-columns: 1fr;
    gap: 1.5rem;
  }

  .stats-chart {
    order: -1;
  }

  .race-results-table {
    font-size: 0.9rem;
  }

  .race-results-table th,
  .race-results-table td {
    padding: 0.5rem;
  }

  .pagination-controls {
    flex-wrap: wrap;
  }
}
</style>
