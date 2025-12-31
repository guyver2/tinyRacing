<script setup lang="ts">
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';

const props = defineProps<{
  authenticated: boolean;
  currentView: string;
}>();

const emit = defineEmits<{
  (e: 'navigate', view: string): void;
  (e: 'login'): void;
  (e: 'register'): void;
  (e: 'logout'): void;
}>();

const router = useRouter();
const route = useRoute();

const navigationLinks = computed(() => [
  { name: 'Game', view: 'game', requiresAuth: false },
  { name: 'My Team', view: 'my-team', requiresAuth: true },
  { name: 'All Teams', view: 'all-teams', requiresAuth: false },
  { name: 'Races', view: 'races', requiresAuth: false },
  { name: 'Tracks', view: 'tracks', requiresAuth: false },
  { name: 'Market', view: 'market', requiresAuth: true },
]);

const visibleLinks = computed(() =>
  navigationLinks.value.filter((link) => !link.requiresAuth || props.authenticated),
);

// Check if a route is active (handles both exact matches and parameterized routes)
const isRouteActive = (viewName: string) => {
  const routeName = route.name as string;
  // Only match exact route names - don't confuse 'my-team' with 'team'
  return routeName === viewName;
};

function handleNavigate(view: string) {
  router.push({ name: view });
  emit('navigate', view);
}

function handleLogin() {
  emit('login');
}

function handleRegister() {
  emit('register');
}

function handleLogout() {
  emit('logout');
}
</script>

<template>
  <header class="app-header">
    <div class="header-container">
      <!-- Navigation links on the left -->
      <nav class="nav-links">
        <button
          v-for="link in visibleLinks"
          :key="link.view"
          :class="['nav-link', { active: isRouteActive(link.view) }]"
          @click="handleNavigate(link.view)"
        >
          {{ link.name }}
        </button>
      </nav>

      <!-- Auth actions on the right -->
      <div class="auth-actions">
        <template v-if="!authenticated">
          <button class="btn btn-secondary" @click="handleRegister">Register</button>
          <button class="btn btn-primary" @click="handleLogin">Login</button>
        </template>
        <template v-else>
          <button class="btn btn-logout" @click="handleLogout">Logout</button>
        </template>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  background-color: #1a1a2e;
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  position: sticky;
  top: 0;
  z-index: 1000;
}

.header-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0.75rem 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 2rem;
}

.nav-links {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.nav-link {
  padding: 0.5rem 1rem;
  background: transparent;
  color: #e0e0e0;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  transition: all 0.2s ease;
  font-weight: 500;
}

.nav-link:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
}

.nav-link.active {
  background-color: #2d4059;
  color: white;
}

.auth-actions {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}

.btn {
  padding: 0.5rem 1.25rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.95rem;
  font-weight: 500;
  transition: all 0.2s ease;
}

.btn-primary {
  background-color: #2d4059;
  color: white;
}

.btn-primary:hover {
  background-color: #3a5273;
}

.btn-secondary {
  background-color: transparent;
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.btn-secondary:hover {
  background-color: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.5);
}

.btn-logout {
  background-color: #d32f2f;
  color: white;
}

.btn-logout:hover {
  background-color: #b71c1c;
}

/* Responsive design */
@media (max-width: 768px) {
  .header-container {
    flex-direction: column;
    padding: 0.75rem 1rem;
    gap: 1rem;
  }

  .nav-links {
    width: 100%;
    justify-content: center;
  }

  .auth-actions {
    width: 100%;
    justify-content: center;
  }

  .nav-link {
    padding: 0.4rem 0.8rem;
    font-size: 0.9rem;
  }

  .btn {
    padding: 0.4rem 1rem;
    font-size: 0.9rem;
  }
}
</style>
