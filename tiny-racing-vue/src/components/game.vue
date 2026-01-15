<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import RaceHeader from './RaceHeader.vue';
import CarsTable from './CarsTable.vue';
import TrackVisualizer from './TrackVisualizer.vue';
import TrackSvg from './TrackSvg.vue';
import ConnectionStatus from './ConnectionStatus.vue';
import DriverControl from './DriverControl.vue';
import { useRaceData } from '../services/WebSocketService';
import { getPlayerId, isAuthenticated } from '../services/ApiService';
import type { Car } from '@/types';

const emit = defineEmits<{
  navigate: [view: string];
}>();

const router = useRouter();
const { raceState, connected } = useRaceData();
const isCarTableCollapsed = ref(true);

// Mobile swipe pane state
const currentPane = ref(0); // 0: race view, 1: table view, 2: driver commands
const touchStartX = ref(0);
const touchEndX = ref(0);
const swipeDirection = ref<'left' | 'right' | null>(null);
const panesContainerRef = ref<HTMLElement | null>(null);

// Get current player ID
const currentPlayerId = computed(() => getPlayerId());

// Check if there's no active race
const hasNoRace = computed(() => {
  return (
    !raceState.value.track.id ||
    raceState.value.track.name === 'No race loaded' ||
    raceState.value.cars.length === 0
  );
});

// Filter cars to only show player's cars, ordered by car number
const playerCars = computed(() => {
  if (!isAuthenticated() || !currentPlayerId.value) {
    return [];
  }
  return raceState.value.cars
    .filter((car: Car) => car.player_uuid === currentPlayerId.value)
    .sort((a: Car, b: Car) => a.car_number - b.car_number);
});

function navigateToRaces() {
  router.push({ name: 'races' });
  emit('navigate', 'races');
}

// Mobile swipe handlers
function handleTouchStart(e: TouchEvent) {
  // Don't start swipe if touching an interactive element
  const target = e.target as HTMLElement;

  // Check if target or any parent is an interactive element
  const isInteractive =
    target.tagName === 'BUTTON' ||
    target.tagName === 'INPUT' ||
    target.tagName === 'SELECT' ||
    target.tagName === 'A' ||
    target.closest('button') !== null ||
    target.closest('a') !== null ||
    target.closest('input') !== null ||
    target.closest('select') !== null ||
    target.closest('.tire-selector') !== null ||
    target.closest('.tire-icon-container') !== null ||
    target.closest('.fuel-gauge') !== null ||
    target.closest('.fuel-gauge-container') !== null ||
    target.closest('.driving-style-btn') !== null ||
    target.closest('.pit-btn') !== null ||
    target.closest('.tire-option-btn') !== null ||
    target.closest('.speed-gauge-mobile') !== null ||
    target.closest('.speed-line-track') !== null ||
    target.closest('.speed-line-fill') !== null ||
    target.closest('.speed-value-mobile') !== null ||
    // Check if element has a click handler (common pattern)
    target.onclick !== null ||
    target.getAttribute('role') === 'button';

  if (isInteractive) {
    touchStartX.value = 0; // Explicitly reset to prevent any tracking
    return;
  }

  touchStartX.value = e.touches[0].clientX;
}

function handleTouchMove(e: TouchEvent) {
  // Don't track movement if we didn't start a swipe
  if (touchStartX.value === 0) return;

  // Cancel swipe if touch moves over an interactive element
  const target = document.elementFromPoint(
    e.touches[0].clientX,
    e.touches[0].clientY,
  ) as HTMLElement;

  if (target) {
    const isInteractive =
      target.tagName === 'BUTTON' ||
      target.tagName === 'INPUT' ||
      target.tagName === 'SELECT' ||
      target.tagName === 'A' ||
      target.closest('button') !== null ||
      target.closest('a') !== null ||
      target.closest('input') !== null ||
      target.closest('select') !== null ||
      target.closest('.tire-selector') !== null ||
      target.closest('.tire-icon-container') !== null ||
      target.closest('.fuel-gauge') !== null ||
      target.closest('.fuel-gauge-container') !== null ||
      target.closest('.driving-style-btn') !== null ||
      target.closest('.pit-btn') !== null ||
      target.closest('.tire-option-btn') !== null;

    if (isInteractive) {
      touchStartX.value = 0;
      touchEndX.value = 0;
      return;
    }
  }

  touchEndX.value = e.touches[0].clientX;
}

function handleTouchEnd() {
  // Don't process swipe if we didn't start one
  if (touchStartX.value === 0) return;

  const swipeDistance = touchStartX.value - touchEndX.value;
  const minSwipeDistance = 50; // Minimum distance for a swipe

  if (Math.abs(swipeDistance) > minSwipeDistance) {
    if (swipeDistance > 0) {
      // Swipe left - go to next pane (always use left animation, even when looping)
      swipeDirection.value = 'left';
      currentPane.value = (currentPane.value + 1) % 3;
    } else {
      // Swipe right - go to previous pane (always use right animation, even when looping)
      swipeDirection.value = 'right';
      currentPane.value = (currentPane.value - 1 + 3) % 3;
    }

    // Clear swipe direction after animation completes
    setTimeout(() => {
      swipeDirection.value = null;
    }, 300);
  }

  touchStartX.value = 0;
  touchEndX.value = 0;
}
</script>

