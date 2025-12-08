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
          <h3>Available Drivers</h3>
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
                    <span v-else class="country-flag-fallback" :title="driver.nationality">🏁</span>
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

const drivers = ref<DriverDb[]>([]);
const cars = ref<CarDb[]>([]);
const loading = ref(true);
const error = ref('');
const expandedDrivers = ref<Set<string>>(new Set());
const expandedCars = ref<Set<string>>(new Set());
const myTeam = ref<TeamDb | null>(null);
const buyingDriverId = ref<string | null>(null);
const buyingCarId = ref<string | null>(null);

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

function getCountryCodeFromNationality(nationality: string): string | null {
  // Map terms like 'french', 'german', 'dutch', etc. to ISO country codes, case-insensitive
  const nationalityToCountry: Record<string, string> = {
    french: 'fr',
    german: 'de',
    dutch: 'nl',
    british: 'gb',
    english: 'gb',
    spanish: 'es',
    italian: 'it',
    swiss: 'ch',
    belgian: 'be',
    austrian: 'at',
    monégasque: 'mc',
    monegasque: 'mc',
    brazilian: 'br',
    argentine: 'ar',
    mexican: 'mx',
    canadian: 'ca',
    australian: 'au',
    'new zealander': 'nz',
    japanese: 'jp',
    chinese: 'cn',
    korean: 'kr',
    'south korean': 'kr',
    indian: 'in',
    russian: 'ru',
    finnish: 'fi',
    swedish: 'se',
    norwegian: 'no',
    danish: 'dk',
    polish: 'pl',
    czech: 'cz',
    hungarian: 'hu',
    romanian: 'ro',
    turkish: 'tr',
    'south african': 'za',
    portuguese: 'pt',
    greek: 'gr',
    irish: 'ie',
    thai: 'th',
    malaysian: 'my',
    singaporean: 'sg',
    indonesian: 'id',
    american: 'us',
    'us-american': 'us',
    usa: 'us',
    briton: 'gb',
    scottish: 'gb',
    welsh: 'gb',
    'northern irish': 'gb',
  };
  // Normalize input
  const key = nationality.trim().toLowerCase();
  return nationalityToCountry[key] || null;
}

function getCountryCode(countryName: string): string | null {
  // Map country names to ISO 3166-1 alpha-2 country codes
  const countryCodes: Record<string, string> = {
    'United Kingdom': 'gb',
    UK: 'gb',
    'United States': 'us',
    USA: 'us',
    US: 'us',
    France: 'fr',
    Germany: 'de',
    Italy: 'it',
    Spain: 'es',
    Netherlands: 'nl',
    Belgium: 'be',
    Switzerland: 'ch',
    Austria: 'at',
    Monaco: 'mc',
    Brazil: 'br',
    Argentina: 'ar',
    Mexico: 'mx',
    Canada: 'ca',
    Australia: 'au',
    'New Zealand': 'nz',
    Japan: 'jp',
    China: 'cn',
    'South Korea': 'kr',
    Korea: 'kr',
    India: 'in',
    Russia: 'ru',
    Finland: 'fi',
    Sweden: 'se',
    Norway: 'no',
    Denmark: 'dk',
    Poland: 'pl',
    'Czech Republic': 'cz',
    Hungary: 'hu',
    Romania: 'ro',
    Turkey: 'tr',
    'South Africa': 'za',
    Portugal: 'pt',
    Greece: 'gr',
    Ireland: 'ie',
    Thailand: 'th',
    Malaysia: 'my',
    Singapore: 'sg',
    Indonesia: 'id',
    Philippines: 'ph',
    Vietnam: 'vn',
    UAE: 'ae',
    'United Arab Emirates': 'ae',
    'Saudi Arabia': 'sa',
    Qatar: 'qa',
    Bahrain: 'bh',
    Kuwait: 'kw',
    Oman: 'om',
    Israel: 'il',
    Lebanon: 'lb',
    Jordan: 'jo',
    Egypt: 'eg',
    Morocco: 'ma',
    Tunisia: 'tn',
    Algeria: 'dz',
    Nigeria: 'ng',
    Kenya: 'ke',
    Ghana: 'gh',
    Senegal: 'sn',
    'Ivory Coast': 'ci',
    "Côte d'Ivoire": 'ci',
    Cameroon: 'cm',
    Chile: 'cl',
    Colombia: 'co',
    Peru: 'pe',
    Venezuela: 've',
    Ecuador: 'ec',
    Uruguay: 'uy',
    Paraguay: 'py',
    Bolivia: 'bo',
  };

  // Try exact match first
  if (countryCodes[countryName]) {
    return countryCodes[countryName];
  }

  // Try case-insensitive match
  const normalizedName = countryName.trim();
  for (const [key, code] of Object.entries(countryCodes)) {
    if (key.toLowerCase() === normalizedName.toLowerCase()) {
      return code;
    }
  }
  // try with the nationality
  const nationalityCode = getCountryCodeFromNationality(countryName);
  if (nationalityCode) {
    return nationalityCode;
  }

  return null;
}

// Store flag URLs in a reactive map to cache them
const flagUrlCache = new Map<string, string>();

function getFlagUrl(countryCode: string): string {
  const code = countryCode.toLowerCase();

  // Check cache first
  if (flagUrlCache.has(code)) {
    return flagUrlCache.get(code)!;
  }

  // Flags are in public/assets/country-flags/svg/
  // In Vite, public folder files are served from root
  const url = `/assets/country-flags/svg/${code}.svg`;
  flagUrlCache.set(code, url);
  return url;
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
    // Reload market to remove purchased driver
    await loadMarket();
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
    // Reload market to remove purchased car
    await loadMarket();
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

  try {
    const [driversData, carsData, teamData] = await Promise.all([
      getUnassignedDrivers(),
      getUnassignedCars(),
      getMyTeam().catch(() => null), // Don't fail if team doesn't exist
    ]);

    drivers.value = driversData;
    cars.value = carsData;
    myTeam.value = teamData;
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
