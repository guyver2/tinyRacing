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
                  <span class="stat-value">{{ driver.skill_level.toFixed(1) }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Stamina:</span>
                  <span class="stat-value">{{ driver.stamina.toFixed(1) }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Experience:</span>
                  <span class="stat-value">{{ driver.experience.toFixed(1) }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Consistency:</span>
                  <span class="stat-value">{{ driver.consistency.toFixed(1) }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Focus:</span>
                  <span class="stat-value">{{ driver.focus.toFixed(1) }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Weather Tolerance:</span>
                  <span class="stat-value">{{ driver.weather_tolerance.toFixed(1) }}</span>
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
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useRoute } from 'vue-router';
import { getDriver, getTeam, type DriverDb, type TeamDb } from '@/services/ApiService';
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
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load driver';
    console.error('Error loading driver:', err);
  } finally {
    loading.value = false;
  }
}

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
    loadDriver();
  },
);

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
}
</style>
