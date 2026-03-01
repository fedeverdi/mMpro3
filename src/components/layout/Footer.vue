<template>
  <footer class="bg-black/50 backdrop-blur-sm border-t border-gray-700 px-4 py-2">
    <div class="flex items-center justify-start gap-4">
      <template v-if="performanceStats">
        <div class="flex items-center gap-4 text-[10px] font-mono">
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">BUFFER:</span>
            <span class="text-gray-300">{{ performanceStats.bufferSize }}</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">SAMPLE RATE:</span>
            <span class="text-gray-300">{{ (performanceStats.sampleRate / 1000).toFixed(1) }}kHz</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">LATENCY:</span>
            <span class="text-gray-300">{{ performanceStats.latencyMs.toFixed(2) }}ms</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">CPU:</span>
            <span :class="cpuClass">
              {{ performanceStats.cpuPercent.toFixed(1) }}%
            </span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">AVG PROCESS:</span>
            <span class="text-gray-300">{{ performanceStats.avgProcessMs.toFixed(2) }}ms</span>
          </div>
        </div>
      </template>
      <template v-else>
        <span class="text-gray-500 text-[10px]">Waiting for audio engine...</span>
      </template>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface PerformanceStats {
  bufferSize: number
  sampleRate: number
  latencyMs: number
  cpuPercent: number
  avgProcessMs: number
}

const props = defineProps<{
  performanceStats?: PerformanceStats | null
}>()

const cpuClass = computed(() => {
  if (!props.performanceStats) return 'text-gray-300'
  const cpu = props.performanceStats.cpuPercent
  if (cpu > 80) return 'text-red-400 font-bold'
  if (cpu > 60) return 'text-yellow-400'
  return 'text-green-400'
})
</script>
