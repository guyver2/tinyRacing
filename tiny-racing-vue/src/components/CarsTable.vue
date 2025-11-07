<template>
  <div class="cars-table-wrapper">
    <table class="cars-table" :class="{ collapsed: collapsed }">
      <thead>
        <tr>
          <th v-if="!collapsed">Pos</th>
          <th>Car#</th>
          <th>Driver</th>
          <th v-if="!collapsed">Team</th>
          <th v-if="!collapsed">Tire</th>
          <th v-if="!collapsed">Wear</th>
          <th v-if="!collapsed">Fuel</th>
          <th v-if="!collapsed">Lap#</th>
          <th v-if="!collapsed">Lap%</th>
          <th v-if="!collapsed">Status</th>
          <th v-if="!collapsed">Style</th>
          <th v-if="!collapsed">Speed</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="cars.length === 0">
          <td :colspan="collapsed ? 3 : 12" style="text-align: center">Waiting for race data...</td>
        </tr>
        <tr
          v-for="car in cars"
          :key="car.car_number"
          :class="getRowClasses(car)"
          @click="selectCar(car)"
          :style="{'background-color': car.team.color}"
        >
          <td v-if="!collapsed" :class="{ 'top-3': car.race_position <= 3 }">
            {{ car.race_position }}
          </td>
          <td :class="{ 'top-3': car.race_position <= 3 }">{{ car.car_number }}</td>
          <td :class="{ 'top-3': car.race_position <= 3 }">{{ car.driver.name }}</td>
          <td v-if="!collapsed">{{ car.team.name }}</td>
          <td v-if="!collapsed">
            <img
              :src="`/assets/tires/${car.tire.type.toLowerCase()}.svg`"
              :alt="`${car.tire.type}`"
              :title="`${car.tire.type}`"
              class="tire-icon"
            />
          </td>
          <td v-if="!collapsed">{{ car.tire.wear.toFixed(1) }}</td>
          <td v-if="!collapsed">{{ car.fuel.toFixed(1) }}</td>
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

    <DriverStrategyForm
      v-if="selectedCar"
      :car="selectedCar"
      :visible="!!selectedCar"
      @close="closeStrategyForm"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { Car } from '@/types';
import DriverStrategyForm from './DriverStrategyForm.vue';

const props = defineProps<{
  cars: Car[];
  collapsed?: boolean;
}>();

const emit = defineEmits<{
  'update:collapsed': [value: boolean];
}>();

const selectedCar = ref<Car | null>(null);

function getRowClasses(car: Car) {
  const classes = [`team`];
  if (car.race_position <= 3) {
    classes.push('top-3');
  }
  return classes;
}

function toggleCollapsed() {
  emit('update:collapsed', !props.collapsed);
}

function selectCar(car: Car) {
  selectedCar.value = car;
}

function closeStrategyForm() {
  selectedCar.value = null;
}

</script>

<style scoped>
.cars-table-wrapper {
  position: relative;
  width: 100%;
}

.team {
  color: #2d4059;
  cursor: pointer;
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
}

.cars-table td {
  padding: 8px;
  border-bottom: 1px solid #e8e8e8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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

.tire-icon {
  width: 24px;
  height: 24px;
}
</style>
