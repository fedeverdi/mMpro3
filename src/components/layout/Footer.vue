<template>
  <footer class="bg-black/20 backdrop-blur-sm border-t border-gray-700 px-4 py-2 relative z-50">
    <div class="flex items-center justify-between gap-4">
      <!-- Left: Performance Stats -->
      <div class="flex items-center gap-4">
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

      <!-- Center/Right: Recording Info, Version, License and Buttons -->
      <div class="flex items-center gap-3">
        <!-- License Badge -->
        <button 
          @click="showLicenseModal = true"
          class="flex items-center gap-1.5 text-[10px] font-mono px-2 py-0.5 rounded border transition-all"
          :class="licenseBadgeClass"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
          </svg>
          <span>{{ licenseType.toUpperCase() }}</span>
        </button>
        <div class="w-px h-4 bg-gray-600"></div>
        
        <!-- Version -->
        <div class="flex items-center gap-1.5 text-[10px] font-mono">
          <span class="text-gray-500">VERSION:</span>
          <span class="text-gray-300">{{ appVersion }}</span>
        </div>
        <div class="w-px h-4 bg-gray-600"></div>
        
        <!-- Recording Info (quando sta registrando) -->
        <div v-if="isRecording" class="flex items-center gap-4 text-[10px] font-mono">
          <div class="flex items-center gap-1.5">
            <div class="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
            <span class="text-red-400 font-bold">REC</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">TIME:</span>
            <span class="text-white font-semibold">{{ recordingTime }}</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">SIZE:</span>
            <span class="text-gray-300">{{ recordingFileSize }}</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
          <div class="flex items-center gap-1.5">
            <span class="text-gray-500">FREE:</span>
            <span class="text-gray-300">{{ availableDiskSpace }}</span>
          </div>
          <div class="w-px h-4 bg-gray-600"></div>
        </div>
        
        <!-- Audio Config Button -->
        <button @click="$emit('open-audio-config')"
          class="px-3 py-0.5 border border-gray-600 hover:border-cyan-500 hover:bg-cyan-500/10 rounded text-[0.6rem] font-semibold text-gray-300 hover:text-cyan-400 transition-all flex items-center gap-1.5">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          Audio Config
        </button>
        
        <!-- Signal Flow Button -->
        <button @click="$emit('open-audio-flow')"
          class="px-3 py-0.5 border border-gray-600 hover:border-purple-500 hover:bg-purple-500/10 rounded text-[0.6rem] font-semibold text-gray-300 hover:text-purple-400 transition-all flex items-center gap-1.5">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
          Signal Flow
        </button>
      </div>
    </div>

    <!-- License Modal -->
    <LicenseModal v-model="showLicenseModal" />
  </footer>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import LicenseModal from '@/components/LicenseModal.vue'
import { useLicense } from '@/composables/useLicense'

interface PerformanceStats {
  bufferSize: number
  sampleRate: number
  latencyMs: number
  cpuPercent: number
  avgProcessMs: number
}

const props = defineProps<{
  performanceStats?: PerformanceStats | null
  isRecording?: boolean
  recordingTime?: string
  recordingFileSize?: string
  availableDiskSpace?: string
}>()

defineEmits<{
  (e: 'open-audio-flow'): void
  (e: 'open-audio-config'): void
}>()

// License
const { licenseType } = useLicense()
const showLicenseModal = ref(false)

const licenseBadgeClass = computed(() => {
  const baseClasses = 'cursor-pointer hover:opacity-80'
  switch (licenseType.value) {
    case 'full':
      return `${baseClasses} bg-green-500/20 border-green-500/50 text-green-400`
    case 'medium':
      return `${baseClasses} bg-blue-500/20 border-blue-500/50 text-blue-400`
    case 'demo':
    default:
      return `${baseClasses} bg-gray-500/20 border-gray-500/50 text-gray-400`
  }
})

const cpuClass = computed(() => {
  if (!props.performanceStats) return 'text-gray-300'
  const cpu = props.performanceStats.cpuPercent
  if (cpu > 80) return 'text-red-400 font-bold'
  if (cpu > 60) return 'text-yellow-400'
  return 'text-green-400'
})

// App version
const appVersion = ref('...')

onMounted(async () => {
  try {
    const electronAPI = (window as any).electronAPI
    if (electronAPI && electronAPI.getAppVersion) {
      appVersion.value = await electronAPI.getAppVersion()
    }
  } catch (error) {
    console.error('Failed to load app version:', error)
  }
})
</script>
