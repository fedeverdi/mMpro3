<template>
  <div class="knob-container flex flex-col items-center gap-1">
    <div class="text-xs text-gray-400 uppercase tracking-wide">{{ label }}</div>
    <div 
      class="knob relative cursor-pointer"
      @mousedown="startDrag"
      @touchstart="startDrag"
      @dblclick="reset440Hz"
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
        
        <!-- Value arc -->
        <path
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
        
        <!-- Indicator line -->
        <line
          :x1="30"
          :y1="30"
          :x2="indicatorX"
          :y2="indicatorY"
          :stroke="color"
          stroke-width="2"
          stroke-linecap="round"
        />
        
        <!-- Center dot -->
        <circle
          cx="30"
          cy="30"
          r="3"
          :fill="color"
        />
      </svg>
    </div>
    
    <!-- Editable frequency input -->
    <input 
      v-model.number="editableValue"
      @focus="(e) => (e.currentTarget as HTMLInputElement).select()"
      @blur="updateFromInput"
      @keydown.enter="(e) => (e.currentTarget as HTMLInputElement).blur()"
      type="number"
      :min="min"
      :max="max"
      class="w-20 px-2 py-0.5 text-xs text-white font-mono bg-gray-800 border border-gray-600 rounded text-center focus:border-blue-500 focus:outline-none"
    />
    <div class="text-[0.6rem] text-gray-500">Hz</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

interface Props {
  modelValue: number
  min?: number
  max?: number
  label?: string
  color?: string
}

const props = withDefaults(defineProps<Props>(), {
  min: 20,
  max: 20000,
  label: 'Frequency',
  color: '#3b82f6'
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const isDragging = ref(false)
const startY = ref(0)
const startValue = ref(0)
const editableValue = ref(props.modelValue)
let rafId: number | null = null
let pendingValue: number | null = null

// Convert frequency to logarithmic position (0-1)
function freqToPosition(freq: number): number {
  const logMin = Math.log(props.min)
  const logMax = Math.log(props.max)
  const logFreq = Math.log(Math.max(props.min, Math.min(props.max, freq)))
  return (logFreq - logMin) / (logMax - logMin)
}

// Convert logarithmic position (0-1) to frequency
function positionToFreq(position: number): number {
  const logMin = Math.log(props.min)
  const logMax = Math.log(props.max)
  const logFreq = logMin + position * (logMax - logMin)
  return Math.round(Math.exp(logFreq))
}

// Calculate angle for the knob (270 degrees range)
const angle = computed(() => {
  const position = freqToPosition(props.modelValue)
  return -90 + (position * 270)
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

// Calculate arc path for value indicator
const arcPath = computed(() => {
  const startAngle = -90
  const endAngle = angle.value
  const radius = 26
  
  const start = polarToCartesian(30, 30, radius, startAngle)
  const end = polarToCartesian(30, 30, radius, endAngle)
  
  const largeArcFlag = endAngle - startAngle <= 180 ? 0 : 1
  
  return [
    'M', start.x, start.y,
    'A', radius, radius, 0, largeArcFlag, 1, end.x, end.y
  ].join(' ')
})

function polarToCartesian(cx: number, cy: number, r: number, angle: number) {
  const rad = (angle * Math.PI) / 180
  return {
    x: cx + r * Math.cos(rad),
    y: cy + r * Math.sin(rad)
  }
}

function startDrag(e: MouseEvent | TouchEvent) {
  isDragging.value = true
  startY.value = 'touches' in e ? e.touches[0].clientY : e.clientY
  startValue.value = props.modelValue
  
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
  document.addEventListener('touchmove', onDrag)
  document.addEventListener('touchend', stopDrag)
  
  e.preventDefault()
}

function onDrag(e: MouseEvent | TouchEvent) {
  if (!isDragging.value) return
  
  const currentY = 'touches' in e ? e.touches[0].clientY : e.clientY
  const deltaY = startY.value - currentY
  
  // Increased sensitivity: 200 pixels for full range
  const sensitivity = 200
  const deltaPosition = deltaY / sensitivity
  
  const currentPosition = freqToPosition(startValue.value)
  const newPosition = Math.max(0, Math.min(1, currentPosition + deltaPosition))
  const newFreq = positionToFreq(newPosition)
  
  // Use requestAnimationFrame to throttle updates
  pendingValue = newFreq
  
  if (rafId === null) {
    rafId = requestAnimationFrame(() => {
      if (pendingValue !== null) {
        editableValue.value = pendingValue
        emit('update:modelValue', pendingValue)
        pendingValue = null
      }
      rafId = null
    })
  }
}

function stopDrag() {
  isDragging.value = false
  
  // Cancel any pending animation frame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  
  // Emit final pending value
  if (pendingValue !== null) {
    editableValue.value = pendingValue
    emit('update:modelValue', pendingValue)
    pendingValue = null
  }
  
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchmove', onDrag)
  document.removeEventListener('touchend', stopDrag)
}

function onWheel(e: WheelEvent) {
  const currentPosition = freqToPosition(props.modelValue)
  const delta = e.deltaY > 0 ? -0.02 : 0.02 // 2% per scroll
  const newPosition = Math.max(0, Math.min(1, currentPosition + delta))
  const newFreq = positionToFreq(newPosition)
  
  // Use requestAnimationFrame to throttle updates
  pendingValue = newFreq
  
  if (rafId === null) {
    rafId = requestAnimationFrame(() => {
      if (pendingValue !== null) {
        editableValue.value = pendingValue
        emit('update:modelValue', pendingValue)
        pendingValue = null
      }
      rafId = null
    })
  }
}

function reset440Hz() {
  const freq = 440 // A4 note
  editableValue.value = freq
  emit('update:modelValue', freq)
}

function updateFromInput() {
  let value = editableValue.value
  
  // Clamp to min/max
  if (value < props.min) value = props.min
  if (value > props.max) value = props.max
  
  editableValue.value = value
  emit('update:modelValue', value)
}

// Watch for external changes
import { watch } from 'vue'
watch(() => props.modelValue, (newVal) => {
  editableValue.value = newVal
})

onMounted(() => {
  editableValue.value = props.modelValue
})

onUnmounted(() => {
  stopDrag()
})
</script>

<style scoped>
/* Remove number input arrows */
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

input[type="number"] {
  appearance: textfield;
  -moz-appearance: textfield;
}

.knob {
  user-select: none;
  -webkit-user-select: none;
}
</style>
