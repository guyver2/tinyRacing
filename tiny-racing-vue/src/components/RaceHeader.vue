<template>
  <header>
    <div class="race-info">
      Race: <span>{{ trackName }}</span> Time elapsed: <span>{{ formattedElapsedTime }}</span>
    </div>
    <div class="race-status">
      Status: <button @click="startStopRace">{{ raceStatus }}</button> Lap:
      <span>{{ currentLap }}</span
      >/<span>{{ totalLaps }}</span>
    </div>
    <div class="weather-container">
      <img :src="`/assets/weather/${weather}.svg`" alt="Weather Icon" class="weather-icon" />
      <div class="wetness-indicator">Wetness: {{ Math.round(wetness * 100) }}%</div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const API_URL = 'http://localhost:3000';
const RACE_ID = 1;

const props = defineProps<{
  trackName: string;
  elapsedTime: number;
  raceStatus: string;
  currentLap: number;
  totalLaps: number;
  weather: string;
  wetness: number;
}>();

const formattedElapsedTime = computed(() => {
  const totalSeconds = props.elapsedTime;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds.toFixed(0)}s`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds.toFixed(0)}s`;
  } else {
    return `${seconds.toFixed(0)}s`;
  }
});

function startStopRace() {
  const requestOptions = {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: '',
  };
  let endpoint = '';
  if (props.raceStatus === 'Paused') {
    endpoint = `/race/${RACE_ID}/start`;
  } else if (props.raceStatus === 'Running') {
    endpoint = `/race/${RACE_ID}/pause`;
  }
  fetch(`${API_URL}${endpoint}`, requestOptions); // Placeholder for starting/stopping
}
</script>

<style scoped>
header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
  font-weight: bold;
  padding: 10px 15px;
  background-color: #c9d6df;
  border-radius: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  width: 100%;
  flex-shrink: 0;
}
.race-info {
  font-size: 1.1em;
}
.race-status {
  text-align: right;
  font-size: 1.1em;
}

.weather-container {
  display: flex;
  align-items: center;
  gap: 10px;
}

.weather-icon {
  width: 32px;
  height: 32px;
}

button {
  background-color: #dbe2ef;
  border: none;
  border-radius: 4px;
  padding: 5px 10px;
  font-weight: bold;
}
</style>
