<template>
  <div class="fader-container flex flex-col items-center gap-2 w-16 pt-2">
    <!-- <div class="text-[0.525rem] text-gray-400 uppercase tracking-wide truncate w-full text-center">{{ label }}</div> -->
    
    <div class="flex items-start gap-1" :style="{ height: trackHeight + 'px' }">
      <!-- Scale labels (left side) -->
      <div class="relative flex-shrink-0 w-6 text-right" :style="{ height: trackHeight + 'px' }">
        <div v-for="mark in scaleMarks" :key="mark.label" 
          class="absolute text-[9px] font-mono text-gray-400 leading-none right-0"
          :class="{
            'text-green-400 font-semibold': mark.value === 0,
            'text-red-400': mark.value > 0
          }"
          :style="{ bottom: mark.position + '%', transform: 'translateY(50%)' }">
          {{ mark.label }}
        </div>
      </div>
      
      <!-- Fader track (center) -->
      <div 
        class="fader-track-wrapper relative cursor-pointer flex justify-center"
        :style="{ width: '32px', height: trackHeight + 'px' }"
        @mousedown="startDrag"
        @touchstart="startDrag"
        @wheel.prevent="onWheel"
        ref="trackRef"
      >
        <!-- Visual track (narrower) -->
        <div 
          class="fader-track-visual bg-gray-800 border border-gray-700"
          :style="{ width: '8px', height: (trackHeight + 2) + 'px' }"
        >
          <!-- Value fill -->
          <div 
            class="fader-fill absolute bottom-0 left-0 right-0 "
            :class="fillClass"
            :style="{ height: fillHeight + 'px' }"
          ></div>
        </div>
        
        <!-- Fader thumb -->
        <div
          class="fader-thumb absolute left-1/2 transform -translate-y-[0.75rem] -translate-x-1/2 w-7 h-10 rounded-lg shadow-lg border-2 cursor-grab active:cursor-grabbing"
          :class="[thumbClass, isDragging ? '' : '']"
          :style="{ bottom: thumbPosition + 'px' }"
        >
          <div class="w-full h-full flex items-center justify-center">
            <div class="w-4 h-0.5 bg-gray-800 rounded"></div>
          </div>
        </div>
        
        <!-- Scale marks -->
        <div class="absolute inset-0 pointer-events-none">
          <div v-for="mark in scaleMarks" :key="mark.label" 
            class="absolute left-0 w-2 h-px bg-gray-500"
            :style="{ bottom: mark.position + '%' }">
          </div>
        </div>
      </div>
    </div>
    
    <div class="text-[0.625rem] font-mono truncate w-full text-center" :class="valueClass">{{ displayValue }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'

interface Props {
  modelValue: number // Value in dB (-60 to 6)
  label?: string
  trackHeight?: number
  color?: 'blue' | 'green' | 'red'
}

const props = withDefaults(defineProps<Props>(), {
  label: 'Volume',
  trackHeight: 200,
  color: 'blue'
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const trackRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
let trackRect: DOMRect | null = null
let rafId: number | null = null
let pendingValue: number | null = null

const min = -60
const max = 12

function onWheel(e: WheelEvent) {
  e.preventDefault()
  
  // Variable sensitivity based on current value
  // High sensitivity at -60dB (bottom), low sensitivity at +12dB (top)
  // This creates a safer control as you approach the top
  const currentValue = props.modelValue
  
  // Map -60dB to +12dB → 1.0 to 0.0 (inverse relationship)
  // At -60dB: factor = 1.0 (high sensitivity)
  // At 0dB: factor = ~0.83
  // At +12dB: factor = 0.0 (low sensitivity)
  const normalizedPosition = (currentValue - min) / (max - min) // 0 to 1
  const sensitivityFactor = 1.0 - normalizedPosition // 1 to 0
  
  // Sensitivity range
  const minSensitivity = 0.0002 // Very precise at top (+12dB)
  const maxSensitivity = 0.001 // Faster at bottom (-60dB)
  
  const sensitivity = minSensitivity + (maxSensitivity - minSensitivity) * sensitivityFactor
  
  const range = max - min
  const delta = e.deltaY * range * sensitivity
  
  let newValue = currentValue + delta
  newValue = Math.max(min, Math.min(max, newValue))
  
  emit('update:modelValue', newValue)
}

// Logarithmic fader mapping for professional audio feel
// 0% = -60dB, 75% = 0dB, 100% = +12dB
function dbToPosition(db: number): number {
  if (db <= 0) {
    // Map -60dB to 0dB across 0% to 75% of fader
    return ((db + 60) / 60) * 0.75
  } else {
    // Map 0dB to +12dB across 75% to 100% of fader
    return 0.75 + (db / 12) * 0.25
  }
}

function positionToDb(position: number): number {
  if (position <= 0.75) {
    // Map 0% to 75% of fader to -60dB to 0dB
    return -60 + (position / 0.75) * 60
  } else {
    // Map 75% to 100% of fader to 0dB to +12dB
    return ((position - 0.75) / 0.25) * 12
  }
}

// Calculate thumb position (0 to trackHeight) - using logarithmic mapping
const thumbPosition = computed(() => {
  const position = dbToPosition(props.modelValue)
  return position * props.trackHeight - 16 // -16 to center the thumb
})

// Calculate fill height - using logarithmic mapping
const fillHeight = computed(() => {
  const position = dbToPosition(props.modelValue)
  return position * props.trackHeight
})

// Display value
const displayValue = computed(() => {
  return props.modelValue >= 0 
    ? `+${props.modelValue.toFixed(1)}dB` 
    : `${props.modelValue.toFixed(1)}dB`
})

// Scale marks with labels at key dB values - using logarithmic positions
const scaleMarks = computed(() => {
  const marks = [12, 6, 0, -6, -12, -18, -24, -30, -40, -50, -60]
  return marks.map(mark => {
    const position = dbToPosition(mark)
    return {
      value: mark,
      position: position * 100,
      label: mark >= 0 ? `+${mark}` : `${mark}`
    }
  })
})

// Color classes
const fillClass = computed(() => {
  const colors = {
    blue: 'bg-blue-600',
    green: 'bg-green-600',
    red: 'bg-red-600'
  }
  return colors[props.color]
})

const thumbClass = computed(() => {
  const colors = {
    blue: 'bg-blue-500 border-blue-700',
    green: 'bg-green-500 border-green-700',
    red: 'bg-red-500 border-red-700'
  }
  return colors[props.color]
})

const valueClass = computed(() => {
  const colors = {
    blue: 'text-blue-400',
    green: 'text-green-400',
    red: 'text-red-400'
  }
  return colors[props.color]
})

function startDrag(e: MouseEvent | TouchEvent) {
  e.preventDefault()
  isDragging.value = true
  
  // Cache the bounding rect at the start of drag
  if (trackRef.value) {
    trackRect = trackRef.value.getBoundingClientRect()
  }
  
  updateValue(e)
  
  document.addEventListener('mousemove', updateValue, { passive: false })
  document.addEventListener('touchmove', updateValue, { passive: false })
  document.addEventListener('mouseup', stopDrag)
  document.addEventListener('touchend', stopDrag)
}

function updateValue(e: MouseEvent | TouchEvent) {
  e.preventDefault()
  
  if (!trackRect) return
  
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY
  
  // Calculate position from bottom (0 to 1)
  const y = trackRect.bottom - clientY
  const position = Math.max(0, Math.min(1, y / props.trackHeight))
  
  // Convert position to dB using logarithmic mapping
  let newValue = positionToDb(position)
  
  // Snap to 0dB (unity gain) when close
  if (Math.abs(newValue) < 0.5) {
    newValue = 0
  }
  
  // Round to 0.1
  newValue = Math.round(newValue * 10) / 10
  
  // Use requestAnimationFrame to throttle updates
  pendingValue = newValue
  
  if (rafId === null) {
    rafId = requestAnimationFrame(() => {
      if (pendingValue !== null) {
        emit('update:modelValue', pendingValue)
        pendingValue = null
      }
      rafId = null
    })
  }
}

function stopDrag() {
  isDragging.value = false
  trackRect = null
  
  // Cancel any pending animation frame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  
  // Emit final pending value
  if (pendingValue !== null) {
    emit('update:modelValue', pendingValue)
    pendingValue = null
  }
  
  document.removeEventListener('mousemove', updateValue)
  document.removeEventListener('touchmove', updateValue)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchend', stopDrag)
}

onUnmounted(() => {
  stopDrag()
})
</script>

<style scoped>
.fader-container {
  user-select: none;
  -webkit-user-select: none;
}

.fader-track-wrapper {
  touch-action: none;
}

.fader-track-visual {
  position: relative;
}

.fader-thumb {
  margin-bottom: -16px; /* Half of thumb height for proper centering */
}
</style>
