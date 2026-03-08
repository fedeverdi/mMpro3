<template>
  <div 
    class="flex flex-col h-full gap-2 relative transition-all duration-300 ease-out rounded-lg overflow-hidden"
    :style="{ width: sectionWidth + 'px' }"
  >
    <!-- Collapse/Expand Button -->
    <button
      @click="toggleCollapse"
      class="absolute top-2 w-5 h-5 -translate-x-1/2 bg-gray-800 hover:bg-blue-600 border border-gray-700 hover:border-blue-500 rounded flex items-center justify-center transition-all shadow-lg"
      :class="{ 'left-1/2' : isCollapsed, 'left-3' : !isCollapsed }"
      :title="isCollapsed ? 'Expand Meters Panel' : 'Collapse Meters Panel'"
    >
      <svg 
        class="w-3 h-3 text-gray-400 transition-transform rotate-180" 
        :class="{ 'rotate-0': isCollapsed }" 
        fill="currentColor" 
        viewBox="0 0 20 20"
      >
        <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
      </svg>
    </button>

    <!-- Meters Container -->
    <div class="flex-1 p-0 overflow-y-auto flex flex-col" :class="{ 'space-y-2': !isCollapsed, 'space-y-1': isCollapsed }">
      <!-- LUFS Card -->
      <div class="bg-gradient-to-b from-purple-900/20 to-gray-900 rounded-lg border-2 border-purple-600/50 p-2" :class="{ 'flex-1 h-0': isCollapsed }">
        <LoudnessMeter 
          v-show="audioEngine?.state.value.isRunning"
          :collapsed="isCollapsed"
          :momentary-lufs="loudnessData?.momentaryLufs"
          :short-term-lufs="loudnessData?.shortTermLufs"
          :integrated-lufs="loudnessData?.integratedLufs"
          :loudness-range-lu="loudnessData?.loudnessRangeLu"
          :true-peak-dbtp="loudnessData?.truePeakDbtp"
          @reset="resetLoudness"
        />
        
        <!-- Engine not running message -->
        <div v-show="!audioEngine?.state.value.isRunning" class="flex items-center justify-center" :class="{ 'h-full': isCollapsed, 'py-8': !isCollapsed }">
          <div class="text-xs text-gray-500 text-center" :class="{ 'transform -rotate-90': isCollapsed }">
            <div class="mb-2">⏸️</div>
            <div v-if="!isCollapsed">Audio engine<br/>not running</div>
          </div>
        </div>
      </div>

      <!-- Dynamic Range Card (always visible, changes layout based on collapsed state) -->
      <div class="bg-gradient-to-b from-orange-900/20 to-gray-900 rounded-lg border-2 border-orange-600/50 p-2" :class="{ 'flex-1 h-0': isCollapsed }">
        <DynamicRangeMeter 
          v-show="audioEngine?.state.value.isRunning"
          :collapsed="isCollapsed"
          :peak-db-l="dynamicRangeData?.peakDbL"
          :peak-db-r="dynamicRangeData?.peakDbR"
          :rms-db-l="dynamicRangeData?.rmsDbL"
          :rms-db-r="dynamicRangeData?.rmsDbR"
          :dynamic-range-l="dynamicRangeData?.dynamicRangeL"
          :dynamic-range-r="dynamicRangeData?.dynamicRangeR"
          :dynamic-range-stereo="dynamicRangeData?.dynamicRangeStereo"
          @reset="resetDynamicRange"
        />
        
        <!-- Engine not running message -->
        <div v-show="!audioEngine?.state.value.isRunning" class="flex items-center justify-center" :class="{ 'h-full': isCollapsed, 'py-8': !isCollapsed }">
          <div class="text-xs text-gray-500 text-center" :class="{ 'transform -rotate-90': isCollapsed }">
            <div class="mb-2">⏸️</div>
            <div v-if="!isCollapsed">Audio engine<br/>not running</div>
          </div>
        </div>
      </div>

      <!-- Phase Correlation Card (always visible, changes layout based on collapsed state) -->
      <div class="bg-gradient-to-b from-cyan-900/20 to-gray-900 rounded-lg border-2 border-cyan-600/50 p-2" :class="{ 'flex-1 h-0': isCollapsed }">
        <MasterPhaseCorrelationMeter 
          v-show="audioEngine?.state.value.isRunning"
          :collapsed="isCollapsed"
          :correlation="phaseCorrelationData?.correlation"
          :mono-compatible="phaseCorrelationData?.monoCompatible"
          @reset="resetPhaseCorrelation"
        />
        
        <!-- Engine not running message -->
        <div v-show="!audioEngine?.state.value.isRunning" class="flex items-center justify-center" :class="{ 'h-full': isCollapsed, 'py-8': !isCollapsed }">
          <div class="text-xs text-gray-500 text-center" :class="{ 'transform -rotate-90': isCollapsed }">
            <div class="mb-2">⏸️</div>
            <div v-if="!isCollapsed">Audio engine<br/>not running</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch, onUnmounted } from 'vue'
import LoudnessMeter from './master/LoudnessMeter.vue'
import DynamicRangeMeter from './master/DynamicRangeMeter.vue'
import MasterPhaseCorrelationMeter from './master/MasterPhaseCorrelationMeter.vue'

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Collapse state
const isCollapsed = ref(false)
const expandedWidth = 220
const collapsedWidth = 20

const sectionWidth = computed(() => {
  return isCollapsed.value ? collapsedWidth : expandedWidth
})

// Loudness data
const loudnessData = computed(() => audioEngine?.state.value.loudnessData)

// Dynamic Range data
const dynamicRangeData = computed(() => audioEngine?.state.value.dynamicRangeData)

// Phase Correlation data
const phaseCorrelationData = computed(() => audioEngine?.state.value.phaseCorrelationData)

// Loudness polling
let loudnessPollingInterval: ReturnType<typeof setInterval> | null = null

const startLoudnessPolling = () => {
  if (loudnessPollingInterval) return // Already running
  loudnessPollingInterval = setInterval(() => {
    if (audioEngine?.state.value.isRunning) {
      audioEngine.getLoudness()
      audioEngine.getDynamicRange()
      audioEngine.getPhaseCorrelation()
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

// Reset dynamic range measurements
function resetDynamicRange() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetDynamicRange()
  }
}

// Reset phase correlation measurements
function resetPhaseCorrelation() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetPhaseCorrelation()
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
