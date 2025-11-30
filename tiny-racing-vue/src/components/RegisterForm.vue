<template>
  <div class="register-container">
    <div class="register-form">
      <h2>Register</h2>
      <form @submit.prevent="handleRegister">
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
          <label for="email">Email (optional):</label>
          <input id="email" v-model="email" type="email" autocomplete="email" :disabled="loading" />
        </div>
        <div class="form-group">
          <label for="password">Password:</label>
          <input
            id="password"
            v-model="password"
            type="password"
            required
            autocomplete="new-password"
            :disabled="loading"
          />
        </div>
        <div class="form-group">
          <label for="confirmPassword">Confirm Password:</label>
          <input
            id="confirmPassword"
            v-model="confirmPassword"
            type="password"
            required
            autocomplete="new-password"
            :disabled="loading"
          />
        </div>
        <div v-if="error" class="error-message">{{ error }}</div>
        <div v-if="success" class="success-message">{{ success }}</div>
        <button type="submit" :disabled="loading">
          {{ loading ? 'Registering...' : 'Register' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { register } from '@/services/ApiService';

const emit = defineEmits<{
  registerSuccess: [];
  close: [];
}>();

const username = ref('');
const email = ref('');
const password = ref('');
const confirmPassword = ref('');
const loading = ref(false);
const error = ref('');
const success = ref('');

async function handleRegister() {
  error.value = '';
  success.value = '';
  loading.value = true;

  // Validate passwords match
  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match';
    loading.value = false;
    return;
  }

  // Validate password length
  if (password.value.length < 6) {
    error.value = 'Password must be at least 6 characters';
    loading.value = false;
    return;
  }

  try {
    const response = await register(username.value, email.value || undefined, password.value);

    if (response.status === 'success') {
      success.value = 'Registration successful! You can now login.';
      // Auto-close after 2 seconds
      setTimeout(() => {
        emit('close');
        emit('registerSuccess');
      }, 2000);
    } else {
      error.value = response.message || 'Registration failed';
    }
  } catch (err) {
    error.value = 'Network error. Please try again.';
    console.error('Register error:', err);
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.register-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background-color: #f0f5f9;
}

.register-form {
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

.success-message {
  color: #2e7d32;
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}
</style>
