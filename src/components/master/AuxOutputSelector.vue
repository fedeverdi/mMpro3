<template>
  <div>
    <button @click="showModal = true"
      class="w-full text-[0.4rem] bg-gray-700 text-gray-300 rounded px-2 py-[0.15rem]  hover:bg-gray-600 hover:border-gray-500 transition-all flex items-center justify-center gap-1">
      <span class="font-bold uppercase tracking-wide">OUT</span>
      <span class="flex items-center gap-0.5">
        <!-- Routing indicators -->
        <span v-if="routeToMaster" class="w-1 h-1 bg-blue-500 rounded-full" title="Routed to Master"></span>
        <span v-if="routedSubgroups.size > 0" class="w-1 h-1 bg-green-500 rounded-full" title="Routed to Subgroups"></span>
      </span>
    </button>

    <AuxOutputModal 
      :is-open="showModal" 
      :title="title" 
      :devices="devices" 
      :selected-device-id="selectedDeviceId"
      :route-to-master="routeToMaster"
      :routed-subgroups="routedSubgroups"
      :subgroups="subgroups"
      @close="showModal = false" 
      @select-device="handleSelectDevice"
      @toggle-master-routing="$emit('toggle-master-routing')"
      @toggle-subgroup-routing="(id) => $emit('toggle-subgroup-routing', id)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import AuxOutputModal from './AuxOutputModal.vue'

interface Props {
  title: string
  devices: RustAudioDevice[]
  selectedDeviceId: string | null
  routeToMaster: boolean
  routedSubgroups: Set<number>
  subgroups?: Array<{ id: number, name: string }>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'select-device', deviceId: string | null): void
  (e: 'toggle-master-routing'): void
  (e: 'toggle-subgroup-routing', subgroupId: number): void
}>()

const showModal = ref(false)

const deviceLabel = computed(() => {
  if (!props.selectedDeviceId || props.selectedDeviceId === 'no-output') {
    return 'Output & Routing'
  }
  
  // Parse device ID (format: "deviceId:channel")
  const parts = props.selectedDeviceId.split(':')
  const deviceId = parts[0]
  const channel = parts[1] ? parseInt(parts[1]) : null
  
  const device = props.devices.find(d => d.id === deviceId)
  if (!device) return 'Select...'
  
  const name = device.name || 'Unknown'
  const shortName = name.length > 10 ? name.substring(0, 10) + '...' : name
  
  return channel ? `${shortName} Ch${channel}` : shortName
})

const deviceIcon = computed(() => {
  if (!props.selectedDeviceId || props.selectedDeviceId === 'no-output') {
    return '🔇'
  }
  return '🔊'
})

function handleSelectDevice(deviceId: string | null) {
  emit('select-device', deviceId)
  showModal.value = false
}
</script>
