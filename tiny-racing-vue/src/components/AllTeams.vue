<template>
  <div class="all-teams-container">
    <div class="all-teams-content">
      <h2>All Teams</h2>

      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading teams...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Teams table -->
      <div v-if="!loading && !error" class="teams-table-container">
        <table class="teams-table">
          <thead>
            <tr>
              <th>Number</th>
              <th>Logo</th>
              <th>Team</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="team in teams"
              :key="team.id"
              class="team-row"
              :class="{ expanded: expandedTeamId === team.id }"
              @click="toggleTeam(team.id)"
            >
              <td class="team-number">#{{ team.number }}</td>
              <td class="team-logo-cell">
                <img v-if="team.logo" :src="team.logo" :alt="team.name" class="team-logo-small" />
                <span v-else class="no-logo">—</span>
              </td>
              <td class="team-name-cell">
                <div class="team-name-wrapper">
                  <div class="team-color-indicator" :style="{ backgroundColor: team.color }"></div>
                  <span class="team-name">{{ team.name }}</span>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Expanded team details (separate div below table) -->
      <div v-if="expandedTeamId" class="expanded-team-container">
        <div v-if="loadingExpandedData" class="loading-small">Loading team details...</div>
        <div v-else-if="errorExpandedData" class="error-small">{{ errorExpandedData }}</div>
        <div v-else-if="expandedTeamData" class="expanded-team-view">
          <div class="team-layout">
            <div class="team-metadata-column">
              <div class="team-card">
                <div class="team-header">
                  <div
                    class="team-color"
                    :style="{ backgroundColor: expandedTeamData.team.color }"
                  ></div>
                  <div class="team-info">
                    <h3>#{{ expandedTeamData.team.number }} - {{ expandedTeamData.team.name }}</h3>
                    <p class="team-details">
                      <span
                        >Pit Efficiency:
                        {{ (expandedTeamData.team.pit_efficiency * 100).toFixed(1) }}%</span
                      >
                      <span style="margin-left: 1rem"
                        >Cash: ${{ expandedTeamData.team.cash.toLocaleString() }}</span
                      >
                    </p>
                  </div>
                </div>
                <div v-if="expandedTeamData.team.logo" class="team-logo">
                  <img :src="expandedTeamData.team.logo" :alt="expandedTeamData.team.name" />
                </div>
              </div>

              <!-- Upcoming Races Section -->
              <div class="upcoming-races-section">
                <h4 class="section-title-small">Upcoming Races</h4>
                <div v-if="upcomingRacesForExpandedTeam.length === 0" class="no-races">
                  <p>No upcoming races</p>
                </div>
                <div v-else class="races-list">
                  <div
                    v-for="raceReg in upcomingRacesForExpandedTeam"
                    :key="raceReg.registration_id"
                    class="race-card-small"
                  >
                    <div class="race-card-line-1">
                      <span class="race-track-name">{{ raceReg.track_name }}</span>
                      <span class="race-laps">{{ raceReg.laps }} laps</span>
                    </div>
                    <div class="race-card-line-2">
                      <span class="race-date">{{ formatRaceDate(raceReg.start_datetime) }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Lineup Section -->
            <div class="lineup-column">
              <div class="lineup-section">
                <!-- Active Section: Cars with Drivers -->
                <div class="active-section">
                  <h3 class="section-title">Lineup</h3>
                  <div class="cars-grid">
                    <div
                      v-for="car in sortedCars"
                      :key="car.id"
                      class="car-slot"
                      :class="{ 'has-driver': getDriverForCar(car) }"
                    >
                      <div class="car-header">
                        <h5>Car #{{ car.number }}</h5>
                      </div>
                      <div class="car-stats">
                        <div class="stat-row">
                          <span class="stat-label">Handling:</span>
                          <span class="stat-value">{{ car.handling.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Acceleration:</span>
                          <span class="stat-value">{{ car.acceleration.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Top Speed:</span>
                          <span class="stat-value">{{ car.top_speed.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Reliability:</span>
                          <span class="stat-value">{{ car.reliability.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Fuel Consumption:</span>
                          <span class="stat-value">{{ car.fuel_consumption.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Tire Wear:</span>
                          <span class="stat-value">{{ car.tire_wear.toFixed(1) }}</span>
                        </div>
                      </div>
                      <div v-if="getDriverForCar(car)" class="driver-assigned">
                        <div class="driver-header">
                          <img
                            v-if="getDriverForCar(car)?.avatar_url"
                            :src="getDriverForCar(car)?.avatar_url"
                            :alt="`${getDriverForCar(car)?.first_name} ${getDriverForCar(car)?.last_name} avatar`"
                            class="driver-avatar"
                          />
                          <div class="driver-name">
                            <h4>
                              {{ getDriverForCar(car)?.first_name }}
                              {{ getDriverForCar(car)?.last_name }}
                            </h4>
                            <span class="driver-nationality">{{
                              getDriverForCar(car)?.nationality
                            }}</span>
                          </div>
                        </div>
                        <div class="driver-stats">
                          <div class="stat-row">
                            <span class="stat-label">Skill:</span>
                            <span class="stat-value">{{
                              getDriverForCar(car)?.skill_level.toFixed(1)
                            }}</span>
                          </div>
                          <div class="stat-row">
                            <span class="stat-label">Stamina:</span>
                            <span class="stat-value">{{
                              getDriverForCar(car)?.stamina.toFixed(1)
                            }}</span>
                          </div>
                          <div class="stat-row">
                            <span class="stat-label">Experience:</span>
                            <span class="stat-value">{{
                              getDriverForCar(car)?.experience.toFixed(1)
                            }}</span>
                          </div>
                          <div class="stat-row">
                            <span class="stat-label">Consistency:</span>
                            <span class="stat-value">{{
                              getDriverForCar(car)?.consistency.toFixed(1)
                            }}</span>
                          </div>
                          <div class="stat-row">
                            <span class="stat-label">Focus:</span>
                            <span class="stat-value">{{
                              getDriverForCar(car)?.focus.toFixed(1)
                            }}</span>
                          </div>
                          <div class="stat-row">
                            <span class="stat-label">Weather:</span>
                            <span class="stat-value">{{
                              getDriverForCar(car)?.weather_tolerance.toFixed(1)
                            }}</span>
                          </div>
                        </div>
                      </div>
                      <div v-else class="empty-slot">
                        <p>No driver assigned</p>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Substitute Drivers Section -->
                <div v-if="substituteDrivers.length > 0" class="substitute-section">
                  <h4 class="section-title">Subs</h4>
                  <div class="subs-grid">
                    <div
                      v-for="subDriver in substituteDrivers"
                      :key="subDriver.id"
                      class="sub-driver-card"
                    >
                      <div class="driver-header">
                        <img
                          v-if="subDriver.avatar_url"
                          :src="subDriver.avatar_url"
                          :alt="`${subDriver.first_name} ${subDriver.last_name} avatar`"
                          class="driver-avatar"
                        />
                        <div class="driver-name">
                          <h4>{{ subDriver.first_name }} {{ subDriver.last_name }}</h4>
                          <span class="driver-nationality">{{ subDriver.nationality }}</span>
                        </div>
                      </div>
                      <div class="driver-stats">
                        <div class="stat-row">
                          <span class="stat-label">Skill:</span>
                          <span class="stat-value">{{ subDriver.skill_level.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Stamina:</span>
                          <span class="stat-value">{{ subDriver.stamina.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Experience:</span>
                          <span class="stat-value">{{ subDriver.experience.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Consistency:</span>
                          <span class="stat-value">{{ subDriver.consistency.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Focus:</span>
                          <span class="stat-value">{{ subDriver.focus.toFixed(1) }}</span>
                        </div>
                        <div class="stat-row">
                          <span class="stat-label">Weather:</span>
                          <span class="stat-value">{{
                            subDriver.weather_tolerance.toFixed(1)
                          }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import {
  getTeams,
  getTeamDrivers,
  getTeamCars,
  getTeamRegistrations,
  type TeamDb,
  type DriverDb,
  type CarDb,
  type RegistrationWithRaceDetails,
} from '@/services/ApiService';

interface ExpandedTeamData {
  team: TeamDb;
  drivers: DriverDb[];
  cars: CarDb[];
  registeredRaces: RegistrationWithRaceDetails[];
}

const teams = ref<TeamDb[]>([]);
const loading = ref(true);
const error = ref('');
const expandedTeamId = ref<string | null>(null);
const expandedTeamData = ref<ExpandedTeamData | null>(null);
const loadingExpandedData = ref(false);
const errorExpandedData = ref('');

// Sort cars by number
const sortedCars = computed(() => {
  if (!expandedTeamData.value) return [];
  return [...expandedTeamData.value.cars].sort((a, b) => a.number - b.number);
});

// Get substitute drivers (those without car_id)
const substituteDrivers = computed(() => {
  if (!expandedTeamData.value) return [];
  return expandedTeamData.value.drivers.filter((driver) => driver.car_id === null);
});

// Helper function to get driver for a car
function getDriverForCar(car: CarDb): DriverDb | undefined {
  if (!expandedTeamData.value) return undefined;
  return expandedTeamData.value.drivers.find((driver) => driver.car_id === car.id);
}

// Filter races to only show non-started races (REGISTRATION_OPEN or REGISTRATION_CLOSED)
const upcomingRacesForExpandedTeam = computed(() => {
  if (!expandedTeamData.value) return [];
  return expandedTeamData.value.registeredRaces.filter(
    (race) =>
      race.race_status === 'REGISTRATION_OPEN' || race.race_status === 'REGISTRATION_CLOSED',
  );
});

async function loadTeams() {
  loading.value = true;
  error.value = '';

  try {
    teams.value = await getTeams();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load teams';
    console.error('Error loading teams:', err);
  } finally {
    loading.value = false;
  }
}

async function toggleTeam(teamId: string) {
  if (expandedTeamId.value === teamId) {
    // Collapse if already expanded
    expandedTeamId.value = null;
    expandedTeamData.value = null;
    errorExpandedData.value = '';
  } else {
    // Expand new team
    expandedTeamId.value = teamId;
    await loadTeamDetails(teamId);
  }
}

async function loadTeamDetails(teamId: string) {
  loadingExpandedData.value = true;
  errorExpandedData.value = '';

  try {
    const team = teams.value.find((t) => t.id === teamId);
    if (!team) {
      throw new Error('Team not found');
    }

    const [drivers, cars, registeredRaces] = await Promise.all([
      getTeamDrivers(teamId),
      getTeamCars(teamId),
      getTeamRegistrations(teamId).catch(() => [] as RegistrationWithRaceDetails[]), // Handle errors gracefully
    ]);

    expandedTeamData.value = {
      team,
      drivers,
      cars,
      registeredRaces,
    };
  } catch (err) {
    errorExpandedData.value = err instanceof Error ? err.message : 'Failed to load team details';
    console.error('Error loading team details:', err);
  } finally {
    loadingExpandedData.value = false;
  }
}

function formatRaceDate(startDatetime: string | null): string {
  if (!startDatetime) return 'Date TBD';
  const date = new Date(startDatetime);
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });
}

onMounted(() => {
  loadTeams();
});
</script>

<style scoped>
.all-teams-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.all-teams-content {
  width: 100%;
}

h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 2rem;
  text-align: center;
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

.teams-table-container {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.teams-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

.teams-table thead {
  background-color: #2d4059;
  color: white;
}

.teams-table th {
  padding: 1rem;
  text-align: left;
  font-weight: 600;
  font-size: 0.95rem;
}

.teams-table th:nth-child(1) {
  width: 100px;
}

.teams-table th:nth-child(2) {
  width: 100px;
}

.teams-table th:nth-child(3) {
  width: auto;
}

.teams-table tbody tr {
  cursor: pointer;
  transition: background-color 0.2s;
  border-bottom: 1px solid #e0e0e0;
  height: 80px;
}

.teams-table tbody tr:hover {
  background-color: #f5f5f5;
}

.teams-table tbody tr.expanded {
  background-color: #e8f5e9;
}

.teams-table td {
  padding: 1rem;
  vertical-align: middle;
  height: 80px;
  box-sizing: border-box;
}

.team-number {
  font-weight: 600;
  color: #2d4059;
  width: 100px;
  max-width: 100px;
  text-align: center;
}

.team-name-cell {
  flex: 1;
  text-align: left;
  padding-left: 1.5rem !important;
}

.team-name-wrapper {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  justify-content: flex-start;
  width: 100%;
}

.team-color-indicator {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  border: 2px solid #ddd;
  flex-shrink: 0;
}

.team-name {
  font-weight: 500;
  color: #1a1a2e;
}

.team-logo-cell {
  width: 100px;
  max-width: 100px;
  text-align: center;
  padding: 1rem 1rem !important;
}

.team-logo-small {
  max-width: 100%;
  max-height: 60px;
  width: auto;
  height: auto;
  object-fit: contain;
  display: block;
  margin: 0 auto;
}

.no-logo {
  color: #999;
}

.expanded-team-container {
  margin-top: 2rem;
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.expanded-team-view {
  padding: 2rem;
}

.loading-small,
.error-small {
  padding: 2rem;
  text-align: center;
}

.loading-small {
  color: #666;
}

.error-small {
  color: #d32f2f;
}

/* Team layout styles (reused from Team.vue) */
.team-layout {
  display: grid;
  grid-template-columns: 1fr 3fr;
  gap: 2rem;
  align-items: start;
}

.team-metadata-column {
  width: 100%;
}

.team-card {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  width: 100%;
}

.team-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.team-color {
  width: 40px;
  height: 40px;
  border-radius: 4px;
  flex-shrink: 0;
}

.team-info {
  flex: 1;
}

.team-info h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.2rem;
  color: #1a1a2e;
}

.team-details {
  color: #666;
  font-size: 0.9rem;
  margin: 0;
}

.team-logo {
  margin-top: 1rem;
  text-align: center;
}

.team-logo img {
  max-width: 100%;
  max-height: 100px;
  object-fit: contain;
}

.upcoming-races-section {
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 1px solid #eee;
}

.section-title-small {
  color: #2d4059;
  font-size: 1.1rem;
  margin-bottom: 1rem;
  margin-top: 0;
}

.no-races {
  text-align: center;
  color: #999;
  font-size: 0.9rem;
  padding: 1rem;
}

.races-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.race-card-small {
  background-color: #f8f9fa;
  border-radius: 6px;
  padding: 0.75rem;
  border: 1px solid #e0e0e0;
}

.race-card-line-1 {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.race-track-name {
  font-weight: 500;
  color: #1a1a2e;
  font-size: 0.95rem;
}

.race-laps {
  color: #666;
  font-size: 0.85rem;
}

.race-card-line-2 {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.race-date {
  color: #666;
  font-size: 0.85rem;
}

.lineup-column {
  width: 100%;
}

.lineup-section {
  width: 100%;
}

.section-title {
  color: #2d4059;
  font-size: 1.3rem;
  margin-bottom: 1rem;
  margin-top: 2rem;
}

.section-title:first-of-type {
  margin-top: 0;
}

.cars-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 1.5rem;
  margin-bottom: 1rem;
}

.car-slot {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  border: 3px solid #e0e0e0;
  min-height: 200px;
}

.car-slot.has-driver {
  border-color: #4caf50;
}

.car-header h5 {
  margin: 0 0 1rem 0;
  color: #2d4059;
  font-size: 1.2rem;
}

.car-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  padding: 0.4rem 0;
  border-bottom: 1px solid #f0f0f0;
}

.stat-label {
  color: #666;
  font-size: 0.9rem;
}

.stat-value {
  color: #1a1a2e;
  font-weight: 500;
  font-size: 0.9rem;
}

.driver-assigned {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 2px solid #4caf50;
}

.driver-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.driver-avatar {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e0e0e0;
}

.driver-name h4 {
  margin: 0 0 0.25rem 0;
  color: #1a1a2e;
  font-size: 1rem;
}

.driver-nationality {
  color: #666;
  font-size: 0.85rem;
}

.driver-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
}

.empty-slot {
  margin-top: 1rem;
  padding: 2rem;
  text-align: center;
  color: #999;
  border: 2px dashed #ddd;
  border-radius: 4px;
}

.substitute-section {
  margin-top: 2rem;
}

.subs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
}

.sub-driver-card {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1rem;
  border-left: 4px solid #ff9800;
}

@media (max-width: 768px) {
  .team-layout {
    grid-template-columns: 1fr;
  }

  .cars-grid,
  .subs-grid {
    grid-template-columns: 1fr;
  }

  .car-stats,
  .driver-stats {
    grid-template-columns: 1fr;
  }

  .teams-table {
    font-size: 0.9rem;
  }

  .teams-table th,
  .teams-table td {
    padding: 0.75rem 0.5rem;
  }

  .expanded-team-view {
    padding: 1rem;
  }
}
</style>
