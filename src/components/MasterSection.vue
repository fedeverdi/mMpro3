<template>
  <div
    class="master-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-2 flex flex-col items-center gap-1 h-full w-full max-w-[12rem]">
    <!-- Master Header -->
    <div class="w-full text-center">
      <div class="text-xs font-bold text-blue-400">MASTER</div>
    </div>

    <!-- Headphones Control -->
    <HeadphonesControl
      :devices="audioOutputs"
      :selected-device-id="selectedHeadphonesOutput"
      :volume="headphonesVolume"
      :level="headphonesLevel"
      @select="onHeadphonesOutputSelect"
      @update:volume="headphonesVolume = $event"
    />

    <!-- VU Meters and Faders -->
    <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-4 min-h-0 mt-2">
      <!-- VU Meters Row -->
      <div v-if="vuMetersHeight > 0"
        class="flex gap-4 w-full justify-center bg-gray-900 rounded p-1 border border-gray-700">
        <VuMeter :level="leftLevel" label="L" :height="vuMetersHeight" :width="25" />
        <VuMeter :level="rightLevel" label="R" :height="vuMetersHeight" :width="25" />
      </div>

      <!-- Faders Row -->
      <div v-if="fadersHeight > 0" class="flex gap-2 items-end mb-6">
        <MasterFader v-model="leftVolume" label="L" :trackHeight="fadersHeight" />
        <MasterFader v-model="rightVolume" label="R" :trackHeight="fadersHeight" />
      </div>
    </div>

    <!-- Master Controls -->
    <div class="w-full mt-2 flex gap-1">
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

      <!-- Recording Button -->
      <button @click="toggleRecording"
        class="flex-1 py-1 text-xs font-bold rounded transition-all flex items-center justify-center gap-1"
        :class="isRecording ? 'bg-red-700 text-white animate-pulse' : 'bg-red-600 hover:bg-red-700 text-white'">
        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="8" />
        </svg>
        <span v-if="isRecording" class="font-mono">{{ recordingTime }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import MasterFader from './MasterFader.vue'
import VuMeter from './VuMeter.vue'
import HeadphonesControl from './HeadphonesControl.vue'
import { ref, watch, onMounted, onUnmounted, nextTick, inject } from 'vue'

// Props
interface Props {
  masterEqDisplay?: any
}

const props = defineProps<Props>()

// Inject Tone.js from App.vue
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Master volumes
const leftVolume = ref(0)
const rightVolume = ref(0)
const headphonesVolume = ref(0)
const isLinked = ref(true)

// VU meter levels
const leftLevel = ref(-60)
const rightLevel = ref(-60)
const headphonesLevel = ref(-60)

// Recording state
const isRecording = ref(false)
const recordingTime = ref('00:00')

// Audio outputs
const audioOutputs = ref<MediaDeviceInfo[]>([])
const selectedHeadphonesOutput = ref<string | null>(null)

// Headphones output routing
let headphonesGain: any = null
let headphonesStreamDest: MediaStreamAudioDestinationNode | null = null
let headphonesAudioElement: HTMLAudioElement | null = null

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let resizeObserver: ResizeObserver | null = null

// Tone.js nodes
let masterChannel: any = null
let leftMeter: any = null
let rightMeter: any = null
let headphonesMeter: any = null
let splitNode: any = null
let leftGain: any = null
let rightGain: any = null
let mergeNode: any = null

// Calculate meters height based on container
function updateMetersHeight() {
  if (metersContainer.value) {
    const height = metersContainer.value.clientHeight
    const availableHeight = Math.max(160, height - 80)
    vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.4))
    fadersHeight.value = Math.max(100, Math.floor(availableHeight * 0.6))
  }
}

// Enumerate audio output devices
async function enumerateAudioOutputs() {
  try {
    // Request microphone permission to unlock device labels
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      stream.getTracks().forEach(track => track.stop())
    } catch (permError) {
      console.warn('[Audio Outputs] Permission denied for device labels')
    }

    const devices = await navigator.mediaDevices.enumerateDevices()
    audioOutputs.value = devices.filter(device => device.kind === 'audiooutput')
  } catch (error) {
    console.error('[Audio Outputs] Error enumerating devices:', error)
  }
}

// Handle headphones output selection
async function onHeadphonesOutputSelect(deviceId: string | null) {
  selectedHeadphonesOutput.value = deviceId
  
  if (!headphonesAudioElement) return
  
  try {
    if (deviceId) {
      // Set device and unmute
      await (headphonesAudioElement as any).setSinkId(deviceId)
      headphonesAudioElement.muted = false
      
      // Ensure playback
      if (headphonesAudioElement.paused) {
        await headphonesAudioElement.play()
      }
      
      console.log('[Headphones Output] Changed to:', deviceId)
    } else {
      // Mute when "Off"
      headphonesAudioElement.muted = true
      console.log('[Headphones Output] Muted (Off)')
    }
  } catch (error) {
    console.error('[Headphones Output] Error changing device:', error)
  }
}

