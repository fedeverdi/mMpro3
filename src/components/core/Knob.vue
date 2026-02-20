<template>
  <div class="knob-container flex flex-col items-center gap-1">
    <div class="text-xs text-gray-400 uppercase tracking-wide">{{ label }}</div>
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
    <div 
      v-if="!isEditing" 
      class="text-xs text-white font-mono cursor-pointer hover:bg-gray-700 px-2 py-0.5 rounded"
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
      class="text-xs text-white font-mono bg-gray-700 border border-blue-500 rounded px-2 py-0.5 text-center w-20"
      @keydown.enter="finishEditing"
      @keydown.escape="cancelEditing"
      @blur="finishEditing"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'

interface Props {
  modelValue: number
  min?: number
  max?: number
  step?: number
  label?: string
  unit?: string
  color?: string
  centerValue?: number // Optional: value that should appear at vertical position (-90deg)
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  step: 1,
  label: 'Knob',
  unit: '',
  color: '#3b82f6',
  centerValue: undefined
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const isDragging = ref(false)
const startY = ref(0)
const startValue = ref(0)
const isEditing = ref(false)
const editValue = ref('')
const editInput = ref<HTMLInputElement | null>(null)

// Calculate angle for the knob (270 degrees range)
const angle = computed(() => {
  const value = props.modelValue ?? props.min
  const range = props.max - props.min
  
  // If centerValue is specified, center the rotation around that value
  if (props.centerValue !== undefined) {
    const normalizedFromCenter = (value - props.centerValue) / range
    return -90 + (normalizedFromCenter * 270)
  }
  
  // Default: start at min value = -90deg (top), rotate 270deg total
  const normalized = (value - props.min) / range
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
  const centerAngle = -90 // Vertical position (top)
  const currentAngle = angle.value
  const radius = 26
  
  // For bipolar knobs (with centerValue), arc always starts from center (-90deg)
  if (props.centerValue !== undefined) {
    if (currentAngle >= centerAngle) {
      // Positive values: arc from center to current (clockwise)
      const start = polarToCartesian(30, 30, radius, centerAngle)
      const end = polarToCartesian(30, 30, radius, currentAngle)
      const largeArcFlag = currentAngle - centerAngle <= 180 ? 0 : 1
      return [
        'M', start.x, start.y,
        'A', radius, radius, 0, largeArcFlag, 1, end.x, end.y
      ].join(' ')
    } else {
      // Negative values: arc from current to center (counter-clockwise)
      const start = polarToCartesian(30, 30, radius, currentAngle)
      const end = polarToCartesian(30, 30, radius, centerAngle)
      const largeArcFlag = centerAngle - currentAngle <= 180 ? 0 : 1
      return [
        'M', start.x, start.y,
        'A', radius, radius, 0, largeArcFlag, 1, end.x, end.y
      ].join(' ')
    }
  }
  
  // Default behavior: arc from -90deg (top) to current angle
  const startAngle = -90
  const endAngle = currentAngle
  const start = polarToCartesian(30, 30, radius, endAngle)
  const end = polarToCartesian(30, 30, radius, startAngle)
  const largeArcFlag = endAngle - startAngle <= 180 ? 0 : 1
  
  return [
    'M', start.x, start.y,
    'A', radius, radius, 0, largeArcFlag, 0, end.x, end.y
  ].join(' ')
})

const displayValue = computed(() => {
  const value = props.modelValue ?? 0
  const val = value.toFixed(props.step < 1 ? 1 : 0)
  return props.unit ? `${val}${props.unit}` : val
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
  
  const range = props.max - props.min
  const sensitivity = range / 200 // Adjust sensitivity
  const delta = deltaY * sensitivity
  
  let newValue = startValue.value + delta
  newValue = Math.max(props.min, Math.min(props.max, newValue))
  
  // Apply step
  newValue = Math.round(newValue / props.step) * props.step
  
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
  // Reset to 0 if it's within valid range, otherwise clamp to nearest valid value
  let resetValue = 0
  resetValue = Math.max(props.min, Math.min(props.max, resetValue))
  
  // Apply step
  resetValue = Math.round(resetValue / props.step) * props.step
  
  emit('update:modelValue', resetValue)
}

function onWheel(e: WheelEvent) {
  e.preventDefault()
  
  // Use deltaY for vertical scroll (two-finger drag on trackpad)
  // Calculate sensitivity based on range for smooth control
  const range = props.max - props.min
  const sensitivity = range * 0.002 // 0.2% of range per deltaY unit
  const delta = e.deltaY * sensitivity
  
  let newValue = props.modelValue + delta
  newValue = Math.max(props.min, Math.min(props.max, newValue))
  
  // Don't apply step rounding on wheel - allow smooth continuous control
  
  emit('update:modelValue', newValue)
}

function startEditing() {
  isEditing.value = true
  // Remove unit from display value for editing
  editValue.value = props.modelValue.toFixed(props.step < 1 ? 3 : 0)
  nextTick(() => {
    editInput.value?.select()
  })
}

function finishEditing() {
  // Parse the input value
  const parsed = parseFloat(editValue.value)
  
  if (!isNaN(parsed)) {
    // Clamp to min/max
    let newValue = Math.max(props.min, Math.min(props.max, parsed))
    
    // Apply step
    newValue = Math.round(newValue / props.step) * props.step
    
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
