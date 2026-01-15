<template>
  <header>
    <div class="desktop-layout">
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
    </div>
    <div class="mobile-layout">
      <span class="mobile-track-name">{{ trackName }}</span>
      <span class="mobile-status">{{ raceStatus }}</span>
      <span class="mobile-time">{{ formattedElapsedTime }}</span>
      <span class="mobile-lap">{{ currentLap }}/{{ totalLaps }}</span>
      <img :src="`/assets/weather/${weather}.svg`" alt="Weather Icon" class="mobile-weather-icon" />
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { apiRequest } from '@/services/ApiService';

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

async function startStopRace() {
  let endpoint = '';
  if (props.raceStatus === 'Paused') {
    endpoint = `/race/${RACE_ID}/start`;
  } else if (props.raceStatus === 'Running') {
    endpoint = `/race/${RACE_ID}/pause`;
  }

  if (endpoint) {
    await apiRequest(endpoint, {
      method: 'POST',
      body: '',
    });
  }
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
  color: #141c27;
}

.desktop-layout {
  display: flex;
  justify-content: space-between;
  width: 100%;
  align-items: center;
}

.mobile-layout {
  display: none;
}

.race-info {
  font-size: 1.1em;
  color: #141c27;
}

.race-status {
  text-align: right;
  font-size: 1.1em;
  color: #141c27;
}

.wetness-indicator {
  color: #141c27;
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
  cursor: pointer;
  transition: background-color 0.2s;
}

button:hover {
  background-color: #c9d6df;
}

button:active {
  background-color: #b8c5d0;
}

/* Mobile Responsive Styles */
@media (max-width: 768px) {
  header {
    padding: 6px 8px;
    margin-bottom: 3px;
  }

  .desktop-layout {
    display: none;
  }

  .mobile-layout {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    width: 100%;
    gap: 8px;
    font-size: 0.8rem;
  }

  .mobile-track-name {
    font-weight: bold;
    color: #141c27;
    flex: 1 1 auto;
    min-width: 0;
    word-wrap: break-word;
    line-height: 1.2;
  }

  .mobile-status {
    color: #141c27;
    font-weight: 500;
    flex-shrink: 0;
  }

  .mobile-time {
    color: #141c27;
    font-weight: 500;
    flex-shrink: 0;
  }

  .mobile-lap {
    color: #141c27;
    font-weight: 500;
    flex-shrink: 0;
  }

  .mobile-weather-icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }
}

@media (max-width: 480px) {
  header {
    padding: 5px 6px;
  }

  .mobile-layout {
    font-size: 0.75rem;
    gap: 6px;
  }

  .mobile-weather-icon {
    width: 18px;
    height: 18px;
  }
}
</style>
