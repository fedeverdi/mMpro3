<template>
  <div
    class="master-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-2 flex flex-col items-center gap-1 h-full w-full w-[12rem] max-w-[12rem]">
    <!-- Master Header -->
    <div class="w-full text-center">
      <div class="text-xs font-bold text-blue-400">MASTER</div>
    </div>

    <!-- Master Output Selector -->
    <div class="w-full bg-gray-900 rounded p-1.5 border border-gray-700">
      <OutputSelector 
        title="Master Output" 
        :devices="audioOutputDevices"
        :selected-device-id="selectedMasterOutput" 
        default-label="Default" 
        default-description="Default audio output"
        icon="🔊"
        default-icon="🔊"
        :show-no-output="false"
        @select="onMasterOutputSelect" 
      />
    </div>

    <!-- Headphones Control -->
    <HeadphonesControl
      :devices="audioOutputDevices"
      :selected-device-id="selectedHeadphonesOutput"
      :volume="headphonesVolume"
      :level="headphonesLevel"
      @select="onHeadphonesOutputSelect"
      @update:volume="headphonesVolume = $event"
    />

    <!-- VU Meters and Faders -->
    <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-4 min-h-0 ">
      <!-- VU Meters Row -->
      <MasterMeter 
        :left-level="leftLevel" 
        :right-level="rightLevel" 
        :vu-meters-height="vuMetersHeight" 
      />

      <!-- Faders Row -->
      <div v-if="fadersHeight > 0" class="flex gap-2 items-end mb-6 mt-2">
        <MasterFader v-model="leftVolume" label="L" :trackHeight="fadersHeight" />
        <MasterFader v-model="rightVolume" label="R" :trackHeight="fadersHeight" />
      </div>
    </div>

    <!-- Master Controls -->
    <div class="w-full mt-2 flex gap-1">
      <!-- Master Mute Button -->
      <button @click="toggleMasterMute" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="masterMuted ? 'bg-red-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        title="Mute master output">
        <div class="flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 576 512">
            <path v-if="masterMuted"
              d="M215.03 71.05L126.06 160H24c-13.26 0-24 10.74-24 24v144c0 13.25 10.74 24 24 24h102.06l88.97 88.95c15.03 15.03 40.97 4.47 40.97-16.97V88.02c0-21.46-25.96-31.98-40.97-16.97zM461.64 256l45.64-45.64c6.3-6.3 6.3-16.52 0-22.82l-22.82-22.82c-6.3-6.3-16.52-6.3-22.82 0L416 210.36l-45.64-45.64c-6.3-6.3-16.52-6.3-22.82 0l-22.82 22.82c-6.3 6.3-6.3 16.52 0 22.82L370.36 256l-45.63 45.63c-6.3 6.3-6.3 16.52 0 22.82l22.82 22.82c6.3 6.3 16.52 6.3 22.82 0L416 301.64l45.64 45.64c6.3 6.3 16.52 6.3 22.82 0l22.82-22.82c6.3-6.3 6.3-16.52 0-22.82L461.64 256z" />
            <path v-else
              d="M215.03 71.05L126.06 160H24c-13.26 0-24 10.74-24 24v144c0 13.25 10.74 24 24 24h102.06l88.97 88.95c15.03 15.03 40.97 4.47 40.97-16.97V88.02c0-21.46-25.96-31.98-40.97-16.97zm233.32-51.08c-14.17-8.18-32.06-3.34-40.24 10.82-8.18 14.17-3.34 32.06 10.82 40.24 65.09 37.54 105.76 107.59 105.76 184.97 0 77.38-40.67 147.43-105.76 184.97-14.17 8.18-19.01 26.07-10.82 40.24 8.18 14.17 26.07 19.01 40.24 10.82 77.62-44.79 126.34-128.31 126.34-236.03s-48.72-191.24-126.34-236.03zm-63.58 79.13c-14.17-8.19-32.06-3.34-40.24 10.82-8.19 14.17-3.34 32.06 10.82 40.24 34.44 19.87 55.89 57.1 55.89 96.84 0 39.74-21.45 76.97-55.89 96.84-14.17 8.19-19.01 26.07-10.82 40.24 8.19 14.17 26.07 19.01 40.24 10.82 50.68-29.23 87.16-84.25 87.16-147.9s-36.48-118.67-87.16-147.9z" />
          </svg>
        </div>
      </button>

      <!-- Link Button -->
      <button @click="toggleLink" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isLinked ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
        <div class="flex items-center justify-center">
          <svg v-if="isLinked" xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 512 512">
            <path
              d="M326.612 185.391c59.747 59.809 58.927 155.698.36 214.59-.11.12-.24.25-.36.37l-67.2 67.2c-59.27 59.27-155.699 59.262-214.96 0-59.27-59.26-59.27-155.7 0-214.96l37.106-37.106c9.84-9.84 26.786-3.3 27.294 10.606.648 17.722 3.826 35.527 9.69 52.721 1.986 5.822.567 12.262-3.783 16.612l-13.087 13.087c-28.026 28.026-28.905 73.66-1.155 101.96 28.024 28.579 74.086 28.749 102.325.51l67.2-67.19c28.191-28.191 28.073-73.757 0-101.83-3.701-3.694-7.429-6.564-10.341-8.569a16.037 16.037 0 0 1-6.947-12.606c-.396-10.567 3.348-21.456 11.698-29.806l21.054-21.055c5.521-5.521 14.182-6.199 20.584-1.731a152.482 152.482 0 0 1 20.522 17.197zM467.547 44.449c-59.261-59.262-155.69-59.27-214.96 0l-67.2 67.2c-.12.12-.25.25-.36.37-58.566 58.892-59.387 154.781.36 214.59a152.454 152.454 0 0 0 20.521 17.196c6.402 4.468 15.064 3.789 20.584-1.731l21.054-21.055c8.35-8.35 12.094-19.239 11.698-29.806a16.037 16.037 0 0 0-6.947-12.606c-2.912-2.005-6.64-4.875-10.341-8.569-28.073-28.073-28.191-73.639 0-101.83l67.2-67.19c28.239-28.239 74.3-28.069 102.325.51 27.75 28.3 26.872 73.934-1.155 101.96l-13.087 13.087c-4.35 4.35-5.769 10.79-3.783 16.612 5.864 17.194 9.042 34.999 9.69 52.721.509 13.906 17.454 20.446 27.294 10.606l37.106-37.106c59.271-59.259 59.271-155.699.001-214.959z" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 448 512">
            <path
              d="M400 224h-24v-72C376 68.2 307.8 0 224 0S72 68.2 72 152v72H48c-26.5 0-48 21.5-48 48v192c0 26.5 21.5 48 48 48h352c26.5 0 48-21.5 48-48V272c0-26.5-21.5-48-48-48zm-104 0H152v-72c0-39.7 32.3-72 72-72s72 32.3 72 72v72z" />
          </svg>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import MasterFader from './master/MasterFader.vue'
