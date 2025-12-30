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
            <tr v-for="team in teams" :key="team.id" class="team-row" @click="toggleTeam(team.id)">
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { getTeams, type TeamDb } from '@/services/ApiService';

const router = useRouter();

const teams = ref<TeamDb[]>([]);
const loading = ref(true);
const error = ref('');

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

function toggleTeam(teamId: string) {
  // Navigate to team route instead of expanding inline
  router.push({ name: 'team', params: { teamId } });
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

@media (max-width: 768px) {
  .teams-table {
    font-size: 0.9rem;
  }

  .teams-table th,
  .teams-table td {
    padding: 0.75rem 0.5rem;
  }
}
</style>
