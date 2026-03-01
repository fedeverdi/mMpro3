<template>
  <button @click="showModal = true"
    class="flex-1 w-full text-[0.65rem] bg-gray-800 text-gray-200 border border-gray-600 rounded px-1.5 py-1 hover:bg-gray-700 hover:border-blue-500 focus:border-blue-500 focus:outline-none transition-all text-left flex items-center justify-between gap-1 min-w-0">
    <span class="truncate">
      <div class="flex">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 352 512" :class="icon ? 'mt-[0.1rem]' : ''" class="w-3 h-3 mr-2" fill="white">
          <path
            d="M176 352c53.02 0 96-42.98 96-96V96c0-53.02-42.98-96-96-96S80 42.98 80 96v160c0 53.02 42.98 96 96 96zm160-160h-16c-8.84 0-16 7.16-16 16v48c0 74.8-64.49 134.82-140.79 127.38C96.71 376.89 48 317.11 48 250.3V208c0-8.84-7.16-16-16-16H16c-8.84 0-16 7.16-16 16v40.16c0 89.64 63.97 169.55 152 181.69V464H96c-8.84 0-16 7.16-16 16v16c0 8.84 7.16 16 16 16h160c8.84 0 16-7.16 16-16v-16c0-8.84-7.16-16-16-16h-56v-33.77C285.71 418.47 352 344.9 352 256v-48c0-8.84-7.16-16-16-16z" />
        </svg>
        {{ icon ? deviceLabel : null }}
      </div>
    </span>
    <svg class="w-2.5 h-2.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  <AudioInputModal :is-open="showModal" :title="title" :devices="devices" :selected-device-id="selectedDeviceId || ''"
    :default-label="defaultLabel" :default-description="defaultDescription" :default-icon="defaultIcon"
    :show-file-option="showFileOption" @close="showModal = false" @select="handleSelect" />
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import AudioInputModal from './AudioInputModal.vue'

interface Props {
  icon?: string
  title: string
  devices: RustAudioDevice[]
  selectedDeviceId: string | null
  defaultLabel: string
  defaultDescription: string
  defaultIcon: string
  showFileOption?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showFileOption: false
})

const emit = defineEmits<{
  select: [deviceId: string | null]
}>()

const showModal = ref(false)

const deviceLabel = computed(() => {
  if (!props.selectedDeviceId) return props.defaultLabel
  if (props.selectedDeviceId === 'file') return 'Audio File'
  const device = props.devices.find(d => d.id === props.selectedDeviceId)
  if (!device) return props.defaultLabel
  const name = device.name || 'Unknown'
  return name.length > 8 ? name.substring(0, 8) + '...' : name
})

function handleSelect(deviceId: string | null) {
  emit('select', deviceId)
  showModal.value = false
}
</script>
