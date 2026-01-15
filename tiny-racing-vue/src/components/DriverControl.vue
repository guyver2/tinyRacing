<template>
  <div class="driver-control" :class="{ 'player-car': isPlayerCar }">
    <!-- First Row: Avatar, Fuel Gauge, Driving Style -->
    <div class="control-row first-row">
      <div class="driver-avatar-section">
        <div class="driver-avatar-container">
          <div
            v-if="car.driver.stress_level !== undefined"
            class="stress-level-overlay"
            :style="{ height: `${(car.driver.stress_level || 0) * 100}%` }"
            :class="getStressColorClass(car.driver.stress_level || 0)"
          ></div>
          <img
            v-if="driverAvatarUrl"
            :src="driverAvatarUrl"
            :alt="car.driver.name"
            class="driver-avatar"
          />
          <div v-else class="driver-avatar-placeholder">{{ getInitials(car.driver.name) }}</div>
        </div>
      </div>

      <div class="fuel-gauge-container">
        <div class="fuel-gauge-label">
          #{{ car.car_number }} - {{ car.driver.name }} - P{{ car.race_position }}
        </div>
        <div
          class="fuel-gauge"
          @click="setRefuelAmount"
          @mousemove="updateRefuelAmountOnHover"
          @mouseleave="isHoveringGauge = false"
          @mouseenter="isHoveringGauge = true"
          ref="fuelGaugeRef"
        >
          <div
            class="fuel-gauge-fill"
            :style="{ width: `${car.fuel}%` }"
            :class="getFuelColorClass(car.fuel)"
          ></div>
          <div
            class="fuel-gauge-cursor"
            :style="{ left: `${isHoveringGauge ? hoverRefuelAmount : targetRefuelAmount}%` }"
            :class="{ visible: isPlayerCar }"
          ></div>
          <div class="fuel-gauge-text">{{ car.fuel.toFixed(1) }}%</div>
        </div>
      </div>

      <button
        class="driving-style-btn"
        :class="{ disabled: !isPlayerCar }"
        @click="cycleDrivingStyle"
        :disabled="!isPlayerCar"
      >
        {{ car.driving_style }}
      </button>
    </div>

    <!-- Second Row: Tire Icon, PIT Button -->
    <div class="control-row second-row">
      <div class="tire-section">
        <div
          ref="tireIconRef"
          class="tire-icon-container"
          :class="{ 'tire-selected': selectedTireForPit, disabled: !isPlayerCar }"
          @click="toggleTireSelector"
          :title="`Tire: ${car.tire.type} - Wear: ${car.tire.wear.toFixed(1)}%`"
        >
          <div class="tire-wear-overlay" :style="{ height: `${car.tire.wear}%` }"></div>
          <img :src="getTireImagePath(car.tire.type)" :alt="car.tire.type" class="tire-icon" />
        </div>

        <!-- Tire Selector Dropdown -->
        <Teleport to="body">
          <div
            v-if="showTireSelector"
            class="tire-selector"
            :style="{
              top: `${tireSelectorPosition.top}px`,
              left: `${tireSelectorPosition.left}px`,
            }"
            @click.stop
          >
            <button
              class="tire-option-btn"
              :class="{ active: selectedTireForPit === null }"
              @click="selectTireForPit(null)"
            >
              <img
                :src="getTireImagePath('Hard')"
                alt="None"
                class="tire-option-icon"
                style="opacity: 0.4; filter: grayscale(100%)"
              />
              <span>None</span>
            </button>
            <button
              v-for="tireType in tireTypes"
              :key="tireType"
              class="tire-option-btn"
              :class="{ active: selectedTireForPit === tireType }"
              @click="selectTireForPit(tireType)"
            >
              <img :src="getTireImagePath(tireType)" :alt="tireType" class="tire-option-icon" />
              <span>{{ getTireDisplayName(tireType) }}</span>
            </button>
          </div>
        </Teleport>
      </div>

      <div class="speed-gauge-container">
        <!-- Desktop: Arc gauge -->
        <div class="speed-gauge speed-gauge-desktop">
          <svg class="speed-gauge-svg" viewBox="0 0 200 120">
            <!-- Gradient definition -->
            <defs>
              <linearGradient
                :id="`speedGradient-${car.car_number}`"
                x1="0%"
                y1="0%"
                x2="100%"
                y2="0%"
              >
                <stop offset="0%" style="stop-color: #e84545; stop-opacity: 1" />
                <stop offset="50%" style="stop-color: #f9a826; stop-opacity: 1" />
                <stop offset="100%" style="stop-color: #7bc74d; stop-opacity: 1" />
              </linearGradient>
            </defs>
            <!-- Semicircular arc -->
            <path
              :d="`M 20 100 A 80 80 0 0 1 180 100`"
              fill="none"
              :stroke="`url(#speedGradient-${car.car_number})`"
              stroke-width="20"
              stroke-linecap="round"
            />
            <!-- Speed indicator dot -->
            <circle
              class="speed-indicator"
              :cx="getSpeedIndicatorX()"
              :cy="getSpeedIndicatorY()"
              r="12"
              fill="#2d4059"
              stroke="#141c27"
              stroke-width="1"
            />
          </svg>
          <div class="speed-value">
            <div class="speed-number">{{ Math.round(car.speed) }}</div>
            <div class="speed-unit">Km/h</div>
          </div>
        </div>
        <!-- Mobile: Horizontal line gauge -->
        <div class="speed-gauge speed-gauge-mobile">
          <div class="speed-line-container">
            <div class="speed-line-track">
              <div class="speed-line-fill" :style="{ width: `${(car.speed / 400) * 100}%` }"></div>
            </div>
            <div class="speed-value-mobile">
              <span class="speed-number-mobile">{{ Math.round(car.speed) }}</span>
              <span class="speed-unit-mobile">Km/h</span>
            </div>
          </div>
        </div>
      </div>

      <div class="pit-info" v-if="isPlayerCar">
        <div class="pit-info-row">
          <span class="pit-info-label">Tire:</span>
          <span class="pit-info-value">
            <img
              v-if="selectedTireForPit"
              :src="getTireImagePath(selectedTireForPit)"
              :alt="selectedTireForPit"
              class="pit-tire-icon"
            />
            {{ selectedTireForPit ? getTireDisplayName(selectedTireForPit) : 'None' }}
          </span>
        </div>
        <div class="pit-info-row">
          <span class="pit-info-label">Refuel:</span>
          <span class="pit-info-value">{{ targetRefuelAmount }}%</span>
        </div>
      </div>
      <button
        class="pit-btn"
        :class="{ 'cancel-btn': hasPitRequest }"
        @click="hasPitRequest ? cancelPitStop() : executePitStop()"
        :disabled="!isPlayerCar || car.status === 'Pit'"
      >
        {{ hasPitRequest ? 'CANCEL' : 'PIT' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import type { Car } from '@/types';
import { apiRequest } from '@/services/ApiService';
import { getPlayerId } from '@/services/ApiService';

const RACE_ID = 1;

const props = defineProps<{
  car: Car;
}>();

const showTireSelector = ref(false);
const selectedTireForPit = ref<string | null>(null);
const driverAvatarUrl = ref<string | null>(null);
const tireSelectorPosition = ref({ top: 0, left: 0 });
const tireIconRef = ref<HTMLElement | null>(null);
const targetRefuelAmount = ref(100);
const hoverRefuelAmount = ref(100);
const fuelGaugeRef = ref<HTMLElement | null>(null);
const isHoveringGauge = ref(false);

const tireTypes = ['Soft', 'Medium', 'Hard', 'Intermediate', 'Wet'];

// Get current player ID
const currentPlayerId = computed(() => getPlayerId());

// Check if this is the player's car
const isPlayerCar = computed(() => {
  if (!currentPlayerId.value) return false;
  return props.car.player_uuid === currentPlayerId.value;
});

// Check if car has a pit request
const hasPitRequest = computed(() => {
  return props.car.pit_requested === true;
});

// Fetch driver avatar URL
async function fetchDriverAvatar() {
  if (!props.car.driver.uid) {
    return;
  }

  try {
    const response = await apiRequest(`/drivers/${props.car.driver.uid}`, {
      method: 'GET',
    });

    if (response.ok) {
      const data = await response.json();
      if (data.status === 'success' && data.data?.avatar_url) {
        driverAvatarUrl.value = data.data.avatar_url;
      }
    }
  } catch (error) {
    console.error('Failed to fetch driver avatar:', error);
  }
}

// Fetch avatar when component mounts or driver changes
onMounted(() => {
  fetchDriverAvatar();
});

watch(
  () => props.car.driver.uid,
  () => {
    fetchDriverAvatar();
  },
);

// Track previous status to detect when pit is completed
const previousStatus = ref<string>(props.car.status);

// Clear selected tire when pit stop is actually completed (status changes from 'Pit' to something else)
watch(
  () => props.car.status,
  (newStatus) => {
    // If status changes from 'Pit' to something else, the pit was completed
    if (previousStatus.value === 'Pit' && newStatus !== 'Pit') {
      selectedTireForPit.value = null;
    }
    previousStatus.value = newStatus;
  },
);

function getInitials(name: string): string {
  const parts = name.split(' ');
  if (parts.length >= 2) {
    return `${parts[0][0]}${parts[parts.length - 1][0]}`.toUpperCase();
  }
  return name.substring(0, 2).toUpperCase();
}

function getTireImagePath(tireType: string): string {
  const normalized = tireType.toLowerCase();
  // Handle "intermediate" -> "inter" for image path
  if (normalized === 'intermediate') {
    return '/assets/tires/inter.svg';
  }
  return `/assets/tires/${normalized}.svg`;
}

function getTireDisplayName(tireType: string): string {
  // Show "Inter" instead of "Intermediate" for better UX
  if (tireType === 'Intermediate') {
    return 'Inter';
  }
  return tireType;
}

function getFuelColorClass(fuel: number): string {
  if (fuel > 50) return 'fuel-high';
  if (fuel > 25) return 'fuel-medium';
  return 'fuel-low';
}

function getStressColorClass(stress: number): string {
  if (stress > 0.7) return 'stress-high';
  if (stress > 0.4) return 'stress-medium';
  return 'stress-low';
}

// Calculate speed indicator position on the gauge (0-400 km/h mapped to semicircle)
// Semicircle goes from left (0 km/h, angle π) to right (400 km/h, angle 0)
function getSpeedIndicatorX(): number {
  const speed = Math.max(0, Math.min(400, props.car.speed));
  const angle = Math.PI - (speed / 400) * Math.PI; // π (left) to 0 (right)
  const centerX = 100;
  const radius = 80;
  return centerX + radius * Math.cos(angle);
}

function getSpeedIndicatorY(): number {
  const speed = Math.max(0, Math.min(400, props.car.speed));
  const angle = Math.PI - (speed / 400) * Math.PI; // π (left) to 0 (right)
  const centerY = 100;
  const radius = 80;
  return centerY - radius * Math.sin(angle);
}

async function cycleDrivingStyle() {
  if (!isPlayerCar.value) return;

  const styles = ['Relax', 'Normal', 'Aggressive'];
  const currentIndex = styles.indexOf(props.car.driving_style);
  const nextIndex = (currentIndex + 1) % styles.length;
  const nextStyle = styles[nextIndex];

  await apiRequest(`/race/${RACE_ID}/car/${props.car.car_number}/driving-style`, {
    method: 'PUT',
    body: JSON.stringify({ style: nextStyle }),
  });
}

function toggleTireSelector() {
  if (!isPlayerCar.value) return;

  if (!showTireSelector.value && tireIconRef.value) {
    // Calculate position when opening
    const rect = tireIconRef.value.getBoundingClientRect();
    tireSelectorPosition.value = {
      top: rect.bottom + 4,
      left: rect.left,
    };
  }

  showTireSelector.value = !showTireSelector.value;
}

function selectTireForPit(tireType: string | null) {
  selectedTireForPit.value = tireType;
  showTireSelector.value = false;
}

function setRefuelAmount(event: MouseEvent) {
  if (!isPlayerCar.value || !fuelGaugeRef.value) return;

  const rect = fuelGaugeRef.value.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = Math.max(0, Math.min(100, (x / rect.width) * 100));
  targetRefuelAmount.value = Math.round(percentage);
}

function updateRefuelAmountOnHover(event: MouseEvent) {
  if (!isPlayerCar.value || !fuelGaugeRef.value || !isHoveringGauge.value) return;

  const rect = fuelGaugeRef.value.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const percentage = Math.max(0, Math.min(100, (x / rect.width) * 100));
  hoverRefuelAmount.value = Math.round(percentage);
}

async function executePitStop() {
  if (!isPlayerCar.value || props.car.status === 'Pit') return;

  const tireType = selectedTireForPit.value ?? null;

  await apiRequest(`/race/${RACE_ID}/car/${props.car.car_number}/pit`, {
    method: 'POST',
    body: JSON.stringify({
      tires: tireType,
      refuel: targetRefuelAmount.value,
    }),
  });

  // Don't clear selectedTireForPit here - keep it until pit actually happens
}

async function cancelPitStop() {
  if (!isPlayerCar.value || props.car.status === 'Pit') return;

  const response = await apiRequest(`/race/${RACE_ID}/car/${props.car.car_number}/pit`, {
    method: 'POST',
    body: JSON.stringify({
      cancel: true,
    }),
  });

  if (response.ok) {
    // Pit request cancelled
  }
}

// Close tire selector when clicking outside
watch(showTireSelector, (isOpen) => {
  if (isOpen) {
    const closeHandler = (e: MouseEvent) => {
      if (!(e.target as HTMLElement).closest('.tire-section')) {
        showTireSelector.value = false;
        document.removeEventListener('click', closeHandler);
      }
    };
    setTimeout(() => {
      document.addEventListener('click', closeHandler);
    }, 0);
  }
});
</script>

<style scoped>
.driver-control {
  background-color: #f9f7f7;
  border: 1px solid #c9d6df;
  border-radius: 4px;
  padding: 10px;
  margin-bottom: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  position: relative;
  overflow: visible;
}

.driver-control.player-car {
  border-color: #2d4059;
  border-width: 2px;
}

.control-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.control-row.second-row {
  margin-bottom: 0;
}

@media (max-width: 768px) {
  .control-row.second-row {
    gap: 0.5rem;
  }

  .tire-section {
    flex: 0 0 auto;
  }
}

.driver-avatar-section {
  position: relative;
  flex-shrink: 0;
}

.driver-avatar-container {
  position: relative;
  width: 64px;
  height: 64px;
  border: 2px solid #c9d6df;
  border-radius: 4px;
  background-color: #f9f7f7;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.stress-level-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  transition:
    height 0.3s ease,
    background-color 0.3s ease;
  pointer-events: none;
}

.stress-level-overlay.stress-low {
  background-color: rgba(123, 199, 77, 0.3); /* Green for low stress */
}

.stress-level-overlay.stress-medium {
  background-color: rgba(249, 168, 38, 0.3); /* Orange for medium stress */
}

.stress-level-overlay.stress-high {
  background-color: rgba(232, 69, 69, 0.3); /* Red for high stress */
}

.driver-avatar {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: 50%;
  position: relative;
  z-index: 1;
}

.driver-avatar-placeholder {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  color: #2d4059;
  font-size: 0.9em;
  border-radius: 50%;
  background-color: #dbe2ef;
  position: relative;
  z-index: 1;
}

.fuel-gauge-container {
  flex: 1;
  min-width: 0;
}

.fuel-gauge-label {
  font-size: 0.75em;
  color: #2d4059;
  margin-bottom: 4px;
  font-weight: bold;
}

.fuel-gauge {
  position: relative;
  width: 100%;
  height: 24px;
  background-color: #dbe2ef;
  border-radius: 12px;
  overflow: visible;
  border: 1px solid #c9d6df;
  cursor: pointer;
}

.fuel-gauge-fill {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  transition:
    width 0.3s ease,
    background-color 0.3s ease;
  border-radius: 12px;
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

.fuel-gauge-text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 0.75em;
  font-weight: bold;
  color: #2d4059;
  z-index: 1;
  text-shadow: 0 0 2px rgba(255, 255, 255, 0.8);
  pointer-events: none;
}

.fuel-gauge-cursor {
  position: absolute;
  top: -2px;
  width: 2px;
  height: 28px;
  background-color: #2d4059;
  transform: translateX(-50%);
  z-index: 3;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.2s;
}

.fuel-gauge-cursor.visible {
  opacity: 1;
}

.fuel-gauge-cursor::before {
  content: '';
  position: absolute;
  top: -4px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 6px solid #2d4059;
}

.driving-style-btn {
  padding: 8px 16px;
  border: 1px solid #c9d6df;
  background-color: #dbe2ef;
  border-radius: 4px;
  font-size: 0.9em;
  font-weight: bold;
  cursor: pointer;
  color: #2d4059;
  transition: background-color 0.2s;
  white-space: nowrap;
  min-width: 90px;
}

.driving-style-btn:hover:not(:disabled) {
  background-color: #c9d6df;
}

.driving-style-btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tire-section {
  position: relative;
  flex: 1;
  overflow: visible;
}

@media (max-width: 768px) {
  .tire-section {
    flex: 0 0 auto;
  }
}

.tire-icon-container {
  position: relative;
  width: 64px;
  height: 64px;
  cursor: pointer;
  border: 2px solid #c9d6df;
  border-radius: 4px;
  background-color: #f9f7f7;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  transition: border-color 0.2s;
}

.tire-icon-container:hover:not(.disabled) {
  border-color: #2d4059;
}

.tire-icon-container.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tire-icon-container.tire-selected {
  border-color: #2d4059;
  border-width: 3px;
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
  width: 48px;
  height: 48px;
  position: relative;
  z-index: 1;
}

.tire-selector {
  position: fixed;
  background-color: #f9f7f7;
  border: 1px solid #c9d6df;
  border-radius: 4px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  z-index: 2000;
  display: flex;
  flex-direction: row;
  gap: 4px;
  padding: 4px;
  flex-wrap: nowrap;
  white-space: nowrap;
  width: max-content;
}

.tire-option-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 6px;
  border: 1px solid #c9d6df;
  background-color: #f9f7f7;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
  text-align: center;
  min-width: 50px;
  flex-shrink: 0;
}

.tire-option-btn:hover {
  background-color: #dbe2ef;
}

.tire-option-btn.active {
  background-color: #dbe2ef;
  border-color: #2d4059;
  border-width: 2px;
}

.tire-option-icon {
  width: 24px;
  height: 24px;
}

.pit-btn {
  padding: 12px 24px;
  border: 1px solid #c9d6df;
  background-color: #2d4059;
  color: white;
  border-radius: 4px;
  font-size: 1em;
  font-weight: bold;
  cursor: pointer;
  transition: background-color 0.2s;
  white-space: nowrap;
}

.pit-btn:hover:not(:disabled) {
  background-color: #1a2d3d;
}

.pit-btn.cancel-btn {
  background-color: #e84545;
  padding: 12px 4px;
}

.pit-btn.cancel-btn:hover:not(:disabled) {
  background-color: #d32f2f;
}

.pit-btn:disabled {
  background-color: #a5b1c2;
  cursor: not-allowed;
  opacity: 0.6;
}

.pit-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-right: 8px;
  padding: 4px 8px;
  background-color: #dbe2ef;
  border-radius: 4px;
  border: 1px solid #c9d6df;
  font-size: 0.7em;
  color: #2d4059;
  min-width: 80px;
}

