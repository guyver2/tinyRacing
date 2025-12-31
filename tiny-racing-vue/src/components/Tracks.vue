<template>
  <div class="tracks-container">
    <div class="tracks-content">
      <h2>Race Tracks</h2>

      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading tracks...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Tracks grid -->
      <div v-if="!loading && !error" class="tracks-grid">
        <div
          v-for="track in tracks"
          :key="track.id"
          class="track-card"
          @click="navigateToTrack(track.track_id || track.id)"
        >
          <div class="track-header">
            <h3 class="track-name">{{ track.name }}</h3>
          </div>

          <div class="track-preview">
            <img
              :src="getTrackSvgPath(track.track_id || track.id)"
              :alt="track.name"
              class="track-svg"
              @error="handleImageError"
            />
          </div>

          <div class="track-details">
            <div v-if="track.description" class="track-description">
              {{ track.description }}
            </div>

            <div class="track-stats">
              <div class="stat-item">
                <span class="stat-label">Lap Length:</span>
                <span class="stat-value">{{ track.lap_length_km }} km</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="!loading && !error && tracks.length === 0" class="empty-state">
        <p>No tracks available.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { getTracks, type TrackDb } from '@/services/ApiService';

const router = useRouter();
const tracks = ref<TrackDb[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

function navigateToTrack(trackId: string) {
  router.push({ name: 'track', params: { trackId } });
}

function getTrackSvgPath(trackId: string): string {
  return `/assets/tracks/${trackId}/track.svg`;
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement;
  // Hide broken images or show a placeholder
  img.style.display = 'none';
}

async function loadTracks() {
  loading.value = true;
  error.value = null;

  try {
    tracks.value = await getTracks();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load tracks';
    console.error('Error loading tracks:', err);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadTracks();
});
</script>

<style scoped>
.tracks-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.tracks-content {
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
  background-color: #e3f2fd;
  color: #1976d2;
}

.error-message {
  background-color: #ffebee;
  color: #c62828;
}

.tracks-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 2rem;
  margin-top: 2rem;
}

.track-card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;
  display: flex;
  flex-direction: column;
  cursor: pointer;
}

.track-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.track-header {
  padding: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
  background: linear-gradient(135deg, #f5f5f5 0%, #ffffff 100%);
}

.track-name {
  color: #1a1a2e;
  font-size: 1.5rem;
  margin: 0;
  font-weight: 600;
}

.track-preview {
  width: 100%;
  height: 250px;
  background: #f9f9f9;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  overflow: hidden;
}

.track-svg {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
}

.track-details {
  padding: 1.5rem;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.track-description {
  color: #666;
  font-size: 0.95rem;
  line-height: 1.6;
}

.track-stats {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding-top: 1rem;
  border-top: 1px solid #e0e0e0;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  color: #666;
  font-size: 0.9rem;
  font-weight: 500;
}

.stat-value {
  color: #1a1a2e;
  font-size: 1rem;
  font-weight: 600;
}

.empty-state {
  text-align: center;
  padding: 3rem;
  color: #666;
}

@media (max-width: 768px) {
  .tracks-grid {
    grid-template-columns: 1fr;
    gap: 1.5rem;
  }

  .tracks-container {
    padding: 1rem;
  }

  h2 {
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .track-preview {
    height: 200px;
  }
}
</style>
