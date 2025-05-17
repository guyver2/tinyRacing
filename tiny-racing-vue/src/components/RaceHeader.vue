<template>
  <header>
    <div class="race-info">Race: <span>{{ trackName }}</span> Time elapsed: <span>{{ timeElapsed }}s</span></div>
    <div class="race-status">Status: <span>{{ raceStatus }}</span> Lap: <span>{{ currentLap }}</span>/<span>{{ totalLaps }}</span></div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  trackName: string;
  timeElapsed: string;
  raceStatus: string;
  currentLap: number;
  totalLaps: number;
}>();
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
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
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
</style> 