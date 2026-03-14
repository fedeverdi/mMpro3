<template>
  <div v-if="isOpen" class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/70" @click.self="close">
    <div class="bg-gray-900 rounded-lg shadow-2xl w-[500px] border border-gray-700">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-700">
        <h2 class="text-lg font-semibold text-white">⚙️ Audio Settings</h2>
        <button @click="close" class="text-gray-400 hover:text-white transition-colors">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-6 max-h-[70vh] overflow-y-auto">
        <!-- Sample Rate Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-3">
            Master Sample Rate
          </label>
          <div class="grid grid-cols-5 gap-2">
            <button
              v-for="rate in sampleRates"
              :key="rate.value"
              @click="selectedSampleRate = rate.value"
              :class="[
                'px-4 py-3 rounded-lg border-2 transition-all',
                selectedSampleRate === rate.value
                  ? 'bg-blue-600 border-blue-500 text-white'
                  : 'bg-gray-800 border-gray-700 text-gray-300 hover:border-gray-600'
              ]"
            >
              <div class="text-lg font-bold">{{ rate.label }}</div>
              <div class="text-xs opacity-75">{{ rate.description }}</div>
            </button>
          </div>
          <div class="mt-2 text-xs text-gray-400">
            💡 <span class="font-semibold">Auto</span>: adapts to device (48kHz for MacBook, 192kHz for Rubix). Manual: forces selected rate on all devices.
          </div>
        </div>

        <!-- Buffer Size Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-3">
            Buffer Size (Latency)
          </label>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="buffer in bufferSizes"
              :key="buffer.value"
              @click="selectedBufferSize = buffer.value"
              :class="[
                'px-4 py-3 rounded-lg border-2 transition-all text-left',
                selectedBufferSize === buffer.value
                  ? 'bg-blue-600 border-blue-500 text-white'
                  : 'bg-gray-800 border-gray-700 text-gray-300 hover:border-gray-600'
              ]"
            >
              <div class="font-bold">{{ buffer.value }} samples</div>
              <div class="text-xs opacity-75">~{{ buffer.latency }}ms</div>
              <div class="text-xs mt-1" :class="buffer.level === 'low' ? 'text-green-400' : buffer.level === 'medium' ? 'text-yellow-400' : 'text-orange-400'">
                {{ buffer.label }}
              </div>
            </button>
          </div>
          <div class="mt-2 text-xs text-gray-400">
            💡 Lower buffer = lower latency but higher CPU usage
          </div>
        </div>

        <!-- Network URL Section -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-3">
            Network Access URL
          </label>
          <div class="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden">
            <div class="p-4">
              <div v-if="!networkUrlLoading" class="flex items-center gap-3">
                <div class="flex-1 bg-gray-900 rounded px-3 py-2 font-mono text-sm text-blue-400 border border-gray-700">
                  {{ networkUrl || 'Loading...' }}
                </div>
                <button
                  @click="copyNetworkUrl"
                  :disabled="!networkUrl"
                  class="px-4 py-2 rounded bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed text-white transition-colors flex items-center gap-2"
                  :title="networkUrl ? 'Copy URL to clipboard' : 'URL not available'"
                >
                  <svg v-if="!urlCopied" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  <span>{{ urlCopied ? 'Copied!' : 'Copy' }}</span>
                </button>
              </div>
              <div v-else class="flex items-center justify-center py-2">
                <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-500"></div>
              </div>
              <div class="mt-3 text-xs text-gray-400">
                💡 Share this URL to access the application from other devices on your network
              </div>
              <div class="mt-1 text-xs text-gray-500">
                WebSocket server listening on port 3001 for remote control
              </div>
            </div>
          </div>
        </div>

        <!-- NDI Streaming Section -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-3">
            NDI Audio Streaming
          </label>
          <button
            @click="openNDISettings"
            class="w-full px-4 py-3 rounded-lg border-2 border-purple-600 bg-purple-900/20 hover:bg-purple-900/40 transition-all"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
                  <svg class="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
                  </svg>
                </div>
                <div class="text-left">
                  <div class="font-semibold text-white">Configure NDI Stream</div>
                  <div class="text-xs text-gray-400">Stream audio over network via NDI protocol</div>
                </div>
              </div>
              <svg class="w-5 h-5 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </div>
          </button>
        </div>

        <!-- Info Box -->
        <div class="bg-blue-900/30 border border-blue-700/50 rounded-lg p-4">
          <div class="text-sm text-blue-200">
            <div class="font-semibold mb-2">ℹ️ Professional Audio Configuration</div>
            <ul class="space-y-1 text-xs">
              <li>• <strong>Auto mode</strong>: adapts to each device's native rate</li>
              <li>• <strong>Manual mode</strong>: forces rate with resampling if needed</li>
              <li>• Lower buffer = better for live performance</li>
              <li>• Higher rate = better quality (more CPU)</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between p-4 border-t border-gray-700">
        <button
          @click="reset"
          class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
        >
          Reset to Default
        </button>
        <div class="flex gap-2">
          <button
            @click="close"
            class="px-4 py-2 text-sm bg-gray-700 hover:bg-gray-600 text-white rounded transition-colors"
          >
            Cancel
          </button>
          <button
            @click="apply"
            class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded transition-colors font-semibold"
          >
            Apply & Restart Audio
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useNetworkUrl } from '~/composables/useNetworkUrl'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
  apply: [config: { sampleRate: number; bufferSize: number }]
  'open-ndi': []
}>()

