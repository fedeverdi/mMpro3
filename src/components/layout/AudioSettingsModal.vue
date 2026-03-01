<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70" @click.self="close">
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
      <div class="p-6 space-y-6">
        <!-- Sample Rate Selection -->
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-3">
            Master Sample Rate
          </label>
          <div class="grid grid-cols-3 gap-2">
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

        <!-- Info Box -->
        <div class="bg-blue-900/30 border border-blue-700/50 rounded-lg p-4">
          <div class="text-sm text-blue-200">
            <div class="font-semibold mb-2">ℹ️ Professional Audio Configuration</div>
            <ul class="space-y-1 text-xs">
              <li>• All inputs/outputs resampled to master rate</li>
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

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
  apply: [config: { sampleRate: number; bufferSize: number }]
}>()

const sampleRates = [
  { value: 44100, label: '44.1 kHz', description: 'CD Quality' },
  { value: 48000, label: '48 kHz', description: 'Standard' },
  { value: 96000, label: '96 kHz', description: 'Hi-Res' },
]

const bufferSizes = [
  { value: 64, latency: '1.3', label: 'Ultra Low', level: 'low' },
  { value: 128, latency: '2.7', label: 'Very Low', level: 'low' },
  { value: 256, latency: '5.3', label: 'Low', level: 'medium' },
  { value: 512, latency: '10.7', label: 'Medium', level: 'medium' },
  { value: 1024, latency: '21.3', label: 'High', level: 'high' },
]

const selectedSampleRate = ref(48000)
const selectedBufferSize = ref(256)

const close = () => {
  emit('close')
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
  selectedSampleRate.value = 48000
  selectedBufferSize.value = 256
}

// Load from localStorage on mount
onMounted(() => {
  const saved = localStorage.getItem('audioConfig')
  if (saved) {
    try {
      const config = JSON.parse(saved)
      selectedSampleRate.value = config.sampleRate || 48000
      selectedBufferSize.value = config.bufferSize || 256
    } catch (e) {
      console.error('Failed to load audio config:', e)
    }
  }
})
</script>