import MasterMeter from './master/MasterMeter.vue'
import HeadphonesControl from './master/HeadphonesControl.vue'
import OutputSelector from './master/OutputSelector.vue'
import { ref, watch, onMounted, onUnmounted, nextTick, inject } from 'vue'
import { useAudioDevices } from '../composables/useAudioDevices'

// Props
interface Props {
  masterFxOutputNode?: any
  masterFxComponent?: any
  loadedTracks?: Array<{ trackNumber: number, fileName: string, fileId: string }>
}

const props = withDefaults(defineProps<Props>(), {
  loadedTracks: () => []
})

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Master volumes
const leftVolume = ref(0) // dB
const rightVolume = ref(0) // dB
const headphonesVolume = ref(-60) // dB
const isLinked = ref(true)
const masterMuted = ref(false)

// VU meter levels (will be updated by Rust engine)
const leftLevel = ref(-60)
const rightLevel = ref(-60)
const headphonesLevel = ref(-60)

// Audio outputs
const { audioOutputDevices } = useAudioDevices()
const selectedHeadphonesOutput = ref<string | null>(null)
const selectedMasterOutput = ref<string | null>(null)

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let resizeObserver: ResizeObserver | null = null

// Calculate meters height based on container
function updateMetersHeight() {
  if (metersContainer.value) {
    const height = metersContainer.value.clientHeight
    const availableHeight = Math.max(160, height - 80)
    vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.4))
    fadersHeight.value = Math.max(100, Math.floor(availableHeight * 0.6))
  }
}

