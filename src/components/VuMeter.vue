<template>
  <div class="vu-meter flex flex-col items-center gap-1">
    <div class="text-xs text-gray-500 uppercase">{{ label }}</div>
    
    <div 
      class="meter-bar relative bg-gray-900 border border-gray-700 rounded-sm overflow-hidden"
      :style="{ width: width + 'px', height: height + 'px' }"
    >
      <!-- Segments -->
      <div class="absolute inset-0 flex flex-col-reverse gap-0.5 p-1">
        <div 
          v-for="(segment, index) in segments" 
          :key="index"
          class="segment transition-all duration-75"
          :class="[
            segment.active ? segment.color : 'bg-gray-800',
            'rounded-sm'
          ]"
          :style="{ flex: '1' }"
        ></div>
      </div>
      
      <!-- Peak indicator -->
      <div 
        v-if="peakHold > 0"
        class="absolute left-0 right-0 h-1 bg-red-500"
        :style="{ bottom: (peakHold * 100) + '%' }"
      ></div>
    </div>
    
    <div v-if="showValue" class="w-12 text-center text-[10px] font-mono truncate" :class="levelClass">
      {{ displayLevel }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'

interface Props {
  level: number // Level in dB (-60 to 0)
  label?: string
  width?: number
  height?: number
  segments?: number
  showValue?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  label: 'L',
  width: 30,
  height: 200,
  segments: 20,
  showValue: true
})

const peakHold = ref(0)
const peakTimer = ref<number | null>(null)

// Create segments with appropriate colors
const segments = computed(() => {
  const segmentCount = props.segments
  const result = []
  
  // Normalize level from dB (-60 to 0) to 0-1
  const normalizedLevel = Math.max(0, Math.min(1, (props.level + 60) / 60))
  const activeSegments = Math.floor(normalizedLevel * segmentCount)
  
  for (let i = 0; i < segmentCount; i++) {
    const position = i / segmentCount
    let color = 'bg-green-500'
    
    // Color zones:
    // 0-60%: Green
    // 60-85%: Yellow
    // 85-100%: Red
    if (position > 0.85) {
      color = 'bg-red-500'
    } else if (position > 0.6) {
      color = 'bg-yellow-500'
    }
    
    result.push({
      active: i < activeSegments,
      color
    })
  }
  
  return result
})

// Display level
const displayLevel = computed(() => {
  if (props.level <= -60) return '-∞'
  return props.level >= 0 ? '0dB' : `${props.level.toFixed(1)}dB`
})

// Color class based on level
const levelClass = computed(() => {
  if (props.level > -6) return 'text-red-400'
  if (props.level > -20) return 'text-yellow-400'
  return 'text-green-400'
})

// Peak hold logic
watch(() => props.level, (newLevel) => {
  const normalized = Math.max(0, Math.min(1, (newLevel + 60) / 60))
  
  if (normalized > peakHold.value) {
    peakHold.value = normalized
    
    // Clear existing timer
    if (peakTimer.value !== null) {
      clearTimeout(peakTimer.value)
    }
    
    // Hold peak for 2 seconds, then decay
    peakTimer.value = window.setTimeout(() => {
      const decayInterval = setInterval(() => {
        peakHold.value = Math.max(0, peakHold.value - 0.01)
        if (peakHold.value <= 0) {
          clearInterval(decayInterval)
        }
      }, 50)
    }, 2000)
  }
})

onUnmounted(() => {
  if (peakTimer.value !== null) {
    clearTimeout(peakTimer.value)
  }
})
</script>

<style scoped>
.vu-meter {
  user-select: none;
  -webkit-user-select: none;
}

.segment {
  min-height: 2px;
}
</style>
