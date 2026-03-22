<template>
  <div
    class="master-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-2 flex flex-col items-center gap-1 h-full w-full w-[12rem] max-w-[12rem]">
    <!-- Master Header -->
    <div class="w-full text-center">
      <div class="text-xs font-bold text-blue-400">MASTER</div>
    </div>
    
    <!-- Master Output Selector -->
    <OutputSelector title="Master Output" :devices="audioOutputDevices" :selected-device-id="selectedMasterOutput"
      default-label="Default" default-description="Default audio output" icon="🔊" default-icon="🔊"
      :show-no-output="false" @select="onMasterOutputSelect" />

    <!-- Headphones Control -->
    <HeadphonesControl :devices="audioOutputDevices" :selected-device-id="selectedHeadphonesOutput"
      :volume="headphonesVolume" :level="headphonesLevel" @select="onHeadphonesOutputSelect"
      @update:volume="headphonesVolume = $event" />

    <!-- VU Meters and Faders -->
    <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-4 min-h-0 ">
      <!-- VU Meters Row -->
      <MasterMeter :left-level="leftLevel" :right-level="rightLevel" :vu-meters-height="vuMetersHeight" />

      <!-- Faders Row -->
      <div v-if="fadersHeight > 0" class="flex gap-2 items-end mb-6 mt-2">
        <MasterFader v-model="leftVolume" label="L" :trackHeight="fadersHeight" @drag-start="onDragStart('left')" @drag-end="onDragEnd('left')" />
        <MasterFader v-model="rightVolume" label="R" :trackHeight="fadersHeight" @drag-start="onDragStart('right')" @drag-end="onDragEnd('right')" />
      </div>
    </div>

    <!-- Master Controls -->
    <div class="w-full mt-2 flex gap-1">
      <!-- Recorder Button -->
      <RecorderButton :is-recording="isRecording" @open="emit('open-recorder')" />

      <!-- NDI Stream Button -->
      <button 
        @click="emit('open-ndi')" 
        class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isNdiStreaming ? 'bg-purple-600 text-white animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        title="NDI Stream Settings">
        <div class="flex items-center justify-center pl-[0.2rem]">
          <img 
            src="@/assets/ndi_logo.png" 
            class="h-2 w-auto"
            :class="isNdiStreaming ? 'invert' : 'invert opacity-70'"
            alt="NDI" />
        </div>
      </button>

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
import RecorderButton from './recorder/RecorderButton.vue'
import { ref, computed, watch, onMounted, onUnmounted, nextTick, inject, type Ref, type ComputedRef } from 'vue'
import { useNDI } from '../composables/useNDI'
import type { AudioDevice } from '../composables/useAudioEngine'

// Props
interface Props {
  masterFxOutputNode?: any
  isRecording?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isRecording: false
})

// Emits
const emit = defineEmits<{
  'open-recorder': []
  'open-ndi': []
}>()

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Master volumes
const leftVolume = ref(0) // dB
const rightVolume = ref(0) // dB
const headphonesVolume = ref(-60) // dB
const isLinked = ref(true)
const masterMuted = ref(false)
const isDraggingLeft = ref(false)
const isDraggingRight = ref(false)
let settlingTimer: ReturnType<typeof setTimeout> | null = null
let isSettling = false
let linkToggledAt = 0

// NDI Stream
const { isStreaming: isNdiStreaming } = useNDI()

// VU meter levels (will be updated by Rust engine)
const leftLevel = ref(-60)
const rightLevel = ref(-60)
const headphonesLevel = ref(-60)

// Audio outputs from engine
const audioOutputDevices: ComputedRef<AudioDevice[]> = computed(() => audioEngine?.state.value.availableOutputDevices || [])
const selectedHeadphonesOutput = ref<string | null>(null)
const selectedMasterOutput = ref<string | null>(null)

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
      const availableHeight = Math.max(160, height - 80)
      vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.4))
      fadersHeight.value = Math.max(100, Math.floor(availableHeight * 0.6))
    }
    updateMetersHeightTimeout = null
  }, 16) // ~60fps
}

