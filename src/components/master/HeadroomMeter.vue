<template>
  <!-- Collapsed View: Vertical Title -->
  <div v-if="collapsed" class="headroom-meter w-full h-full flex items-center justify-center">
    <div class="text-[10px] font-bold text-amber-400 uppercase tracking-wider transform -rotate-90 whitespace-nowrap">
      Headroom
    </div>
  </div>

  <!-- Expanded View: Full Meter -->
  <div v-else class="headroom-meter w-full">
    <!-- Header with Reset Button -->
    <div class="flex items-center justify-between mb-2">
      <div class="text-[10px] font-bold text-amber-400 uppercase tracking-wider">Headroom</div>
      <button 
        @click="emit('reset')"
        class="text-[9px] px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded transition-colors"
        title="Reset headroom measurements">
        Reset
      </button>
    </div>

    <!-- Measurements Display -->
    <div class="space-y-1.5">
      <!-- Headroom Stereo (Primary Metric) -->
      <div class="measurement-row">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">Headroom (min)</span>
          <span class="text-[12px] font-mono font-bold" :class="getHeadroomColor(headroomStereo)">
            {{ formatDb(headroomStereo) }}
          </span>
        </div>
        <!-- Headroom bar (inverse - less is worse) -->
        <div class="h-2 bg-gray-800 rounded-full overflow-hidden">
          <div 
            class="h-full transition-all duration-200"
            :class="getHeadroomBarColor(headroomStereo)"
            :style="{ width: getHeadroomBarWidth(headroomStereo) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Status Badge -->
      <div class="px-2 py-1 rounded" :class="getStatusBackground(headroomStereo)">
        <div class="text-[9px] font-semibold text-center" :class="getStatusTextColor(headroomStereo)">
          {{ getStatusText(headroomStereo) }}
        </div>
      </div>

      <!-- Divider -->
      <div class="border-t border-gray-700/50 my-1"></div>

      <!-- Peak Levels L/R -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Peak L</span>
        <span class="text-[9px] font-mono" :class="getPeakColor(peakL)">
          {{ formatDb(peakL) }}
        </span>
      </div>

      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Peak R</span>
        <span class="text-[9px] font-mono" :class="getPeakColor(peakR)">
          {{ formatDb(peakR) }}
        </span>
      </div>

      <!-- Divider -->
      <div class="border-t border-gray-700/50 my-1"></div>

      <!-- Individual Channel Headroom -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Headroom L</span>
        <span class="text-[9px] font-mono" :class="getHeadroomColor(headroomL)">
          {{ formatDb(headroomL) }}
        </span>
      </div>

      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Headroom R</span>
        <span class="text-[9px] font-mono" :class="getHeadroomColor(headroomR)">
          {{ formatDb(headroomR) }}
        </span>
      </div>
    </div>

    <!-- Reference Guide -->
    <div class="mt-2 pt-2 border-t border-gray-700/50">
      <div class="text-[8px] text-gray-500 space-y-0.5">
        <div class="flex justify-between">
          <span>Safe:</span>
          <span class="text-green-400">&gt;6 dB</span>
        </div>
        <div class="flex justify-between">
          <span>Caution:</span>
          <span class="text-yellow-400">3-6 dB</span>
        </div>
        <div class="flex justify-between">
          <span>Danger:</span>
          <span class="text-red-400">&lt;3 dB</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  peakL?: number
  peakR?: number
  headroomL?: number
  headroomR?: number
  headroomStereo?: number
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  peakL: -90,
  peakR: -90,
  headroomL: 90,
  headroomR: 90,
  headroomStereo: 90,
  collapsed: false
})

const emit = defineEmits<{
  reset: []
}>()

// Format dB values
const formatDb = (db: number | null | undefined): string => {
  if (db == null || !isFinite(db)) {
    return '-∞'
  }
  if (db < -89) return '-∞'
  if (db > 80) return '∞' // Extremely high headroom
  return `${db >= 0 ? '+' : ''}${db.toFixed(1)} dB`
}

// Headroom bar width (map 0-20dB to 0-100%)
const getHeadroomBarWidth = (headroom: number | null | undefined): number => {
  if (headroom == null || !isFinite(headroom) || headroom < 0) {
    return 0
  }
  // Map 0-20 dB to 0-100%
  return Math.min((headroom / 20.0) * 100, 100)
}

// Color coding for headroom (less headroom = worse)
const getHeadroomColor = (headroom: number | null | undefined): string => {
  if (headroom == null || !isFinite(headroom)) return 'text-gray-500'
  if (headroom < 1) return 'text-red-500 font-bold'       // Critical - clipping imminent
  if (headroom < 3) return 'text-red-400'                 // Danger zone
  if (headroom < 6) return 'text-yellow-400'              // Caution
  if (headroom < 12) return 'text-green-400'              // Safe
  return 'text-gray-400'                                  // Plenty of room
}

const getHeadroomBarColor = (headroom: number | null | undefined): string => {
  if (headroom == null || !isFinite(headroom)) return 'bg-gray-600'
  if (headroom < 1) return 'bg-red-600'
  if (headroom < 3) return 'bg-red-500'
  if (headroom < 6) return 'bg-yellow-500'
  if (headroom < 12) return 'bg-green-500'
  return 'bg-gray-500'
}

// Peak level color (closer to 0 dBFS = worse)
const getPeakColor = (peak: number | null | undefined): string => {
  if (peak == null || !isFinite(peak) || peak < -80) return 'text-gray-500'
  if (peak >= -1) return 'text-red-500 font-bold'         // Almost clipping
  if (peak >= -3) return 'text-red-400'                   // Very hot
  if (peak >= -6) return 'text-yellow-400'                // Hot
  if (peak >= -12) return 'text-green-400'                // Good level
  return 'text-gray-400'                                  // Quiet
}

// Status text based on headroom
const getStatusText = (headroom: number | null | undefined): string => {
  if (headroom == null || !isFinite(headroom)) return 'No Signal'
  if (headroom < 0.5) return '🚨 CLIPPING!'
  if (headroom < 1) return '🔥 Critical - Reduce Gain'
  if (headroom < 3) return '⚠️ Danger Zone'
  if (headroom < 6) return 'Caution'
  if (headroom < 12) return '✓ Safe Level'
  return 'Plenty of Headroom'
}

const getStatusBackground = (headroom: number | null | undefined): string => {
  if (headroom == null || !isFinite(headroom)) return 'bg-gray-800'
  if (headroom < 3) return 'bg-red-900/40'
  if (headroom < 6) return 'bg-yellow-900/30'
  if (headroom < 12) return 'bg-green-900/30'
  return 'bg-gray-800'
}

const getStatusTextColor = (headroom: number | null | undefined): string => {
  if (headroom == null || !isFinite(headroom)) return 'text-gray-400'
  if (headroom < 3) return 'text-red-300'
  if (headroom < 6) return 'text-yellow-300'
  if (headroom < 12) return 'text-green-300'
  return 'text-gray-400'
}
</script>
