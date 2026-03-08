<template>
  <div class="loudness-meter w-full">
    <!-- Header with Reset Button -->
    <div class="flex items-center justify-between mb-2">
      <div class="text-[10px] font-bold text-purple-400 uppercase tracking-wider">LUFS Metering</div>
      <button 
        @click="emit('reset')"
        class="text-[9px] px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded transition-colors"
        title="Reset loudness measurements">
        Reset
      </button>
    </div>

    <!-- Measurements Display -->
    <div class="space-y-1.5">
      <!-- Momentary Loudness (400ms) -->
      <div class="measurement-row">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">Momentary</span>
          <span class="text-[10px] font-mono font-bold" :class="getMomentaryColor()">
            {{ formatLufs(momentaryLufs) }}
          </span>
        </div>
        <div class="h-1 bg-gray-800 rounded-full overflow-hidden">
          <div 
            class="h-full transition-all duration-100"
            :class="getMomentaryBarColor()"
            :style="{ width: getMeterWidth(momentaryLufs) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Short-term Loudness (3s) -->
      <div class="measurement-row">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">Short-term</span>
          <span class="text-[10px] font-mono font-bold" :class="getShortTermColor()">
            {{ formatLufs(shortTermLufs) }}
          </span>
        </div>
        <div class="h-1 bg-gray-800 rounded-full overflow-hidden">
          <div 
            class="h-full transition-all duration-200"
            :class="getShortTermBarColor()"
            :style="{ width: getMeterWidth(shortTermLufs) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Integrated Loudness (gated) -->
      <div class="measurement-row">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">Integrated</span>
          <span class="text-[10px] font-mono font-bold" :class="getIntegratedColor()">
            {{ formatLufs(integratedLufs) }}
          </span>
        </div>
        <div class="h-1 bg-gray-800 rounded-full overflow-hidden">
          <div 
            class="h-full transition-all duration-500"
            :class="getIntegratedBarColor()"
            :style="{ width: getMeterWidth(integratedLufs) + '%' }"
          ></div>
        </div>
      </div>

      <!-- Divider -->
      <div class="border-t border-gray-700/50 my-1"></div>

      <!-- Loudness Range (LRA) -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Loudness Range</span>
        <span class="text-[10px] font-mono font-bold text-blue-400">
          {{ formatLra(loudnessRangeLu) }}
        </span>
      </div>

      <!-- True Peak -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">True Peak</span>
        <span class="text-[10px] font-mono font-bold" :class="getTruePeakColor()">
          {{ formatDbtp(truePeakDbtp) }}
        </span>
      </div>
    </div>

    <!-- Broadcast Standards Reference -->
    <div class="mt-2 pt-2 border-t border-gray-700/50">
      <div class="text-[8px] text-gray-500 space-y-0.5">
        <div class="flex justify-between">
          <span>EBU R128 Target:</span>
          <span class="text-gray-400">-23 LUFS</span>
        </div>
        <div class="flex justify-between">
          <span>Max True Peak:</span>
          <span class="text-gray-400">-1.0 dBTP</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  momentaryLufs?: number
  shortTermLufs?: number
  integratedLufs?: number
  loudnessRangeLu?: number
  truePeakDbtp?: number
}

const props = withDefaults(defineProps<Props>(), {
  momentaryLufs: -Infinity,
  shortTermLufs: -Infinity,
  integratedLufs: -Infinity,
  loudnessRangeLu: 0,
  truePeakDbtp: -Infinity
})

const emit = defineEmits<{
  reset: []
}>()

// Format LUFS values
const formatLufs = (lufs: number | null | undefined): string => {
  if (lufs == null || !isFinite(lufs) || lufs < -90) {
    return '-∞'
  }
  return lufs.toFixed(1)
}

// Format LRA values
const formatLra = (lra: number | null | undefined): string => {
  if (lra == null || !isFinite(lra) || lra <= 0) {
    return '0.0 LU'
  }
  return `${lra.toFixed(1)} LU`
}

// Format dBTP values
const formatDbtp = (dbtp: number | null | undefined): string => {
  if (dbtp == null || !isFinite(dbtp) || dbtp < -90) {
    return '-∞'
  }
  return `${dbtp >= 0 ? '+' : ''}${dbtp.toFixed(1)}`
}

// Get meter width percentage (map -60 to 0 LUFS to 0-100%)
const getMeterWidth = (lufs: number | null | undefined): number => {
  if (lufs == null || !isFinite(lufs) || lufs < -60) return 0
  if (lufs > 0) return 100
  return ((lufs + 60) / 60) * 100
}

// Color coding for momentary loudness
const getMomentaryColor = (): string => {
  const lufs = props.momentaryLufs
  if (!isFinite(lufs)) return 'text-gray-500'
  if (lufs > -3) return 'text-yellow-400'
  if (lufs > -18) return 'text-green-400'
  return 'text-gray-300'
}

const getMomentaryBarColor = (): string => {
  const lufs = props.momentaryLufs
  if (!isFinite(lufs)) return 'bg-gray-600'
  if (lufs > -3) return 'bg-yellow-500'
  if (lufs > -18) return 'bg-green-500'
  return 'bg-blue-500'
}

// Color coding for short-term loudness
const getShortTermColor = (): string => {
  const lufs = props.shortTermLufs
  if (!isFinite(lufs)) return 'text-gray-500'
  if (lufs > -3) return 'text-yellow-400'
  if (lufs > -18) return 'text-green-400'
  return 'text-gray-300'
}

const getShortTermBarColor = (): string => {
  const lufs = props.shortTermLufs
  if (!isFinite(lufs)) return 'bg-gray-600'
  if (lufs > -3) return 'bg-yellow-500'
  if (lufs > -18) return 'bg-green-500'
  return 'bg-blue-500'
}

// Color coding for integrated loudness (-23 LUFS target)
const getIntegratedColor = (): string => {
  const lufs = props.integratedLufs
  if (!isFinite(lufs)) return 'text-gray-500'
  const diff = Math.abs(lufs - (-23))
  if (diff < 1) return 'text-green-400'  // Very close to target
  if (lufs > -18) return 'text-yellow-400'  // Too loud
  return 'text-blue-400'  // Below target
}

const getIntegratedBarColor = (): string => {
  const lufs = props.integratedLufs
  if (!isFinite(lufs)) return 'bg-gray-600'
  const diff = Math.abs(lufs - (-23))
  if (diff < 1) return 'bg-green-500'
  if (lufs > -18) return 'bg-yellow-500'
  return 'bg-blue-500'
}

// Color coding for true peak (warning at -1 dBTP, critical at 0 dBTP)
const getTruePeakColor = (): string => {
  const dbtp = props.truePeakDbtp
  if (!isFinite(dbtp)) return 'text-gray-500'
  if (dbtp >= 0) return 'text-red-500'  // Clipping!
  if (dbtp > -1) return 'text-yellow-400'  // Warning
  return 'text-green-400'  // Safe
}
</script>
