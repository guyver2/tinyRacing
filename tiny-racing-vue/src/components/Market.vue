<template>
  <div class="market-container">
    <div class="market-content">
      <h2>Market</h2>
      <p class="market-description">Browse available drivers and cars for your team</p>

      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading market...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Market content -->
      <div v-if="!loading && !error" class="market-sections">
        <!-- Team Info -->
        <div v-if="myTeam" class="team-info">
          <h3>Your Team</h3>
          <div class="team-stats">
            <div class="stat-item">
              <span class="stat-label">Cash:</span>
              <span class="stat-value">${{ myTeam.cash }}</span>
            </div>
          </div>
        </div>
        <!-- Drivers Section -->
        <section class="market-section">
          <div class="section-header-with-toggle">
            <h3>Available Drivers</h3>
            <button
              class="collapse-toggle"
              @click="driversSectionCollapsed = !driversSectionCollapsed"
              :aria-expanded="!driversSectionCollapsed"
              aria-label="Toggle drivers section"
            >
              <span class="toggle-icon" :class="{ collapsed: driversSectionCollapsed }">▼</span>
            </button>
          </div>
          <div v-if="!driversSectionCollapsed">
            <div v-if="drivers.length === 0" class="empty-state">
              <p>No drivers available in the market</p>
            </div>
            <div v-else class="items-grid">
              <div
                v-for="driver in drivers"
                :key="driver.id"
                class="market-item driver-card"
                :class="{ expanded: expandedDrivers.has(driver.id) }"
              >
                <div class="item-header clickable" @click="toggleDriver(driver.id)">
                  <div class="header-content">
                    <h4>{{ driver.first_name }} {{ driver.last_name }}</h4>
                    <div class="header-meta">
                      <img
                        v-if="getCountryCode(driver.nationality)"
                        :src="getFlagUrl(getCountryCode(driver.nationality)!)"
                        :alt="driver.nationality"
                        :title="driver.nationality"
                        class="country-flag"
                      />
                      <span v-else class="country-flag-fallback" :title="driver.nationality"
                        >🏁</span
                      >
                      <span class="gender-symbol">{{ getGenderSymbol(driver.gender) }}</span>
                      <span class="average-stat"
                        >Avg: {{ getDriverAverageStat(driver).toFixed(1) }}</span
                      >
                    </div>
                  </div>
                  <img
                    v-if="driver.avatar_url"
                    :src="driver.avatar_url"
                    :alt="`${driver.first_name} ${driver.last_name} avatar`"
                    class="driver-avatar"
                    :class="{ expanded: expandedDrivers.has(driver.id) }"
                  />
                </div>
                <div v-if="expandedDrivers.has(driver.id)" class="item-details">
                  <div class="detail-row">
                    <span class="detail-label">Skill Level:</span>
                    <span class="detail-value">{{ driver.skill_level.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Stamina:</span>
                    <span class="detail-value">{{ driver.stamina.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Experience:</span>
                    <span class="detail-value">{{ driver.experience.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Consistency:</span>
                    <span class="detail-value">{{ driver.consistency.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Focus:</span>
                    <span class="detail-value">{{ driver.focus.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Weather Tolerance:</span>
                    <span class="detail-value">{{ driver.weather_tolerance.toFixed(1) }}</span>
                  </div>
                </div>
                <div v-if="expandedDrivers.has(driver.id)" class="buy-section">
                  <div class="price-info">
                    <span class="price-label">Price:</span>
                    <span class="price-value">${{ getDriverPrice(driver) }}</span>
                  </div>
                  <button
                    class="buy-button"
                    :disabled="!canBuyDriver(driver) || buyingDriverId === driver.id"
                    @click="handleBuyDriver(driver)"
                  >
                    <span v-if="buyingDriverId === driver.id">Processing...</span>
                    <span v-else-if="!myTeam">No Team</span>
                    <span v-else-if="!canBuyDriver(driver)">Insufficient Cash</span>
                    <span v-else>Buy Driver</span>
                  </button>
                </div>
              </div>
            </div>
            <div v-if="drivers.length > 0 && hasMoreDrivers" class="load-more-container">
              <button
                class="load-more-button"
                :disabled="loadingMoreDrivers"
                @click="loadMoreDrivers"
              >
                <span v-if="loadingMoreDrivers">Loading...</span>
                <span v-else>Load More Drivers</span>
              </button>
            </div>
          </div>
        </section>

        <!-- Cars Section -->
        <section class="market-section">
          <div class="section-header-with-toggle">
            <h3>Available Cars</h3>
            <button
              class="collapse-toggle"
              @click="carsSectionCollapsed = !carsSectionCollapsed"
              :aria-expanded="!carsSectionCollapsed"
              aria-label="Toggle cars section"
            >
              <span class="toggle-icon" :class="{ collapsed: carsSectionCollapsed }">▼</span>
            </button>
          </div>
          <div v-if="!carsSectionCollapsed">
            <div v-if="cars.length === 0" class="empty-state">
              <p>No cars available in the market</p>
            </div>
            <div v-else class="items-grid">
              <div
                v-for="car in cars"
                :key="car.id"
                class="market-item car-card"
                :class="{ expanded: expandedCars.has(car.id) }"
              >
                <div class="item-header clickable" @click="toggleCar(car.id)">
                  <div class="header-content">
                    <h4>Car #{{ car.number }}</h4>
                    <div class="header-meta">
                      <span class="item-badge">Unassigned</span>
                      <span class="average-stat">Avg: {{ getCarAverageStat(car).toFixed(1) }}</span>
                    </div>
                  </div>
                  <span class="expand-icon" :class="{ expanded: expandedCars.has(car.id) }">▼</span>
                </div>
                <div v-if="expandedCars.has(car.id)" class="item-details">
                  <div class="detail-row">
                    <span class="detail-label">Handling:</span>
                    <span class="detail-value">{{ car.handling.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Acceleration:</span>
                    <span class="detail-value">{{ car.acceleration.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Top Speed:</span>
                    <span class="detail-value">{{ car.top_speed.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Reliability:</span>
                    <span class="detail-value">{{ car.reliability.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Fuel Consumption:</span>
                    <span class="detail-value">{{ car.fuel_consumption.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Tire Wear:</span>
                    <span class="detail-value">{{ car.tire_wear.toFixed(1) }}</span>
                  </div>
                  <div class="detail-row">
                    <span class="detail-label">Base Performance:</span>
                    <span class="detail-value">{{ car.base_performance.toFixed(1) }}</span>
                  </div>
                </div>
                <div v-if="expandedCars.has(car.id)" class="buy-section">
                  <div class="price-info">
                    <span class="price-label">Price:</span>
                    <span class="price-value">${{ getCarPrice(car) }}</span>
                  </div>
                  <button
                    class="buy-button"
                    :disabled="!canBuyCar(car) || buyingCarId === car.id"
                    @click="handleBuyCar(car)"
                  >
                    <span v-if="buyingCarId === car.id">Processing...</span>
                    <span v-else-if="!myTeam">No Team</span>
                    <span v-else-if="!canBuyCar(car)">Insufficient Cash</span>
                    <span v-else>Buy Car</span>
                  </button>
                </div>
              </div>
            </div>
            <div v-if="cars.length > 0 && hasMoreCars" class="load-more-container">
              <button class="load-more-button" :disabled="loadingMoreCars" @click="loadMoreCars">
                <span v-if="loadingMoreCars">Loading...</span>
                <span v-else>Load More Cars</span>
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  getUnassignedDrivers,
  getUnassignedCars,
  buyDriver,
  buyCar,
  getMyTeam,
  type DriverDb,
  type CarDb,
  type TeamDb,
} from '@/services/ApiService';
import { DEFAULT_PAGE_SIZE } from '@/utils/constants';
import { getCountryCode, getFlagUrl } from '@/utils/countryFlags';

const drivers = ref<DriverDb[]>([]);
const cars = ref<CarDb[]>([]);
const loading = ref(true);
const error = ref('');
const expandedDrivers = ref<Set<string>>(new Set());
const expandedCars = ref<Set<string>>(new Set());
const driversSectionCollapsed = ref(false);
const carsSectionCollapsed = ref(false);
const myTeam = ref<TeamDb | null>(null);
const buyingDriverId = ref<string | null>(null);
const buyingCarId = ref<string | null>(null);

// Pagination state for drivers
const driversLimit = ref(DEFAULT_PAGE_SIZE);
const driversOffset = ref(0);
const hasMoreDrivers = ref(false);
const loadingMoreDrivers = ref(false);

// Pagination state for cars
const carsLimit = ref(DEFAULT_PAGE_SIZE);
const carsOffset = ref(0);
const hasMoreCars = ref(false);
const loadingMoreCars = ref(false);

function toggleDriver(driverId: string) {
  if (expandedDrivers.value.has(driverId)) {
    expandedDrivers.value.delete(driverId);
  } else {
    expandedDrivers.value.add(driverId);
  }
}

function toggleCar(carId: string) {
  if (expandedCars.value.has(carId)) {
    expandedCars.value.delete(carId);
  } else {
    expandedCars.value.add(carId);
  }
}

function getGenderSymbol(gender: string): string {
  switch (gender.toLowerCase()) {
    case 'male':
      return '♂';
    case 'female':
      return '♀';
    case 'non-binary':
      return '⚧';
    default:
      return '?';
  }
}

function getDriverAverageStat(driver: DriverDb): number {
  return (
    (driver.skill_level +
      driver.stamina +
      driver.weather_tolerance +
      driver.experience +
      driver.consistency +
      driver.focus) /
    6
  );
}

function getDriverPrice(driver: DriverDb): number {
  const avgStat = getDriverAverageStat(driver);
  return Math.round(avgStat * 100);
}

function canBuyDriver(driver: DriverDb): boolean {
  if (!myTeam.value) return false;
  const price = getDriverPrice(driver);
  return myTeam.value.cash >= price;
}

async function handleBuyDriver(driver: DriverDb) {
  if (buyingDriverId.value === driver.id) return; // Already processing

  buyingDriverId.value = driver.id;
  error.value = '';

  try {
    const updatedTeam = await buyDriver(driver.id);
    myTeam.value = updatedTeam;
    // Remove purchased driver from list
    drivers.value = drivers.value.filter((d) => d.id !== driver.id);

    // If there are more drivers available, try to load one more to fill the gap
    if (hasMoreDrivers.value) {
      const currentOffset = driversOffset.value;
      try {
        const driversData = await getUnassignedDrivers(1, currentOffset);
        if (driversData.length > 0) {
          drivers.value = [...drivers.value, ...driversData];
          driversOffset.value = currentOffset + driversData.length;
          // We got 1 driver, so if we had more before, we might still have more
          // (we don't know for sure without checking, but it's likely)
        } else {
          hasMoreDrivers.value = false;
        }
      } catch (err) {
        // Silently fail - we already removed the driver
        console.error('Error loading replacement driver:', err);
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to buy driver';
    console.error('Error buying driver:', err);
  } finally {
    buyingDriverId.value = null;
  }
}

function getCarAverageStat(car: CarDb): number {
  return (
    (car.handling +
      car.acceleration +
      car.top_speed +
      car.reliability +
      car.fuel_consumption +
      car.tire_wear +
      car.base_performance) /
    7
  );
}

function getCarPrice(car: CarDb): number {
  const avgStat = getCarAverageStat(car);
  return Math.round(avgStat * 100);
}

function canBuyCar(car: CarDb): boolean {
  if (!myTeam.value) return false;
  const price = getCarPrice(car);
  return myTeam.value.cash >= price;
}

async function handleBuyCar(car: CarDb) {
  if (buyingCarId.value === car.id) return; // Already processing

  buyingCarId.value = car.id;
  error.value = '';

  try {
    const updatedTeam = await buyCar(car.id);
    myTeam.value = updatedTeam;
    // Remove purchased car from list
    cars.value = cars.value.filter((c) => c.id !== car.id);

    // If there are more cars available, try to load one more to fill the gap
    if (hasMoreCars.value) {
      const currentOffset = carsOffset.value;
      try {
        const carsData = await getUnassignedCars(1, currentOffset);
        if (carsData.length > 0) {
          cars.value = [...cars.value, ...carsData];
          carsOffset.value = currentOffset + carsData.length;
          // We got 1 car, so if we had more before, we might still have more
          // (we don't know for sure without checking, but it's likely)
        } else {
          hasMoreCars.value = false;
        }
      } catch (err) {
        // Silently fail - we already removed the car
        console.error('Error loading replacement car:', err);
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to buy car';
    console.error('Error buying car:', err);
  } finally {
    buyingCarId.value = null;
  }
}

async function loadMarket() {
  loading.value = true;
  error.value = '';

  // Reset pagination when loading market fresh
  driversOffset.value = 0;
  hasMoreDrivers.value = false;
  carsOffset.value = 0;
  hasMoreCars.value = false;

  try {
    const [driversData, carsData, teamData] = await Promise.all([
      getUnassignedDrivers(driversLimit.value, driversOffset.value),
      getUnassignedCars(carsLimit.value, carsOffset.value),
      getMyTeam().catch(() => null), // Don't fail if team doesn't exist
    ]);

    drivers.value = driversData;
    cars.value = carsData;
    myTeam.value = teamData;

    // Update offsets to reflect what we've loaded
    driversOffset.value = driversData.length;
    carsOffset.value = carsData.length;

    // If we got a full page, there might be more
    hasMoreDrivers.value = driversData.length === driversLimit.value;
    hasMoreCars.value = carsData.length === carsLimit.value;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load market';
    console.error('Error loading market:', err);
  } finally {
    loading.value = false;
  }
}

async function loadMoreDrivers() {
  if (loadingMoreDrivers.value) return;

  loadingMoreDrivers.value = true;
  error.value = '';

  try {
    const newOffset = driversOffset.value;
    const driversData = await getUnassignedDrivers(driversLimit.value, newOffset);

    // Append new drivers to existing list
    drivers.value = [...drivers.value, ...driversData];
    driversOffset.value = driversOffset.value + driversData.length;

    // If we got fewer than the limit, there are no more drivers
    hasMoreDrivers.value = driversData.length === driversLimit.value;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load more drivers';
    console.error('Error loading more drivers:', err);
  } finally {
    loadingMoreDrivers.value = false;
  }
}

async function loadMoreCars() {
  if (loadingMoreCars.value) return;

  loadingMoreCars.value = true;
  error.value = '';

  try {
    const newOffset = carsOffset.value;
    const carsData = await getUnassignedCars(carsLimit.value, newOffset);

    // Append new cars to existing list
    cars.value = [...cars.value, ...carsData];
    carsOffset.value = carsOffset.value + carsData.length;

    // If we got fewer than the limit, there are no more cars
    hasMoreCars.value = carsData.length === carsLimit.value;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load more cars';
    console.error('Error loading more cars:', err);
  } finally {
    loadingMoreCars.value = false;
  }
}

onMounted(() => {
  loadMarket();
});
</script>

<style scoped>
.market-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.market-content {
  width: 100%;
}

h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 0.5rem;
  text-align: center;
}

.market-description {
  text-align: center;
  color: #666;
  margin-bottom: 2rem;
  font-size: 1.1rem;
}

.loading-message,
.error-message {
  text-align: center;
  padding: 1rem;
  margin-bottom: 1rem;
  border-radius: 4px;
}

.loading-message {
  color: #666;
  background-color: #f5f5f5;
}

.error-message {
  color: #d32f2f;
  background-color: #ffebee;
}

.market-sections {
  display: flex;
  flex-direction: column;
  gap: 3rem;
}

.market-section {
  width: 100%;
}

.market-section h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin: 0;
}

.section-header-with-toggle {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 2px solid #e0e0e0;
}

.collapse-toggle {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background-color 0.2s;
  color: #2d4059;
}

.collapse-toggle:hover {
  background-color: #f5f5f5;
}

.toggle-icon {
  font-size: 1rem;
  transition: transform 0.3s ease;
  display: inline-block;
  line-height: 1;
}

.toggle-icon.collapsed {
  transform: rotate(-90deg);
}

.empty-state {
  text-align: center;
  padding: 2rem;
  color: #666;
  background-color: #f5f5f5;
  border-radius: 8px;
}

.items-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.market-item {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  transition:
    transform 0.2s,
    box-shadow 0.2s;
}

.market-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid #e0e0e0;
}

.item-header.clickable {
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
}

.item-header.clickable:hover {
  background-color: #f5f5f5;
  margin: -0.5rem -0.5rem 0.5rem -0.5rem;
  padding: 0.5rem;
  border-radius: 4px;
}

.header-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.item-header h4 {
  margin: 0;
  color: #1a1a2e;
  font-size: 1.2rem;
}

.header-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.item-badge {
  background-color: #2d4059;
  color: white;
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: 500;
}

.country-flag {
  width: 24px;
  height: 18px;
  object-fit: cover;
  cursor: help;
  display: inline-block;
  transition: transform 0.2s;
  border: 1px solid #e0e0e0;
  border-radius: 2px;
  vertical-align: middle;
}

.country-flag:hover {
  transform: scale(1.2);
  border-color: #2d4059;
}

.country-flag-fallback {
  font-size: 1.5rem;
  cursor: help;
  line-height: 1;
  display: inline-block;
}

.gender-symbol {
  font-size: 1.2rem;
  color: #2d4059;
  font-weight: 600;
}

.average-stat {
  color: #666;
  font-size: 0.9rem;
  font-weight: 500;
}

.driver-avatar {
  /* width: 96px;
  height: 96px; */
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e0e0e0;
  transition: all 0.3s ease;
  margin-left: 0.5rem;
  cursor: pointer;
  flex-shrink: 0;
}

.driver-avatar:hover {
  border-color: #2d4059;
  transform: scale(1.05);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.driver-avatar.expanded {
  border-color: #2d4059;
  box-shadow: 0 0 0 2px rgba(45, 64, 89, 0.2);
}

.item-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
  animation: slideDown 0.3s ease;
}

@keyframes slideDown {
  from {
    opacity: 0;
    max-height: 0;
  }
  to {
    opacity: 1;
    max-height: 500px;
  }
}

.detail-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.25rem 0;
}

.detail-label {
  color: #666;
  font-size: 0.9rem;
}

.detail-value {
  color: #1a1a2e;
  font-weight: 600;
  font-size: 0.95rem;
}

.team-info {
  background-color: #f5f5f5;
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 2rem;
}

.team-info h3 {
  margin-top: 0;
  margin-bottom: 0.75rem;
  color: #2d4059;
  font-size: 1.2rem;
}

.team-stats {
  display: flex;
  gap: 2rem;
  flex-wrap: wrap;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.stat-label {
  color: #666;
  font-size: 0.9rem;
}

.stat-value {
  color: #1a1a2e;
  font-weight: 600;
  font-size: 1rem;
}

.buy-section {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #e0e0e0;
}

.price-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
}

.price-label {
  color: #666;
  font-size: 0.9rem;
}

.price-value {
  color: #2d4059;
  font-weight: 600;
  font-size: 1.1rem;
}

.buy-button {
  width: 100%;
  padding: 0.75rem;
  background-color: #2d4059;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.buy-button:hover:not(:disabled) {
  background-color: #1e2a3a;
}

.buy-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  opacity: 0.7;
}

.load-more-container {
  display: flex;
  justify-content: center;
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 1px solid #e0e0e0;
}

.load-more-button {
  padding: 0.75rem 2rem;
  background-color: #2d4059;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.load-more-button:hover:not(:disabled) {
  background-color: #1e2a3a;
}

.load-more-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  opacity: 0.7;
}

/* Responsive design */
@media (max-width: 768px) {
  .market-container {
    padding: 1rem;
  }

  h2 {
    font-size: 1.5rem;
  }

  .market-description {
    font-size: 1rem;
  }

  .items-grid {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .market-item {
    padding: 1rem;
  }

  .market-section h3 {
    font-size: 1.25rem;
  }

  .section-header-with-toggle {
    margin-bottom: 1rem;
  }

  .collapse-toggle {
    padding: 0.4rem;
  }

  .toggle-icon {
    font-size: 0.9rem;
  }

  .item-header h4 {
    font-size: 1.1rem;
  }

  .team-info {
    padding: 0.75rem;
  }

  .team-info h3 {
    font-size: 1.1rem;
  }

  .team-stats {
    flex-direction: column;
    gap: 0.75rem;
  }

  .buy-button {
    padding: 0.65rem;
    font-size: 0.95rem;
  }

  .load-more-button {
    padding: 0.65rem 1.5rem;
    font-size: 0.95rem;
  }
}

@media (max-width: 480px) {
  .market-container {
    padding: 0.75rem;
  }

  h2 {
    font-size: 1.25rem;
  }

  .market-description {
    font-size: 0.95rem;
  }

  .market-item {
    padding: 0.75rem;
  }

  .item-header h4 {
    font-size: 1rem;
  }

  .header-meta {
    font-size: 0.85rem;
  }

  .detail-label,
  .detail-value {
    font-size: 0.85rem;
  }

  .driver-avatar {
    width: 60px;
    height: 60px;
  }
}
</style>
