<template>
  <div class="knob-container flex flex-col items-center gap-1">
    <!-- <div class="text-xs text-gray-400 uppercase tracking-wide">{{ label }}</div> -->
    <div 
      class="knob relative cursor-pointer"
      @mousedown="startDrag"
      @touchstart="startDrag"
      @dblclick="resetToCenter"
      @wheel.prevent="onWheel"
    >
      <svg width="60" height="60" viewBox="0 0 60 60">
        <!-- Background circle -->
        <circle
          cx="30"
          cy="30"
          r="26"
          fill="#1a1a1a"
          stroke="#333"
          stroke-width="2"
        />
        
        <!-- L/R text markers -->
        <text
          x="8"
          y="35"
          font-size="10"
          fill="#666"
          font-weight="bold"
        >L</text>
        <text
          x="48"
          y="35"
          font-size="10"
          fill="#666"
          font-weight="bold"
        >R</text>
        
        <!-- Value arc (only shows when not centered) -->
        <path
          v-if="!isCentered"
          :d="arcPath"
          fill="none"
          :stroke="color"
          stroke-width="3"
          stroke-linecap="round"
        />
        
        <!-- Center circle -->
        <circle
          cx="30"
          cy="30"
          r="20"
          fill="#2a2a2a"
          stroke="#444"
          stroke-width="1"
        />
        
        <!-- Indicator line (vertical at center) -->
        <line
          :x1="30"
          :y1="30"
          :x2="indicatorX"
          :y2="indicatorY"
          :stroke="isCentered ? '#666' : color"
          stroke-width="2"
          stroke-linecap="round"
        />
        
        <!-- Center dot -->
        <circle
          cx="30"
          cy="30"
          r="3"
          :fill="isCentered ? '#666' : color"
        />
      </svg>
    </div>
    <div class="text-xs text-white font-mono">{{ displayValue }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'

interface Props {
  modelValue: number // -1 to +1
  label?: string
  color?: string
}

const props = withDefaults(defineProps<Props>(), {
  label: 'Pan',
  color: '#3b82f6'
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const isDragging = ref(false)
const startY = ref(0)
const startValue = ref(0)

// Check if centered (within threshold)
const isCentered = computed(() => Math.abs(props.modelValue) < 0.05)

// Calculate angle (vertical at center = -90deg, range ±110deg for wider rotation)
const angle = computed(() => {
  // Map -1..+1 to -110..+110 degrees (full 220deg range)
  const degrees = props.modelValue * 110
  return -90 + degrees // -90 is vertical (center position)
})

// Calculate indicator position
const indicatorX = computed(() => {
  const rad = (angle.value * Math.PI) / 180
  return 30 + Math.cos(rad) * 15
})

const indicatorY = computed(() => {
  const rad = (angle.value * Math.PI) / 180
  return 30 + Math.sin(rad) * 15
})

// Calculate arc path showing deviation from center
const arcPath = computed(() => {
  const centerAngle = -90 // Vertical (center position)
  const currentAngle = angle.value
  const radius = 26
  
  // Draw arc from center to current position
  const start = polarToCartesian(30, 30, radius, centerAngle)
  const end = polarToCartesian(30, 30, radius, currentAngle)
  
  // Determine if we need large arc (shouldn't happen with ±110deg range)
  const angleDiff = Math.abs(currentAngle - centerAngle)
  const largeArcFlag = angleDiff > 180 ? 1 : 0
  
  // Sweep direction: 1 for clockwise, 0 for counter-clockwise
  const sweepFlag = currentAngle > centerAngle ? 1 : 0
  
  return [
    'M', start.x, start.y,
    'A', radius, radius, 0, largeArcFlag, sweepFlag, end.x, end.y
  ].join(' ')
})

// Display value
const displayValue = computed(() => {
  if (isCentered.value) return 'C'
  
  const percent = Math.round(Math.abs(props.modelValue) * 100)
  return props.modelValue < 0 ? `L${percent}` : `R${percent}`
})

function polarToCartesian(centerX: number, centerY: number, radius: number, angleInDegrees: number) {
  const angleInRadians = (angleInDegrees * Math.PI) / 180
  return {
    x: centerX + (radius * Math.cos(angleInRadians)),
    y: centerY + (radius * Math.sin(angleInRadians))
  }
}

function startDrag(e: MouseEvent | TouchEvent) {
  e.preventDefault()
  isDragging.value = true
  startY.value = 'touches' in e ? e.touches[0].clientY : e.clientY
  startValue.value = props.modelValue
  
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('touchmove', onDrag)
  document.addEventListener('mouseup', stopDrag)
  document.addEventListener('touchend', stopDrag)
}

function onDrag(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return
  
  const currentY = 'touches' in e ? e.touches[0].clientY : e.clientY
  const deltaY = startY.value - currentY
  
  const sensitivity = 0.005 // Adjust sensitivity
  const delta = deltaY * sensitivity
  
  let newValue = startValue.value + delta
  
  // Clamp between -1 and +1
  newValue = Math.max(-1, Math.min(1, newValue))
  
  // Snap to center if very close
  if (Math.abs(newValue) < 0.05) {
    newValue = 0
  }
  
  emit('update:modelValue', newValue)
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('touchmove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchend', stopDrag)
}

function resetToCenter() {
  emit('update:modelValue', 0)
}

function onWheel(e: WheelEvent) {
  e.preventDefault()
  
  // Use deltaY for vertical scroll (two-finger drag on trackpad)
  const delta = e.deltaY * 0.003
  
  let newValue = props.modelValue + delta
  
  // Clamp between -1 and +1
  newValue = Math.max(-1, Math.min(1, newValue))
  
  // NO snap to center during wheel control - allow fine control
  
  emit('update:modelValue', newValue)
}

onUnmounted(() => {
  stopDrag()
})
</script>

<style scoped>
.knob-container {
  user-select: none;
  -webkit-user-select: none;
}

.knob {
  touch-action: none;
}
</style>
