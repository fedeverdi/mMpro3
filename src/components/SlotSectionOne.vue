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
      <div 
        class="bg-gradient-to-b from-purple-900/20 to-gray-900 rounded-lg border-2 border-purple-600/50 p-2" 
        :class="{ 'flex-1 min-h-0 cursor-pointer hover:border-purple-400 transition-colors': isCollapsed }"
        @click="isCollapsed ? openMeterPopover('loudness', $event) : null"
      >
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
      <div 
        class="bg-gradient-to-b from-orange-900/20 to-gray-900 rounded-lg border-2 border-orange-600/50 p-2" 
        :class="{ 'flex-1 min-h-0 cursor-pointer hover:border-orange-400 transition-colors': isCollapsed }"
        @click="isCollapsed ? openMeterPopover('dynamicRange', $event) : null"
      >
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
      <div 
        class="bg-gradient-to-b from-cyan-900/20 to-gray-900 rounded-lg border-2 border-cyan-600/50 p-2" 
        :class="{ 'flex-1 min-h-0 cursor-pointer hover:border-cyan-400 transition-colors': isCollapsed }"
        @click="isCollapsed ? openMeterPopover('phaseCorrelation', $event) : null"
      >
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

      <!-- Stereo Width Card (always visible, changes layout based on collapsed state) -->
      <div 
        class="bg-gradient-to-b from-pink-900/20 to-gray-900 rounded-lg border-2 border-pink-600/50 p-2" 
        :class="{ 'flex-1 min-h-0 cursor-pointer hover:border-pink-400 transition-colors': isCollapsed }"
        @click="isCollapsed ? openMeterPopover('stereoWidth', $event) : null"
      >
        <StereoWidthMeter 
          v-show="audioEngine?.state.value.isRunning"
          :collapsed="isCollapsed"
          :width-percent="stereoWidthData?.widthPercent"
          :mid-rms="stereoWidthData?.midRms"
          :side-rms="stereoWidthData?.sideRms"
          :balance="stereoWidthData?.balance"
          @reset="resetStereoWidth"
        />
        
        <!-- Engine not running message -->
        <div v-show="!audioEngine?.state.value.isRunning" class="flex items-center justify-center" :class="{ 'h-full': isCollapsed, 'py-8': !isCollapsed }">
          <div class="text-xs text-gray-500 text-center" :class="{ 'transform -rotate-90': isCollapsed }">
            <div class="mb-2">⏸️</div>
            <div v-if="!isCollapsed">Audio engine<br/>not running</div>
          </div>
        </div>
      </div>

      <!-- Headroom Card (always visible, changes layout based on collapsed state) -->
      <div 
        class="bg-gradient-to-b from-amber-900/20 to-gray-900 rounded-lg border-2 border-amber-600/50 p-2" 
        :class="{ 'flex-1 min-h-0 cursor-pointer hover:border-amber-400 transition-colors': isCollapsed }"
        @click="isCollapsed ? openMeterPopover('headroom', $event) : null"
      >
        <HeadroomMeter 
          v-show="audioEngine?.state.value.isRunning"
          :collapsed="isCollapsed"
          :peak-l="headroomData?.peakL"
          :peak-r="headroomData?.peakR"
          :headroom-l="headroomData?.headroomL"
          :headroom-r="headroomData?.headroomR"
          :headroom-stereo="headroomData?.headroomStereo"
          @reset="resetHeadroom"
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

    <!-- Popover for single meter (when collapsed) -->
    <Teleport to="body">
      <div 
        v-if="popoverMeter !== null"
        class="fixed z-[9999] bg-gray-900 border-2 rounded-lg shadow-2xl p-3 w-64"
        :class="{
          'border-purple-600/70': popoverMeter === 'loudness',
          'border-orange-600/70': popoverMeter === 'dynamicRange',
          'border-cyan-600/70': popoverMeter === 'phaseCorrelation',
          'border-pink-600/70': popoverMeter === 'stereoWidth',
          'border-amber-600/70': popoverMeter === 'headroom'
        }"
        :style="{ left: popoverPosition.x + 'px', top: popoverPosition.y + 'px' }"
      >
        <!-- Close button -->
        <button 
          @click="closePopover"
          class="absolute -top-4 -right-3 w-6 h-6 bg-gray-800 hover:bg-red-600 rounded-full flex items-center justify-center text-gray-400 hover:text-white transition-colors shadow-lg border border-gray-700"
          title="Close"
        >
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>

        <!-- Meter content -->
        <LoudnessMeter 
          v-if="popoverMeter === 'loudness'"
          :collapsed="false"
          :momentary-lufs="loudnessData?.momentaryLufs"
          :short-term-lufs="loudnessData?.shortTermLufs"
          :integrated-lufs="loudnessData?.integratedLufs"
          :loudness-range-lu="loudnessData?.loudnessRangeLu"
          :true-peak-dbtp="loudnessData?.truePeakDbtp"
          @reset="resetLoudness"
        />

        <DynamicRangeMeter 
          v-if="popoverMeter === 'dynamicRange'"
          :collapsed="false"
          :peak-db-l="dynamicRangeData?.peakDbL"
          :peak-db-r="dynamicRangeData?.peakDbR"
          :rms-db-l="dynamicRangeData?.rmsDbL"
          :rms-db-r="dynamicRangeData?.rmsDbR"
          :dynamic-range-l="dynamicRangeData?.dynamicRangeL"
          :dynamic-range-r="dynamicRangeData?.dynamicRangeR"
          :dynamic-range-stereo="dynamicRangeData?.dynamicRangeStereo"
          @reset="resetDynamicRange"
        />

        <MasterPhaseCorrelationMeter 
          v-if="popoverMeter === 'phaseCorrelation'"
          :collapsed="false"
          :correlation="phaseCorrelationData?.correlation"
          :mono-compatible="phaseCorrelationData?.monoCompatible"
          @reset="resetPhaseCorrelation"
        />

        <StereoWidthMeter 
          v-if="popoverMeter === 'stereoWidth'"
          :collapsed="false"
          :width-percent="stereoWidthData?.widthPercent"
          :mid-rms="stereoWidthData?.midRms"
          :side-rms="stereoWidthData?.sideRms"
          :balance="stereoWidthData?.balance"
          @reset="resetStereoWidth"
        />

        <HeadroomMeter 
          v-if="popoverMeter === 'headroom'"
          :collapsed="false"
          :peak-l="headroomData?.peakL"
          :peak-r="headroomData?.peakR"
          :headroom-l="headroomData?.headroomL"
          :headroom-r="headroomData?.headroomR"
          :headroom-stereo="headroomData?.headroomStereo"
          @reset="resetHeadroom"
        />
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch, onMounted, onUnmounted } from 'vue'
import LoudnessMeter from './master/LoudnessMeter.vue'
import DynamicRangeMeter from './master/DynamicRangeMeter.vue'
import MasterPhaseCorrelationMeter from './master/MasterPhaseCorrelationMeter.vue'
import StereoWidthMeter from './master/StereoWidthMeter.vue'
import HeadroomMeter from './master/HeadroomMeter.vue'

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Collapse state
const isCollapsed = ref(true) // Start collapsed
const expandedWidth = 220
const collapsedWidth = 20

