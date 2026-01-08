<template>
  <div class="car-detail-container">
    <div class="car-detail-content">
      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading car...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Car display -->
      <div v-if="!loading && !error && car" class="car-display">
        <div class="car-header-section">
          <div class="car-icon-large">
            <img src="/assets/f1.svg" alt="Car" class="car-icon-img" />
          </div>
          <div class="car-info">
            <h2>Car #{{ car.number }}</h2>
            <div class="car-meta">
              <div v-if="team && team.logo" class="team-logo-header">
                <img :src="team.logo" :alt="team.name" class="team-logo-img" />
              </div>
              <div class="team-info-group">
                <router-link
                  v-if="team"
                  :to="{ name: 'team', params: { teamId: team.id } }"
                  class="team-name-link"
                >
                  {{ team.name }}
                </router-link>
                <span v-else class="team-name-link">Loading...</span>
                <span v-if="isPlayerCar && playerTeam" class="team-cash-header">
                  Available Cash: ${{ playerTeam.cash.toLocaleString() }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div class="car-stats-section">
          <div class="stats-layout">
            <div class="stats-column stats-values">
              <h3>Car Statistics</h3>
              <div class="stats-grid">
                <div class="stat-item">
                  <span class="stat-label">Handling:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="isPlayerCar && canImprove && canImproveStat('handling')"
                      @click="improve('handling')"
                      class="improve-btn"
                      :disabled="improving"
                    >
                      Improve (${{ getImprovementCost('handling') }})
                    </button>
                    <span v-else-if="car.handling >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ car.handling.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Acceleration:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="isPlayerCar && canImprove && canImproveStat('acceleration')"
                      @click="improve('acceleration')"
                      class="improve-btn"
                      :disabled="improving"
                    >
                      Improve (${{ getImprovementCost('acceleration') }})
                    </button>
                    <span v-else-if="car.acceleration >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ car.acceleration.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Top Speed:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="isPlayerCar && canImprove && canImproveStat('top_speed')"
                      @click="improve('top_speed')"
                      class="improve-btn"
                      :disabled="improving"
                    >
                      Improve (${{ getImprovementCost('top_speed') }})
                    </button>
                    <span v-else-if="car.top_speed >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ car.top_speed.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Reliability:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="isPlayerCar && canImprove && canImproveStat('reliability')"
                      @click="improve('reliability')"
                      class="improve-btn"
                      :disabled="improving"
                    >
                      Improve (${{ getImprovementCost('reliability') }})
                    </button>
                    <span v-else-if="car.reliability >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ car.reliability.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Fuel Consumption:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="isPlayerCar && canImprove && canImproveStat('fuel_consumption')"
                      @click="improve('fuel_consumption')"
                      class="improve-btn"
                      :disabled="improving"
                    >
                      Improve (${{ getImprovementCost('fuel_consumption') }})
                    </button>
                    <span v-else-if="car.fuel_consumption >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ car.fuel_consumption.toFixed(1) }}</span>
                  </div>
                </div>
                <div class="stat-item">
                  <span class="stat-label">Tire Wear:</span>
                  <div class="stat-right-group">
                    <button
                      v-if="isPlayerCar && canImprove && canImproveStat('tire_wear')"
                      @click="improve('tire_wear')"
                      class="improve-btn"
                      :disabled="improving"
                    >
                      Improve (${{ getImprovementCost('tire_wear') }})
                    </button>
                    <span v-else-if="car.tire_wear >= 1.0" class="max-stat-badge">MAX</span>
                    <span class="stat-value">{{ car.tire_wear.toFixed(1) }}</span>
                  </div>
                </div>
              </div>
            </div>
            <div class="stats-column stats-chart">
              <CarStatsRadarChart
                :handling="car.handling"
                :acceleration="car.acceleration"
                :top-speed="car.top_speed"
                :reliability="car.reliability"
                :fuel-consumption="car.fuel_consumption"
                :tire-wear="car.tire_wear"
              />
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
  getCar,
  getTeam,
  improveCar,
  getMyTeam,
  getPlayerId,
  isAuthenticated,
  type CarDb,
  type TeamDb,
} from '@/services/ApiService';
import CarStatsRadarChart from './CarStatsRadarChart.vue';

const route = useRoute();
const props = defineProps<{
  carId?: string;
}>();

// Get carId from route params if not provided as prop
const carId = computed(() => {
  return props.carId || (route.params.carId as string) || null;
});

const car = ref<CarDb | null>(null);
const team = ref<TeamDb | null>(null);
const playerTeam = ref<TeamDb | null>(null);
const loading = ref(true);
const error = ref('');
const improving = ref(false);

// Computed property to check if car belongs to player's team
const isPlayerCar = computed(() => {
  if (!isAuthenticated() || !car.value || !playerTeam.value) {
    return false;
  }
  return car.value.team_id === playerTeam.value.id;
});

// Computed property for available cash
const availableCash = computed(() => {
  return playerTeam.value?.cash || 0;
});

const canImprove = computed(() => {
  return isPlayerCar.value && availableCash.value > 0;
});

