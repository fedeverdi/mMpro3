<template>
  <div
    class="subgroups-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-gray-600 p-2 flex flex-col items-center gap-1 h-full w-full max-w-[5rem]">
    <!-- Subgroup Header -->
    <div class="w-full flex items-center justify-between gap-1">
      <div class="text-xs font-bold text-gray-400 flex-1 text-center">{{ subgroupName }}</div>
      <button @click="$emit('remove')"
        class="w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
        title="Remove Subgroup">
        ×
      </button>
    </div>

    <!-- Output Device Selector -->
    <OutputSelector title="Select Subgroup Output" :devices="audioOutputDevices" :selected-device-id="selectedOutput"
      default-label="Default" default-description="Default audio output" default-icon="🔊" :show-no-output="true"
      mode="stereo" @select="handleOutputSelect" />

    <!-- VU Meters and Faders -->
    <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-2 min-h-0 mt-6">
      <!-- VU Meters Row -->
      <div v-if="vuMetersHeight > 0"
        class="flex flex-col items-center w-full justify-center bg-gray-900 rounded p-1 border border-gray-700">
        <div class="flex gap-0 relative">
          <VuMeter :level="leftLevel" label="L" :height="vuMetersHeight" :width="10" class="-mr-3"
            :value-font-size="5" />
          <VuMeter :level="rightLevel" label="R" :height="vuMetersHeight" :width="10" class="-ml-3"
            :value-font-size="5" />
        </div>
      </div>

      <!-- Fader -->
      <div v-if="fadersHeight > 0" class="flex gap-1 items-end mt-3 pb-6">
        <SubgroupFader v-model="volume" label="SUB" :trackHeight="fadersHeight" 
          @drag-start="isDraggingVolume = true" 
          @drag-end="isDraggingVolume = false" />
      </div>
    </div>

    <!-- Route to Master Button -->
    <div class="w-full">
      <button @click="toggleRouteToMaster" class="w-full py-1 text-[0.5rem] font-bold rounded transition-all"
        :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
        {{ routeToMaster ? '→ MASTER' : '→ DIRECT' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import VuMeter from './core/VuMeter.vue'
import OutputSelector from './master/OutputSelector.vue'
import SubgroupFader from './subgroups/SubgroupFader.vue'
import { ref, computed, watch, onMounted, onUnmounted, nextTick, inject, type Ref } from 'vue'
import { useAudioDevices } from '../composables/useAudioDevices'

// Props
interface Props {
  masterChannel?: any
  subgroupId?: number
  subgroupName?: string
  volume?: number
  routeToMaster?: boolean
  selectedOutput?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  subgroupName: 'SUBGROUP',
  volume: 0,
  routeToMaster: false,
  selectedOutput: 'no-output'
})

const emit = defineEmits<{
  remove: []
  'update:volume': [value: number]
  'update:routeToMaster': [value: boolean]
  'update:selectedOutput': [value: string | null]
}>()

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Use computed for two-way binding with props
const volume = computed({
  get: () => props.volume ?? 0,
  set: (value: number) => emit('update:volume', value)
})

const routeToMaster = computed({
  get: () => props.routeToMaster ?? false,
  set: (value: boolean) => emit('update:routeToMaster', value)
})

const selectedOutput = computed({
  get: () => props.selectedOutput ?? 'no-output',
  set: (value: string | null) => emit('update:selectedOutput', value)
})

// VU meter levels (will be updated by Rust engine)
const leftLevel = ref(-60)
const rightLevel = ref(-60)

// Anti-loop flags
const isUpdatingFromEngine = ref(false)
const isDraggingVolume = ref(false)

// Audio outputs
const { audioOutputDevices } = useAudioDevices()

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let updateMetersHeightTimeout: ReturnType<typeof setTimeout> | null = null

// Calculate meters height based on container
function updateMetersHeight() {
  // Throttle resize calculations to prevent blocking during window animations
  if (updateMetersHeightTimeout) return

  updateMetersHeightTimeout = setTimeout(() => {
    if (metersContainer.value) {
      const height = metersContainer.value.clientHeight
      const availableHeight = Math.max(120, height - 60)
      vuMetersHeight.value = Math.max(40, Math.floor(availableHeight * 0.4))
      fadersHeight.value = Math.max(80, Math.floor(availableHeight * 0.6))
    }
    updateMetersHeightTimeout = null
  }, 16) // ~60fps
}

// Use centralized resize trigger from parent
const resizeTrigger = inject<Ref<number>>('resizeTrigger', ref(0))

// Handle output device selection
function handleOutputSelect(deviceId: string | null) {
  selectedOutput.value = deviceId

  if (!audioEngine || props.subgroupId === undefined) return

  // Parse device ID (format: "deviceId" or "deviceId:leftCh:rightCh")
  const parts = deviceId?.split(':') || []
  const actualDeviceId = parts[0]
  const leftChannel = parts[1] ? parseInt(parts[1]) : 0
  const rightChannel = parts[2] ? parseInt(parts[2]) : 1

  // If "no-output" is selected, disable direct output
  if (actualDeviceId === 'no-output' || actualDeviceId === null) {
    audioEngine.setSubgroupOutputEnabled(props.subgroupId, false)
  } else {
    // Enable direct output and set channel selection
    audioEngine.setSubgroupOutputEnabled(props.subgroupId, true)
    audioEngine.setSubgroupOutputChannels(props.subgroupId, leftChannel, rightChannel)
  }
}

// Toggle route to master
function toggleRouteToMaster() {
  routeToMaster.value = !routeToMaster.value
}

// Watchers - Send changes to Rust engine
watch(volume, (newVolume) => {
  if (isUpdatingFromEngine.value) return
  if (isDraggingVolume.value) return // Don't update while dragging
  
  if (audioEngine && props.subgroupId !== undefined) {
    // Convert dB to linear gain: gain = 10^(dB/20)
    let gainValue: number
    if (newVolume <= -90) {
      gainValue = 0.0 // Mute
    } else {
      gainValue = Math.pow(10, newVolume / 20)
    }

    audioEngine.setSubgroupGain(props.subgroupId, gainValue)
  }
})

watch(routeToMaster, (route) => {
  if (isUpdatingFromEngine.value) return
  
  if (audioEngine && props.subgroupId !== undefined) {
    audioEngine.setSubgroupRouteToMaster(props.subgroupId, route)
  }
})

// Watch for meter level updates from audio engine
watch(
  () => audioEngine?.state.value.subgroupLevels.get(props.subgroupId ?? 0),
  (levels) => {
    if (levels) {
      leftLevel.value = levels.left
      rightLevel.value = levels.right
      
      // Sync subgroup parameters from Rust engine
      if (!isDraggingVolume.value) {
        isUpdatingFromEngine.value = true
        
        // Convert gain to dB: dB = 20 * log10(gain)
        const gainDb = levels.gain > 0 ? 20 * Math.log10(levels.gain) : -90
        volume.value = gainDb
        routeToMaster.value = levels.routeToMaster
        
        isUpdatingFromEngine.value = false
      }
    }
  },
  { immediate: true, flush: 'sync' }
)

// Initialize
onMounted(async () => {
  // Audio output devices are already enumerated during app initialization
  // No need to refresh them here

  // Calculate initial height
  await nextTick()
  updateMetersHeight()

  // Watch for centralized resize trigger instead of using ResizeObserver
  watch(resizeTrigger, () => {
    updateMetersHeight()
  })

  // Cleanup on unmount
  onUnmounted(() => {
    if (updateMetersHeightTimeout) {
      clearTimeout(updateMetersHeightTimeout)
    }
  })
})

// Remove defineExpose - now using props/emit pattern
</script>

<style scoped>
/* Add any component-specific styles here */
</style>
