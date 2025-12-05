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
        <!-- Drivers Section -->
        <section class="market-section">
          <h3>Available Drivers</h3>
          <div v-if="drivers.length === 0" class="empty-state">
            <p>No drivers available in the market</p>
          </div>
          <div v-else class="items-grid">
            <div
              v-for="driver in drivers"
              :key="driver.id"
              class="market-item driver-card"
            >
              <div class="item-header">
                <h4>{{ driver.first_name }} {{ driver.last_name }}</h4>
                <span class="item-badge">{{ driver.nationality }}</span>
              </div>
              <div class="item-details">
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
              <button class="buy-button" disabled>Buy (Coming Soon)</button>
            </div>
          </div>
        </section>

        <!-- Cars Section -->
        <section class="market-section">
          <h3>Available Cars</h3>
          <div v-if="cars.length === 0" class="empty-state">
            <p>No cars available in the market</p>
          </div>
          <div v-else class="items-grid">
            <div
              v-for="car in cars"
              :key="car.id"
              class="market-item car-card"
            >
              <div class="item-header">
                <h4>Car #{{ car.number }}</h4>
                <span class="item-badge">Unassigned</span>
              </div>
              <div class="item-details">
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
              <button class="buy-button" disabled>Buy (Coming Soon)</button>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getUnassignedDrivers, getUnassignedCars, type DriverDb, type CarDb } from '@/services/ApiService';

const drivers = ref<DriverDb[]>([]);
const cars = ref<CarDb[]>([]);
const loading = ref(true);
const error = ref('');

async function loadMarket() {
  loading.value = true;
  error.value = '';
  
  try {
    const [driversData, carsData] = await Promise.all([
      getUnassignedDrivers(),
      getUnassignedCars(),
    ]);
    
    drivers.value = driversData;
    cars.value = carsData;
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load market';
    console.error('Error loading market:', err);
  } finally {
    loading.value = false;
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
  margin-bottom: 1.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 2px solid #e0e0e0;
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
  transition: transform 0.2s, box-shadow 0.2s;
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

.item-header h4 {
  margin: 0;
  color: #1a1a2e;
  font-size: 1.2rem;
}

.item-badge {
  background-color: #2d4059;
  color: white;
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: 500;
}

.item-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
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
  margin-top: 0.5rem;
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

/* Responsive design */
@media (max-width: 768px) {
  .items-grid {
    grid-template-columns: 1fr;
  }
  
  .market-container {
    padding: 1rem;
  }
}
</style>

