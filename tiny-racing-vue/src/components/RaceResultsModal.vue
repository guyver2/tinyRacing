<template>
  <div v-if="visible" class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <button @click="$emit('close')" class="btn-close" type="button" aria-label="Close">
          ×
        </button>
        <div class="modal-header-content">
          <h3>Race Results</h3>
          <div v-if="race" class="race-details">
            <div class="race-detail-item">
              <span class="detail-label">Track:</span>
              <span class="detail-value">{{ trackName || 'N/A' }}</span>
            </div>
            <div class="race-detail-item">
              <span class="detail-label">Laps:</span>
              <span class="detail-value">{{ race.laps }}</span>
            </div>
            <div class="race-detail-item">
              <span class="detail-label">Date:</span>
              <span class="detail-value">{{
                race.start_datetime ? formatDate(race.start_datetime) : 'N/A'
              }}</span>
            </div>
          </div>
        </div>
      </div>
      <div class="modal-body">
        <div v-if="loading" class="loading-message">Loading results...</div>
        <div v-else-if="error" class="error-message">{{ error }}</div>
        <div v-else-if="results.length === 0" class="empty-state">
          <p>No results available for this race.</p>
        </div>
        <div v-else class="results-table-wrapper">
          <table class="results-table">
            <thead>
              <tr>
                <th class="mobile-visible">Position</th>
                <th class="mobile-visible">Driver</th>
                <th class="mobile-visible">Team</th>
                <th class="mobile-visible">Status</th>
                <th class="mobile-hidden">Laps</th>
                <th class="mobile-hidden">Time</th>
                <th class="mobile-hidden">Distance</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="result in results"
                :key="result.id"
                :class="{
                  'result-finished': result.status === 'FINISHED',
                  'result-dnf': result.status === 'DNF',
                }"
              >
                <td
                  class="position-cell mobile-visible"
                  :class="{
                    'position-1': result.final_position === 1,
                    'position-2': result.final_position === 2,
                    'position-3': result.final_position === 3,
                  }"
                >
                  <div class="position-content">
                    <img
                      v-if="result.final_position === 1"
                      src="/assets/awards/laurels_gold.svg"
                      alt="Gold"
                      class="laurel-icon"
                    />
                    <img
                      v-if="result.final_position === 2"
                      src="/assets/awards/laurels_silver.svg"
                      alt="Silver"
                      class="laurel-icon"
                    />
                    <img
                      v-if="result.final_position === 3"
                      src="/assets/awards/laurels_bronze.svg"
                      alt="Bronze"
                      class="laurel-icon"
                    />
                    <span>{{ result.final_position }}</span>
                  </div>
                </td>
                <td class="mobile-visible">
                  <router-link
                    :to="{ name: 'driver', params: { driverId: result.driver_id } }"
                    class="driver-cell-link"
                  >
                    <div class="driver-cell">
                      <img
                        v-if="getDriverAvatar(result.driver_id)"
                        :src="getDriverAvatar(result.driver_id) || ''"
                        :alt="getDriverName(result.driver_id)"
                        class="driver-avatar"
                      />
                      <span>{{ getDriverName(result.driver_id) }}</span>
                    </div>
                  </router-link>
                </td>
                <td class="mobile-visible">
                  <router-link
                    :to="{ name: 'team', params: { teamId: result.team_id } }"
                    class="team-cell-link"
                  >
                    <div class="team-cell">
                      <img
                        v-if="getTeamLogo(result.team_id)"
                        :src="getTeamLogo(result.team_id) || ''"
                        :alt="getTeamName(result.team_id)"
                        class="team-logo"
                      />
                      <span class="team-name-mobile-hidden">{{ getTeamName(result.team_id) }}</span>
                    </div>
                  </router-link>
                </td>
                <td class="mobile-visible">
                  <span
                    class="status-badge"
                    :class="{
                      'status-finished': result.status === 'FINISHED',
                      'status-dnf': result.status === 'DNF',
                    }"
                  >
                    {{ result.status.toLowerCase() }}
                  </span>
                </td>
                <td class="mobile-hidden">{{ result.laps_completed }}</td>
                <td class="mobile-hidden">{{ formatRaceTime(result.race_time_seconds) }}</td>
                <td class="mobile-hidden">{{ result.total_distance_km.toFixed(2) }} km</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { RaceDb, RaceResultDb, DriverDb, TeamDb } from '@/services/ApiService';

