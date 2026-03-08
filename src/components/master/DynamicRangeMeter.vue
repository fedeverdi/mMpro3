<template>
  <!-- Collapsed View: Vertical Title -->
  <div v-if="collapsed" class="dynamic-range-meter w-full h-full flex items-center justify-center">
    <div class="text-[10px] font-bold text-orange-400 uppercase tracking-wider transform -rotate-90 whitespace-nowrap">
      Dynamic Range
    </div>
  </div>

  <!-- Expanded View: Full Meters -->
  <div v-else class="dynamic-range-meter w-full">
    <!-- Header with Reset Button -->
    <div class="flex items-center justify-between mb-2">
      <div class="text-[10px] font-bold text-orange-400 uppercase tracking-wider">Dynamic Range</div>
      <button 
        @click="emit('reset')"
        class="text-[9px] px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded transition-colors"
        title="Reset dynamic range measurements">
        Reset
      </button>
    </div>

    <!-- Measurements Display -->
    <div class="space-y-1.5">
      <!-- Dynamic Range Stereo (Primary) -->
      <div class="measurement-row">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">DR (Stereo)</span>
          <span class="text-[11px] font-mono font-bold" :class="getDynamicRangeColor(dynamicRangeStereo)">
            {{ formatDb(dynamicRangeStereo) }}
          </span>
        </div>
        <div class="h-1.5 bg-gray-800 rounded-full overflow-hidden">
          <div 
            class="h-full transition-all duration-200"
            :class="getDynamicRangeBarColor(dynamicRangeStereo)"
            :style="{ width: getDynamicRangeWidth(dynamicRangeStereo) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Divider -->
      <div class="border-t border-gray-700/50 my-1"></div>

      <!-- Peak L/R -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Peak L/R</span>
        <span class="text-[9px] font-mono text-gray-300">
          {{ formatDb(peakDbL) }} / {{ formatDb(peakDbR) }}
        </span>
      </div>

      <!-- RMS L/R -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">RMS L/R</span>
        <span class="text-[9px] font-mono text-gray-300">
          {{ formatDb(rmsDbL) }} / {{ formatDb(rmsDbR) }}
        </span>
      </div>

      <!-- DR L/R Channels -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">DR L/R</span>
        <span class="text-[9px] font-mono" :class="getDynamicRangeColor((dynamicRangeL + dynamicRangeR) / 2)">
          {{ formatDb(dynamicRangeL) }} / {{ formatDb(dynamicRangeR) }}
        </span>
      </div>
    </div>

    <!-- Target Reference -->
    <div class="mt-2 pt-2 border-t border-gray-700/50">
      <div class="text-[8px] text-gray-500 space-y-0.5">
        <div class="flex justify-between">
          <span>Good Range:</span>
          <span class="text-green-400">&gt;10 dB</span>
        </div>
        <div class="flex justify-between">
          <span>Over-compressed:</span>
          <span class="text-red-400">&lt;6 dB</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  peakDbL?: number
  peakDbR?: number
  rmsDbL?: number
  rmsDbR?: number
  dynamicRangeL?: number
  dynamicRangeR?: number
  dynamicRangeStereo?: number
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  peakDbL: -90,
  peakDbR: -90,
  rmsDbL: -90,
  rmsDbR: -90,
  dynamicRangeL: 0,
  dynamicRangeR: 0,
  dynamicRangeStereo: 0,
  collapsed: false
})

const emit = defineEmits<{
  reset: []
}>()

// Format dB values
const formatDb = (db: number | null | undefined): string => {
  if (db == null || !isFinite(db) || db < -89) {
    return '-∞'
  }
  return `${db >= 0 ? '+' : ''}${db.toFixed(1)} dB`
}

// Get meter width percentage (map 0-20 dB to 0-100%)
const getDynamicRangeWidth = (dr: number | null | undefined): number => {
  if (dr == null || !isFinite(dr) || dr <= 0) return 0
  if (dr > 20) return 100
  return (dr / 20) * 100
}

// Color coding for dynamic range
// > 12 dB: Excellent (green)
// 10-12 dB: Good (light green)
// 6-10 dB: Acceptable (yellow)
// < 6 dB: Over-compressed (red)
const getDynamicRangeColor = (dr: number | null | undefined): string => {
  if (dr == null || !isFinite(dr) || dr <= 0) return 'text-gray-500'
  if (dr >= 12) return 'text-green-400'
  if (dr >= 10) return 'text-green-300'
  if (dr >= 6) return 'text-yellow-400'
  return 'text-red-400'
}

const getDynamicRangeBarColor = (dr: number | null | undefined): string => {
  if (dr == null || !isFinite(dr) || dr <= 0) return 'bg-gray-600'
  if (dr >= 12) return 'bg-green-500'
  if (dr >= 10) return 'bg-green-400'
  if (dr >= 6) return 'bg-yellow-500'
  return 'bg-red-500'
}
</script>
