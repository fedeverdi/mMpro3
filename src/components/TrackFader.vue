<template>
  <div class="track-fader-container flex flex-col items-center gap-1 w-14">
    <!-- Fader wrapper -->
    <div class="flex items-start gap-1" :style="{ height: trackHeight + 'px' }">
      <!-- Scale marks (left) -->
      <div class="relative flex-shrink-0 w-5 text-right pr-0.5" :style="{ height: trackHeight + 'px' }">
        <div v-for="mark in scaleMarks" :key="mark.label" 
          class="absolute text-[8px] font-mono leading-none right-0"
          :class="{
            'text-green-500 font-bold': mark.value === 0,
            'text-orange-400': mark.value > 0 && mark.value < 6,
            'text-red-500 font-bold': mark.value >= 6,
            'text-gray-500': mark.value < 0
          }"
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
            height: '55px',
            transition: isDragging ? 'none' : 'transform 0.1s ease'
          }"
        >
          <!-- Cap body with professional metallic look -->
          <div class="relative w-full h-full rounded-[3px] overflow-hidden border border-gray-700 shadow-xl">
            <!-- Main metallic gradient (aluminum/steel look) -->
            <div class="absolute inset-0 bg-gradient-to-r from-gray-400 via-gray-300 to-gray-400"></div>
            
            <!-- Central vertical highlight (brushed metal effect) -->
            <div class="absolute inset-y-0 left-1/2 -translate-x-1/2 w-2 bg-gradient-to-r from-transparent via-white/30 to-transparent"></div>
            
            <!-- Top bevel/edge highlight -->
            <div class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-b from-white/30 via-white/10 to-transparent"></div>
            
            <!-- Bottom bevel/edge shadow -->
            <div class="absolute bottom-0 left-0 right-0 h-1.5 bg-gradient-to-t from-black/25 via-black/10 to-transparent"></div>
            
            <!-- Left edge shadow (3D depth) -->
            <div class="absolute left-0 top-0 bottom-0 w-px bg-gradient-to-r from-black/20 to-transparent"></div>
            
            <!-- Right edge highlight (3D depth) -->
            <div class="absolute right-0 top-0 bottom-0 w-px bg-gradient-to-l from-white/20 to-transparent"></div>
            
            <!-- Grip lines container with inset effect -->
            <div class="absolute inset-0 flex flex-col justify-center items-center gap-[3px] px-2 py-3">
              <div v-for="i in 10" :key="i" class="w-full relative">
                <!-- Groove shadow (top) -->
                <div class="w-full h-[1px] bg-black/40"></div>
                <!-- Groove highlight (bottom) -->
                <div class="w-full h-[1px] bg-white/20 -mt-[1px]"></div>
              </div>
            </div>
            
            <!-- Central horizontal groove (engraved effect) -->
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-3/4">
              <!-- Upper shadow of groove -->
              <div class="w-full h-[2px] bg-black/50"></div>
              <!-- Lower highlight of groove -->
              <div class="w-full h-[1px] bg-white/40"></div>
            </div>
            
            <!-- Corner highlights for extra depth -->
            <div class="absolute top-1 left-1 w-1 h-1 bg-white/25 rounded-full blur-[0.5px]"></div>
            <div class="absolute top-1 right-1 w-1 h-1 bg-white/20 rounded-full blur-[0.5px]"></div>
            
            <!-- Inner border for definition -->
            <div class="absolute inset-[1px] rounded-[2px] border border-white/10"></div>
          </div>
          
          <!-- Cap shadow (projected on track) -->
          <div class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-6 h-3 bg-black/50 blur-md rounded-full"></div>
        </div>
      </div>
      
      <!-- Scale marks (right) - mirrored -->
      <div class="relative flex-shrink-0 w-5 text-left pl-0.5" :style="{ height: trackHeight + 'px' }">
        <div v-for="mark in scaleMarks" :key="mark.label" 
          class="absolute text-[8px] font-mono leading-none left-0"
          :class="{
            'text-green-500 font-bold': mark.value === 0,
            'text-orange-400': mark.value > 0 && mark.value < 6,
            'text-red-500 font-bold': mark.value >= 6,
            'text-gray-500': mark.value < 0
          }"
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
  modelValue: number // Value in dB (-60 to 12)
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

const min = -60
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

// Logarithmic mapping: 0% = -60dB, 75% = 0dB, 100% = +12dB
function dbToPosition(db: number): number {
  if (db <= 0) {
    return ((db + 60) / 60) * 0.75
  } else {
    return 0.75 + (db / 12) * 0.25
  }
}

function positionToDb(position: number): number {
  if (position <= 0.75) {
    return -60 + (position / 0.75) * 60
  } else {
    return ((position - 0.75) / 0.25) * 12
  }
}

const thumbPosition = computed(() => {
  const position = dbToPosition(props.modelValue)
  return position * props.trackHeight - 22.5 // -22.5 to center the 45px cap
})

const displayValue = computed(() => {
  if (props.modelValue === 0) return '0.0'
  return props.modelValue >= 0 
    ? `+${props.modelValue.toFixed(1)}` 
    : `${props.modelValue.toFixed(1)}`
})

const scaleMarks = computed(() => {
  const marks = [12, 6, 0, -6, -12, -18, -24, -30, -40, -60]
  return marks.map(mark => {
    const position = dbToPosition(mark)
    return {
      value: mark,
      position: position * 100,
      label: mark >= 0 ? `+${mark}` : `${mark}`
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
  filter: drop-shadow(0 3px 6px rgba(0, 0, 0, 0.4)) drop-shadow(0 1px 2px rgba(0, 0, 0, 0.3));
}

.fader-cap:active {
  filter: drop-shadow(0 5px 10px rgba(0, 0, 0, 0.5)) drop-shadow(0 2px 4px rgba(0, 0, 0, 0.4));
}
</style>
