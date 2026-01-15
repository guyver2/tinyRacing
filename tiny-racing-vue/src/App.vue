<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import AppHeader from './components/AppHeader.vue';
import { isAuthenticated, logout } from './services/ApiService';

const router = useRouter();
const route = useRoute();

const authenticated = ref(false);

function checkAuth() {
  authenticated.value = isAuthenticated();
}

function handleNavigate(view: string) {
  router.push({ name: view });
  // Re-check auth when navigating to ensure state is up to date
  checkAuth();
}

function handleLoginSuccess() {
  authenticated.value = true;
  router.push({ name: 'game' });
}

function handleRegisterSuccess() {
  // Switch to login view after successful registration
  router.push({ name: 'login' });
}

async function handleLogout() {
  await logout();
  authenticated.value = false;
  // Redirect to initial game view if not already on it
  if (route.name !== 'game') {
    router.push({ name: 'game' });
  }
}

function showLogin() {
  router.push({ name: 'login' });
}

function showRegister() {
  router.push({ name: 'register' });
}

const currentView = computed(() => {
  return (route.name as string) || 'game';
});

// Watch for route changes and re-check auth to keep state synchronized
watch(
  () => route.name,
  () => {
    checkAuth();
  },
);

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
      <router-view v-slot="{ Component }">
        <component
          :is="Component"
          :authenticated="authenticated"
          @navigate="handleNavigate"
          @login-success="handleLoginSuccess"
          @register-success="handleRegisterSuccess"
        />
      </router-view>
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

/* Mobile Responsive Styles */
@media (max-width: 768px) {
  .placeholder-view {
    padding: 1.5rem 1rem;
  }

  .placeholder-view h2 {
    font-size: 1.5rem;
  }

  .placeholder-view p {
    font-size: 1rem;
  }

  .form-view {
    padding: 1.5rem 1rem;
    min-height: calc(100vh - 80px);
  }
}

@media (max-width: 480px) {
  .placeholder-view {
    padding: 1rem 0.75rem;
  }

  .placeholder-view h2 {
    font-size: 1.25rem;
  }

  .placeholder-view p {
    font-size: 0.95rem;
  }

  .form-view {
    padding: 1rem 0.75rem;
    min-height: calc(100vh - 70px);
  }
}
</style>
