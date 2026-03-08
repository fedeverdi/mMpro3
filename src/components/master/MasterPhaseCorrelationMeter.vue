<template>
  <!-- Collapsed View: Vertical Title -->
  <div v-if="collapsed" class="phase-correlation-meter w-full h-full flex items-center justify-center">
    <div class="text-[10px] font-bold text-cyan-400 uppercase tracking-wider transform -rotate-90 whitespace-nowrap">
      Phase Correlation
    </div>
  </div>

  <!-- Expanded View: Full Meter -->
  <div v-else class="phase-correlation-meter w-full">
    <!-- Header with Reset Button -->
    <!-- <div class="flex items-center justify-between mb-2">
      <div class="text-[10px] font-bold text-cyan-400 uppercase tracking-wider">Phase Correlation</div>
      <button 
        @click="emit('reset')"
        class="text-[9px] px-1.5 py-0.5 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded transition-colors"
        title="Reset phase correlation measurements">
        Reset
      </button>
    </div> -->

    <!-- Correlation Value Display -->
    <div class="space-y-2">
      <!-- Main Correlation Value -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Correlation</span>
        <span class="text-[12px] font-mono font-bold" :class="getCorrelationColor()">
          {{ formatCorrelation(correlation) }}
        </span>
      </div>

      <!-- Correlation Bar Meter -->
      <div class="relative h-2 bg-gray-800 rounded overflow-hidden">
        <!-- Background gradient (red on left, yellow center, green on right) -->
        <div class="absolute inset-0 flex">
          <div class="flex-1 bg-gradient-to-r from-red-600 to-red-400"></div>
          <div class="flex-1 bg-gradient-to-r from-red-400 via-yellow-500 to-green-400"></div>
          <div class="flex-1 bg-gradient-to-r from-green-400 to-green-600"></div>
        </div>
        
        <!-- Correlation indicator -->
        <div 
          class="absolute top-0 bottom-0 w-0.5 bg-white shadow-lg transition-all duration-100"
          :style="{ left: getCorrelationPosition() + '%' }"
        ></div>
        
        <!-- Center marker (0.0) -->
        <div class="absolute top-0 bottom-0 left-1/2 w-px bg-gray-400 opacity-50"></div>
      </div>

      <!-- Correlation scale labels -->
      <div class="flex justify-between text-[8px] text-gray-500">
        <span>-1.0</span>
        <span>0.0</span>
        <span>+1.0</span>
      </div>

      <!-- Divider -->
      <div class="border-t border-gray-700/50 my-1"></div>

      <!-- Mono Compatibility Status -->
      <div class="flex justify-between items-center">
        <span class="text-[9px] text-gray-400">Mono Compatible</span>
        <span class="text-[10px] font-bold" :class="monoCompatible ? 'text-green-400' : 'text-red-400'">
          {{ monoCompatible ? '✓ YES' : '✗ NO' }}
        </span>
      </div>

      <!-- Status Indicator -->
      <div class="px-2 py-1 rounded" :class="getStatusBackground()">
        <div class="text-[9px] font-semibold" :class="getStatusTextColor()">
          {{ getStatusText() }}
        </div>
      </div>
    </div>

    <!-- Reference Guide -->
    <div class="mt-2 pt-2 border-t border-gray-700/50">
      <div class="text-[8px] text-gray-500 space-y-0.5">
        <div class="flex justify-between">
          <span>Good (Stereo):</span>
          <span class="text-green-400">+0.7 to +1.0</span>
        </div>
        <div class="flex justify-between">
          <span>Caution:</span>
          <span class="text-yellow-400">0.0 to +0.7</span>
        </div>
        <div class="flex justify-between">
          <span>Phase Issues:</span>
          <span class="text-red-400">&lt; 0.0</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  correlation?: number
  monoCompatible?: boolean
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  correlation: 1.0,
  monoCompatible: true,
  collapsed: false
})

const emit = defineEmits<{
  reset: []
}>()

// Format correlation value
const formatCorrelation = (value: number | null | undefined): string => {
  if (value == null || !isFinite(value)) {
    return '0.00'
  }
  return value >= 0 ? `+${value.toFixed(2)}` : value.toFixed(2)
}

// Get correlation position on bar (map -1 to 0%, +1 to 100%)
const getCorrelationPosition = (): number => {
  if (props.correlation == null || !isFinite(props.correlation)) {
    return 50 // Center (0.0)
  }
  // Map -1.0 to 1.0 → 0% to 100%
  return ((props.correlation + 1.0) / 2.0) * 100
}

// Color coding for correlation value
const getCorrelationColor = (): string => {
  const corr = props.correlation
  if (corr == null || !isFinite(corr)) return 'text-gray-500'
  if (corr >= 0.7) return 'text-green-400'
  if (corr >= 0.0) return 'text-yellow-400'  
  return 'text-red-400'
}

// Get status text based on correlation
const getStatusText = (): string => {
  const corr = props.correlation
  if (corr == null || !isFinite(corr)) return 'No Signal'
  if (corr >= 0.9) return 'Excellent Stereo'
  if (corr >= 0.7) return 'Good Stereo'
  if (corr >= 0.3) return 'Wide Stereo'
  if (corr >= 0.0) return 'Very Wide - Check Mix'
  if (corr >= -0.3) return '⚠️ Phase Issues'
  return '🚨 Severe Phase Issues'
}

const getStatusBackground = (): string => {
  const corr = props.correlation
  if (corr == null || !isFinite(corr)) return 'bg-gray-800'
  if (corr >= 0.7) return 'bg-green-900/30'
  if (corr >= 0.0) return 'bg-yellow-900/30'
  return 'bg-red-900/30'
}

const getStatusTextColor = (): string => {
  const corr = props.correlation
  if (corr == null || !isFinite(corr)) return 'text-gray-400'
  if (corr >= 0.7) return 'text-green-300'
  if (corr >= 0.0) return 'text-yellow-300'
  return 'text-red-300'
}
</script>
