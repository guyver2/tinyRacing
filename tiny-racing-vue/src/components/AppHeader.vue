<script setup lang="ts">
import { computed, ref } from 'vue';
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

// Mobile menu state
const isMobileMenuOpen = ref(false);

function toggleMobileMenu() {
  isMobileMenuOpen.value = !isMobileMenuOpen.value;
}

function closeMobileMenu() {
  isMobileMenuOpen.value = false;
}

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
  closeMobileMenu(); // Close mobile menu after navigation
}

function handleLogin() {
  emit('login');
  closeMobileMenu();
}

function handleRegister() {
  emit('register');
  closeMobileMenu();
}

function handleLogout() {
  emit('logout');
  closeMobileMenu();
}
</script>

<template>
  <header class="app-header">
    <div class="header-container">
      <!-- Mobile menu button -->
      <button class="mobile-menu-toggle" @click="toggleMobileMenu" aria-label="Toggle menu">
        <span class="hamburger-icon" :class="{ active: isMobileMenuOpen }">
          <span></span>
          <span></span>
          <span></span>
        </span>
      </button>

      <!-- Navigation links on the left -->
      <nav class="nav-links" :class="{ 'mobile-open': isMobileMenuOpen }">
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
      <div class="auth-actions" :class="{ 'mobile-open': isMobileMenuOpen }">
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

/* Mobile menu toggle button */
.mobile-menu-toggle {
  display: none;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0.5rem;
  color: white;
  z-index: 1001;
}

.hamburger-icon {
  display: flex;
  flex-direction: column;
  justify-content: space-around;
  width: 24px;
  height: 20px;
  transition: all 0.3s ease;
}

.hamburger-icon span {
  width: 100%;
  height: 2px;
  background-color: white;
  border-radius: 2px;
  transition: all 0.3s ease;
  transform-origin: center;
}

.hamburger-icon.active span:nth-child(1) {
  transform: rotate(45deg) translate(6px, 6px);
}

.hamburger-icon.active span:nth-child(2) {
  opacity: 0;
}

.hamburger-icon.active span:nth-child(3) {
  transform: rotate(-45deg) translate(6px, -6px);
}

/* Responsive design */
@media (max-width: 768px) {
  .header-container {
    flex-wrap: wrap;
    padding: 0.75rem 1rem;
    gap: 0;
    position: relative;
  }

  .mobile-menu-toggle {
    display: block;
    order: 1;
  }

  .nav-links {
    display: none;
    width: 100%;
    flex-direction: column;
    gap: 0;
    order: 3;
    background-color: #1a1a2e;
    margin-top: 0.5rem;
    padding: 0.5rem 0;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .nav-links.mobile-open {
    display: flex;
  }

  .nav-link {
    padding: 0.75rem 1rem;
    font-size: 0.9rem;
    width: 100%;
    text-align: left;
    border-radius: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .nav-link:hover {
    background-color: rgba(255, 255, 255, 0.15);
  }

  .auth-actions {
    display: none;
    width: 100%;
    flex-direction: column;
    gap: 0.5rem;
    order: 4;
    padding-top: 0.5rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    margin-top: 0.5rem;
  }

  .auth-actions.mobile-open {
    display: flex;
  }

  .btn {
    padding: 0.75rem 1rem;
    font-size: 0.9rem;
    width: 100%;
    text-align: center;
  }
}

@media (max-width: 480px) {
  .header-container {
    padding: 0.5rem 0.75rem;
  }

  .nav-link {
    padding: 0.65rem 0.75rem;
    font-size: 0.85rem;
  }

  .btn {
    padding: 0.65rem 0.9rem;
    font-size: 0.85rem;
  }
}
</style>
