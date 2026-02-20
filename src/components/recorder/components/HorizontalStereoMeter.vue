<template>
  <div class="horizontal-stereo-meter flex flex-col gap-2">
    <!-- Left Channel -->
    <div class="flex items-center gap-2">
      <div class="text-[10px] text-gray-400 font-mono w-4">L</div>
      <div 
        class="meter-bar relative bg-gray-900 border border-gray-700 rounded-sm overflow-hidden flex-1"
        :style="{ height: height + 'px' }"
      >
        <!-- Segments -->
        <div class="absolute inset-0 flex gap-px p-0.5">
          <div 
            v-for="(segment, index) in leftSegments" 
            :key="index"
            class="segment transition-all duration-75"
            :class="[
              segment.active ? segment.color : 'bg-gray-800',
              'rounded-[1px]'
            ]"
            :style="{ flex: '1' }"
          ></div>
        </div>
        
        <!-- Peak indicator -->
        <div 
          v-if="leftPeakHold > 0"
          class="absolute top-0 bottom-0 w-1 bg-red-500"
          :style="{ left: (leftPeakHold * 100) + '%' }"
        ></div>
      </div>
    </div>

    <!-- Right Channel -->
    <div class="flex items-center gap-2">
      <div class="text-[10px] text-gray-400 font-mono w-4">R</div>
      <div 
        class="meter-bar relative bg-gray-900 border border-gray-700 rounded-sm overflow-hidden flex-1"
        :style="{ height: height + 'px' }"
      >
        <!-- Segments -->
        <div class="absolute inset-0 flex gap-px p-0.5">
          <div 
            v-for="(segment, index) in rightSegments" 
            :key="index"
            class="segment transition-all duration-75"
            :class="[
              segment.active ? segment.color : 'bg-gray-800',
              'rounded-[1px]'
            ]"
            :style="{ flex: '1' }"
          ></div>
        </div>
        
        <!-- Peak indicator -->
        <div 
          v-if="rightPeakHold > 0"
          class="absolute top-0 bottom-0 w-1 bg-red-500"
          :style="{ left: (rightPeakHold * 100) + '%' }"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue'

interface Props {
  leftLevel: number // Level in dB (-60 to 0)
  rightLevel: number // Level in dB (-60 to 0)
  width?: number
  height?: number
  segments?: number
}

const props = withDefaults(defineProps<Props>(), {
  width: 200,
  height: 8,
  segments: 20
})

const leftPeakHold = ref(0)
const rightPeakHold = ref(0)
const leftPeakTimer = ref<number | null>(null)
const rightPeakTimer = ref<number | null>(null)

// Create segments for left channel
const leftSegments = computed(() => {
  return createSegments(props.leftLevel)
})

// Create segments for right channel
const rightSegments = computed(() => {
  return createSegments(props.rightLevel)
})

function createSegments(level: number) {
  const segmentCount = props.segments
  const result = []
  
  // Normalize level from dB (-60 to 0) to 0-1
  const normalizedLevel = Math.max(0, Math.min(1, (level + 60) / 60))
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
}

// Peak hold logic for left channel
watch(() => props.leftLevel, (newLevel) => {
  updatePeakHold(newLevel, leftPeakHold, leftPeakTimer)
})

// Peak hold logic for right channel
watch(() => props.rightLevel, (newLevel) => {
  updatePeakHold(newLevel, rightPeakHold, rightPeakTimer)
})

function updatePeakHold(
  level: number,
  peakHoldRef: typeof leftPeakHold,
  peakTimerRef: typeof leftPeakTimer
) {
  const normalized = Math.max(0, Math.min(1, (level + 60) / 60))
  
  if (normalized > peakHoldRef.value) {
    peakHoldRef.value = normalized
    
    // Clear existing timer
    if (peakTimerRef.value !== null) {
      clearTimeout(peakTimerRef.value)
    }
    
    // Hold peak for 2 seconds, then decay
    peakTimerRef.value = window.setTimeout(() => {
      const decayInterval = setInterval(() => {
        peakHoldRef.value = Math.max(0, peakHoldRef.value - 0.01)
        if (peakHoldRef.value <= 0) {
          clearInterval(decayInterval)
        }
      }, 50)
    }, 2000)
  }
}

onUnmounted(() => {
  if (leftPeakTimer.value !== null) {
    clearTimeout(leftPeakTimer.value)
  }
  if (rightPeakTimer.value !== null) {
    clearTimeout(rightPeakTimer.value)
  }
})
</script>

<style scoped>
.horizontal-stereo-meter {
  user-select: none;
  -webkit-user-select: none;
}

.segment {
  min-width: 2px;
}
</style>
