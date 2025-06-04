<template>
  <div class="track-container" ref="trackContainer">
    <!-- SVG will be loaded dynamically -->
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { Car, Track } from '@/types';
import { ROOT_URL } from '@/constants';

interface TrackData {
  id: string;
  svg_start_offset: number;
}

const props = defineProps<{
  cars: Car[];
  trackId: string;
}>();

const trackContainer = ref<HTMLElement | null>(null);
const trackData = ref<TrackData | null>(null);
const teamColors = {
  'Red Bull': '#aed9e0',
  'Ferrari': '#ffa5a5',
  'Mercedes': '#b5e8b5',
  'McLaren': '#ffe3a3',
  'Alpine': '#739ff2'
};

async function loadTrack() {
  if (!props.trackId) return;
  
  try {
    // Load SVG
    const svgResponse = await fetch(`${ROOT_URL}/assets/tracks/${props.trackId}/track.svg`);
    const svgData = await svgResponse.text();
    if (trackContainer.value) {
      trackContainer.value.innerHTML = svgData;
      
      // Make the SVG responsive
      const svg = document.getElementById('track-svg');
      if (svg) {
        svg.setAttribute('width', '100%');
        svg.setAttribute('height', '100%');
        
        // Optimize display based on aspect ratio
        const containerWidth = trackContainer.value.clientWidth;
        const containerHeight = trackContainer.value.clientHeight;
        
        // Choose the appropriate preserveAspectRatio based on container dimensions
        const aspectRatio = containerWidth / containerHeight;
        if (aspectRatio > 1.5) {
          // Wide container - prioritize height
          svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
        } else {
          // Taller container or close to square - use the full width
          svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
        }
        
        // Ensure viewBox is set properly to show the entire track
        const viewBox = svg.getAttribute('viewBox');
        if (!viewBox) {
          const bbox = (svg as SVGSVGElement).getBBox();
          svg.setAttribute('viewBox', `${bbox.x - 10} ${bbox.y - 10} ${bbox.width + 20} ${bbox.height + 20}`);
        }
      }
    }
    
    // Load track data
    const dataResponse = await fetch(`${ROOT_URL}/assets/tracks/${props.trackId}/track.json`);
    trackData.value = await dataResponse.json();
  } catch (error) {
    console.error('Error loading track:', error);
  }
}

function updateCarPositions() {
  if (!trackData.value) return;
  
  const svg = document.getElementById('track-svg');
  if (!svg) return;
  
  const path = document.getElementById('track');
  if (!path) return;
  
  const totalLength = (path as SVGPathElement).getTotalLength();
  if (!isFinite(totalLength)) return;
  
  props.cars.forEach(car => {
    // Get point in untransformed path coordinates
    const point = (path as SVGPathElement).getPointAtLength(
      totalLength * ((car.track_position + trackData.value!.svg_start_offset) % 1)
    );
    
    // Transform point to account for any SVG transformations
    const svgPoint = (svg as SVGSVGElement).createSVGPoint();
    svgPoint.x = point.x;
    svgPoint.y = point.y;
    
    // Get transformation matrix for the path and transform the point
    const pathCTM = (path as SVGPathElement).getScreenCTM();
    const svgCTM = (svg as SVGSVGElement).getScreenCTM();
    
    if (pathCTM && svgCTM) {
      // Transform from path coordinates to screen coordinates, then back to SVG coordinates
      const transformedPoint = svgPoint.matrixTransform(pathCTM).matrixTransform(svgCTM.inverse());
      
      // Create or update car circle
      let svgCar = document.getElementById(`car_${car.car_number}`);
      if (!svgCar) {
        svgCar = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        svgCar.setAttribute('id', `car_${car.car_number}`);
        svgCar.setAttribute('r', '10');
        svgCar.setAttribute('fill', teamColors[car.team_name as keyof typeof teamColors] || '#aaaaaa');
        svgCar.setAttribute('style', 'transition: cx 0.5s ease-out, cy 0.5s ease-out;');
        svg.appendChild(svgCar);
        
        // Create car number text
        const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        label.textContent = car.car_number.toString();
        label.setAttribute('id', `car_number_${car.car_number}`);
        label.setAttribute('text-anchor', 'middle');
        label.setAttribute('dominant-baseline', 'middle');
        label.setAttribute('fill', 'black');
        label.setAttribute('font-size', '20px');
        label.setAttribute('font-weight', 'bold');
        label.setAttribute('font-family', 'Arial, sans-serif');
        label.setAttribute('x', '0');
        label.setAttribute('y', '0');
        label.setAttribute('style', 'transition: transform 0.5s ease-out;');
        svg.appendChild(label);
      }
      
      svgCar.setAttribute('cx', transformedPoint.x.toString());
      svgCar.setAttribute('cy', transformedPoint.y.toString());
      
      const label = document.getElementById(`car_number_${car.car_number}`);
      if (label) {
        // Ensure the label has the proper transition style applied
        const currentStyle = label.getAttribute('style') || '';
        if (!currentStyle.includes('transition')) {
          label.setAttribute('style', (currentStyle + '; transition: transform 0.5s ease-out;').replace(/^;/, ''));
        }
        // Use transform instead of x/y attributes for smoother transitions
        label.setAttribute('transform', `translate(${transformedPoint.x}, ${transformedPoint.y + 1.5})`);
      }
    }
  });
}

function resizeHandler() {
  // Update SVG sizing based on container dimensions
  if (trackContainer.value) {
    const svg = document.getElementById('track-svg');
    if (svg) {
      const containerWidth = trackContainer.value.clientWidth;
      const containerHeight = trackContainer.value.clientHeight;
      
      // Adjust display based on actual dimensions
      const aspectRatio = containerWidth / containerHeight;
      if (aspectRatio > 1.5) {
        svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
      } else {
        svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
      }
    }
  }
  
  // Update car positions
  updateCarPositions();
}

onMounted(() => {
  loadTrack();
  
  // Add window resize handler to ensure track is properly displayed
  window.addEventListener('resize', () => {
    // Small delay to ensure the DOM has been updated
    setTimeout(resizeHandler, 100);
  });
});

watch(() => props.trackId, loadTrack);
watch(() => props.cars, updateCarPositions, { deep: true });
</script>

<style scoped>
.track-container {
  position: relative;
  width: 100%;
  height: 100%;
  border: 1px solid #dbe2ef;
  border-radius: 0;
  overflow: hidden;
  background-color: #f9f7f7;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  display: flex;
  justify-content: center;
  align-items: center;
  max-width: 100%;
  max-height: 100%;
  margin: 0;
}

:deep(#track-svg) {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

:deep(circle[id^="car_"]) {
  transition: cx 0.5s ease-out, cy 0.5s ease-out;
}

:deep(text[id^="car_number_"]) {
  transition: transform 0.5s ease-out;
}
</style> 