<template>
  <div class="track-line-container">
    <div class="track-line"></div>
    <div v-for="car in cars" :key="car.car_number" class="car-marker" 
         :class="`team-${car.team_number}`"
         :style="{ left: `${(car.track_position % 1) * 100}%` }">
      {{ car.car_number }}
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Car } from '@/types';

defineProps<{
  cars: Car[];
}>();
</script>

<style scoped>
.track-line-container {
  position: relative;
  height: 60px;
  margin: 5px 0;
  border: 1px solid #dbe2ef;
  border-radius: 0;
  overflow: hidden;
  background-color: #f9f7f7;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  width: 100%;
}
.track-line {
  position: absolute;
  top: 50%;
  left: 0;
  width: 100%;
  height: 2px;
  background-color: #a5b1c2;
  transform: translateY(-50%);
}
.car-marker {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 24px;
  height: 24px;
  border-radius: 50%;
  text-align: center;
  line-height: 24px;
  font-size: 0.8em;
  font-weight: bold;
  transition: left 0.3s ease-out;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
  border: 2px solid white;
}
</style> 