// Initialize master channel
function initMasterChannel() {
  if (masterChannel || !Tone) {
    return
  }

  // Get output node from MasterEQDisplay
  if (props.masterEqDisplay && props.masterEqDisplay.getOutputNode) {
    masterChannel = props.masterEqDisplay.getOutputNode()
  } else {
    return
  }

  if (!masterChannel) return

  // Create stereo routing: Split → [LeftGain, RightGain] → Merge
  splitNode = new Tone.Split()
  leftGain = new Tone.Gain(1)
  rightGain = new Tone.Gain(1)
  mergeNode = new Tone.Merge()

  // Create meters
  leftMeter = new Tone.Meter()
  rightMeter = new Tone.Meter()
  headphonesMeter = new Tone.Meter()

  // Create headphones routing
  headphonesGain = new Tone.Gain(1)
  const audioContext = Tone.context.rawContext as AudioContext
  headphonesStreamDest = audioContext.createMediaStreamDestination()

  // Build main chain
  masterChannel.connect(splitNode)
  
  // Left channel: split → gain → meter → merge
  splitNode.connect(leftGain, 0)
  leftGain.connect(leftMeter)
  leftGain.connect(mergeNode, 0, 0)
  
  // Right channel: split → gain → meter → merge
  splitNode.connect(rightGain, 1)
  rightGain.connect(rightMeter)
  rightGain.connect(mergeNode, 0, 1)
  
  // Main output to default destination (uses main output selector)
  mergeNode.toDestination()
  
  // Headphones output: separate routing via MediaStream
  mergeNode.connect(headphonesGain)
  headphonesGain.connect(headphonesMeter)
  headphonesGain.connect(headphonesStreamDest as any)

  // Create headphones audio element
  if (!headphonesAudioElement && headphonesStreamDest) {
    headphonesAudioElement = new Audio()
    headphonesAudioElement.srcObject = headphonesStreamDest.stream
    headphonesAudioElement.muted = true // Start muted (Off)
    document.body.appendChild(headphonesAudioElement)
    
    // Start playback immediately (muted)
    headphonesAudioElement.play().catch(err => {
      console.warn('[Headphones] Autoplay blocked, will start on first interaction:', err)
    })
  }

  // Update initial volumes
  updateMasterVolume()
  updateHeadphonesVolume()
}

// Level monitoring
let levelMonitorInterval: number | null = null

function startLevelMonitoring() {
  levelMonitorInterval = window.setInterval(() => {
    if (leftMeter && rightMeter && Tone) {
      const leftValue = leftMeter.getValue() as number
      const rightValue = rightMeter.getValue() as number

      leftLevel.value = Math.max(-60, leftValue)
      rightLevel.value = Math.max(-60, rightValue)
    }

    if (headphonesMeter && Tone) {
      const hpValue = headphonesMeter.getValue() as number
      headphonesLevel.value = Math.max(-60, hpValue)
    }
  }, 50)
}

// Update master volume
function updateMasterVolume() {
  if (!leftGain || !rightGain || !Tone) {
    initMasterChannel()
    if (!leftGain || !rightGain) return
  }

  leftGain.gain.value = Tone.dbToGain(leftVolume.value)
  rightGain.gain.value = Tone.dbToGain(rightVolume.value)
}

// Update headphones volume
function updateHeadphonesVolume() {
  if (!headphonesGain || !Tone) return

  headphonesGain.gain.value = Tone.dbToGain(headphonesVolume.value)
}

// Watch for volume changes
watch([leftVolume, rightVolume], updateMasterVolume)
watch(headphonesVolume, updateHeadphonesVolume)

// When linked, sync right to left
watch(leftVolume, (newVal) => {
  if (isLinked.value) {
    rightVolume.value = newVal
  }
})

// Watch for masterEQDisplay to become available and init
watch(() => props.masterEqDisplay, (newVal) => {
  if (newVal && newVal.getOutputNode && !masterChannel) {
    setTimeout(() => {
      initMasterChannel()
    }, 100)
  }
}, { immediate: true })

// Link/unlink channels
function toggleLink() {
  isLinked.value = !isLinked.value
  if (isLinked.value) {
    rightVolume.value = leftVolume.value
  }
}

// Recording functions
function toggleRecording() {
  if (isRecording.value) {
    stopRecording()
  } else {
    startRecording()
  }
}

function startRecording() {
  // TODO: Implement recording
  isRecording.value = true
  console.log('[Recording] Started')
}

function stopRecording() {
  // TODO: Implement recording stop
  isRecording.value = false
  recordingTime.value = '00:00'
  console.log('[Recording] Stopped')
}

// Initialize
onMounted(async () => {
  // Get Tone.js from inject
  if (ToneRef?.value) {
    Tone = ToneRef.value
  } else {
    const checkTone = setInterval(() => {
      if (ToneRef?.value) {
        Tone = ToneRef.value
        clearInterval(checkTone)
      }
    }, 100)
  }

  // Enumerate audio outputs
  await enumerateAudioOutputs()

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

  // Start level monitoring
  startLevelMonitoring()

  // Listen for device changes
  navigator.mediaDevices.addEventListener('devicechange', enumerateAudioOutputs)
})

// Expose minimal interface
defineExpose({})

// Cleanup
onUnmounted(() => {
  if (leftMeter) leftMeter.dispose()
  if (rightMeter) rightMeter.dispose()
  if (headphonesMeter) headphonesMeter.dispose()
  if (splitNode) splitNode.dispose()
  if (leftGain) leftGain.dispose()
  if (rightGain) rightGain.dispose()
  if (mergeNode) mergeNode.dispose()
  if (headphonesGain) headphonesGain.dispose()

  // Cleanup headphones audio element
  if (headphonesAudioElement) {
    headphonesAudioElement.pause()
    headphonesAudioElement.srcObject = null
    headphonesAudioElement.remove()
    headphonesAudioElement = null
  }

  // Remove device change listener
  navigator.mediaDevices.removeEventListener('devicechange', enumerateAudioOutputs)

  if (resizeObserver) {
    resizeObserver.disconnect()
  }

  if (levelMonitorInterval) {
    clearInterval(levelMonitorInterval)
  }
})
</script>
