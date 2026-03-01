<template>
  <Teleport to="body">
    <Transition name="modal">
      <div 
        v-if="isOpen" 
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/70"
        @click.self="$emit('close')"
      >
        <div class="bg-gray-900 rounded-lg shadow-2xl max-w-md w-full mx-4 border border-gray-700">
          <!-- Header -->
          <div class="flex items-center justify-between p-4 border-b border-gray-700">
            <h3 class="text-lg font-semibold text-white">{{ title }}</h3>
            <button 
              @click="$emit('close')"
              class="text-gray-400 hover:text-white transition-colors"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Content -->
          <div class="p-6">
            <div class="space-y-2 max-h-96 overflow-y-auto custom-scrollbar">
            <!-- No Output Option -->
            <button
              v-if="showNoOutput"
              @click="selectDevice('no-output')"
              class="w-full p-4 rounded-lg border-2 transition-all text-left"
              :class="getDeviceSelectionClass('no-output')"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="text-2xl">🔇</div>
                  <div>
                    <div class="font-semibold">No Output</div>
                    <div class="text-xs opacity-70">Disable audio output</div>
                  </div>
                </div>
                <div v-if="isSelected('no-output')" class="text-blue-300">
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
              :class="getDeviceSelectionClass('')"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="text-2xl">{{ defaultIcon }}</div>
                  <div>
                    <div class="font-semibold">{{ defaultLabel }}</div>
                    <div class="text-xs opacity-70">{{ defaultDescription }}</div>
                  </div>
                </div>
                <div v-if="isSelected('')" class="text-blue-300">
                  <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                  </svg>
                </div>
              </div>
            </button>

            <!-- Audio Output Devices with Channel Selection -->
            <div v-for="device in devices" :key="device.id" class="space-y-1">
              <!-- Device Button -->
              <button
                @click="toggleDevice(device)"
                class="w-full p-4 rounded-lg border-2 transition-all text-left"
                :class="getDeviceSelectionClass(device.id)"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3 flex-1 min-w-0">
                    <div class="text-2xl flex-shrink-0">🔊</div>
                    <div class="flex-1 min-w-0">
                      <div class="font-semibold">
                        {{ device.name || `Audio Output ${device.id.substring(0, 12)}...` }}
                      </div>
                      <div class="text-xs opacity-70">
                        {{ device.output_channels }} channel{{ device.output_channels > 1 ? 's' : '' }}
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <div v-if="isSelected(device.id)" class="text-blue-300">
                      <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                      </svg>
                    </div>
                    <!-- Expand icon if multi-channel device -->
                    <div v-if="device.output_channels > 2" class="text-gray-400">
                      <svg class="w-5 h-5 transition-transform" :class="{ 'rotate-180': expandedDevice === device.id }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                      </svg>
                    </div>
                  </div>
                </div>
              </button>

              <!-- Channel Selection (for multi-channel devices) -->
              <div v-if="expandedDevice === device.id && device.output_channels > 2" class="ml-4 space-y-1">
                <!-- Stereo Mode: Channel Pairs -->
                <div v-if="mode === 'stereo'">
                  <button
                    v-for="pair in getChannelPairs(device.output_channels)"
                    :key="`${device.id}:${pair.left}:${pair.right}`"
                    @click="selectDeviceWithChannels(device.id, pair.left, pair.right)"
                    class="w-full p-3 rounded-lg border transition-all text-left text-sm"
                    :class="isChannelPairSelected(device.id, pair.left, pair.right) 
                      ? 'bg-blue-900/50 border-blue-500 text-white' 
                      : 'bg-gray-800 border-gray-600 text-gray-300 hover:bg-gray-700 hover:border-gray-500'"
                  >
                    <div class="flex items-center justify-between">
                      <span>Channels {{ pair.left + 1 }}-{{ pair.right + 1 }}</span>
                      <div v-if="isChannelPairSelected(device.id, pair.left, pair.right)" class="text-blue-300">
                        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                        </svg>
                      </div>
                    </div>
                  </button>
                </div>

                <!-- Mono Mode: Individual Channels -->
                <div v-else>
                  <button
                    v-for="channel in device.output_channels"
                    :key="`${device.id}:${channel - 1}`"
                    @click="selectDeviceWithChannels(device.id, channel - 1)"
                    class="w-full p-3 rounded-lg border transition-all text-left text-sm"
                    :class="isMonoChannelSelected(device.id, channel - 1) 
                      ? 'bg-blue-900/50 border-blue-500 text-white' 
                      : 'bg-gray-800 border-gray-600 text-gray-300 hover:bg-gray-700 hover:border-gray-500'"
                  >
                    <div class="flex items-center justify-between">
                      <span>Channel {{ channel }}</span>
                      <div v-if="isMonoChannelSelected(device.id, channel - 1)" class="text-blue-300">
                        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                        </svg>
                      </div>
                    </div>
                  </button>
                </div>
              </div>
            </div>
            
            <!-- No devices message -->
            <div v-if="devices.length === 0" class="text-center py-8 text-gray-400">
              <div class="text-4xl mb-2">🔇</div>
              <div class="text-sm">No audio output devices found</div>
            </div>
          </div>
        </div>
      </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = withDefaults(defineProps<{
  isOpen: boolean
  title: string
  devices: RustAudioDevice[]
  selectedDeviceId: string
  defaultLabel?: string
  defaultDescription?: string
  defaultIcon?: string
  showNoOutput?: boolean
  mode?: 'stereo' | 'mono' // stereo for master/subgroups, mono for aux
}>(), {
  showNoOutput: false,
  mode: 'stereo'
})

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', deviceId: string): void
}>()

