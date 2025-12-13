<script setup lang="ts">
import { ref, computed } from 'vue';
import RaceHeader from './RaceHeader.vue';
import CarsTable from './CarsTable.vue';
import TrackVisualizer from './TrackVisualizer.vue';
import TrackSvg from './TrackSvg.vue';
import ConnectionStatus from './ConnectionStatus.vue';
import DriverControl from './DriverControl.vue';
import { useRaceData } from '../services/WebSocketService';
import { getPlayerId, isAuthenticated } from '../services/ApiService';
import type { Car } from '@/types';

const { raceState, connected } = useRaceData();
const isCarTableCollapsed = ref(true);

// Get current player ID
const currentPlayerId = computed(() => getPlayerId());

// Filter cars to only show player's cars, ordered by car number
const playerCars = computed(() => {
  if (!isAuthenticated() || !currentPlayerId.value) {
    return [];
  }
  return raceState.value.cars
    .filter((car: Car) => car.player_uuid === currentPlayerId.value)
    .sort((a: Car, b: Car) => a.car_number - b.car_number);
});
</script>

<template>
  <div class="app-container">
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
