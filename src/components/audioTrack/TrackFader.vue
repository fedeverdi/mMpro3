<template>
  <div class="track-fader-container flex flex-col items-center gap-1 w-14">
    <!-- Fader wrapper -->
    <div class="flex items-start gap-1" :style="{ height: trackHeight + 'px' }">
      <!-- Scale marks (left) -->
      <div class="relative flex-shrink-0 w-5 text-right pr-0.5 pb-3" :style="{ height: trackHeight + 'px' }">
        <div v-for="mark in scaleMarks" :key="mark.label" 
          class="absolute leading-none right-0"
          :class="[
            mark.label === '-∞' ? 'text-[10px] font-bold text-gray-500' : 'text-[8px] font-mono',
            {
              'text-green-500 font-bold': mark.value === 0,
              'text-orange-400': mark.value > 0 && mark.value < 6,
              'text-red-500 font-bold': mark.value >= 6,
              'text-gray-500': mark.value < 0 && mark.label !== '-∞'
            }
          ]"
          :style="{ bottom: mark.position + '%', transform: 'translateY(50%)' }">
          {{ mark.label }}
        </div>
      </div>
      
      <!-- Fader track -->
      <div 
        class="fader-track-wrapper relative cursor-ns-resize"
        :style="{ width: '24px', height: trackHeight + 'px' }"
        @mousedown="startDrag"
        @touchstart="startDrag"
        @wheel.prevent="onWheel"
        ref="trackRef"
      >
        <!-- Background track slot -->
        <div 
          class="absolute left-1/2 -translate-x-1/2 w-1 bg-gradient-to-b from-gray-900 via-gray-800 to-gray-900 shadow-inner"
          :style="{ height: trackHeight + 'px' }"
        >
          <!-- Center line detail -->
          <div class="absolute left-1/2 -translate-x-1/2 w-px h-full bg-gray-700"></div>
        </div>
        
        <!-- Scale ticks -->
        <div class="absolute inset-0 pointer-events-none">
          <div v-for="mark in scaleMarks" :key="mark.label" 
            class="absolute left-1/2 -translate-x-1/2 h-px"
            :class="{
              'w-3 bg-green-500': mark.value === 0,
              'w-2 bg-gray-600': mark.value !== 0
            }"
            :style="{ bottom: mark.position + '%' }">
          </div>
        </div>
        
        <!-- Fader cap/thumb -->
        <div
          class="fader-cap absolute left-1/2 -translate-x-1/2 cursor-grab active:cursor-grabbing"
          :class="{ 'scale-105': isDragging }"
          :style="{ 
            bottom: thumbPosition + 'px',
            width: '26px',
            height: '52px',
            transition: isDragging ? 'none' : 'bottom 0.4s ease-out, transform 0.1s ease'
          }"
        >
          <!-- Cap body with clean Tailwind design -->
          <div class="relative w-full h-full rounded overflow-hidden border-2 border-gray-600 bg-gray-400 shadow-lg">
            <!-- Subtle top highlight -->
            <div class="absolute top-0 left-0 right-0 h-1 bg-white/20"></div>
            
            <!-- Subtle bottom shadow -->
            <div class="absolute bottom-0 left-0 right-0 h-1 bg-black/20"></div>
            
            <!-- Central horizontal line -->
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-3/4 h-0.5 bg-gray-600/60 rounded-full"></div>
          </div>
          
          <!-- Cap shadow (projected on track) -->
          <div class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-6 h-3 bg-black/50 blur-md rounded-full"></div>
        </div>
      </div>
      
      <!-- Scale marks (right) - mirrored -->
      <div class="relative flex-shrink-0 w-5 text-left pl-0.5 pb-3" :style="{ height: trackHeight + 'px' }">
        <div v-for="mark in scaleMarks" :key="mark.label" 
          class="absolute leading-none left-0"
          :class="[
            mark.label === '-∞' ? 'text-[10px] font-bold text-gray-500' : 'text-[8px] font-mono',
            {
              'text-green-500 font-bold': mark.value === 0,
              'text-orange-400': mark.value > 0 && mark.value < 6,
              'text-red-500 font-bold': mark.value >= 6,
              'text-gray-500': mark.value < 0 && mark.label !== '-∞'
            }
          ]"
          :style="{ bottom: mark.position + '%', transform: 'translateY(50%)' }">
          {{ mark.label }}
        </div>
      </div>
    </div>
    
    <!-- Value display -->
    <div class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-gray-800/50 border border-gray-700"
      :class="{
        'text-green-400': modelValue === 0,
        'text-orange-400': modelValue > 0 && modelValue < 6,
        'text-red-400': modelValue >= 6,
        'text-gray-400': modelValue < 0
      }">
      {{ displayValue }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'

interface Props {
  modelValue: number // Value in dB (-90 to 12, where -90 = -∞ mute)
  trackHeight?: number
}

const props = withDefaults(defineProps<Props>(), {
  trackHeight: 200
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const trackRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
let trackRect: DOMRect | null = null
let rafId: number | null = null
let pendingValue: number | null = null

const min = -90 // -90 represents -∞ (mute)
const max = 12

function onWheel(e: WheelEvent) {
  e.preventDefault()
  
  const currentValue = props.modelValue
  const normalizedPosition = (currentValue - min) / (max - min)
  const sensitivityFactor = 1.0 - normalizedPosition
  
  const minSensitivity = 0.0002
  const maxSensitivity = 0.001
  const sensitivity = minSensitivity + (maxSensitivity - minSensitivity) * sensitivityFactor
  
  const range = max - min
  const delta = e.deltaY * range * sensitivity
  
  let newValue = currentValue + delta
  newValue = Math.max(min, Math.min(max, newValue))
  
  emit('update:modelValue', newValue)
}

// Logarithmic mapping with compressed low end
// Bottom 5% = -90 to -60dB (compressed), 5-75% = -60dB to 0dB, 75-100% = 0dB to +12dB
function dbToPosition(db: number): number {
  if (db <= -60) {
    // -90 to -60dB: Compressed into bottom 5% of fader
    return 0.01 + ((db + 90) / 30) * 0.04
  } else if (db <= 0) {
    // -60dB to 0dB: Map across 5% to 75% of fader (normal distribution)
    return 0.05 + ((db + 60) / 60) * 0.70
  } else {
    // 0dB to +12dB: Map across 75% to 100% of fader
    return 0.75 + (db / 12) * 0.25
  }
}

function positionToDb(position: number): number {
  if (position <= 0.05) {
    // Bottom 5% of fader: -90dB to -60dB (compressed zone)
    return -90 + ((position - 0.01) / 0.04) * 30
  } else if (position <= 0.75) {
    // 5% to 75% of fader: -60dB to 0dB
    return -60 + ((position - 0.05) / 0.70) * 60
  } else {
    // 75% to 100% of fader: 0dB to +12dB
    return ((position - 0.75) / 0.25) * 12
  }
}

const thumbPosition = computed(() => {
  const position = dbToPosition(props.modelValue)
  return position * props.trackHeight - 22.5 // -22.5 to center the 45px cap
})

const displayValue = computed(() => {
  if (props.modelValue <= -85) return '-∞'
  if (props.modelValue === 0) return '0.0'
  return props.modelValue >= 0 
    ? `+${props.modelValue.toFixed(1)}` 
    : `${props.modelValue.toFixed(1)}`
})

const scaleMarks = computed(() => {
  const marks = [12, 6, 0, -6, -12, -18, -24, -30, -40, -60, -90]
  return marks.map(mark => {
    const position = dbToPosition(mark)
    return {
      value: mark,
      position: position * 100,
      label: mark === -90 ? '-∞' : (mark >= 0 ? `+${mark}` : `${mark}`)
    }
  })
})

function startDrag(e: MouseEvent | TouchEvent) {
  e.preventDefault()
  isDragging.value = true
  
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
  const y = trackRect.bottom - clientY
  const position = Math.max(0, Math.min(1, y / props.trackHeight))
  
  let newValue = positionToDb(position)
  
  // Snap to 0dB (unity) when close
  if (Math.abs(newValue) < 0.5) {
    newValue = 0
  }
  
  newValue = Math.round(newValue * 10) / 10
  
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
  
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  
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
.track-fader-container {
  user-select: none;
  -webkit-user-select: none;
}

.fader-track-wrapper {
  touch-action: none;
}

.fader-cap {
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.fader-cap:active {
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.4));
}
</style>