<template>
  <div class="app-container">
    <!-- No race state -->
    <div v-if="hasNoRace" class="no-race-container">
      <div class="no-race-message">
        <h2>No Active Race</h2>
        <p>There is currently no active race. Start a race from the races page to begin racing.</p>
        <button @click="navigateToRaces" class="btn-primary">Go to Races</button>
      </div>
      <div class="footer-content">
        <ConnectionStatus :connected="connected" />
      </div>
    </div>

    <!-- Active race state -->
    <template v-else>
      <div class="main-content">
        <div class="header-content">
          <RaceHeader
            :trackName="raceState.track.name"
            :elapsedTime="raceState.track.elapsed_time"
            :raceStatus="raceState.race_status"
            :currentLap="raceState.current_lap"
            :totalLaps="raceState.total_laps"
            :weather="raceState.track.current_weather"
            :wetness="raceState.track.wetness"
          />
        </div>
        <div class="game-content">
          <!-- Desktop layout -->
          <div class="desktop-layout">
            <div class="sidebar" :class="{ expanded: !isCarTableCollapsed }">
              <div class="cars-table-container">
                <CarsTable :cars="raceState.cars" v-model:collapsed="isCarTableCollapsed" />
              </div>
              <div v-if="playerCars.length > 0" class="driver-controls-container">
                <DriverControl v-for="car in playerCars" :key="car.car_number" :car="car" />
              </div>
            </div>

            <div class="track-content">
              <TrackSvg :cars="raceState.cars" :trackId="raceState.track.id" />
            </div>
          </div>

          <!-- Mobile swipeable panes -->
          <div
            ref="panesContainerRef"
            class="mobile-panes-container"
            @touchstart="handleTouchStart"
            @touchmove="handleTouchMove"
            @touchend="handleTouchEnd"
          >
            <Transition :name="swipeDirection === 'right' ? 'slide-right' : 'slide-left'">
              <!-- Pane 0: Race View -->
              <div v-if="currentPane === 0" key="race" class="mobile-pane mobile-pane-race">
                <div class="mobile-race-container">
                  <TrackSvg :cars="raceState.cars" :trackId="raceState.track.id" />
                </div>
              </div>

              <!-- Pane 1: Table View -->
              <div v-else-if="currentPane === 1" key="table" class="mobile-pane mobile-pane-table">
                <div class="mobile-table-container">
                  <CarsTable :cars="raceState.cars" :collapsed="true" />
                </div>
              </div>

              <!-- Pane 2: Driver Commands -->
              <div
                v-else-if="currentPane === 2"
                key="commands"
                class="mobile-pane mobile-pane-commands"
              >
                <div v-if="playerCars.length > 0" class="mobile-driver-controls">
                  <DriverControl v-for="car in playerCars" :key="car.car_number" :car="car" />
                </div>
                <div v-else class="mobile-no-drivers">
                  <p>No driver controls available</p>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </div>

      <div class="footer-content">
        <TrackVisualizer :cars="raceState.cars" />
        <ConnectionStatus :connected="connected" />
      </div>
    </template>
  </div>
</template>

<style scoped>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

.app-container {
  max-width: 100%;
  width: 100%;
  margin: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  overflow-x: hidden;
}

.main-content {
  display: flex;
  flex-direction: column;
  position: relative;
  margin-bottom: 0;
  height: calc(100% - 150px);
  overflow-x: hidden;
}

.game-content {
  display: flex;
  flex: 1;
  position: relative;
  margin-bottom: 0;
  height: 100%;
  width: 100%;
  overflow-x: hidden;
}

.desktop-layout {
  display: flex;
  width: 100%;
  height: 100%;
}

.mobile-panes-container {
  display: none;
}

.sidebar {
  width: 30%;
  max-width: 375px;
  min-width: 225px;
  transition: width 0.3s ease;
  position: absolute;
  background-color: #f9f7f7;
  border-radius: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  overflow-y: auto;
  overflow-x: visible;
  height: 100%;
  z-index: 5;
  left: 0;
  display: flex;
  flex-direction: column;
}

.sidebar.expanded {
  width: 100%;
  min-width: fit-content;
  z-index: 10;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
}

.cars-table-container {
  overflow-y: auto;
  overflow-x: hidden;
  flex-shrink: 0;
}

.driver-controls-container {
  padding: 10px;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
  max-width: 375px;
}

.track-content {
  position: absolute;
  left: min(30%, 375px);
  right: 0;
  height: 100%;
  border-radius: 0;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.footer-content {
  margin-top: 15px;
  margin-bottom: 15px;
  width: 100%;
  height: auto;
  display: flex;
  flex-direction: column;
  padding: 0 1%;
}

.header-content {
  width: 100%;
  height: auto;
  display: flex;
  flex-direction: column;
  margin-top: 15px;
  margin-bottom: 15px;
}

.no-race-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100%;
  padding: 2rem;
}

