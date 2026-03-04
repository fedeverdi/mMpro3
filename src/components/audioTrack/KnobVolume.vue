<template>
  <div class="knob-container flex flex-col items-center gap-1">
    <div 
      class="knob relative cursor-pointer"
      @mousedown="startDrag"
      @touchstart="startDrag"
      @dblclick="resetToZero"
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
          :stroke="volumeColor"
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
          :stroke="volumeColor"
          stroke-width="2"
          stroke-linecap="round"
        />
        
        <!-- Center dot -->
        <circle
          cx="30"
          cy="30"
          r="3"
          :fill="volumeColor"
        />
      </svg>
    </div>
    <div 
      v-if="!isEditing" 
      class="text-xs font-mono cursor-pointer hover:bg-gray-700 px-2 py-0.5 rounded"
      :class="valueColorClass"
      @click="startEditing"
      title="Click to edit value"
    >
      {{ displayValue }}
    </div>
    <input
      v-else
      ref="editInput"
      type="text"
      v-model="editValue"
      class="text-xs text-white font-mono bg-gray-700 border border-blue-500 rounded px-2 py-0.5 text-center w-16"
      @keydown.enter="finishEditing"
      @keydown.escape="cancelEditing"
      @blur="finishEditing"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted, nextTick } from 'vue'

interface Props {
  modelValue: number // -60 to +12 dB
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const isDragging = ref(false)
const startY = ref(0)
const startValue = ref(0)
const isEditing = ref(false)
const editValue = ref('')
const editInput = ref<HTMLInputElement | null>(null)

// Min/Max values for volume in dB
const MIN_DB = -60
const MAX_DB = 12

// Calculate color based on volume level
const volumeColor = computed(() => {
  const value = props.modelValue
  if (value <= -40) return '#666' // Very low - gray
  if (value <= -12) return '#3b82f6' // Low - blue
  if (value <= 0) return '#22c55e' // Normal - green
  if (value <= 6) return '#eab308' // Hot - yellow
  return '#ef4444' // Clipping - red
})

// Calculate text color class
const valueColorClass = computed(() => {
  const value = props.modelValue
  if (value <= -40) return 'text-gray-500'
  if (value <= -12) return 'text-blue-400'
  if (value <= 0) return 'text-green-500'
  if (value <= 6) return 'text-yellow-400'
  return 'text-red-500'
})

// Calculate angle (270 degrees range, -60dB at bottom, +12dB at top)
const angle = computed(() => {
  const value = props.modelValue
  const range = MAX_DB - MIN_DB
  const normalized = (value - MIN_DB) / range
  return -90 + (normalized * 270)
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
  const bottomAngle = -90 + 270 // Bottom position (min value)
  const currentAngle = angle.value
  const radius = 26
  
  const start = polarToCartesian(30, 30, radius, bottomAngle)
  const end = polarToCartesian(30, 30, radius, currentAngle)
  
  const angleDiff = Math.abs(currentAngle - bottomAngle)
  const largeArcFlag = angleDiff > 180 ? 1 : 0
  
  return [
    'M', start.x, start.y,
    'A', radius, radius, 0, largeArcFlag, 0, end.x, end.y
  ].join(' ')
})

// Display value
const displayValue = computed(() => {
  const value = props.modelValue
  if (value <= -60) return '-∞'
  return value > 0 ? `+${value.toFixed(1)}` : value.toFixed(1)
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
  
  const sensitivity = 0.15 // dB per pixel
  const delta = deltaY * sensitivity
  
  let newValue = startValue.value + delta
  
  // Clamp between MIN_DB and MAX_DB
  newValue = Math.max(MIN_DB, Math.min(MAX_DB, newValue))
  
  // Round to 0.1 dB
  newValue = Math.round(newValue * 10) / 10
  
  emit('update:modelValue', newValue)
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('touchmove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchend', stopDrag)
}

function resetToZero() {
  emit('update:modelValue', 0)
}

function onWheel(e: WheelEvent) {
  e.preventDefault()
  
  const delta = -e.deltaY * 0.05 // 0.5 dB per wheel tick
  
  let newValue = props.modelValue + delta
  
  // Clamp between MIN_DB and MAX_DB
  newValue = Math.max(MIN_DB, Math.min(MAX_DB, newValue))
  
  // Round to 0.1 dB
  newValue = Math.round(newValue * 10) / 10
  
  emit('update:modelValue', newValue)
}

function startEditing() {
  isEditing.value = true
  editValue.value = props.modelValue.toFixed(1)
  nextTick(() => {
    editInput.value?.select()
  })
}

function finishEditing() {
  const parsed = parseFloat(editValue.value)
  if (!isNaN(parsed)) {
    let newValue = Math.max(MIN_DB, Math.min(MAX_DB, parsed))
    newValue = Math.round(newValue * 10) / 10
    emit('update:modelValue', newValue)
  }
  isEditing.value = false
}

function cancelEditing() {
  isEditing.value = false
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