const expandedDevice = ref<string | null>(null)

// Parse selected device ID (format: "deviceId" or "deviceId:leftCh:rightCh" or "deviceId:ch")
const parsedSelection = computed(() => {
  const parts = props.selectedDeviceId.split(':')
  return {
    deviceId: parts[0] || '',
    leftChannel: parts[1] ? parseInt(parts[1]) : null,
    rightChannel: parts[2] ? parseInt(parts[2]) : null
  }
})

function toggleDevice(device: RustAudioDevice) {
  if (device.output_channels <= 2) {
    // Simple 2-channel device: select directly with default channels
    if (props.mode === 'stereo') {
      selectDeviceWithChannels(device.id, 0, 1)
    } else {
      selectDeviceWithChannels(device.id, 0)
    }
  } else {
    // Multi-channel device: toggle expansion
    if (expandedDevice.value === device.id) {
      expandedDevice.value = null
    } else {
      expandedDevice.value = device.id
    }
  }
}

function selectDevice(deviceId: string) {
  emit('select', deviceId)
  emit('close')
}

function selectDeviceWithChannels(deviceId: string, leftChannel: number, rightChannel?: number) {
  if (props.mode === 'stereo' && rightChannel !== undefined) {
    // Stereo: deviceId:leftCh:rightCh
    emit('select', `${deviceId}:${leftChannel}:${rightChannel}`)
  } else {
    // Mono: deviceId:ch
    emit('select', `${deviceId}:${leftChannel}`)
  }
  emit('close')
}

function getChannelPairs(channelCount: number): Array<{ left: number; right: number }> {
  const pairs: Array<{ left: number; right: number }> = []
  for (let i = 0; i < channelCount - 1; i += 2) {
    pairs.push({ left: i, right: i + 1 })
  }
  return pairs
}

function isSelected(deviceId: string): boolean {
  return parsedSelection.value.deviceId === deviceId && 
         parsedSelection.value.leftChannel === null
}

function isChannelPairSelected(deviceId: string, left: number, right: number): boolean {
  return parsedSelection.value.deviceId === deviceId && 
         parsedSelection.value.leftChannel === left &&
         parsedSelection.value.rightChannel === right
}

function isMonoChannelSelected(deviceId: string, channel: number): boolean {
  return parsedSelection.value.deviceId === deviceId && 
         parsedSelection.value.leftChannel === channel &&
         parsedSelection.value.rightChannel === null
}

function getDeviceSelectionClass(deviceId: string): string {
  const selected = isSelected(deviceId) || 
                   parsedSelection.value.deviceId === deviceId
  
  return selected
    ? 'bg-blue-600 border-blue-500 text-white' 
    : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'
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