const props = defineProps<{
  visible: boolean;
  race: RaceDb | null;
  results: RaceResultDb[];
  loading: boolean;
  error: string | null;
  trackName?: string;
  drivers: Map<string, DriverDb>;
  teams: Map<string, TeamDb>;
}>();

const emit = defineEmits<{
  close: [];
}>();

function formatDate(dateString: string | null): string {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: true,
  });
}

function formatRaceTime(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const secs = (seconds % 60).toFixed(2);
  return `${minutes}:${secs.padStart(5, '0')}`;
}

function getDriverName(driverId: string): string {
  const driver = props.drivers.get(driverId);
  if (driver) {
    return `${driver.first_name} ${driver.last_name}`;
  }
  return `Driver ${driverId.slice(0, 8)}`;
}

function getDriverAvatar(driverId: string): string | null {
  const driver = props.drivers.get(driverId);
  return driver?.avatar_url || null;
}

function getTeamName(teamId: string): string {
  const team = props.teams.get(teamId);
  return team?.name || `Team ${teamId.slice(0, 8)}`;
}

function getTeamLogo(teamId: string): string | null {
  const team = props.teams.get(teamId);
  return team?.logo || null;
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 8px;
  max-width: 90%;
  max-height: 90vh;
  width: 100%;
  max-width: 1200px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.modal-header .btn-close {
  order: -1;
  margin-right: 1rem;
  margin-top: 0;
}

.modal-header-content {
  flex: 1;
}

.modal-header h3 {
  margin: 0 0 1rem 0;
  color: #1a1a2e;
}

.race-details {
  display: flex;
  flex-wrap: wrap;
  gap: 1.5rem;
  margin-top: 0.5rem;
}

.race-detail-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.detail-label {
  font-weight: 600;
  color: #666;
  font-size: 0.875rem;
}

.detail-value {
  color: #1a1a2e;
  font-size: 0.875rem;
}

.btn-close {
  background: none;
  border: none;
  font-size: 2rem;
  color: #666;
  cursor: pointer;
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s ease;
}

.btn-close:hover {
  background-color: #f0f0f0;
  color: #333;
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
  flex: 1;
}

.loading-message,
.error-message,
.empty-state {
  text-align: center;
  padding: 2rem;
  color: #666;
}

.error-message {
  color: #d32f2f;
  background-color: #ffebee;
}

.results-table-wrapper {
  overflow-x: auto;
  width: 100%;
}

.results-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 1rem;
  table-layout: auto;
  display: table;
}

.results-table th {
  padding: 0.375rem 1rem;
  text-align: left;
  font-weight: 600;
  color: #1a1a2e;
  border-bottom: 2px solid #e0e0e0;
  background-color: #f5f5f5;
  position: sticky;
  top: 0;
}

.results-table td {
  padding: 0.375rem 1rem;
  border-bottom: 1px solid #e0e0e0;
  color: #333;
  vertical-align: middle;
}

.driver-cell-link,
.team-cell-link {
  text-decoration: none;
  color: inherit;
  transition: opacity 0.2s;
  padding: 0;
}

.driver-cell-link:hover,
.team-cell-link:hover {
  opacity: 0.7;
}

.driver-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.driver-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e0e0e0;
  flex-shrink: 0;
}

.team-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.results-table .team-logo {
  width: 32px;
  height: 32px;
  object-fit: cover;
  object-position: center;
  border-radius: 4px;
}

.results-table tbody tr:hover {
  background-color: #f9f9f9;
}

.results-table tbody tr.result-finished {
  border-left: 4px solid #4caf50;
}

