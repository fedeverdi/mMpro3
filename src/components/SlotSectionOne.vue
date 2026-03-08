<template>
  <div 
    class="flex flex-col h-full gap-2 relative transition-all duration-300 ease-out rounded-lg overflow-hidden"
    :style="{ width: sectionWidth + 'px' }"
  >
    <!-- Collapse/Expand Button -->
    <button
      @click="toggleCollapse"
      class="absolute top-2 right-2 z-50 p-1.5 rounded bg-gray-800/80 hover:bg-gray-700/80 transition-colors"
      :title="isCollapsed ? 'Expand Meters Panel' : 'Collapse Meters Panel'"
    >
      <svg 
        class="w-3 h-3 text-gray-400 transition-transform" 
        :class="{ 'rotate-180': isCollapsed }" 
        fill="currentColor" 
        viewBox="0 0 20 20"
      >
        <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
      </svg>
    </button>

    <!-- Content (hidden when collapsed) -->
    <div v-show="!isCollapsed" class="flex-1 pb-2 px-0 pt-0 overflow-y-auto space-y-2">
      <!-- LUFS Metering Card -->
      <div class="bg-gradient-to-b from-purple-900/20 to-gray-900 rounded-lg border-2 border-purple-600/50 p-2">
        <!-- Loudness Meter -->
        <LoudnessMeter 
          v-show="audioEngine?.state.value.isRunning"
          :momentary-lufs="loudnessData?.momentaryLufs"
          :short-term-lufs="loudnessData?.shortTermLufs"
          :integrated-lufs="loudnessData?.integratedLufs"
          :loudness-range-lu="loudnessData?.loudnessRangeLu"
          :true-peak-dbtp="loudnessData?.truePeakDbtp"
          @reset="resetLoudness"
        />
        
        <!-- Engine not running message -->
        <div v-show="!audioEngine?.state.value.isRunning" class="flex items-center justify-center py-8">
          <div class="text-xs text-gray-500 text-center">
            <div class="mb-2">⏸️</div>
            <div>Audio engine<br/>not running</div>
          </div>
        </div>
      </div>

      <!-- Future cards will go here -->
      <!-- Example:
      <div class="bg-gradient-to-b from-blue-900/20 to-gray-900 rounded-lg border-2 border-blue-600/50 p-2">
        <div class="text-xs text-gray-400">Another meter/tool</div>
      </div>
      -->
    </div>

    <!-- Collapsed state indicator -->
    <div v-if="isCollapsed" class="flex-1 flex flex-col items-center justify-center gap-2 text-gray-400 text-xs">
      <div class="transform -rotate-90 whitespace-nowrap font-mono">
        <div v-if="audioEngine?.state.value.isRunning && loudnessData">
          {{ formatLufs(loudnessData.momentaryLufs) }}
        </div>
        <div v-else class="text-gray-500">—</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch, onUnmounted } from 'vue'
import LoudnessMeter from './master/LoudnessMeter.vue'

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Collapse state
const isCollapsed = ref(false)
const expandedWidth = 220
const collapsedWidth = 40

const sectionWidth = computed(() => {
  return isCollapsed.value ? collapsedWidth : expandedWidth
})

// Loudness data
const loudnessData = computed(() => audioEngine?.state.value.loudnessData)

// Loudness polling
let loudnessPollingInterval: ReturnType<typeof setInterval> | null = null

const startLoudnessPolling = () => {
  if (loudnessPollingInterval) return // Already running
  loudnessPollingInterval = setInterval(() => {
    if (audioEngine?.state.value.isRunning) {
      audioEngine.getLoudness()
    }
  }, 100)
}

const stopLoudnessPolling = () => {
  if (loudnessPollingInterval) {
    clearInterval(loudnessPollingInterval)
    loudnessPollingInterval = null
  }
}

// Watch engine state to manage polling lifecycle
watch(
  () => audioEngine?.state.value.isRunning,
  (isRunning) => {
    if (isRunning) {
      startLoudnessPolling()
    } else {
      stopLoudnessPolling()
    }
  },
  { immediate: true }
)

// Cleanup on unmount
onUnmounted(() => {
  stopLoudnessPolling()
})

// Toggle collapse/expand
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

// Reset loudness measurements
function resetLoudness() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetLoudness()
  }
}

// Format LUFS values
const formatLufs = (lufs: number | null | undefined): string => {
  if (lufs == null || !isFinite(lufs) || lufs < -90) {
    return '-∞'
  }
  return lufs.toFixed(1)
}
</script>