function canImproveStat(statName: string): boolean {
  if (!car.value) return false;

  const currentValue = (() => {
    switch (statName) {
      case 'handling':
        return car.value!.handling;
      case 'acceleration':
        return car.value!.acceleration;
      case 'top_speed':
        return car.value!.top_speed;
      case 'reliability':
        return car.value!.reliability;
      case 'fuel_consumption':
        return car.value!.fuel_consumption;
      case 'tire_wear':
        return car.value!.tire_wear;
      default:
        return 0;
    }
  })();

  // Can improve if stat is less than 1.0
  return currentValue < 1.0;
}

function getImprovementCost(statName: string): number {
  if (!car.value) return 0;

  const currentValue = (() => {
    switch (statName) {
      case 'handling':
        return car.value!.handling;
      case 'acceleration':
        return car.value!.acceleration;
      case 'top_speed':
        return car.value!.top_speed;
      case 'reliability':
        return car.value!.reliability;
      case 'fuel_consumption':
        return car.value!.fuel_consumption;
      case 'tire_wear':
        return car.value!.tire_wear;
      default:
        return 0;
    }
  })();

  // Cost ranges from $2 for stats below 0.1 to $100 for stats above 0.9
  // Linear interpolation: cost = 2 + (currentValue / 0.9) * 98
  // But we want: 0.0 -> $2, 0.9 -> $100
  if (currentValue < 0.1) {
    return 2;
  } else if (currentValue >= 0.9) {
    return 100;
  } else {
    // Linear interpolation between 0.1 and 0.9
    const normalized = (currentValue - 0.1) / 0.8; // 0 to 1
    return Math.round(2 + normalized * 98);
  }
}

async function loadCar() {
  if (!carId.value) {
    error.value = 'No car ID provided';
    loading.value = false;
    return;
  }

  loading.value = true;
  error.value = '';

  try {
    car.value = await getCar(carId.value);

    // Load team if car has a team
    if (car.value.team_id) {
      try {
        team.value = await getTeam(car.value.team_id);
      } catch (err) {
        console.error('Failed to load team:', err);
        // Don't fail the whole component if team load fails
      }
    }

    // Load player's team to check ownership
    if (isAuthenticated()) {
      try {
        playerTeam.value = await getMyTeam();
      } catch (err) {
        console.error('Failed to load player team:', err);
        // Don't fail the whole component if player team load fails
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load car';
    console.error('Error loading car:', err);
  } finally {
    loading.value = false;
  }
}

// Watch for route changes (carId changes)
watch(
  () => carId.value,
  () => {
    loadCar();
  },
);

async function improve(stat: string) {
  if (!carId.value || !canImprove.value || improving.value) {
    return;
  }

  const cost = getImprovementCost(stat);
  if (availableCash.value < cost) {
    alert(`Insufficient cash. Need $${cost}, but only have $${availableCash.value}`);
    return;
  }

  improving.value = true;
  try {
    const updatedCar = await improveCar(carId.value, stat);
    car.value = updatedCar.car;
    // Update both team (car's team) and playerTeam (player's team)
    if (team.value && team.value.id === updatedCar.team.id) {
      team.value = updatedCar.team;
    }
    if (playerTeam.value && playerTeam.value.id === updatedCar.team.id) {
      playerTeam.value = updatedCar.team;
    }
  } catch (err) {
    alert(err instanceof Error ? err.message : 'Failed to improve car');
    console.error('Error improving car:', err);
  } finally {
    improving.value = false;
  }
}

onMounted(() => {
  loadCar();
});
</script>

<style scoped>
.car-detail-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.car-detail-content {
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

.car-display {
  width: 100%;
}

.car-header-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
  margin-bottom: 2rem;
  display: flex;
  align-items: center;
  gap: 2rem;
}

.car-icon-large {
  width: 120px;
  height: 120px;
  border-radius: 8px;
  overflow: hidden;
  border: 4px solid #e0e0e0;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
}

.car-icon-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  object-position: center;
}

.car-info {
  flex: 1;
}

.car-info h2 {
  margin: 0 0 1rem 0;
  color: #1a1a2e;
  font-size: 2rem;
}

.car-meta {
  display: flex;
  gap: 1rem;
  align-items: flex-start;
  color: #666;
  font-size: 1rem;
}

.team-logo-header {
  flex-shrink: 0;
}

.team-logo-img {
  width: 60px;
  height: 60px;
  object-fit: contain;
  object-position: center;
  border-radius: 4px;
}

.team-info-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.team-name-link {
  color: #2d4059;
  text-decoration: none;
  font-weight: 500;
  transition: color 0.2s;
  background-color: transparent !important;
}

.team-name-link:hover {
  color: #1a1a2e;
  text-decoration: underline;
  background-color: transparent !important;
}

.team-cash-header {
  color: #666;
  font-size: 0.9rem;
}

.car-stats-section {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 2rem;
  margin-bottom: 2rem;
}

.car-stats-section h3 {
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
  align-items: center;
  padding: 0.125rem 0;
  gap: 0.5rem;
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

.stat-right-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  justify-content: flex-end;
}

.improve-btn {
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

.improve-btn:hover:not(:disabled) {
  background-color: #1a1a2e;
}

.improve-btn:disabled {
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
  .car-header-section {
    flex-direction: column;
    text-align: center;
  }

  .car-meta {
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