.results-table tbody tr.result-dnf {
  border-left: 4px solid #f44336;
}

.position-cell {
  font-weight: 600;
  font-size: 1.1rem;
  text-align: center;
}

.position-cell.position-1,
.position-cell.position-2,
.position-cell.position-3 {
  font-weight: 700;
}

.position-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.laurel-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.875rem;
  font-weight: 500;
  display: inline-block;
}

.status-finished {
  color: #2e7d32;
}

.status-dnf {
  color: #c62828;
}

/* Smaller status badge in results table */
.results-table .status-badge {
  font-size: 0.7rem;
  padding: 0.15rem 0.5rem;
}

@media (max-width: 768px) {
  .modal-header {
    flex-direction: row;
    align-items: flex-start;
    padding: 0.75rem 1rem;
    gap: 0.75rem;
  }

  .modal-header .btn-close {
    order: -1;
    margin-right: 0;
    margin-top: 0;
    flex-shrink: 0;
  }

  .modal-header-content {
    flex: 1;
    min-width: 0;
  }

  .modal-header h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
  }

  .race-details {
    flex-direction: row;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-top: 0;
  }

  .race-detail-item {
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
  }

  .race-detail-item .detail-label {
    font-size: 0.8rem;
    margin: 0;
  }

  .race-detail-item .detail-value {
    font-size: 0.85rem;
    margin: 0;
  }

  .modal-content {
    max-width: 95%;
    max-height: 95vh;
  }

  .modal-body {
    padding: 1rem;
  }

  .results-table-wrapper {
    overflow-x: visible;
    width: 100%;
  }

  .results-table {
    width: 100%;
    table-layout: fixed;
  }

  .results-table th.mobile-hidden,
  .results-table td.mobile-hidden {
    display: none;
  }

  .results-table th.mobile-visible,
  .results-table td.mobile-visible {
    display: table-cell;
  }

  /* Set column widths for mobile results table */
  .results-table th.mobile-visible:nth-child(1),
  .results-table td.mobile-visible:nth-child(1) {
    width: 15%;
  }

  .results-table th.mobile-visible:nth-child(2),
  .results-table td.mobile-visible:nth-child(2) {
    width: 45%;
  }

  .results-table th.mobile-visible:nth-child(3),
  .results-table td.mobile-visible:nth-child(3) {
    width: 20%;
  }

  .results-table th.mobile-visible:nth-child(4),
  .results-table td.mobile-visible:nth-child(4) {
    width: 20%;
  }

  /* Hide team name text on mobile, show only logo */
  .results-table .team-name-mobile-hidden {
    display: none;
  }

  .results-table .team-cell {
    justify-content: center;
  }

  .results-table .team-logo {
    margin: 0 auto;
  }

  .results-table .status-badge {
    font-size: 0.65rem;
    padding: 0.15rem 0.4rem;
  }
}

@media (max-width: 480px) {
  .modal-content {
    max-width: 98%;
    margin: 0.5rem;
  }

  .results-table {
    font-size: 0.75rem;
  }

  .results-table th,
  .results-table td {
    padding: 0.4rem 0.3rem;
    font-size: 0.75rem;
  }

  .results-table th.mobile-hidden,
  .results-table td.mobile-hidden {
    display: none;
  }

  .results-table th.mobile-visible,
  .results-table td.mobile-visible {
    display: table-cell;
  }

  /* Set column widths for mobile results table on small screens */
  .results-table th.mobile-visible:nth-child(1),
  .results-table td.mobile-visible:nth-child(1) {
    width: 15%;
  }

  .results-table th.mobile-visible:nth-child(2),
  .results-table td.mobile-visible:nth-child(2) {
    width: 45%;
  }

  .results-table th.mobile-visible:nth-child(3),
  .results-table td.mobile-visible:nth-child(3) {
    width: 20%;
  }

  .results-table th.mobile-visible:nth-child(4),
  .results-table td.mobile-visible:nth-child(4) {
    width: 20%;
  }
}
</style>
