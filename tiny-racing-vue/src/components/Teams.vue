<template>
  <div class="teams-container">
    <div class="teams-content">
      <h2>My Team</h2>

      <!-- Loading state -->
      <div v-if="loading" class="loading-message">Loading teams...</div>

      <!-- Error state -->
      <div v-if="error" class="error-message">{{ error }}</div>

      <!-- Create team form (shown when user has no team) -->
      <div v-if="!loading && !error && !team" class="create-team-section">
        <div class="create-team-form">
          <h3>Create Your First Team</h3>
          <p class="form-description">You don't have any teams yet. Create one to get started!</p>
          <form @submit.prevent="handleCreateTeam">
            <div class="form-group">
              <label for="teamName">Team Name:</label>
              <input
                id="teamName"
                v-model="formData.name"
                type="text"
                required
                :disabled="creating"
              />
            </div>
            <div class="form-group">
              <label for="teamLogo">Logo URL:</label>
              <input
                id="teamLogo"
                v-model="formData.logo"
                type="text"
                placeholder="https://example.com/logo.png"
                :disabled="creating"
              />
            </div>
            <div class="form-group">
              <label for="teamColor">Color:</label>
              <input
                id="teamColor"
                v-model="formData.color"
                type="color"
                required
                :disabled="creating"
              />
            </div>
            <div v-if="createError" class="error-message">{{ createError }}</div>
            <button type="submit" :disabled="creating">
              {{ creating ? 'Creating...' : 'Create Team' }}
            </button>
          </form>
        </div>
      </div>

      <!-- Team display (shown when user has a team) -->
      <div v-if="!loading && !error && team" class="team-display">
        <div class="team-card">
          <div class="team-header">
            <div class="team-color" :style="{ backgroundColor: team.color }"></div>
            <div class="team-info">
              <h3>#{{ team.number }} - {{ team.name }}</h3>
              <p class="team-details">
                <span>Pit Efficiency: {{ (team.pit_efficiency * 100).toFixed(1) }}%</span>
              </p>
            </div>
          </div>
          <div v-if="team.logo" class="team-logo">
            <img :src="team.logo" :alt="team.name" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getMyTeam, createTeam, type TeamDb, type CreateTeamRequest } from '@/services/ApiService';

const team = ref<TeamDb | null>(null);
const loading = ref(true);
const error = ref('');
const creating = ref(false);
const createError = ref('');

const formData = ref<CreateTeamRequest>({
  name: '',
  logo: '',
  color: '#2d4059',
});

async function loadTeam() {
  loading.value = true;
  error.value = '';
  
  try {
    team.value = await getMyTeam();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load team';
    console.error('Error loading team:', err);
  } finally {
    loading.value = false;
  }
}

async function handleCreateTeam() {
  createError.value = '';
  creating.value = true;

  try {
    const request: CreateTeamRequest = {
      ...formData.value,
    };

    await createTeam(request);
    
    // Reset form
    formData.value = {
      name: '',
      logo: '',
      color: '#2d4059',
    };

    // Reload team
    await loadTeam();
  } catch (err) {
    createError.value = err instanceof Error ? err.message : 'Failed to create team';
    console.error('Error creating team:', err);
  } finally {
    creating.value = false;
  }
}

onMounted(() => {
  loadTeam();
});
</script>

<style scoped>
.teams-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.teams-content {
  width: 100%;
}

h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 2rem;
  text-align: center;
}

h3 {
  color: #2d4059;
  font-size: 1.5rem;
  margin-bottom: 1rem;
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

.create-team-section {
  display: flex;
  justify-content: center;
  margin-top: 2rem;
}

.create-team-form {
  background-color: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 500px;
}

.form-description {
  color: #666;
  margin-bottom: 1.5rem;
  text-align: center;
}

.form-group {
  margin-bottom: 1rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  color: #2d4059;
  font-weight: 500;
}

input[type="text"],
input[type="number"],
input[type="color"] {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
  box-sizing: border-box;
}

input[type="color"] {
  height: 50px;
  cursor: pointer;
}

input:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

input:focus {
  outline: none;
  border-color: #2d4059;
}

button {
  width: 100%;
  padding: 0.75rem;
  background-color: #2d4059;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  margin-top: 1rem;
}

button:hover:not(:disabled) {
  background-color: #1e2a3a;
}

button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.team-display {
  display: flex;
  justify-content: center;
  margin-top: 2rem;
}

.team-card {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 1.5rem;
  transition: transform 0.2s, box-shadow 0.2s;
  width: 100%;
  max-width: 500px;
}

.team-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
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
</style>

