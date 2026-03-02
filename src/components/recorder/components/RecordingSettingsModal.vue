<template>
  <Transition
    enter-from-class="opacity-0"
    enter-active-class="transition-opacity duration-300"
    enter-to-class="opacity-100"
    leave-from-class="opacity-100"
    leave-active-class="transition-opacity duration-300"
    leave-to-class="opacity-0">
    <div v-if="modelValue" @click="closeModal" class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[15000]">
      <div @click.stop class="bg-gradient-to-br from-gray-800 to-gray-900 rounded-xl border-2 border-gray-600 shadow-2xl p-6 w-[500px] max-h-[80vh] overflow-y-auto">
        <!-- Header -->
        <div class="flex items-center justify-between mb-6">
          <div>
            <h2 class="text-xl font-bold text-white">Recording Settings</h2>
            <p class="text-xs text-gray-400 mt-1">Configure audio recording parameters</p>
          </div>
          <button @click="closeModal" class="text-gray-400 hover:text-white transition-colors">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Settings Form -->
        <div class="space-y-6">
          <!-- File Format -->
          <div>
            <label class="block text-sm font-semibold text-white mb-2">File Format</label>
            <div class="space-y-2">
              <label 
                v-for="format in formatOptions" 
                :key="format.value"
                class="flex items-center p-3 bg-gray-800/50 rounded-lg border border-gray-700 hover:border-gray-600 cursor-pointer transition-colors"
                :class="{ 'border-red-500 bg-red-500/10': localSettings.format === format.value }"
              >
                <input 
                  type="radio" 
                  :value="format.value"
                  v-model="localSettings.format"
                  class="w-4 h-4 text-red-600 focus:ring-red-500"
                />
                <div class="ml-3 flex-1">
                  <div class="text-sm font-medium text-white">{{ format.label }}</div>
                  <div class="text-xs text-gray-400">{{ format.description }}</div>
                </div>
                <span v-if="!format.available" class="text-xs text-yellow-500 font-semibold">Coming Soon</span>
              </label>
            </div>
          </div>

          <!-- Sample Rate Info (always matches device) -->
          <div v-if="localSettings.format === 'wav'" class="bg-gray-800/30 rounded-lg p-4 border border-gray-700">
            <div class="flex items-center gap-2 mb-1">
              <svg class="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <label class="text-sm font-semibold text-white">Sample Rate</label>
            </div>
            <p class="text-sm text-gray-300">Uses audio device rate (typically 48 kHz)</p>
            <p class="text-xs text-gray-500 mt-1">Recording uses the system audio device's native sample rate for optimal quality</p>
          </div>

          <!-- Bit Depth (only for WAV) -->
          <div v-if="localSettings.format === 'wav'">
            <label class="block text-sm font-semibold text-white mb-2">Bit Depth</label>
            <div class="space-y-2">
              <label 
                v-for="depth in bitDepthOptions" 
                :key="depth.value"
                class="flex items-center p-3 bg-gray-800/50 rounded-lg border border-gray-700 hover:border-gray-600 cursor-pointer transition-colors"
                :class="{ 'border-red-500 bg-red-500/10': localSettings.bitDepth === depth.value }"
              >
                <input 
                  type="radio" 
                  :value="depth.value"
                  v-model="localSettings.bitDepth"
                  class="w-4 h-4 text-red-600 focus:ring-red-500"
                />
                <div class="ml-3 flex-1">
                  <div class="text-sm font-medium text-white">{{ depth.label }}</div>
                  <div class="text-xs text-gray-400">{{ depth.description }}</div>
                </div>
              </label>
            </div>
            <p class="text-xs text-gray-500 mt-2">Higher bit depth provides better dynamic range and lower noise floor</p>
          </div>

          <!-- Bitrate (for compressed formats) -->
          <div v-if="localSettings.format === 'mp3' || localSettings.format === 'opus'">
            <label class="block text-sm font-semibold text-white mb-2">Bitrate</label>
            <select 
              v-model="localSettings.bitrate"
              class="w-full bg-gray-800 border border-gray-600 rounded-lg px-4 py-2.5 text-white text-sm focus:border-red-500 focus:ring-1 focus:ring-red-500"
            >
              <option value="128">128 kbps (Good)</option>
              <option value="192">192 kbps (High)</option>
              <option value="256">256 kbps (Very High)</option>
              <option value="320">320 kbps (Maximum)</option>
            </select>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-3 mt-8">
          <button 
            @click="closeModal"
            class="flex-1 px-4 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-semibold transition-colors"
          >
            Cancel
          </button>
          <button 
            @click="saveSettings"
            class="flex-1 px-4 py-2.5 bg-red-600 hover:bg-red-700 text-white rounded-lg font-semibold transition-colors"
          >
            Apply Settings
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

export interface RecordingSettings {
  format: 'wav' | 'mp3' | 'opus'
  sampleRate: number
  bitDepth: number
  bitrate: number
}

interface Props {
  modelValue: boolean
  settings: RecordingSettings
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'update:settings': [settings: RecordingSettings]
}>()

const formatOptions = [
  {
    value: 'wav',
    label: 'WAV (Uncompressed)',
    description: 'Lossless quality, larger file size',
    available: true
  }
]

const bitDepthOptions = [
  {
    value: 16,
    label: '16-bit',
    description: 'Standard quality, compatible with all players'
  },
  {
    value: 24,
    label: '24-bit',
    description: 'Professional quality, ideal for production'
  },
  {
    value: 32,
    label: '32-bit Float',
    description: 'Maximum dynamic range, no clipping distortion'
  }
]

const localSettings = ref<RecordingSettings>({ ...props.settings })

// Update local settings when props change
watch(() => props.settings, (newSettings) => {
  localSettings.value = { ...newSettings }
}, { deep: true })

function closeModal() {
  emit('update:modelValue', false)
}

function saveSettings() {
  // Convert string values from <select> to numbers
  emit('update:settings', {
    format: localSettings.value.format,
    sampleRate: Number(localSettings.value.sampleRate),
    bitDepth: Number(localSettings.value.bitDepth),
    bitrate: Number(localSettings.value.bitrate)
  })
  closeModal()
}
</script>

<style scoped>
input[type="radio"] {
  appearance: none;
  width: 1rem;
  height: 1rem;
  border: 2px solid #6b7280;
  border-radius: 50%;
  cursor: pointer;
  position: relative;
}

input[type="radio"]:checked {
  border-color: #dc2626;
}

input[type="radio"]:checked::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0.5rem;
  height: 0.5rem;
  background: #dc2626;
  border-radius: 50%;
}
</style>
