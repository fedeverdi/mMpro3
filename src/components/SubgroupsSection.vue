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
    <div class="w-full bg-gray-900 rounded p-1.5 border border-gray-700">
      <OutputSelector title="Select Subgroup Output" :devices="audioOutputDevices" :selected-device-id="selectedOutput"
        default-label="Default" default-description="Default audio output" default-icon="🔊" :show-no-output="true"
        @select="handleOutputSelect" />
    </div>

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
        <SubgroupFader v-model="volume" label="SUB" :trackHeight="fadersHeight" />
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
import { ref, watch, onMounted, onUnmounted, nextTick, inject } from 'vue'
import { useAudioDevices } from '../composables/useAudioDevices'

// Props
interface Props {
  masterChannel?: any
  subgroupId?: number
  subgroupName?: string
}

const props = withDefaults(defineProps<Props>(), {
  subgroupName: 'SUBGROUP'
})

defineEmits<{
  remove: []
}>()

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Subgroup volume
const volume = ref(0) // dB
const routeToMaster = ref(false)

// VU meter levels (will be updated by Rust engine)
const leftLevel = ref(-60)
const rightLevel = ref(-60)

// Audio outputs
const { audioOutputDevices, refreshAudioOutputs } = useAudioDevices()
const selectedOutput = ref<string | null>('no-output')

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let resizeObserver: ResizeObserver | null = null

// Calculate meters height based on container
function updateMetersHeight() {
  if (metersContainer.value) {
    const height = metersContainer.value.clientHeight
    const availableHeight = Math.max(120, height - 60)
    vuMetersHeight.value = Math.max(40, Math.floor(availableHeight * 0.4))
    fadersHeight.value = Math.max(80, Math.floor(availableHeight * 0.6))
  }
}

// Handle output device selection
function handleOutputSelect(deviceId: string | null) {
  selectedOutput.value = deviceId
  console.log(`[Subgroup ${props.subgroupId}] Output selected:`, deviceId)

  if (!audioEngine || props.subgroupId === undefined) return

  // If "no-output" is selected, disable direct output
  // The subgroup can still route to master if route_to_master is enabled
  if (deviceId === 'no-output' || deviceId === null) {
    audioEngine.setSubgroupOutputEnabled(props.subgroupId, false)
    console.log(`[Subgroup ${props.subgroupId}] Direct output disabled (can still route to master)`)
  } else {
    // Enable direct output when a device is selected
    audioEngine.setSubgroupOutputEnabled(props.subgroupId, true)
    console.log(`[Subgroup ${props.subgroupId}] Direct output enabled`)

    // Note: Currently all subgroups use the main output device
    // Multi-device routing would require separate audio streams per subgroup
  }
}

// Toggle route to master
function toggleRouteToMaster() {
  routeToMaster.value = !routeToMaster.value
}

// Watchers - Send changes to Rust engine
watch(volume, (newVolume) => {
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
    }
  },
  { immediate: true }
)

// Initialize
onMounted(async () => {
  // Enumerate audio outputs
  await refreshAudioOutputs()

  // Calculate initial height
  await nextTick()
  updateMetersHeight()

  // Watch for container size changes
  if (metersContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateMetersHeight()
    })
    resizeObserver.observe(metersContainer.value)
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})

// Expose refs for parent component
defineExpose({
  volume,
  routeToMaster,
  selectedOutput
})
</script>

<style scoped>
/* Add any component-specific styles here */
</style>
