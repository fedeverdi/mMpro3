<template>
  <button @click="showModal = true"
    class="w-full bg-gray-900 rounded border border-gray-700 px-2 py-1.5 hover:bg-gray-800 hover:border-blue-500 transition-all flex items-center justify-center gap-2">
    <span class="truncate">
      <div class="flex">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 576 512" :class="icon ? 'mt-[0.1rem]' : ''"
          class="w-3 h-3" fill="white">
          <path
            d="M215.03 71.05L126.06 160H24c-13.26 0-24 10.74-24 24v144c0 13.25 10.74 24 24 24h102.06l88.97 88.95c15.03 15.03 40.97 4.47 40.97-16.97V88.02c0-21.46-25.96-31.98-40.97-16.97zm233.32-51.08c-11.17-7.33-26.18-4.24-33.51 6.95-7.34 11.17-4.22 26.18 6.95 33.51 66.27 43.49 105.82 116.6 105.82 195.58 0 78.98-39.55 152.09-105.82 195.58-11.17 7.32-14.29 22.34-6.95 33.5 7.04 10.71 21.93 14.56 33.51 6.95C528.27 439.58 576 351.33 576 256S528.27 72.43 448.35 19.97zM480 256c0-63.53-32.06-121.94-85.77-156.24-11.19-7.14-26.03-3.82-33.12 7.46s-3.78 26.21 7.41 33.36C408.27 165.97 432 209.11 432 256s-23.73 90.03-63.48 115.42c-11.19 7.14-14.5 22.07-7.41 33.36 6.51 10.36 21.12 15.14 33.12 7.46C447.94 377.94 480 319.54 480 256zm-141.77-76.87c-11.58-6.33-26.19-2.16-32.61 9.45-6.39 11.61-2.16 26.2 9.45 32.61C327.98 228.28 336 241.63 336 256c0 14.38-8.02 27.72-20.92 34.81-11.61 6.41-15.84 21-9.45 32.61 6.43 11.66 21.05 15.8 32.61 9.45 28.23-15.55 45.77-45 45.77-76.88s-17.54-61.32-45.78-76.86z" />
        </svg>
        <span class="text-xs text-gray-400 ml-2">{{ icon ? deviceLabel : null }}</span>
      </div>
    </span>
  </button>

  <AudioOutputModal :is-open="showModal" :title="title" :devices="devices" :selected-device-id="selectedDeviceId || ''"
    :default-label="defaultLabel" :default-description="defaultDescription" :default-icon="defaultIcon"
    :show-no-output="showNoOutput" :mode="mode" @close="showModal = false" @select="handleSelect" />
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import AudioOutputModal from './AudioOutputModal.vue'

interface Props {
  icon?: string
  title: string
  devices: RustAudioDevice[]
  selectedDeviceId: string | null
  defaultLabel: string
  defaultDescription: string
  defaultIcon: string
  showNoOutput?: boolean
  mode?: 'stereo' | 'mono' // stereo for master/subgroups, mono for aux
}

const props = withDefaults(defineProps<Props>(), {
  showNoOutput: false,
  mode: 'stereo'
})

const emit = defineEmits<{
  select: [deviceId: string | null]
}>()

const showModal = ref(false)

const deviceLabel = computed(() => {
  if (!props.selectedDeviceId) return props.defaultLabel
  if (props.selectedDeviceId === 'no-output') return 'No Output'

  // Parse device ID (format: "deviceId" or "deviceId:leftCh:rightCh" or "deviceId:ch")
  const parts = props.selectedDeviceId.split(':')
  const deviceId = parts[0]
  const leftCh = parts[1] ? parseInt(parts[1]) : null
  const rightCh = parts[2] ? parseInt(parts[2]) : null

  const device = props.devices.find(d => d.id === deviceId)
  if (!device) return props.defaultLabel

  let name = device.name || 'Unknown'

  // Add channel info if specific channels are selected
  if (leftCh !== null) {
    if (rightCh !== null) {
      // Stereo: show L-R
      name = `${name.substring(0, 6)}.. ${leftCh + 1}-${rightCh + 1}`
    } else {
      // Mono: show single channel
      name = `${name.substring(0, 8)}.. Ch${leftCh + 1}`
    }
  } else {
    // No channel info: truncate name
    name = name.length > 8 ? name.substring(0, 8) + '..' : name
  }

  return name
})

function handleSelect(deviceId: string | null) {
  emit('select', deviceId)
  showModal.value = false
}
</script>
