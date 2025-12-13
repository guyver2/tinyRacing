<template>
  <div class="cars-table-wrapper">
    <table class="cars-table" :class="{ collapsed: collapsed }">
      <thead>
        <tr>
          <th v-if="!collapsed">Pos</th>
          <th>Car#</th>
          <th>Driver</th>
          <th>Tire</th>
          <th>Fuel</th>
          <th v-if="!collapsed">Team</th>
          <th v-if="!collapsed">Wear</th>
          <th v-if="!collapsed">Lap#</th>
          <th v-if="!collapsed">Lap%</th>
          <th v-if="!collapsed">Status</th>
          <th v-if="!collapsed">Style</th>
          <th v-if="!collapsed">Speed</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="cars.length === 0">
          <td :colspan="collapsed ? 5 : 13" style="text-align: center">Waiting for race data...</td>
        </tr>
        <tr
          v-for="car in cars"
          :key="car.car_number"
          :class="getRowClasses(car)"
          :style="{ 'background-color': hexToPastel(car.team.color) }"
        >
          <td v-if="!collapsed" :class="{ 'top-3': car.race_position <= 3 }">
            {{ car.race_position }}
          </td>
          <td :class="{ 'top-3': car.race_position <= 3 }">{{ car.car_number }}</td>
          <td :class="{ 'top-3': car.race_position <= 3 }">{{ car.driver.name }}</td>
          <td>
            <div
              class="tire-icon-container"
              :title="`Tire: ${car.tire.type} - Wear: ${car.tire.wear.toFixed(1)}%`"
            >
              <div class="tire-wear-overlay" :style="{ height: `${car.tire.wear}%` }"></div>
              <img :src="getTireImagePath(car.tire.type)" :alt="car.tire.type" class="tire-icon" />
            </div>
          </td>
          <td>
            <div class="fuel-gauge-vertical" :title="`Fuel: ${car.fuel.toFixed(1)}%`">
              <div
                class="fuel-gauge-fill"
                :style="{ height: `${car.fuel}%` }"
                :class="getFuelColorClass(car.fuel)"
              ></div>
            </div>
          </td>
          <td v-if="!collapsed">{{ car.team.name }}</td>
          <td v-if="!collapsed">{{ car.tire.wear.toFixed(1) }}</td>
          <td v-if="!collapsed">{{ Math.floor(car.track_position) }}</td>
          <td v-if="!collapsed">{{ ((car.track_position % 1) * 100).toFixed(1) }}%</td>
          <td v-if="!collapsed">{{ car.status }}</td>
          <td v-if="!collapsed">{{ car.driving_style }}</td>
          <td v-if="!collapsed">{{ car.speed.toFixed(1) }}</td>
        </tr>
      </tbody>
    </table>
    <div class="table-footer">
      <div class="toggle-button" @click="toggleCollapsed">
        {{ collapsed ? '>>' : '<<' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Car } from '@/types';
import { getPlayerId } from '@/services/ApiService';
import { hexToPastel } from '@/utils/colorUtils';

const props = defineProps<{
  cars: Car[];
  collapsed?: boolean;
}>();

const emit = defineEmits<{
  'update:collapsed': [value: boolean];
}>();

// Get current player ID
const currentPlayerId = computed(() => getPlayerId());

// Check if a car belongs to the current player
function isPlayerCar(car: Car): boolean {
  if (!currentPlayerId.value) {
    return false;
  }
  return car.player_uuid === currentPlayerId.value;
}

function getRowClasses(car: Car) {
  const classes = [`team`];
  if (car.race_position <= 3) {
    classes.push('top-3');
  }
  // Add a class to indicate if this is the player's car
  if (isPlayerCar(car)) {
    classes.push('player-car');
  } else {
    classes.push('not-player-car');
  }
  return classes;
}

function toggleCollapsed() {
  emit('update:collapsed', !props.collapsed);
}

function getTireImagePath(tireType: string): string {
  const normalized = tireType.toLowerCase();
  // Handle "intermediate" -> "inter" for image path
  if (normalized === 'intermediate') {
    return '/assets/tires/inter.svg';
  }
  return `/assets/tires/${normalized}.svg`;
}

function getFuelColorClass(fuel: number): string {
  if (fuel > 50) return 'fuel-high';
  if (fuel > 25) return 'fuel-medium';
  return 'fuel-low';
}
</script>

<style scoped>
.cars-table-wrapper {
  position: relative;
  width: 100%;
}

.team {
  color: #141c27;
}

.table-footer {
  display: flex;
  justify-content: flex-end;
  padding: 5px;
}

.toggle-button {
  background-color: #dbe2ef;
  border-radius: 4px;
  padding: 4px 8px;
  cursor: pointer;
  font-weight: bold;
  display: inline-block;
}

.cars-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 0;
  background-color: #f9f7f7;
  border-radius: 0;
  overflow: hidden;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.cars-table.collapsed {
  width: 100%;
}

.cars-table th {
  background-color: #dbe2ef;
  padding: 10px;
  text-align: left;
  border-bottom: 1px solid #c9d6df;
  color: #141c27;
  font-weight: bold;
}

.cars-table td {
  padding: 8px;
  border-bottom: 1px solid #e8e8e8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #141c27;
}

.cars-table.collapsed th,
.cars-table.collapsed td {
  padding: 8px;
}

.cars-table.collapsed th:first-child,
.cars-table.collapsed td:first-child {
  padding-left: 12px;
}

.top-3 {
  font-weight: bold;
}

.bold {
  font-weight: bold;
}

.tire-icon-container {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid #c9d6df;
  border-radius: 4px;
  background-color: #f9f7f7;
  overflow: hidden;
}

.tire-wear-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: rgba(232, 69, 69, 0.3);
  transition: height 0.3s ease;
  pointer-events: none;
}

.tire-icon {
  width: 24px;
  height: 24px;
  position: relative;
  z-index: 1;
}

.fuel-gauge-vertical {
  position: relative;
  width: 20px;
  height: 32px;
  background-color: #dbe2ef;
  border: 1px solid #c9d6df;
  border-radius: 4px;
  overflow: hidden;
  display: inline-block;
}

.fuel-gauge-fill {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  transition:
    height 0.3s ease,
    background-color 0.3s ease;
  border-radius: 0 0 3px 3px;
}

.fuel-gauge-fill.fuel-high {
  background-color: #7bc74d;
}

.fuel-gauge-fill.fuel-medium {
  background-color: #f9a826;
}

.fuel-gauge-fill.fuel-low {
  background-color: #e84545;
}
</style>
