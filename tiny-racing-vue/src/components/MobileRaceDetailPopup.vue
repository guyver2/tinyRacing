<template>
  <div v-if="visible && race" class="mobile-race-popup-overlay" @click="$emit('close')">
    <div class="mobile-race-popup-content" @click.stop>
      <div class="mobile-race-popup-header">
        <h3>{{ trackName }}</h3>
        <button @click="$emit('close')" class="btn-close" type="button" aria-label="Close">
          ×
        </button>
      </div>
      <div class="mobile-race-popup-body">
        <div class="mobile-race-detail-item">
          <span class="detail-label">Race Date:</span>
          <span class="detail-value">{{
            race.start_datetime ? formatDate(race.start_datetime) : 'N/A'
          }}</span>
        </div>
        <div class="mobile-race-detail-item">
          <span class="detail-label">Laps:</span>
          <span class="detail-value">{{ race.laps }}</span>
        </div>
        <div class="mobile-race-detail-item">
          <span class="detail-label">Status:</span>
          <span class="status-badge" :class="getRaceStatusClass(race.status)">
            {{ formatStatus(race.status) }}
          </span>
        </div>
        <div v-if="race.description" class="mobile-race-detail-item">
          <span class="detail-label">Description:</span>
          <span class="detail-value">{{ race.description }}</span>
        </div>
        <div class="mobile-race-popup-actions">
          <!-- Registration buttons for authenticated users with a team -->
          <div
            v-if="
              authenticated &&
              myTeam &&
              (race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED')
            "
            class="action-buttons"
          >
            <button
              v-if="canRegister(race)"
              type="button"
              @click.prevent="handleRegister"
              class="btn btn-success btn-small"
              :disabled="registering.get(race.id) || starting.get(race.id)"
            >
              <span v-if="registering.get(race.id)">Registering...</span>
              <span v-else>Register</span>
            </button>
            <button
              v-if="canUnregister(race)"
              type="button"
              @click.prevent="handleUnregister"
              class="btn btn-warning btn-small"
              :disabled="registering.get(race.id) || starting.get(race.id)"
            >
              <span v-if="registering.get(race.id)">Unregistering...</span>
              <span v-else>Unregister</span>
            </button>
          </div>
          <!-- Start now button for upcoming races -->
          <button
            v-if="race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED'"
            type="button"
            @click.prevent="handleStartNow"
            class="btn btn-primary btn-small"
            :disabled="starting.get(race.id)"
          >
            <span v-if="starting.get(race.id)">Starting...</span>
            <span v-else>Start now</span>
          </button>
          <!-- View race button for ongoing races -->
          <button
            v-if="race.status === 'ONGOING'"
            type="button"
            @click.prevent="handleViewRace"
            class="btn btn-primary btn-small"
          >
            View Race
          </button>
          <!-- View results button for finished races -->
          <button
            v-if="race.status === 'FINISHED'"
            type="button"
            @click.prevent="handleViewResults"
            class="btn btn-primary btn-small"
          >
            View Results
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { RaceDb, TeamDb } from '@/services/ApiService';

const props = defineProps<{
  visible: boolean;
  race: RaceDb | null;
  trackName: string;
  authenticated?: boolean;
  myTeam?: TeamDb | null;
  isRegistered: (raceId: string) => boolean;
  registering: Map<string, boolean>;
  starting: Map<string, boolean>;
}>();

const emit = defineEmits<{
  close: [];
  register: [raceId: string];
  unregister: [raceId: string];
  'start-now': [raceId: string];
  'view-race': [raceId: string];
  'view-results': [raceId: string];
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

function formatStatus(status: string): string {
  return status.replace(/_/g, ' ').toLowerCase();
}

function getRaceStatusClass(status: string): string {
  const statusLower = status.toLowerCase();
  if (statusLower === 'finished') return 'status-finished';
  if (statusLower === 'ongoing') return 'status-ongoing';
  if (statusLower === 'registration_open') return 'status-open';
  if (statusLower === 'registration_closed') return 'status-closed';
  if (statusLower === 'canceled') return 'status-canceled';
  return '';
}

function canRegister(race: RaceDb): boolean {
  return race.status === 'REGISTRATION_OPEN' && !props.isRegistered(race.id);
}

function canUnregister(race: RaceDb): boolean {
  return (
    (race.status === 'REGISTRATION_OPEN' || race.status === 'REGISTRATION_CLOSED') &&
    props.isRegistered(race.id)
  );
}

function handleRegister() {
  if (props.race) {
    emit('register', props.race.id);
    emit('close');
  }
}

function handleUnregister() {
  if (props.race) {
    emit('unregister', props.race.id);
    emit('close');
  }
}

function handleStartNow() {
  if (props.race) {
    emit('start-now', props.race.id);
    emit('close');
  }
}

function handleViewRace() {
  if (props.race) {
    emit('view-race', props.race.id);
    emit('close');
  }
}

function handleViewResults() {
  if (props.race) {
    emit('view-results', props.race.id);
    emit('close');
  }
}
</script>

<style scoped>
.mobile-race-popup-overlay {
  display: none;
}

.mobile-race-popup-content {
  display: none;
}

@media (max-width: 768px) {
  .mobile-race-popup-overlay {
    display: flex;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    align-items: center;
    justify-content: center;
    z-index: 1001;
    padding: 1rem;
  }

  .mobile-race-popup-content {
    display: flex;
    flex-direction: column;
    background: white;
    border-radius: 8px;
    max-width: 90%;
    max-height: 90vh;
    width: 100%;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .mobile-race-popup-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #e0e0e0;
  }

  .mobile-race-popup-header h3 {
    margin: 0;
    color: #1a1a2e;
    font-size: 1.25rem;
  }

  .mobile-race-popup-body {
    padding: 1rem;
    overflow-y: auto;
    flex: 1;
  }

  .mobile-race-detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #f0f0f0;
  }

  .mobile-race-detail-item:last-child {
    border-bottom: none;
    margin-bottom: 0;
    padding-bottom: 0;
  }

  .mobile-race-detail-item .detail-label {
    font-weight: 600;
    color: #666;
    font-size: 0.875rem;
  }

  .mobile-race-detail-item .detail-value {
    color: #1a1a2e;
    font-size: 1rem;
  }

  .mobile-race-popup-actions {
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .mobile-race-popup-actions .btn {
    width: 100%;
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

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-primary {
    background-color: #2d4059;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #3a5273;
  }

  .btn-success {
    background-color: #2e7d32;
    color: white;
  }

  .btn-success:hover:not(:disabled) {
    background-color: #1b5e20;
  }

  .btn-warning {
    background-color: #f57c00;
    color: white;
  }

  .btn-warning:hover:not(:disabled) {
    background-color: #e65100;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-small {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
  }

  .action-buttons {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
  }

  .action-buttons .btn {
    width: 100%;
  }

  .status-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.875rem;
    font-weight: 500;
    display: inline-block;
  }

  .status-open {
    color: #2e7d32;
  }

  .status-closed {
    color: #e65100;
  }

  .status-ongoing {
    color: #1976d2;
  }

  .status-finished {
    color: #7b1fa2;
  }

  .status-canceled {
    color: #c62828;
  }
}
</style>
