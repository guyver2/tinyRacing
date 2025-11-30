<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Game from './components/game.vue';
import LoginForm from './components/LoginForm.vue';
import RegisterForm from './components/RegisterForm.vue';
import { isAuthenticated, logout } from './services/ApiService';

const authenticated = ref(false);
const showLoginForm = ref(false);
const showRegisterForm = ref(false);

function checkAuth() {
  authenticated.value = isAuthenticated();
}

function handleLoginSuccess() {
  authenticated.value = true;
  showLoginForm.value = false;
}

function handleRegisterSuccess() {
  showRegisterForm.value = false;
  // Optionally auto-show login form after successful registration
  showLoginForm.value = true;
}

async function handleLogout() {
  await logout();
  authenticated.value = false;
}

function showLogin() {
  showLoginForm.value = true;
  showRegisterForm.value = false;
}

function showRegister() {
  showRegisterForm.value = true;
  showLoginForm.value = false;
}

function hideLogin() {
  showLoginForm.value = false;
}

function hideRegister() {
  showRegisterForm.value = false;
}

onMounted(() => {
  checkAuth();
});
</script>

<template>
  <div>
    <!-- Auth buttons in top-right corner -->
    <div
      style="position: absolute; top: 10px; right: 10px; z-index: 1000; display: flex; gap: 10px"
    >
      <button
        v-if="!authenticated"
        @click="showRegister"
        style="
          padding: 0.5rem 1rem;
          background-color: #2d4059;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        "
      >
        Register
      </button>
      <button
        v-if="!authenticated"
        @click="showLogin"
        style="
          padding: 0.5rem 1rem;
          background-color: #2d4059;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        "
      >
        Login
      </button>
      <button
        v-if="authenticated"
        @click="handleLogout"
        style="
          padding: 0.5rem 1rem;
          background-color: #d32f2f;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        "
      >
        Logout
      </button>
    </div>

    <!-- Login form overlay (shown when login button is clicked) -->
    <div
      v-if="showLoginForm"
      style="
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 2000;
      "
      @click.self="hideLogin"
    >
      <div style="position: relative">
        <button
          @click="hideLogin"
          style="
            position: absolute;
            top: -40px;
            right: 0;
            background-color: white;
            border: none;
            border-radius: 4px;
            padding: 0.5rem 1rem;
            cursor: pointer;
          "
        >
          Close
        </button>
        <LoginForm @login-success="handleLoginSuccess" />
      </div>
    </div>

    <!-- Register form overlay (shown when register button is clicked) -->
    <div
      v-if="showRegisterForm"
      style="
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 2000;
      "
      @click.self="hideRegister"
    >
      <div style="position: relative">
        <button
          @click="hideRegister"
          style="
            position: absolute;
            top: -40px;
            right: 0;
            background-color: white;
            border: none;
            border-radius: 4px;
            padding: 0.5rem 1rem;
            cursor: pointer;
          "
        >
          Close
        </button>
        <RegisterForm @register-success="handleRegisterSuccess" @close="hideRegister" />
      </div>
    </div>

    <!-- Game is always shown (default page) -->
    <Game />
  </div>
</template>