.no-race-message {
  text-align: center;
  max-width: 600px;
  padding: 3rem;
  background-color: #ffffff;
  border-radius: 12px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.no-race-message h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 1rem;
  font-weight: 600;
}

.no-race-message p {
  color: #666;
  font-size: 1.1rem;
  margin-bottom: 2rem;
  line-height: 1.6;
}

.btn-primary {
  background-color: #5c7aea;
  color: white;
  border: none;
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.3s ease;
  font-weight: 500;
}

.btn-primary:hover {
  background-color: #4a6fd8;
}

.btn-primary:active {
  background-color: #3d5bc4;
}

/* Pastel Tire Colors */
.tire-soft {
  color: #e84545;
} /* Softer Red */
.tire-medium {
  color: #f9a826;
} /* Softer Yellow */
.tire-hard {
  color: #4d4d4d;
} /* Softer White/Gray */
.tire-intermediate {
  color: #7bc74d;
} /* Softer Green */
.tire-wet {
  color: #5c7aea;
} /* Softer Blue */

/* Mobile Responsive Styles */
@media (max-width: 768px) {
  .app-container {
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    height: calc(100% - 200px);
  }

  .game-content {
    flex-direction: column;
    height: 100%;
    position: relative;
    display: flex;
  }

  .desktop-layout {
    display: none;
  }

  .mobile-panes-container {
    display: block;
    width: 100%;
    height: 100%;
    overflow: hidden;
    position: relative;
    touch-action: pan-y;
  }

  .mobile-pane {
    width: 100%;
    height: 100%;
    overflow: hidden;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    flex-direction: column;
  }

  .mobile-pane-race {
    display: flex;
    flex-direction: column;
  }

  .mobile-race-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    min-height: 0;
  }

  .mobile-race-container :deep(.track-container) {
    width: 100%;
    height: 100%;
    min-height: 0;
  }

  .mobile-pane-table {
    display: flex;
    flex-direction: column;
  }

  .mobile-table-container {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    display: flex;
    flex-direction: column;
  }

  .mobile-table-container :deep(.cars-table-wrapper) {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .mobile-table-container :deep(.cars-table) {
    flex: 1;
  }

  .mobile-pane-commands {
    display: flex;
    flex-direction: column;
  }

  .mobile-driver-controls {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0.5rem;
    -webkit-overflow-scrolling: touch;
  }

  .mobile-no-drivers {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    color: #666;
  }

  /* Slide transition animations - swipe left (next pane) */
  .slide-left-enter-active {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 1;
  }

  .slide-left-leave-active {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 0;
  }

  /* When swiping left: new pane enters from right (100%), old pane leaves to left (-100%) */
  .slide-left-enter-from {
    transform: translateX(100%);
  }

  .slide-left-enter-to {
    transform: translateX(0);
  }

  .slide-left-leave-from {
    transform: translateX(0);
  }

  .slide-left-leave-to {
    transform: translateX(-100%);
  }

  /* Slide transition animations - swipe right (previous pane) */
  .slide-right-enter-active {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 1;
  }

  .slide-right-leave-active {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 0;
  }

  /* When swiping right: new pane enters from left (-100%), old pane leaves to right (100%) */
  .slide-right-enter-from {
    transform: translateX(-100%);
  }

  .slide-right-enter-to {
    transform: translateX(0);
  }

  .slide-right-leave-from {
    transform: translateX(0);
  }

  .slide-right-leave-to {
    transform: translateX(100%);
  }

  .sidebar {
    width: 100%;
    max-width: 100%;
    min-width: 100%;
    position: relative;
    height: auto;
    max-height: 40vh;
    order: 2;
    border-radius: 0;
  }

  .sidebar.expanded {
    max-height: 60vh;
  }

  .track-content {
    position: relative;
    left: 0;
    width: 100%;
    height: 60vh;
    min-height: 300px;
    order: 1;
    flex-shrink: 0;
  }

  .driver-controls-container {
    max-width: 100%;
  }

  .footer-content {
    padding: 0.5rem;
    margin-top: 10px;
    margin-bottom: 10px;
  }

  .header-content {
    margin-top: 10px;
    margin-bottom: 10px;
    padding: 0 0.5rem;
  }

  .no-race-container {
    padding: 1rem;
  }

  .no-race-message {
    padding: 2rem 1.5rem;
  }

  .no-race-message h2 {
    font-size: 1.5rem;
  }

  .no-race-message p {
    font-size: 1rem;
  }
}

@media (max-width: 480px) {
  .main-content {
    height: calc(100% - 180px);
  }

  .mobile-pane-indicators {
    padding: 0.4rem;
  }

  .pane-indicator {
    padding: 0.3rem 0.6rem;
    font-size: 0.7rem;
  }

  .header-content {
    margin-top: 5px;
    margin-bottom: 5px;
  }

  .footer-content {
    margin-top: 5px;
    margin-bottom: 5px;
  }
}
</style>