// Use centralized resize trigger from parent
const resizeTrigger = inject<Ref<number>>('resizeTrigger', ref(0))

// Handle master output selection
async function onMasterOutputSelect(deviceId: string | null) {
  if (!audioEngine) return

  // Parse device ID (format: "deviceId" or "deviceId:leftCh:rightCh")
  const parts = deviceId?.split(':') || []
  const actualDeviceId = parts[0]
  const leftChannel = parts[1] ? parseInt(parts[1]) : 0
  const rightChannel = parts[2] ? parseInt(parts[2]) : 1

  // Save current device BEFORE any updates
  const previousDeviceId = selectedMasterOutput.value?.split(':')[0]
  
  // Check if we're just changing channels on the same device
  if (actualDeviceId === previousDeviceId && previousDeviceId && actualDeviceId !== 'no-output' && actualDeviceId !== '') {
    // Same device, different channels - just update channels without restart
    audioEngine.setMasterOutputChannels(leftChannel, rightChannel)
    
    // Update selected_master_output to include new channels
    await audioEngine.setSelectedMasterOutput(deviceId && deviceId !== '' ? deviceId : null)
  } else {
    // Different device - need to restart audio engine
    
    // Save selected output to backend first
    await audioEngine.setSelectedMasterOutput(deviceId && deviceId !== '' ? deviceId : null)
    
    if (!deviceId || deviceId === '' || actualDeviceId === 'no-output') {
      // Default device
      await audioEngine.restartWithDevices(undefined, undefined)
    } else {
      // Specific device
      const device = audioOutputDevices.value.find(d => d.id === actualDeviceId)
      if (device) {
        const deviceLabel = device.name || `Device ${actualDeviceId.substring(0, 8)}`
        await audioEngine.restartWithDevices(undefined, deviceLabel)

        // Wait for engine to be ready (max 2 seconds)
        const startTime = Date.now()
        while (!audioEngine.state.value.isRunning && Date.now() - startTime < 2000) {
          await new Promise(resolve => setTimeout(resolve, 50))
        }

        if (audioEngine.state.value.isRunning) {
          // Set channel selection after restart
          audioEngine.setMasterOutputChannels(leftChannel, rightChannel)
        }
      }
    }
  }
}

// Handle headphones output selection
function onHeadphonesOutputSelect(deviceId: string | null) {
  selectedHeadphonesOutput.value = deviceId
  // TODO: Send to Rust engine to set headphones output device
  // audioEngine.setHeadphonesOutput(deviceId)
}

function onDragStart(side: 'left' | 'right') {
  if (settlingTimer) clearTimeout(settlingTimer)
  isSettling = false
  if (side === 'left') isDraggingLeft.value = true
  else isDraggingRight.value = true
}

function onDragEnd(side: 'left' | 'right') {
  if (side === 'left') isDraggingLeft.value = false
  else isDraggingRight.value = false
  // Keep ignoring backend echoes for 400ms after release (clears in-flight responses)
  isSettling = true
  if (settlingTimer) clearTimeout(settlingTimer)
  settlingTimer = setTimeout(() => { isSettling = false }, 400)
}

// Link/unlink channels
function toggleLink() {
  linkToggledAt = Date.now()
  isLinked.value = !isLinked.value
}

// Toggle master mute
function toggleMasterMute() {
  masterMuted.value = !masterMuted.value
}

// Watchers - Send changes to Rust engine

// Fader moved by user → send to backend
watch(leftVolume, (val) => {
  if (!audioEngine?.state.value.isRunning) return
  const gain = val <= -90 ? 0.0 : Math.pow(10, val / 20)
  if (isLinked.value) {
    rightVolume.value = val           // keep right in sync (UI only)
    audioEngine.setMasterGain(gain)
    audioEngine.setMasterGainLeft(gain)
    audioEngine.setMasterGainRight(gain)
  } else {
    audioEngine.setMasterGainLeft(gain)
  }
})

