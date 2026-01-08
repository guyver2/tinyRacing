<template>
  <div class="radar-chart-container">
    <canvas ref="chartCanvas"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onBeforeUnmount } from 'vue';
import {
  Chart,
  RadialLinearScale,
  PointElement,
  LineElement,
  Filler,
  Tooltip,
  Legend,
  RadarController,
  type ChartConfiguration,
} from 'chart.js';
import ChartDataLabels from 'chartjs-plugin-datalabels';

Chart.register(
  RadarController,
  RadialLinearScale,
  PointElement,
  LineElement,
  Filler,
  Tooltip,
  Legend,
  ChartDataLabels,
);

interface Props {
  handling: number;
  acceleration: number;
  topSpeed: number;
  reliability: number;
  fuelConsumption: number;
  tireWear: number;
}

const props = defineProps<Props>();

const chartCanvas = ref<HTMLCanvasElement | null>(null);
let chartInstance: Chart<'radar'> | null = null;

const statNames = [
  'Handling',
  'Acceleration',
  'Top Speed',
  'Reliability',
  'Fuel Consumption',
  'Tire Wear',
];

function getLabelsWithValues(): string[][] {
  const values = [
    props.handling,
    props.acceleration,
    props.topSpeed,
    props.reliability,
    props.fuelConsumption,
    props.tireWear,
  ];
  return statNames.map((name, index) => [name, values[index].toFixed(2)]);
}

function createChart() {
  if (!chartCanvas.value) return;

  // Destroy existing chart if it exists
  if (chartInstance) {
    chartInstance.destroy();
  }

  const config: ChartConfiguration<'radar'> = {
    type: 'radar',
    data: {
      labels: getLabelsWithValues(),
      datasets: [
        {
          label: 'Car Stats',
          data: [
            props.handling,
            props.acceleration,
            props.topSpeed,
            props.reliability,
            props.fuelConsumption,
            props.tireWear,
          ],
          backgroundColor: 'rgba(33, 150, 243, 0.2)',
          borderColor: 'rgba(33, 150, 243, 1)',
          borderWidth: 2,
          pointBackgroundColor: 'rgba(33, 150, 243, 1)',
          pointBorderColor: '#fff',
          pointHoverBackgroundColor: '#fff',
          pointHoverBorderColor: 'rgba(33, 150, 243, 1)',
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false,
      scales: {
        r: {
          beginAtZero: true,
          max: 1,
          ticks: {
            stepSize: 0.2,
            display: false,
          },
          grid: {
            color: 'rgba(0, 0, 0, 0.1)',
          },
          pointLabels: {
            font: {
              size: 11,
              weight: 'bold',
            },
            color: '#666',
            padding: 8,
          },
        },
      },
      plugins: {
        legend: {
          display: false,
        },
        tooltip: {
          callbacks: {
            label: (context) => {
              return `${context.label}: ${context.parsed.r.toFixed(2)}`;
            },
          },
        },
        datalabels: {
          display: false,
        },
      },
    },
  };

  chartInstance = new Chart(chartCanvas.value, config);
}

onMounted(() => {
  createChart();
});

watch(
  () => [
    props.handling,
    props.acceleration,
    props.topSpeed,
    props.reliability,
    props.fuelConsumption,
    props.tireWear,
  ],
  () => {
    if (chartInstance) {
      chartInstance.data.labels = getLabelsWithValues();
      chartInstance.data.datasets[0].data = [
        props.handling,
        props.acceleration,
        props.topSpeed,
        props.reliability,
        props.fuelConsumption,
        props.tireWear,
      ];
      chartInstance.update('active');
    }
  },
);

onBeforeUnmount(() => {
  if (chartInstance) {
    chartInstance.destroy();
    chartInstance = null;
  }
});
</script>

<style scoped>
.radar-chart-container {
  width: 100%;
  height: 250px;
  margin-top: 0.5rem;
}
</style>
