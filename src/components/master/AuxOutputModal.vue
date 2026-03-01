<template>
  <Teleport to="body">
    <Transition name="modal">
      <div 
        v-if="isOpen" 
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/70"
        @click.self="$emit('close')"
      >
        <div class="bg-gray-900 rounded-lg shadow-2xl max-w-lg w-full mx-4 border border-gray-700 max-h-[90vh] flex flex-col">
          <!-- Header -->
          <div class="flex items-center justify-between p-4 border-b border-gray-700 flex-shrink-0">
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
          <div class="p-6 flex-1 overflow-y-auto custom-scrollbar space-y-6">
            <!-- Output Device Section -->
            <div class="space-y-2">
              <h4 class="text-sm font-bold text-teal-300 uppercase tracking-wide">Physical Output</h4>
              
              <!-- No Output Option -->
              <button
                @click="selectDevice('no-output')"
                class="w-full p-3 rounded-lg border-2 transition-all text-left"
                :class="getDeviceSelectionClass('no-output')"
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <div class="text-xl">🔇</div>
                    <div>
                      <div class="font-semibold text-sm">No Output</div>
                      <div class="text-xs opacity-70">Disable physical output</div>
                    </div>
                  </div>
                  <div v-if="isSelected('no-output')" class="text-teal-300">
                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                    </svg>
                  </div>
                </div>
              </button>

              <!-- Audio Output Devices -->
              <div v-for="device in devices" :key="device.id" class="space-y-1">
                <button
                  @click="toggleDevice(device)"
                  class="w-full p-3 rounded-lg border-2 transition-all text-left"
                  :class="getDeviceSelectionClass(device.id)"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3 flex-1 min-w-0">
                      <div class="text-xl flex-shrink-0">🔊</div>
                      <div class="flex-1 min-w-0">
                        <div class="font-semibold text-sm">
                          {{ device.name || `Audio Output ${device.id.substring(0, 12)}...` }}
                        </div>
                        <div class="text-xs opacity-70">
                          {{ device.output_channels }} channel{{ device.output_channels > 1 ? 's' : '' }}
                        </div>
                      </div>
                    </div>
                    <div class="flex items-center gap-2 flex-shrink-0">
                      <div v-if="isSelected(device.id)" class="text-teal-300">
                        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                        </svg>
                      </div>
                      <div v-if="device.output_channels > 2" class="text-gray-400">
                        <svg class="w-4 h-4 transition-transform" :class="{ 'rotate-180': expandedDevice === device.id }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                      </div>
                    </div>
                  </div>
                </button>

                <!-- Channel Selection (Mono for Aux) -->
                <div v-if="expandedDevice === device.id && device.output_channels > 1" 
                     class="pl-10 pr-4 pb-3 space-y-1">
                  <div class="text-xs text-gray-400 mb-2">Select mono channel:</div>
                  <div class="grid grid-cols-4 gap-2">
                    <button
                      v-for="ch in device.output_channels"
                      :key="ch"
                      @click="selectDeviceWithChannel(device.id, ch)"
                      class="px-2 py-1.5 text-xs rounded border transition-all"
                      :class="isChannelSelected(device.id, ch) 
                        ? 'bg-teal-600 border-teal-500 text-white' 
                        : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600'"
                    >
                      Ch {{ ch }}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Routing Section -->
            <div class="space-y-3 border-t-2 border-teal-600/30 pt-4">
              <h4 class="text-sm font-bold text-teal-300 uppercase tracking-wide">Internal Routing</h4>
              
              <!-- Route to Master -->
              <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg border border-gray-700">
                <div class="flex items-center gap-3">
                  <div class="text-xl">🎚️</div>
                  <div>
                    <div class="font-semibold text-sm">Route to Master</div>
                    <div class="text-xs opacity-70">Send aux output to master bus</div>
                  </div>
                </div>
                <button
                  @click="toggleMasterRouting"
                  class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors"
                  :class="localRouteToMaster ? 'bg-blue-600' : 'bg-gray-600'"
                >
                  <span
                    class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
                    :class="localRouteToMaster ? 'translate-x-6' : 'translate-x-1'"
                  />
                </button>
              </div>

              <!-- Route to Subgroups -->
              <div v-if="subgroups && subgroups.length > 0" 
                   class="p-3 bg-gray-800/50 rounded-lg border border-gray-700">
                <div class="flex items-center gap-3 mb-3">
                  <div class="text-xl">🎛️</div>
                  <div>
                    <div class="font-semibold text-sm">Route to Subgroups</div>
                    <div class="text-xs opacity-70">Send aux output to subgroup buses</div>
                  </div>
                </div>
                <div class="grid grid-cols-2 gap-2 mt-2">
                  <button
                    v-for="subgroup in subgroups"
                    :key="subgroup.id"
                    @click="toggleSubgroupRouting(subgroup.id)"
                    class="px-3 py-2 text-sm rounded border transition-all flex items-center justify-between"
                    :class="isSubgroupRouted(subgroup.id)
                      ? 'bg-green-600 border-green-500 text-white'
                      : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600'"
                  >
                    <span>{{ subgroup.name }}</span>
                    <svg v-if="isSubgroupRouted(subgroup.id)" class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- No devices message -->
          <div v-if="devices.length === 0" class="text-center py-8 text-gray-400">
            <div class="text-4xl mb-2">🔊</div>
            <div class="text-sm">No audio output devices found</div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'

