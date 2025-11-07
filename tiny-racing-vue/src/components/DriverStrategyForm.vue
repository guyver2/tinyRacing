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

const API_URL = 'http://localhost:3000';

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

function selectDrivingStyle(style: string) {
  selectedStyle.value = style;
  const requestOptions = {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ style: style }),
  };
  fetch(`${API_URL}/cars/${props.car.car_number}/driving-style`, requestOptions);
}

function selectTire(type: string) {
  selectedTire.value = type;
}

function executePitStop() {
  const requestOptions = {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ tires: selectedTire.value, refuel: refuelAmount.value }),
  };
  fetch(`${API_URL}/cars/${props.car.car_number}/pit`, requestOptions);
}

function close() {
  emit('close');
}
</script>

<style scoped>
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
  background-color: white;
  border: 2px solid black;
  width: 90%;
  max-width: 600px;
  padding: 20px;
  position: relative;
}

.header {
  display: flex;
  justify-content: space-between;
  border-bottom: 2px solid black;
  padding-bottom: 15px;
  margin-bottom: 15px;
  position: relative;
}

.driver-info,
.car-info {
  flex: 1;
}

h2 {
  font-size: 24px;
  margin-top: 0;
  margin-bottom: 10px;
}

h3 {
  font-size: 20px;
  margin-top: 0;
  margin-bottom: 10px;
}

.close-btn {
  position: absolute;
  top: -15px;
  right: -15px;
  background-color: white;
  border: 2px solid black;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 20px;
  font-weight: bold;
}

.driving-styles {
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
}

.style-btn {
  flex: 1;
  margin: 0 5px;
  padding: 15px;
  border: 2px solid black;
  background-color: white;
  font-size: 18px;
  font-weight: bold;
  cursor: pointer;
}

.style-btn.active {
  background-color: #dbe2ef;
}

.pit-strategy {
  border: 2px solid black;
  padding: 15px;
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
  height: 10px;
  -webkit-appearance: none;
  background: #ddd;
  outline: none;
  margin: 10px 0;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  background: black;
  cursor: pointer;
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  font-size: 18px;
  font-weight: bold;
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
  border: 2px solid black;
  background-color: white;
  padding: 2px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tire-btn.active {
  border-width: 4px;
  padding: 0;
}

.tire-icon {
  width: 36px;
  height: 36px;
}

.pit-btn {
  margin-left: auto;
  padding: 10px 20px;
  border: 2px solid black;
  background-color: white;
  font-size: 18px;
  font-weight: bold;
  cursor: pointer;
}
</style>
