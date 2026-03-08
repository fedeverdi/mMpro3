<template>
  <!-- Collapsed View: Vertical Title -->
  <div v-if="collapsed" class="stereo-width-meter w-full h-full flex items-center justify-center">
    <div class="text-[10px] font-bold text-pink-400 uppercase tracking-wider transform -rotate-90 whitespace-nowrap">
      Stereo Width
    </div>
  </div>

  <!-- Expanded View: Full Meter -->
  <div v-else class="stereo-width-meter w-full">
    <!-- Header with Reset Button -->
    <div class="flex items-center justify-between mb-2">
      <div class="text-[10px] font-bold text-pink-400 uppercase tracking-wider">Stereo Width</div>
      <button 
        @click="emit('reset')"
        class="text-[9px] px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded transition-colors"
        title="Reset stereo width measurements">
        Reset
      </button>
    </div>

    <!-- Measurements Display -->
    <div class="space-y-1.5">
      <!-- Width Percentage (Primary) -->
      <div class="measurement-row">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">Width</span>
          <span class="text-[12px] font-mono font-bold" :class="getWidthColor()">
            {{ formatPercent(widthPercent) }}
          </span>
        </div>
        <div class="h-2 bg-gray-800 rounded-full overflow-hidden">
          <div 
            class="h-full transition-all duration-200"
            :class="getWidthBarColor()"
            :style="{ width: getWidthBarWidth() + '%' }"
          ></div>
        </div>
      </div>

      <!-- Width Status -->
      <div class="px-2 py-1 rounded" :class="getStatusBackground()">
        <div class="text-[9px] font-semibold text-center" :class="getStatusTextColor()">
          {{ getStatusText() }}
        </div>
      </div>

      <!-- Divider -->
      <div class="border-t border-gray-700/50 my-1"></div>

      <!-- Mid/Side RMS -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Mid (Mono)</span>
        <span class="text-[9px] font-mono text-gray-300">
          {{ formatDb(midRms) }}
        </span>
      </div>

      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Side (Width)</span>
        <span class="text-[9px] font-mono text-gray-300">
          {{ formatDb(sideRms) }}
        </span>
      </div>

      <!-- L/R Balance -->
      <div class="mt-1">
        <div class="flex justify-between items-center mb-0.5">
          <span class="text-[9px] text-gray-400">L/R Balance</span>
          <span class="text-[9px] font-mono" :class="getBalanceColor()">
            {{ formatBalance(balance) }}
          </span>
        </div>
        <!-- Balance bar -->
        <div class="relative h-1 bg-gray-800 rounded-full overflow-hidden">
          <div class="absolute inset-0 flex">
            <div class="flex-1 bg-gradient-to-r from-blue-500 to-gray-600"></div>
            <div class="flex-1 bg-gradient-to-r from-gray-600 to-blue-500"></div>
          </div>
          <!-- Balance indicator -->
          <div 
            class="absolute top-0 bottom-0 w-0.5 bg-white shadow-lg transition-all duration-100"
            :style="{ left: getBalancePosition() + '%' }"
          ></div>
          <!-- Center marker -->
          <div class="absolute top-0 bottom-0 left-1/2 w-px bg-gray-400 opacity-70"></div>
        </div>
      </div>
    </div>

    <!-- Reference Guide -->
    <div class="mt-2 pt-2 border-t border-gray-700/50">
      <div class="text-[8px] text-gray-500 space-y-0.5">
        <div class="flex justify-between">
          <span>Normal Stereo:</span>
          <span class="text-green-400">80-120%</span>
        </div>
        <div class="flex justify-between">
          <span>Wide Stereo:</span>
          <span class="text-yellow-400">120-150%</span>
        </div>
        <div class="flex justify-between">
          <span>Mono/Narrow:</span>
          <span class="text-blue-400">&lt;80%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  widthPercent?: number
  midRms?: number
  sideRms?: number
  balance?: number
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  widthPercent: 100,
  midRms: -90,
  sideRms: -90,
  balance: 0,
  collapsed: false
})

const emit = defineEmits<{
  reset: []
}>()

// Format percentage
const formatPercent = (value: number | null | undefined): string => {
  if (value == null || !isFinite(value)) {
    return '0%'
  }
  return `${Math.round(value)}%`
}

// Format dB values
const formatDb = (db: number | null | undefined): string => {
  if (db == null || !isFinite(db) || db < -89) {
    return '-∞'
  }
  return `${db >= 0 ? '+' : ''}${db.toFixed(1)} dB`
}

// Format balance (-1 to +1 → L100 to R100)
const formatBalance = (bal: number | null | undefined): string => {
  if (bal == null || !isFinite(bal)) {
    return 'Center'
  }
  if (Math.abs(bal) < 0.05) return 'Center'
  if (bal < 0) return `L${Math.round(Math.abs(bal) * 100)}`
  return `R${Math.round(bal * 100)}`
}

// Get balance position on bar (map -1 to 0%, +1 to 100%)
const getBalancePosition = (): number => {
  if (props.balance == null || !isFinite(props.balance)) {
    return 50
  }
  return ((props.balance + 1.0) / 2.0) * 100
}

// Get width bar width (cap at 100% for display)
const getWidthBarWidth = (): number => {
  if (props.widthPercent == null || !isFinite(props.widthPercent)) {
    return 0
  }
  return Math.min(props.widthPercent, 100)
}

// Color coding for width
const getWidthColor = (): string => {
  const width = props.widthPercent
  if (width == null || !isFinite(width)) return 'text-gray-500'
  if (width >= 150) return 'text-red-400'      // Too wide
  if (width >= 120) return 'text-yellow-400'   // Wide
  if (width >= 80) return 'text-green-400'     // Normal
  if (width >= 50) return 'text-blue-400'      // Narrow
  return 'text-purple-400'                     // Very narrow/mono
}

const getWidthBarColor = (): string => {
  const width = props.widthPercent
  if (width == null || !isFinite(width)) return 'bg-gray-600'
  if (width >= 150) return 'bg-red-500'
  if (width >= 120) return 'bg-yellow-500'
  if (width >= 80) return 'bg-green-500'
  if (width >= 50) return 'bg-blue-500'
  return 'bg-purple-500'
}

// Status text based on width
const getStatusText = (): string => {
  const width = props.widthPercent
  if (width == null || !isFinite(width)) return 'No Signal'
  if (width >= 180) return '🚨 Extremely Wide'
  if (width >= 150) return '⚠️ Very Wide Stereo'
  if (width >= 120) return 'Wide Stereo'
  if (width >= 80) return '✓ Normal Stereo'
  if (width >= 50) return 'Narrow Stereo'
  if (width >= 20) return 'Very Narrow'
  return 'Near Mono'
}

const getStatusBackground = (): string => {
  const width = props.widthPercent
  if (width == null || !isFinite(width)) return 'bg-gray-800'
  if (width >= 150 || width < 50) return 'bg-yellow-900/30'
  if (width >= 80) return 'bg-green-900/30'
  return 'bg-blue-900/30'
}

const getStatusTextColor = (): string => {
  const width = props.widthPercent
  if (width == null || !isFinite(width)) return 'text-gray-400'
  if (width >= 150 || width < 50) return 'text-yellow-300'
  if (width >= 80) return 'text-green-300'
  return 'text-blue-300'
}

// Balance color
const getBalanceColor = (): string => {
  if (Math.abs(props.balance) < 0.1) return 'text-green-400'
  if (Math.abs(props.balance) < 0.3) return 'text-yellow-400'
  return 'text-red-400'
}
</script>
