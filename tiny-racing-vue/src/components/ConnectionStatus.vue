<template>
  <div :class="['connection-status', connected ? 'connected' : 'disconnected']">
    {{ connected ? 'Connected' : 'Disconnected' }}
  </div>
</template>

<script setup lang="ts">
defineProps<{
  connected: boolean;
}>();
</script>

<style scoped>
.connection-status {
  position: fixed;
  bottom: 15px;
  right: 15px;
  padding: 8px 15px;
  border-radius: 20px;
  font-size: 0.8em;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
  z-index: 20;
  opacity: 0.8;
}
.connected {
  background-color: #b5e8b5;
  color: #2d4059;
}
.disconnected {
  background-color: #ffa5a5;
  color: #2d4059;
}
</style> 