.pit-info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  line-height: 1.2;
}

.pit-info-label {
  font-weight: bold;
  color: #2d4059;
}

.pit-info-value {
  display: flex;
  align-items: center;
  gap: 3px;
  color: #141c27;
}

.pit-tire-icon {
  width: 14px;
  height: 14px;
}

.speed-gauge-container {
  position: relative;
  width: 80px;
  height: 40px;
  flex-shrink: 0;
}

.speed-gauge {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.speed-gauge-desktop {
  display: flex;
}

.speed-gauge-mobile {
  display: none;
}

.speed-gauge-svg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

.speed-indicator {
  transition:
    cx 0.3s ease,
    cy 0.3s ease;
}

.speed-value {
  position: relative;
  z-index: 1;
  text-align: center;
  pointer-events: none;
  margin-top: 20px;
}

.speed-number {
  font-size: 0.9em;
  font-weight: bold;
  color: #2d4059;
  line-height: 1;
}

.speed-unit {
  font-size: 0.5em;
  color: #2d4059;
  margin-top: 1px;
}

/* Mobile Responsive Styles */
@media (max-width: 768px) {
  .pit-info {
    display: none;
  }

  .speed-gauge-container {
    width: 100%;
    height: auto;
    flex: 1;
    min-width: 0;
  }

  .speed-gauge-desktop {
    display: none;
  }

  .speed-gauge-mobile {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 0.75rem;
  }

  .speed-line-container {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-width: 0;
    width: 100%;
    margin-left: 0.5rem;
  }

  .speed-line-track {
    flex: 1;
    height: 12px;
    background-color: #dbe2ef;
    border-radius: 6px;
    overflow: hidden;
    position: relative;
    min-width: 0;
  }

  .speed-line-fill {
    height: 100%;
    background: linear-gradient(to right, #e84545 0%, #f9a826 50%, #7bc74d 100%);
    border-radius: 6px;
    transition: width 0.3s ease;
  }

  .speed-value-mobile {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .speed-number-mobile {
    font-size: 0.9em;
    font-weight: bold;
    color: #2d4059;
    line-height: 1;
  }

  .speed-unit-mobile {
    font-size: 0.65em;
    color: #2d4059;
    line-height: 1;
  }
}
</style>
