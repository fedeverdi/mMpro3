<template>
  <Teleport to="body">
    <Transition name="modal">
      <div 
        v-if="isOpen" 
        class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-70"
        @click.self="$emit('close')"
      >
        <div class="bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-6 max-w-md w-full mx-4 shadow-2xl">
          <!-- Modal Header -->
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-bold text-blue-400">{{ title }}</h3>
            <button 
              @click="$emit('close')"
              class="text-gray-400 hover:text-white transition-colors"
            >
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Devices List -->
          <div class="space-y-2 max-h-96 overflow-y-auto custom-scrollbar">
            <!-- File Option -->
            <button
              v-if="showFileOption"
              @click="selectDevice('file')"
              class="w-full p-4 rounded-lg border-2 transition-all text-left"
              :class="selectedDeviceId === 'file' 
                ? 'bg-blue-600 border-blue-500 text-white' 
                : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="text-2xl">📁</div>
                  <div>
                    <div class="font-semibold">Audio File</div>
                    <div class="text-xs opacity-70">Use audio file playback</div>
                  </div>
                </div>
                <div v-if="selectedDeviceId === 'file'" class="text-blue-300">
                  <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                  </svg>
                </div>
              </div>
            </button>
            
            <!-- Default/Off Option -->
            <button
              @click="selectDevice('')"
              class="w-full p-4 rounded-lg border-2 transition-all text-left"
              :class="selectedDeviceId === '' 
                ? 'bg-blue-600 border-blue-500 text-white' 
                : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="text-2xl">{{ defaultIcon }}</div>
                  <div>
                    <div class="font-semibold">{{ defaultLabel }}</div>
                    <div class="text-xs opacity-70">{{ defaultDescription }}</div>
                  </div>
                </div>
                <div v-if="selectedDeviceId === ''" class="text-blue-300">
                  <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                  </svg>
                </div>
              </div>
            </button>

            <!-- Audio Input Devices -->
            <button
              v-for="device in devices"
              :key="device.id"
              @click="selectDevice(device.id)"
              class="w-full p-4 rounded-lg border-2 transition-all text-left"
              :class="selectedDeviceId === device.id 
                ? 'bg-blue-600 border-blue-500 text-white' 
                : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="text-2xl">🎤</div>
                  <div>
                    <div class="font-semibold">
                      {{ device.name || `Audio Input ${device.id.substring(0, 12)}...` }}
                    </div>
                    <div class="text-xs opacity-70 font-mono">
                      {{ device.id.substring(0, 20) }}...
                    </div>
                  </div>
                </div>
                <div v-if="selectedDeviceId === device.id" class="text-blue-300">
                  <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                  </svg>
                </div>
              </div>
            </button>
          </div>

          <!-- No devices message -->
          <div v-if="devices.length === 0" class="text-center py-8 text-gray-400">
            <div class="text-4xl mb-2">🎤</div>
            <div class="text-sm">No audio input devices found</div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
const props = withDefaults(defineProps<{
  isOpen: boolean
  title: string
  devices: RustAudioDevice[]
  selectedDeviceId: string
  defaultLabel?: string
  defaultDescription?: string
  defaultIcon?: string
  showFileOption?: boolean
}>(), {
  showFileOption: false
})

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', deviceId: string): void
}>()

function selectDevice(deviceId: string) {
  emit('select', deviceId)
  emit('close')
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .bg-gradient-to-b,
.modal-leave-active .bg-gradient-to-b {
  transition: transform 0.2s ease;
}

.modal-enter-from .bg-gradient-to-b {
  transform: scale(0.9);
}

.modal-leave-to .bg-gradient-to-b {
  transform: scale(0.9);
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(17, 24, 39, 0.5);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(59, 130, 246, 0.5);
  border-radius: 4px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(59, 130, 246, 0.8);
}

/* Firefox */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgba(59, 130, 246, 0.5) rgba(17, 24, 39, 0.5);
}
</style>
