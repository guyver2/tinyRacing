<script setup lang="ts">
import { ref } from 'vue';
import RaceHeader from './components/RaceHeader.vue';
import CarsTable from './components/CarsTable.vue';
import TrackVisualizer from './components/TrackVisualizer.vue';
import TrackSvg from './components/TrackSvg.vue';
import ConnectionStatus from './components/ConnectionStatus.vue';
import { useRaceData } from './services/WebSocketService';

const { raceState, connected } = useRaceData();
const isCarTableCollapsed = ref(true);


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
            <CarsTable
              :cars="raceState.cars"
              v-model:collapsed="isCarTableCollapsed"
            />
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

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
  font-family: 'Courier New', monospace;
}

body {
  background-color: #f0f5f9;
  color: #2d4059;
  padding: 0;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  margin: 0;
}

.app-container {
  max-width: 100%;
  width: 100%;
  margin: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-content {
  display: flex;
  flex-direction: column;
  position: relative;
  margin-bottom: 0;
  height: calc(100% - 150px);
}

.game-content {
  display: flex;
  flex: 1;
  position: relative;
  margin-bottom: 0;
  height: 100%;
  width: 100%;
}

.sidebar {
  width: 20%;
  max-width: 250px;
  min-width: 150px;
  transition: width 0.3s ease;
  position: absolute;
  background-color: #f9f7f7;
  border-radius: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  height: auto;
  z-index: 5;
  left: 0;
}

.sidebar.expanded {
  width: 100%;
  min-width: fit-content;
  z-index: 10;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
}

.cars-table-container {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}

.track-content {
  position: absolute;
  left: min(20%, 250px);
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
}

.header-content {
  width: 100%;
  height: auto;
  display: flex;
  flex-direction: column;
  margin-top: 15px;
  margin-bottom: 15px;
}

/* Pastel Team Colors */
.team-1 {
  color: #2d4059;
  background-color: #aed9e0;
} /* Pastel Blue */
.team-2 {
  color: #2d4059;
  background-color: #ffa5a5;
} /* Pastel Red */
.team-3 {
  color: #2d4059;
  background-color: #b5e8b5;
} /* Pastel Green */
.team-4 {
  color: #2d4059;
  background-color: #ffe3a3;
} /* Pastel Yellow */
.team-5 {
  color: #2d4059;
  background-color: #739ff2;
} /* Pastel Purple */

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