// Handle master output selection
async function onMasterOutputSelect(deviceId: string | null) {
  selectedMasterOutput.value = deviceId
  console.log('[Master Output] Selected:', deviceId)
  
  // Restart audio engine with new output device
  if (audioEngine) {
    // '' or null means default device (undefined in Rust)
    if (!deviceId || deviceId === '') {
      console.log('[Master Output] Restarting engine with default output device')
      await audioEngine.restartWithDevices(undefined, undefined)
      console.log('[Master Output] Engine restarted with default output device')
    } else {
      const device = audioOutputDevices.value.find(d => d.id === deviceId)
      if (device) {
        const deviceLabel = device.name || `Device ${deviceId.substring(0, 8)}`
        console.log('[Master Output] Restarting engine with:', deviceLabel)
        await audioEngine.restartWithDevices(undefined, deviceLabel)
        console.log('[Master Output] Engine restarted with new output device')
      } else {
        console.warn('[Master Output] Device not found:', deviceId)
      }
    }
  }
}

// Handle headphones output selection
function onHeadphonesOutputSelect(deviceId: string | null) {
  selectedHeadphonesOutput.value = deviceId
  console.log('[Headphones Output] Selected:', deviceId)
  // TODO: Send to Rust engine to set headphones output device
  // audioEngine.setHeadphonesOutput(deviceId)
}

// Link/unlink channels
function toggleLink() {
  isLinked.value = !isLinked.value
  if (isLinked.value) {
    rightVolume.value = leftVolume.value
  }
}

// Toggle master mute
function toggleMasterMute() {
  masterMuted.value = !masterMuted.value
}

// Watchers - Send changes to Rust engine
watch([leftVolume, rightVolume], ([left, right]) => {
  if (audioEngine?.state.value.isRunning) {
    // Use average of L/R for master gain (since Rust has single master gain)
    const avgDb = (left + right) / 2
    
    // Convert dB to linear gain: gain = 10^(dB/20)
    let gainValue: number
    if (avgDb <= -90) {
      gainValue = 0.0 // Mute
    } else {
      gainValue = Math.pow(10, avgDb / 20)
    }
    
    audioEngine.setMasterGain(gainValue)
  }
})

watch(headphonesVolume, (volume) => {
  if (audioEngine?.state.value.isRunning) {
    // TODO: Send headphones volume to Rust engine
    // audioEngine.setHeadphonesVolume(volume)
    console.log('[Headphones] Volume changed:', volume)
  }
})

watch(masterMuted, (muted) => {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setMasterMute(muted)
  }
})

// Watch for meter level updates from audio engine
watch(
  () => audioEngine?.state.value.masterLevels,
  (levels) => {
    if (levels) {
      // Convert linear (0-1) to dB (-60 to 0)
      // dB = 20 * log10(linear)
      leftLevel.value = levels.left > 0 ? 20 * Math.log10(levels.left) : -60
      rightLevel.value = levels.right > 0 ? 20 * Math.log10(levels.right) : -60
    }
  },
  { deep: true }
)

// When linked, sync right to left
watch(leftVolume, (newVal) => {
  if (isLinked.value) {
    rightVolume.value = newVal
  }
})

// Initialize
onMounted(async () => {
  // Audio output devices are already enumerated during app initialization
  // No need to refresh them here

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

  // TODO: Start receiving meter levels from Rust engine
  // Set up periodic updates from Rust engine
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})

// Method to get current meter values for FX visualization
function getMeterValues() {
  return {
    left: leftLevel.value,
    right: rightLevel.value
  }
}

// Method to get pre-limiter meter values for limiter visualization
function getPreLimiterValues() {
  return {
    left: leftLevel.value,
    right: rightLevel.value
  }
}

// Expose refs and methods for parent component
defineExpose({
  leftVolume,
  rightVolume,
  masterMuted,
  selectedMasterOutput,
  selectedHeadphonesOutput,
  leftLevel,
  rightLevel,
  getMeterValues,
  getPreLimiterValues
})
</script>

<style scoped>
/* Add any component-specific styles here */
</style>