interface Props {
  isOpen: boolean
  title: string
  devices: RustAudioDevice[]
  selectedDeviceId: string | null
  routeToMaster: boolean
  routedSubgroups: Set<number>
  subgroups?: Array<{ id: number, name: string }>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select-device', deviceId: string | null): void
  (e: 'toggle-master-routing'): void
  (e: 'toggle-subgroup-routing', subgroupId: number): void
}>()

const expandedDevice = ref<string | null>(null)
const localRouteToMaster = ref(props.routeToMaster)
const localRoutedSubgroups = ref(new Set(props.routedSubgroups))

// Watch for prop changes
watch(() => props.routeToMaster, (newVal) => {
  localRouteToMaster.value = newVal
})

watch(() => props.routedSubgroups, (newVal) => {
  localRoutedSubgroups.value = new Set(newVal)
}, { deep: true })

function getDeviceSelectionClass(deviceId: string) {
  return isSelected(deviceId)
    ? 'bg-teal-600 border-teal-500 text-white'
    : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'
}

function isSelected(deviceId: string): boolean {
  if (!props.selectedDeviceId) return deviceId === 'no-output'
  return props.selectedDeviceId.startsWith(deviceId)
}

function isChannelSelected(deviceId: string, channel: number): boolean {
  if (!props.selectedDeviceId) return false
  return props.selectedDeviceId === `${deviceId}:${channel}`
}

function toggleDevice(device: RustAudioDevice) {
  if (device.output_channels > 1) {
    // Multi-channel device: expand/collapse
    expandedDevice.value = expandedDevice.value === device.id ? null : device.id
  } else {
    // Single channel: select directly
    selectDevice(`${device.id}:1`)
  }
}

function selectDevice(deviceId: string | null) {
  emit('select-device', deviceId)
  expandedDevice.value = null
}

function selectDeviceWithChannel(deviceId: string, channel: number) {
  selectDevice(`${deviceId}:${channel}`)
}

function toggleMasterRouting() {
  localRouteToMaster.value = !localRouteToMaster.value
  emit('toggle-master-routing')
}

function toggleSubgroupRouting(subgroupId: number) {
  if (localRoutedSubgroups.value.has(subgroupId)) {
    localRoutedSubgroups.value.delete(subgroupId)
  } else {
    localRoutedSubgroups.value.add(subgroupId)
  }
  emit('toggle-subgroup-routing', subgroupId)
}

function isSubgroupRouted(subgroupId: number): boolean {
  return localRoutedSubgroups.value.has(subgroupId)
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
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(17, 24, 39, 0.5);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(20, 184, 166, 0.5);
  border-radius: 3px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(20, 184, 166, 0.8);
}

/* Firefox */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgba(20, 184, 166, 0.5) rgba(17, 24, 39, 0.5);
}
</style>
