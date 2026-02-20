<template>
  <button
    @click="showModal = true"
    class="flex-1 w-full text-[0.65rem] bg-gray-800 text-gray-200 border border-gray-600 rounded px-1.5 py-1 hover:bg-gray-700 hover:border-blue-500 focus:border-blue-500 focus:outline-none transition-all text-left flex items-center justify-between gap-1 min-w-0"
  >
    <span class="truncate">{{ icon }} {{ deviceLabel }}</span>
    <svg class="w-2.5 h-2.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  <AudioOutputModal
    :is-open="showModal"
    :title="title"
    :devices="devices"
    :selected-device-id="selectedDeviceId || ''"
    :default-label="defaultLabel"
    :default-description="defaultDescription"
    :default-icon="defaultIcon"
    @close="showModal = false"
    @select="handleSelect"
  />
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import AudioOutputModal from './AudioOutputModal.vue'

interface Props {
  icon: string
  title: string
  devices: MediaDeviceInfo[]
  selectedDeviceId: string | null
  defaultLabel: string
  defaultDescription: string
  defaultIcon: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [deviceId: string | null]
}>()

const showModal = ref(false)

const deviceLabel = computed(() => {
  if (!props.selectedDeviceId) return props.defaultLabel
  const device = props.devices.find(d => d.deviceId === props.selectedDeviceId)
  if (!device) return props.defaultLabel
  const name = device.label || 'Unknown'
  return name.length > 8 ? name.substring(0, 8) + '...' : name
})

function handleSelect(deviceId: string | null) {
  emit('select', deviceId)
  showModal.value = false
}
</script>