const sampleRates = [
  { value: 0, label: 'Auto', description: 'Device Native' },
  { value: 44100, label: '44.1 kHz', description: 'CD Quality' },
  { value: 48000, label: '48 kHz', description: 'Standard' },
  { value: 96000, label: '96 kHz', description: 'Hi-Res' },
  { value: 192000, label: '192 kHz', description: 'Ultra Hi-Res' },
]

const bufferSizes = [
  { value: 64, latency: '1.3', label: 'Ultra Low', level: 'low' },
  { value: 128, latency: '2.7', label: 'Very Low', level: 'low' },
  { value: 256, latency: '5.3', label: 'Low', level: 'medium' },
  { value: 512, latency: '10.7', label: 'Medium', level: 'medium' },
  { value: 1024, latency: '21.3', label: 'High', level: 'high' },
]

const selectedSampleRate = ref(0) // Default to Auto
const selectedBufferSize = ref(256)

// Network URL composable
const { networkUrl, isLoading: networkUrlLoading, copyToClipboard } = useNetworkUrl()
const urlCopied = ref(false)

const close = () => {
  emit('close')
}

const openNDISettings = () => {
  emit('open-ndi')
  close()
}

const copyNetworkUrl = async () => {
  const success = await copyToClipboard()
  if (success) {
    urlCopied.value = true
    setTimeout(() => {
      urlCopied.value = false
    }, 2000)
  }
}

const apply = () => {
  console.log('[AudioSettingsModal] Applying config:', {
    sampleRate: selectedSampleRate.value,
    bufferSize: selectedBufferSize.value,
  })
  emit('apply', {
    sampleRate: selectedSampleRate.value,
    bufferSize: selectedBufferSize.value,
  })
  close()
}

const reset = () => {
  selectedSampleRate.value = 0 // Auto
  selectedBufferSize.value = 256
}

// Load from Rust engine on mount
onMounted(async () => {
  try {
    const config = await window.audioEngine.getAudioConfig()
    if (config) {
      selectedSampleRate.value = config.sample_rate ?? 0 // Use ?? to allow 0 (Auto)
      selectedBufferSize.value = config.buffer_size || 256
      console.log('[AudioSettingsModal] Loaded config from Rust:', config)
    }
  } catch (e) {
    console.error('[AudioSettingsModal] Failed to load audio config:', e)
  }
})
</script>