// Popover state
type MeterType = 'loudness' | 'dynamicRange' | 'phaseCorrelation' | 'stereoWidth' | 'headroom' | null
const popoverMeter = ref<MeterType>(null)
const popoverPosition = ref({ x: 0, y: 0 })

const sectionWidth = computed(() => {
  return isCollapsed.value ? collapsedWidth : expandedWidth
})

// Loudness data
const loudnessData = computed(() => audioEngine?.state.value.loudnessData)

// Dynamic Range data
const dynamicRangeData = computed(() => audioEngine?.state.value.dynamicRangeData)

// Phase Correlation data
const phaseCorrelationData = computed(() => audioEngine?.state.value.phaseCorrelationData)

// Stereo Width data
const stereoWidthData = computed(() => audioEngine?.state.value.stereoWidthData)

// Headroom data
const headroomData = computed(() => audioEngine?.state.value.headroomData)

// Note: All meter data is now pushed automatically from the audio engine
// every ~50ms via the Response::Levels stream. No polling is needed.
// The computed properties above are reactive and will update automatically.

// Toggle collapse/expand
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
  // Close popover when expanding section
  if (!isCollapsed.value) {
    popoverMeter.value = null
  }
}

// Open meter in popover
function openMeterPopover(meter: MeterType, event: MouseEvent) {
  if (!isCollapsed.value) return // Only when collapsed
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  
  // Estimate popover height based on meter type (approximate values)
  const popoverHeights: Record<Exclude<MeterType, null>, number> = {
    loudness: 400,
    dynamicRange: 350,
    phaseCorrelation: 280,
    stereoWidth: 380,
    headroom: 330
  }
  
  const estimatedHeight = meter ? popoverHeights[meter] : 300
  let yPosition = rect.top
  
  // Check if popover would overflow below viewport
  if (yPosition + estimatedHeight > window.innerHeight) {
    // Adjust to align bottom with viewport bottom (with 10px margin)
    yPosition = window.innerHeight - estimatedHeight + 20
    // Ensure it doesn't go above viewport top
    if (yPosition < 10) yPosition = 10
  }
  
  popoverPosition.value = {
    x: rect.left - 256 - 10, // 10px to the left (256px = w-64)
    y: yPosition
  }
  popoverMeter.value = meter
}

// Close popover
function closePopover() {
  popoverMeter.value = null
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

// Reset stereo width measurements
function resetStereoWidth() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetStereoWidth()
  }
}

// Reset headroom measurements
function resetHeadroom() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetHeadroom()
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
