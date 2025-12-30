<template>
  <div class="cars-table-wrapper">
    <table class="cars-table" :class="{ collapsed: collapsed }">
      <thead>
        <tr>
          <th v-if="!collapsed">Pos</th>
          <th class="car-number-col">Car#</th>
          <th>Driver</th>
          <th>Status</th>
          <!-- <th>Speed</th> -->
          <th v-if="!collapsed">Team</th>
          <th v-if="!collapsed">Lap#</th>
          <th v-if="!collapsed">Status</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="cars.length === 0">
          <td :colspan="collapsed ? 3 : 7" style="text-align: center">Waiting for race data...</td>
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
          <td class="car-number-col" :class="{ 'top-3': car.race_position <= 3 }">
            {{ car.car_number }}
          </td>
          <td :class="{ 'top-3': car.race_position <= 3 }" class="driver-cell">
            <img
              v-if="car.team.logo"
              :src="car.team.logo"
              :alt="car.team.name"
              :title="car.team.name"
              class="team-logo"
            />
            <span>{{ car.driver.name }}</span>
          </td>
          <td class="status-column">
            <div class="status-column-content">
              <div
                class="tire-icon-container"
                :title="`Tire: ${car.tire.type} - Wear: ${car.tire.wear.toFixed(1)}%`"
              >
                <div class="tire-wear-overlay" :style="{ height: `${car.tire.wear}%` }"></div>
                <img
                  :src="getTireImagePath(car.tire.type)"
                  :alt="car.tire.type"
                  class="tire-icon"
                />
              </div>
              <div class="fuel-gauge-vertical" :title="`Fuel: ${car.fuel.toFixed(1)}%`">
                <div
                  class="fuel-gauge-fill"
                  :style="{ height: `${car.fuel}%` }"
                  :class="getFuelColorClass(car.fuel)"
                ></div>
              </div>
              <div
                class="status-indicator"
                :class="getStatusIndicatorClass(car)"
                :title="getStatusTooltip(car)"
              ></div>
            </div>
          </td>
          <!-- <td>
            <div class="speed-gauge-vertical" :title="`Speed: ${car.speed.toFixed(1)} km/h`">
              <div
                class="speed-gauge-fill"
                :style="{ 
                  height: `${(car.speed / 400) * 100}%`,
                  backgroundColor: getSpeedColor(car.speed)
                }"
              ></div>
            </div>
          </td> -->
          <td v-if="!collapsed">{{ car.team.name }}</td>
          <td v-if="!collapsed">{{ Math.floor(car.track_position) }}</td>
          <td v-if="!collapsed">{{ car.status }}</td>
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

// Get status indicator class based on car status
function getStatusIndicatorClass(car: Car): string {
  if (car.pit_requested === true) {
    return 'status-pit-requested';
  }
  const status = car.status?.toLowerCase() || '';
  switch (status) {
    case 'racing':
      return 'status-racing';
    case 'pit':
      return 'status-pit';
    case 'dnf':
      return 'status-dnf';
    case 'finished':
      return 'status-finished';
    default:
      return 'status-racing';
  }
}

// Get tooltip text for status indicator
function getStatusTooltip(car: Car): string {
  if (car.pit_requested === true) {
    return 'Pit Stop Requested';
  }
  const status = car.status?.toLowerCase() || '';
  switch (status) {
    case 'racing':
      return 'Racing';
    case 'pit':
      return 'In Pit';
    case 'dnf':
      return 'Did Not Finish';
    case 'finished':
      return 'Finished';
    default:
      return car.status || 'Unknown';
  }
}

// Calculate speed color based on speed (0-400 km/h)
// Returns a color that transitions from red (0) -> orange (200) -> green (400)
function getSpeedColor(speed: number): string {
  const clampedSpeed = Math.max(0, Math.min(400, speed));
  const ratio = clampedSpeed / 400;

  // Interpolate between colors: red (#e84545) -> orange (#f9a826) -> green (#7bc74d)
  if (ratio <= 0.5) {
    // Red to orange (0 to 0.5)
    const t = ratio * 2; // 0 to 1
    const r = Math.round(232 + (249 - 232) * t); // e8 -> f9
    const g = Math.round(69 + (168 - 69) * t); // 45 -> a8
    const b = Math.round(69 + (38 - 69) * t); // 45 -> 26
    return `rgb(${r}, ${g}, ${b})`;
  } else {
    // Orange to green (0.5 to 1)
    const t = (ratio - 0.5) * 2; // 0 to 1
    const r = Math.round(249 + (123 - 249) * t); // f9 -> 7b
    const g = Math.round(168 + (199 - 168) * t); // a8 -> c7
    const b = Math.round(38 + (77 - 38) * t); // 26 -> 4d
    return `rgb(${r}, ${g}, ${b})`;
  }
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
  padding: 2px 8px;
  text-align: left;
  border-bottom: 1px solid #c9d6df;
  color: #141c27;
  font-weight: bold;
}

.cars-table td {
  padding: 2px 8px;
  border-bottom: 1px solid #e8e8e8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #141c27;
}

.cars-table th.car-number-col,
.cars-table td.car-number-col {
  width: 1%;
  white-space: nowrap;
}

.cars-table.collapsed th,
.cars-table.collapsed td {
  padding: 2px 8px;
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

.driver-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.team-logo {
  width: 32px;
  height: 32px;
  object-fit: cover;
  object-position: center;
  flex-shrink: 0;
  border-radius: 4px;
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
  padding: 2px;
  width: 100%;
  height: 100%;
  position: relative;
  z-index: 1;
}

.fuel-gauge-vertical {
  position: relative;
  width: 18px;
  height: 28px;
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

.status-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: inline-block;
  cursor: help;
}

.status-indicator.status-racing {
  background-color: #7bc74d;
}

.status-indicator.status-pit {
  background-color: #5c7aea;
}

.status-indicator.status-dnf {
  background-color: #e84545;
}

.status-indicator.status-pit-requested {
  background-color: #f9a826;
}

.status-indicator.status-finished {
  background-color: #2d4059;
}

.status-column {
  padding: 2px 4px;
}

.status-column-content {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: center;
}

.speed-gauge-vertical {
  position: relative;
  width: 18px;
  height: 28px;
  background-color: #dbe2ef;
  border: 1px solid #c9d6df;
  border-radius: 4px;
  overflow: hidden;
  display: inline-block;
}

.speed-gauge-fill {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  transition:
    height 0.3s ease,
    background-color 0.3s ease;
  border-radius: 0 0 3px 3px;
}
</style>
