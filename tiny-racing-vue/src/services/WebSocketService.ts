import { ref } from 'vue';
import type { RaceState } from '@/types';

// Default empty state
const defaultRaceState: RaceState = {
  track: {
    id: '',
    name: 'Loading...',
    svg_start_offset: 0,
    current_weather: 'clear',
    wetness: 0.0,
    elapsed_time: 0.0,
  },
  cars: [],
  race_status: 'Loading...',
  current_lap: 0,
  total_laps: 0,
};

// State
const raceState = ref<RaceState>({ ...defaultRaceState });
const connectedState = ref(false);
const startTime = ref(Date.now());

// WebSocket connection
let socket: WebSocket;

const connectWebSocket = () => {
  socket = new WebSocket('ws://127.0.0.1:3030/ws');

  socket.addEventListener('open', () => {
    connectedState.value = true;
    startTime.value = Date.now();
  });

  socket.addEventListener('message', (event) => {
    try {
      const data = JSON.parse(event.data);
      raceState.value = data;
    } catch (error) {
      console.error('Error parsing race state:', error);
    }
  });

  socket.addEventListener('close', () => {
    connectedState.value = false;
    // Try to reconnect after a delay
    setTimeout(connectWebSocket, 5000);
  });

  socket.addEventListener('error', () => {
    connectedState.value = false;
  });
};

// Send car strategy updates to server
const updateCarStrategy = (strategy: {
  carNumber: number;
  style: string;
  tire?: string;
  refuel?: number;
}) => {
  if (socket && socket.readyState === WebSocket.OPEN) {
    const message = {
      type: 'strategy',
      data: strategy,
    };
    socket.send(JSON.stringify(message));
    return true;
  }
  return false;
};

// Initialize connection
connectWebSocket();

export const useRaceData = () => {
  return {
    raceState,
    connected: connectedState,
    updateCarStrategy,
  };
};