watch(rightVolume, (val) => {
  if (!audioEngine?.state.value.isRunning) return
  if (isLinked.value) return          // handled by leftVolume watcher
  const gain = val <= -90 ? 0.0 : Math.pow(10, val / 20)
  audioEngine.setMasterGainRight(gain)
})

watch(masterMuted, (muted) => {
  if (audioEngine?.state.value.isRunning) audioEngine.setMasterMute(muted)
})

watch(isLinked, (linked) => {
  if (!audioEngine?.state.value.isRunning) return
  audioEngine.setMasterLinked(linked)
  if (linked) {
    // Re-link: align right to left and send unified gain
    rightVolume.value = leftVolume.value
    const gain = leftVolume.value <= -90 ? 0.0 : Math.pow(10, leftVolume.value / 20)
    audioEngine.setMasterGain(gain)
    audioEngine.setMasterGainLeft(gain)
    audioEngine.setMasterGainRight(gain)
  }
})

// VU meter levels — updated at 60fps from meters stream
watch(
  () => ({
    left: audioEngine?.state.value.masterLevels.left,
    right: audioEngine?.state.value.masterLevels.right,
    selectedMasterOutput: audioEngine?.state.value.masterLevels.selectedMasterOutput,
  }),
  (v) => {
    if (!v) return
    leftLevel.value = v.left ?? -60
    rightLevel.value = v.right ?? -60
    if (v.selectedMasterOutput !== selectedMasterOutput.value) {
      selectedMasterOutput.value = v.selectedMasterOutput ?? null
    }
  }
)

// Fader / link / mute sync — only fires when these specific primitives change
watch(
  [
    () => audioEngine?.state.value.masterLevels.gain,
    () => audioEngine?.state.value.masterLevels.gainLeft,
    () => audioEngine?.state.value.masterLevels.gainRight,
    () => audioEngine?.state.value.masterLevels.linked,
    () => audioEngine?.state.value.masterLevels.mute,
  ],
  ([gain, gainLeft, gainRight, linked, mute]) => {
    if (isDraggingLeft.value || isDraggingRight.value || isSettling) return

    // Don't let stale backend echoes override link state for 500ms after user toggle
    if (Date.now() - linkToggledAt > 500) {
      isLinked.value = linked ?? true
    }

    if (linked) {
      const db = gain > 0 ? 20 * Math.log10(gain) : -90
      leftVolume.value = db
      rightVolume.value = db
    } else {
      leftVolume.value  = gainLeft  > 0 ? 20 * Math.log10(gainLeft)  : -90
      rightVolume.value = gainRight > 0 ? 20 * Math.log10(gainRight) : -90
    }

    masterMuted.value = mute
  }
)

// Initialize
onMounted(async () => {
  // Audio output devices are already enumerated during app initialization
  // No need to refresh them here

  // Initialize selectedMasterOutput from backend
  // The Rust backend auto-selects the default device on startup
  const backendSelected = audioEngine?.state.value.masterLevels.selectedMasterOutput
  
  if (backendSelected) {
    selectedMasterOutput.value = backendSelected
  }

  // Calculate initial height
  await nextTick()
  updateMetersHeight()

  // Watch for centralized resize trigger instead of using ResizeObserver
  watch(resizeTrigger, () => {
    updateMetersHeight()
  })

  // TODO: Start receiving meter levels from Rust engine
  // Set up periodic updates from Rust engine
})

onUnmounted(() => {
  if (updateMetersHeightTimeout) {
    clearTimeout(updateMetersHeightTimeout)
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
  getPreLimiterValues,
})
</script>

<style scoped>
/* Add any component-specific styles here */
</style>
