<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import RaceHeader from './RaceHeader.vue';
import CarsTable from './CarsTable.vue';
import TrackVisualizer from './TrackVisualizer.vue';
import TrackSvg from './TrackSvg.vue';
import ConnectionStatus from './ConnectionStatus.vue';
import DriverControl from './DriverControl.vue';
import { useRaceData } from '../services/WebSocketService';
import { getPlayerId, isAuthenticated } from '../services/ApiService';
import type { Car } from '@/types';

const emit = defineEmits<{
  navigate: [view: string];
}>();

const router = useRouter();
const { raceState, connected } = useRaceData();
const isCarTableCollapsed = ref(true);

// Get current player ID
const currentPlayerId = computed(() => getPlayerId());

// Check if there's no active race
const hasNoRace = computed(() => {
  return (
    !raceState.value.track.id ||
    raceState.value.track.name === 'No race loaded' ||
    raceState.value.cars.length === 0
  );
});

// Filter cars to only show player's cars, ordered by car number
const playerCars = computed(() => {
  if (!isAuthenticated() || !currentPlayerId.value) {
    return [];
  }
  return raceState.value.cars
    .filter((car: Car) => car.player_uuid === currentPlayerId.value)
    .sort((a: Car, b: Car) => a.car_number - b.car_number);
});

function navigateToRaces() {
  router.push({ name: 'races' });
  emit('navigate', 'races');
}
</script>

<template>
  <div class="app-container">
    <!-- No race state -->
    <div v-if="hasNoRace" class="no-race-container">
      <div class="no-race-message">
        <h2>No Active Race</h2>
        <p>There is currently no active race. Start a race from the races page to begin racing.</p>
        <button @click="navigateToRaces" class="btn-primary">Go to Races</button>
      </div>
      <div class="footer-content">
        <ConnectionStatus :connected="connected" />
      </div>
    </div>

    <!-- Active race state -->
    <template v-else>
      <div class="main-content">
        <div class="header-content">
          <RaceHeader
            :trackName="raceState.track.name"
            :elapsedTime="raceState.track.elapsed_time"
            :raceStatus="raceState.race_status"
            :currentLap="raceState.current_lap"
            :totalLaps="raceState.total_laps"
            :weather="raceState.track.current_weather"
            :wetness="raceState.track.wetness"
          />
        </div>
        <div class="game-content">
          <div class="sidebar" :class="{ expanded: !isCarTableCollapsed }">
            <div class="cars-table-container">
              <CarsTable :cars="raceState.cars" v-model:collapsed="isCarTableCollapsed" />
            </div>
            <div v-if="playerCars.length > 0" class="driver-controls-container">
              <DriverControl v-for="car in playerCars" :key="car.car_number" :car="car" />
            </div>
          </div>

          <div class="track-content">
            <TrackSvg :cars="raceState.cars" :trackId="raceState.track.id" />
          </div>
        </div>
      </div>

      <div class="footer-content">
        <TrackVisualizer :cars="raceState.cars" />
        <ConnectionStatus :connected="connected" />
      </div>
    </template>
  </div>
</template>

<style scoped>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

.app-container {
  max-width: 100%;
  width: 100%;
  margin: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  overflow-x: hidden;
}

.main-content {
  display: flex;
  flex-direction: column;
  position: relative;
  margin-bottom: 0;
  height: calc(100% - 150px);
  overflow-x: hidden;
}

.game-content {
  display: flex;
  flex: 1;
  position: relative;
  margin-bottom: 0;
  height: 100%;
  width: 100%;
  overflow-x: hidden;
}

.sidebar {
  width: 30%;
  max-width: 375px;
  min-width: 225px;
  transition: width 0.3s ease;
  position: absolute;
  background-color: #f9f7f7;
  border-radius: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  overflow-y: auto;
  overflow-x: visible;
  height: 100%;
  z-index: 5;
  left: 0;
  display: flex;
  flex-direction: column;
}

.sidebar.expanded {
  width: 100%;
  min-width: fit-content;
  z-index: 10;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
}

.cars-table-container {
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
}

.driver-controls-container {
  padding: 10px;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
  max-width: 375px;
}

.track-content {
  position: absolute;
  left: min(30%, 375px);
  right: 0;
  height: 100%;
  border-radius: 0;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.footer-content {
  margin-top: 15px;
  margin-bottom: 15px;
  width: 100%;
  height: auto;
  display: flex;
  flex-direction: column;
  padding: 0 1%;
}

.header-content {
  width: 100%;
  height: auto;
  display: flex;
  flex-direction: column;
  margin-top: 15px;
  margin-bottom: 15px;
}

.no-race-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100%;
  padding: 2rem;
}

.no-race-message {
  text-align: center;
  max-width: 600px;
  padding: 3rem;
  background-color: #ffffff;
  border-radius: 12px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.no-race-message h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 1rem;
  font-weight: 600;
}

.no-race-message p {
  color: #666;
  font-size: 1.1rem;
  margin-bottom: 2rem;
  line-height: 1.6;
}

.btn-primary {
  background-color: #5c7aea;
  color: white;
  border: none;
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.3s ease;
  font-weight: 500;
}

.btn-primary:hover {
  background-color: #4a6fd8;
}

.btn-primary:active {
  background-color: #3d5bc4;
}

/* Pastel Tire Colors */
.tire-soft {
  color: #e84545;
} /* Softer Red */
.tire-medium {
  color: #f9a826;
} /* Softer Yellow */
.tire-hard {
  color: #4d4d4d;
} /* Softer White/Gray */
.tire-intermediate {
  color: #7bc74d;
} /* Softer Green */
.tire-wet {
  color: #5c7aea;
} /* Softer Blue */
</style>
