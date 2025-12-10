<template>
  <div v-if="visible" class="strategy-form-overlay">
    <div class="strategy-form" @click.stop>
      <div class="header">
        <div class="driver-info">
          <h2>Driver Info</h2>
          <div>{{ car.driver.name }} - Car #{{ car.car_number }}</div>
          <div>Team: {{ car.team.name }}</div>
        </div>
        <div class="car-info">
          <h2>Car Info</h2>
          <div>Fuel: {{ car.fuel.toFixed(1) }}</div>
          <div>Tire: {{ car.tire.type }} (Wear: {{ car.tire.wear.toFixed(1) }})</div>
          <div>Speed: {{ car.speed.toFixed(1) }}</div>
        </div>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="driving-styles">
        <button
          class="style-btn"
          :class="{ active: selectedStyle === 'Relax' }"
          @click="selectDrivingStyle('Relax')"
        >
          Relax
        </button>
        <button
          class="style-btn"
          :class="{ active: selectedStyle === 'Normal' }"
          @click="selectDrivingStyle('Normal')"
        >
          Normal
        </button>
        <button
          class="style-btn"
          :class="{ active: selectedStyle === 'Aggressive' }"
          @click="selectDrivingStyle('Aggressive')"
        >
          Aggressive
        </button>
      </div>

      <div class="pit-strategy">
        <div class="refuel-section">
          <h3>refuel</h3>
          <div class="refuel-slider">
            <input type="range" min="0" max="100" v-model="refuelAmount" class="slider" />
            <div class="slider-labels">
              <span>0</span>
              <span>100</span>
            </div>
          </div>
        </div>

        <div class="tire-section">
          <h3>Change tire</h3>
          <div class="tire-options">
            <button
              class="tire-btn"
              @click="selectTire('Soft')"
              :class="{ active: selectedTire === 'Soft' }"
            >
              <img :src="`/assets/tires/soft.svg`" alt="Soft" title="Soft" class="tire-icon" />
            </button>
            <button
              class="tire-btn"
              @click="selectTire('Medium')"
              :class="{ active: selectedTire === 'Medium' }"
            >
              <img
                :src="`/assets/tires/medium.svg`"
                alt="Medium"
                title="Medium"
                class="tire-icon"
              />
            </button>
            <button
              class="tire-btn"
              @click="selectTire('Hard')"
              :class="{ active: selectedTire === 'Hard' }"
            >
              <img :src="`/assets/tires/hard.svg`" alt="Hard" title="Hard" class="tire-icon" />
            </button>
            <button
              class="tire-btn"
              @click="selectTire('Inter')"
              :class="{ active: selectedTire === 'Inter' }"
            >
              <img :src="`/assets/tires/inter.svg`" alt="Inter" title="Inter" class="tire-icon" />
            </button>
            <button
              class="tire-btn"
              @click="selectTire('Wet')"
              :class="{ active: selectedTire === 'Wet' }"
            >
              <img :src="`/assets/tires/wet.svg`" alt="Wet" title="Wet" class="tire-icon" />
            </button>

            <button class="pit-btn" @click="executePitStop">Pit</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { Car } from '@/types';
import { apiRequest } from '@/services/ApiService';

const RACE_ID = 1;

const props = defineProps<{
  car: Car;
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const selectedStyle = ref(props.car.driving_style);
const selectedTire = ref<string | null>(null);
const refuelAmount = ref(100);

async function selectDrivingStyle(style: string) {
  selectedStyle.value = style;
  await apiRequest(`/race/${RACE_ID}/car/${props.car.car_number}/driving-style`, {
    method: 'PUT',
    body: JSON.stringify({ style: style }),
  });
}

function selectTire(type: string) {
  selectedTire.value = type;
}

async function executePitStop() {
  const response = await apiRequest(`/race/${RACE_ID}/car/${props.car.car_number}/pit`, {
    method: 'POST',
    body: JSON.stringify({
      tires: selectedTire.value,
      refuel: Number(refuelAmount.value),
    }),
  });
  if (response.ok) {
    close();
  }
}

function close() {
  emit('close');
}
</script>

<style scoped>
* {
  box-sizing: border-box;
  font-family: 'Courier New', monospace;
}

.strategy-form-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.strategy-form {
  background-color: #f9f7f7;
  border: 1px solid #c9d6df;
  border-radius: 0;
  width: 90%;
  max-width: 600px;
  padding: 20px;
  position: relative;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  color: #2d4059;
}

.header {
  display: flex;
  justify-content: space-between;
  border-bottom: 1px solid #c9d6df;
  padding-bottom: 15px;
  margin-bottom: 15px;
  position: relative;
}

.driver-info,
.car-info {
  flex: 1;
}

h2 {
  font-size: 1.1em;
  margin-top: 0;
  margin-bottom: 10px;
  color: #2d4059;
  font-weight: bold;
}

h3 {
  font-size: 1em;
  margin-top: 0;
  margin-bottom: 10px;
  color: #2d4059;
  font-weight: bold;
}

.close-btn {
  position: absolute;
  top: -10px;
  right: -10px;
  background-color: #dbe2ef;
  border: 1px solid #c9d6df;
  width: 30px;
  height: 30px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 20px;
  font-weight: bold;
  color: #2d4059;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.close-btn:hover {
  background-color: #c9d6df;
}

.driving-styles {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 20px;
}

.style-btn {
  flex: 1;
  padding: 10px 15px;
  border: 1px solid #c9d6df;
  background-color: #f9f7f7;
  border-radius: 4px;
  font-size: 1em;
  font-weight: bold;
  cursor: pointer;
  color: #2d4059;
  transition: background-color 0.2s;
}

.style-btn:hover {
  background-color: #dbe2ef;
}

.style-btn.active {
  background-color: #dbe2ef;
  border-color: #c9d6df;
}

.pit-strategy {
  border: 1px solid #c9d6df;
  border-radius: 0;
  padding: 15px;
  background-color: #f9f7f7;
}

.refuel-section {
  margin-bottom: 20px;
}

.refuel-slider {
  position: relative;
  width: 100%;
}

.slider {
  width: 100%;
  height: 8px;
  -webkit-appearance: none;
  background: #dbe2ef;
  outline: none;
  margin: 10px 0;
  border-radius: 4px;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  background: #2d4059;
  cursor: pointer;
  border-radius: 50%;
  border: 1px solid #c9d6df;
}

.slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  background: #2d4059;
  cursor: pointer;
  border-radius: 50%;
  border: 1px solid #c9d6df;
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  font-size: 0.9em;
  font-weight: bold;
  color: #2d4059;
}

.tire-options {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.tire-btn {
  width: 48px;
  height: 48px;
  border: 1px solid #c9d6df;
  background-color: #f9f7f7;
  padding: 2px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.tire-btn:hover {
  background-color: #dbe2ef;
}

.tire-btn.active {
  border: 2px solid #2d4059;
  background-color: #dbe2ef;
  padding: 0;
}

.tire-icon {
  width: 36px;
  height: 36px;
}

.pit-btn {
  margin-left: auto;
  padding: 10px 20px;
  border: 1px solid #c9d6df;
  background-color: #dbe2ef;
  border-radius: 4px;
  font-size: 1em;
  font-weight: bold;
  cursor: pointer;
  color: #2d4059;
  transition: background-color 0.2s;
}

.pit-btn:hover {
  background-color: #c9d6df;
}
</style>
