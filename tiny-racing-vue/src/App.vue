<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import AppHeader from './components/AppHeader.vue';
import Game from './components/game.vue';
import LoginForm from './components/LoginForm.vue';
import RegisterForm from './components/RegisterForm.vue';
import Team from './components/Team.vue';
import AllTeams from './components/AllTeams.vue';
import Market from './components/Market.vue';
import Races from './components/Races.vue';
import { isAuthenticated, logout } from './services/ApiService';

const authenticated = ref(false);
const currentView = ref('game');

function checkAuth() {
  authenticated.value = isAuthenticated();
}

function handleNavigate(view: string) {
  currentView.value = view;
  // Re-check auth when navigating to ensure state is up to date
  checkAuth();
}

function handleLoginSuccess() {
  authenticated.value = true;
  currentView.value = 'game';
}

function handleRegisterSuccess() {
  // Switch to login view after successful registration
  currentView.value = 'login';
}

async function handleLogout() {
  await logout();
  authenticated.value = false;
  // Redirect to initial game view if not already on it
  if (currentView.value !== 'game') {
    currentView.value = 'game';
  }
}

function showLogin() {
  currentView.value = 'login';
}

function showRegister() {
  currentView.value = 'register';
}

// Watch for view changes and re-check auth to keep state synchronized
watch(currentView, () => {
  checkAuth();
});

onMounted(() => {
  checkAuth();
});
</script>

<template>
  <div id="app">
    <!-- Header visible on all pages -->
    <AppHeader
      :authenticated="authenticated"
      :current-view="currentView"
      @navigate="handleNavigate"
      @login="showLogin"
      @register="showRegister"
      @logout="handleLogout"
    />

    <!-- Main content area -->
    <main class="main-content" :class="{ 'no-overflow': currentView === 'game' }">
      <!-- Game view -->
      <div v-show="currentView === 'game'" class="view">
        <Game />
      </div>

      <!-- Team view -->
      <div v-show="currentView === 'my-team'" class="view">
        <Team :authenticated="authenticated" />
      </div>

      <!-- All Teams view -->
      <div v-show="currentView === 'all-teams'" class="view">
        <AllTeams />
      </div>

      <!-- Market view -->
      <div v-show="currentView === 'market'" class="view market-view">
        <Market />
      </div>

      <!-- Races view -->
      <div v-show="currentView === 'races'" class="view races-view">
        <Races :authenticated="authenticated" @navigate="handleNavigate" />
      </div>

      <!-- Login view -->
      <div v-show="currentView === 'login'" class="view">
        <div class="form-view">
          <LoginForm @login-success="handleLoginSuccess" />
        </div>
      </div>

      <!-- Register view -->
      <div v-show="currentView === 'register'" class="view">
        <div class="form-view">
          <RegisterForm @register-success="handleRegisterSuccess" />
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
#app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 0;
}

.main-content {
  flex: 1;
  background-color: #f5f5f5;
  margin-top: 0;
  padding-top: 0;
  overflow-y: auto;
  color: #1a1a2e;
}

.main-content.no-overflow {
  overflow: hidden;
}

.view {
  width: 100%;
  height: 100%;
}

.view.market-view,
.view.races-view {
  overflow-y: auto;
  height: auto;
  min-height: 100%;
}

.placeholder-view {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
  text-align: center;
}

.placeholder-view h2 {
  color: #1a1a2e;
  font-size: 2rem;
  margin-bottom: 1rem;
}

.placeholder-view p {
  color: #666;
  font-size: 1.1rem;
}

.form-view {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-height: calc(100vh - 100px);
}
</style>
