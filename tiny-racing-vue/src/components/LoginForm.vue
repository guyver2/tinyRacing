<template>
  <div class="login-container">
    <div class="login-form">
      <h2>Login</h2>
      <form @submit.prevent="handleLogin">
        <div class="form-group">
          <label for="username">Username:</label>
          <input
            id="username"
            v-model="username"
            type="text"
            required
            autocomplete="username"
            :disabled="loading"
          />
        </div>
        <div class="form-group">
          <label for="password">Password:</label>
          <input
            id="password"
            v-model="password"
            type="password"
            required
            autocomplete="current-password"
            :disabled="loading"
          />
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
        <button type="submit" :disabled="loading">
          {{ loading ? 'Logging in...' : 'Login' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { login, isAuthenticated } from '@/services/ApiService';

const emit = defineEmits<{
  loginSuccess: [];
}>();

const username = ref('');
const password = ref('');
const loading = ref(false);
const error = ref('');

async function handleLogin() {
  error.value = '';
  loading.value = true;

  try {
    const response = await login(username.value, password.value);

    if (response.status === 'success' && isAuthenticated()) {
      emit('loginSuccess');
    } else {
      error.value = response.message || 'Login failed';
    }
  } catch (err) {
    error.value = 'Network error. Please try again.';
    console.error('Login error:', err);
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background-color: #f0f5f9;
}

.login-form {
  background-color: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 400px;
}

h2 {
  margin-top: 0;
  margin-bottom: 1.5rem;
  text-align: center;
  color: #2d4059;
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

input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
  box-sizing: border-box;
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

.error-message {
  color: #d32f2f;
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}
</style>
