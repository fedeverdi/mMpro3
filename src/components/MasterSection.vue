<template>
  <div
    class="master-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-2 flex flex-col items-center gap-1 h-full w-full max-w-[12rem]">
    <!-- Master Header -->
    <div class="w-full text-center">
      <div class="text-xs font-bold text-blue-400">MASTER</div>
    </div>

    <!-- Audio Output Selectors -->
    <div class="w-full flex gap-1 bg-gray-900 rounded p-1.5 border border-gray-700">
      <button @click="showMainOutputModal = true"
        class="flex-1 text-[0.65rem] bg-gray-800 text-gray-200 border border-gray-600 rounded px-1.5 py-1 hover:bg-gray-700 hover:border-blue-500 focus:border-blue-500 focus:outline-none transition-all text-left flex items-center justify-between gap-1 min-w-0">
        <span class="truncate">🔊 {{ getDeviceLabel(selectedMainOutput, 'Default') }}</span>
        <svg class="w-2.5 h-2.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      <button @click="showHeadphonesModal = true"
        class="flex-1 text-[0.65rem] bg-gray-800 text-gray-200 border border-gray-600 rounded px-1.5 py-1 hover:bg-gray-700 hover:border-blue-500 focus:border-blue-500 focus:outline-none transition-all text-left flex items-center justify-between gap-1 min-w-0">
        <span class="truncate">🎧 {{ getDeviceLabel(selectedHeadphonesOutput, 'Off') }}</span>
        <svg class="w-2.5 h-2.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
    </div>

    <!-- Headphones Volume Control -->
    <div class="w-full flex items-center justify-center gap-2 bg-gray-900 rounded border border-gray-700 px-2 py-1">
      <div class="scale-75">
        <Knob v-model="headphonesVolume" :min="-60" :max="6" :step="0.5" label="HP Vol" unit="dB" color="#14b8a6" />
      </div>
      <!-- HP Level Indicator -->
      <div class="flex flex-col gap-1">
        <div class="w-3 h-3 rounded-full transition-all" :class="headphonesLevel > -6 ? 'bg-red-500 shadow-lg shadow-red-500/50' : 'bg-gray-700'"></div>
        <div class="w-3 h-3 rounded-full transition-all" :class="headphonesLevel > -20 ? 'bg-yellow-500 shadow-lg shadow-yellow-500/50' : 'bg-gray-700'"></div>
        <div class="w-3 h-3 rounded-full transition-all" :class="headphonesLevel > -35 ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-700'"></div>
        <div class="w-3 h-3 rounded-full transition-all" :class="headphonesLevel > -50 ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-700'"></div>
      </div>
    </div>

    <!-- Peak Levels Display -->
    <!-- <div class="w-full bg-gray-900 rounded px-3 py-2 border border-gray-700 mb-2">
      <div class="flex justify-around text-[0.65rem] font-mono tracking-wide">
        <div>
          <span class="truncate" :class="getLevelColor(leftLevel)">{{ formatLevel(leftLevel) }}</span>
        </div>
        <div>
          <span class="truncate" :class="getLevelColor(rightLevel)">{{ formatLevel(rightLevel) }}</span>
        </div>
      </div>
    </div> -->

    <!-- VU Meters and Faders -->
    <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-4 min-h-0 mt-2">
      <!-- VU Meters Row -->
      <div v-if="vuMetersHeight > 0"
        class="flex gap-4 w-full justify-center bg-gray-900 rounded p-1 border border-gray-700">
        <VuMeter :level="leftLevel" label="L" :height="vuMetersHeight" :width="25" />
        <VuMeter :level="rightLevel" label="R" :height="vuMetersHeight" :width="25" />
      </div>

      <!-- Faders Row -->
      <div v-if="fadersHeight > 0" class="flex gap-2 items-end">
        <MasterFader v-model="leftVolume" label="Left" :trackHeight="fadersHeight" />
        <MasterFader v-model="rightVolume" label="Right" :trackHeight="fadersHeight" />
      </div>
    </div>

    <!-- Master Controls -->
    <div class="w-full mt-4 flex gap-1">
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
        <svg v-if="isRecording" class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="8" />
        </svg>
        <svg v-else class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="8" />
        </svg>
        <span v-if="isRecording" class="font-mono">{{ recordingTime }}</span>
      </button>
    </div>

    <!-- Audio Output Modals -->
    <AudioOutputModal :is-open="showMainOutputModal" title="Select Main Output" :devices="audioOutputs"
      :selected-device-id="selectedMainOutput" default-label="System Default"
      default-description="Use default audio output" default-icon="🔊" @close="showMainOutputModal = false"
      @select="onMainOutputSelect" />

    <AudioOutputModal :is-open="showHeadphonesModal" title="Select Headphones Output" :devices="audioOutputs"
      :selected-device-id="selectedHeadphonesOutput" default-label="Off" default-description="No headphones output"
      default-icon="🔇" @close="showHeadphonesModal = false" @select="onHeadphonesOutputSelect" />
  </div>

</template>

<script setup lang="ts">
import AudioOutputModal from './AudioOutputModal.vue'
import MasterFader from './MasterFader.vue'
import Knob from './Knob.vue'
import VuMeter from './VuMeter.vue'
import { ref, watch, onMounted, onUnmounted, nextTick, inject, toRaw } from 'vue'

// Props
interface Props {
  masterChannel?: any
}

const props = defineProps<Props>()

// Inject Tone.js from App.vue
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Master volumes
const leftVolume = ref(0)
const rightVolume = ref(0)
const isLinked = ref(true)

// VU meter levels
const leftLevel = ref(-60)
const rightLevel = ref(-60)
const headphonesLevel = ref(-60)

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let resizeObserver: ResizeObserver | null = null

// Audio outputs
const audioOutputs = ref<MediaDeviceInfo[]>([])
const selectedMainOutput = ref<string>('')
const selectedHeadphonesOutput = ref<string>('')
const showMainOutputModal = ref(false)
const showHeadphonesModal = ref(false)
const headphonesVolume = ref(0)

// Audio elements for routing
let mainAudioElement: HTMLAudioElement | null = null
let headphonesAudioElement: HTMLAudioElement | null = null
let mainStreamDestination: MediaStreamAudioDestinationNode | null = null
let headphonesStreamDestination: MediaStreamAudioDestinationNode | null = null
let mainOutputGain: any = null
let headphonesOutputGain: any = null

// Recording state
const isRecording = ref(false)
const recordingTime = ref('00:00')
let mediaRecorder: MediaRecorder | null = null
let recordedChunks: Blob[] = []
let recordingStartTime: number = 0
let recordingInterval: number | null = null

// Tone.js nodes
let masterChannel: any = null
let leftMeter: any = null
let rightMeter: any = null
let headphonesMeter: any = null  // Meter for headphones output
let preLimiterLeftMeter: any = null  // Meter before limiter
let preLimiterRightMeter: any = null  // Meter before limiter
let preLimiterSplit: any = null  // Split for pre-limiter metering
let splitNode: any = null
let masterParametricEQ: any = null
let analysisTap: any = null // Punto stabile per spectrum meter
let fxChainOutput: any = null // Output after FX chain (before faders)
let masterFaderSplit: any = null // Split stereo for L/R fader control
let masterFaderLeftGain: any = null // Left channel gain
let masterFaderRightGain: any = null // Right channel gain
let masterFaderMerge: any = null // Native ChannelMerger for stereo (Tone.Merge collapses to mono)
let masterFaderMergeWrapper: any = null // Tone.Gain wrapper around native merger

// Master EQ filters data for scenes
const masterEQFiltersData = ref<any[]>([])

// FX nodes
let compressor: any = null
let reverb: any = null
let delay: any = null
let limiter: any = null
let limiterGain: any = null  // Pre-gain for limiter
let limiterWaveShaper: any = null  // Hard clipper
let limiterUpdateTimeout: number | null = null

// FX enabled states
const compressorEnabled = ref(false)
const reverbEnabled = ref(false)
const delayEnabled = ref(false)
const limiterEnabled = ref(false)

// Reactive FX references for components
const compressorNode = ref<any>(null)
const reverbNode = ref<any>(null)
const delayNode = ref<any>(null)
const limiterNode = ref<any>(null)

// Reactive master output
const masterOutput = ref<any>(null)
const analysisOutput = ref<any>(null)

// Calculate meters height based on container
function updateMetersHeight() {
  if (metersContainer.value) {
    const height = metersContainer.value.clientHeight
    const availableHeight = Math.max(160, height - 80)
    // VU meters get 30% of space, faders get 70%
    vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.4))
    fadersHeight.value = Math.max(100, Math.floor(availableHeight * 0.6))
  }
}

// Enumerate audio output devices
async function enumerateAudioOutputs() {
  try {
    // Request microphone permission to unlock device labels (browser privacy requirement)
    // We immediately stop the stream after getting permission
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      stream.getTracks().forEach(track => track.stop())
    } catch (permError) {
      console.warn('[Audio Outputs] Permission denied for device labels, will show generic names')
    }

    const devices = await navigator.mediaDevices.enumerateDevices()
    audioOutputs.value = devices.filter(device => device.kind === 'audiooutput')
  } catch (error) {
    console.error('[Audio Outputs] Error enumerating devices:', error)
  }
}

// Initialize audio elements and stream destinations
function initAudioOutputs() {
  console.log('[initAudioOutputs] Called. Tone:', !!Tone, 'Tone.context:', !!Tone?.context)
  
  if (!Tone || !Tone.context) return

  console.log('[initAudioOutputs] Creating media stream destinations...')
  
  // Create media stream destinations
  mainStreamDestination = Tone.context.rawContext.createMediaStreamDestination()
  headphonesStreamDestination = Tone.context.rawContext.createMediaStreamDestination()

  console.log('[initAudioOutputs] Created destinations:', !!mainStreamDestination, !!headphonesStreamDestination)

  // Create Tone.js gain nodes for routing
  mainOutputGain = new Tone.Gain(1)
  headphonesOutputGain = new Tone.Gain(1)

  // Create hidden audio elements
  if (!mainAudioElement && mainStreamDestination) {
    console.log('[initAudioOutputs] Creating mainAudioElement...')
    mainAudioElement = document.createElement('audio')
    mainAudioElement.autoplay = true
    mainAudioElement.volume = 1.0 // Full volume
    mainAudioElement.muted = false // ALWAYS ACTIVE - this is the main output
    mainAudioElement.srcObject = mainStreamDestination.stream
    document.body.appendChild(mainAudioElement)

    console.log('[initAudioOutputs] mainAudioElement created and appended to body')

    // Force play (handle autoplay policy)
    mainAudioElement.play().then(() => {
      console.log('[initAudioOutputs] Main audio element started successfully')
    }).catch(err => {
      console.warn('[Audio Outputs] Main autoplay blocked, will start on user interaction:', err)
    })
  } else {
    console.log('[initAudioOutputs] Skipping mainAudioElement creation - already exists or no destination')
  }

  if (!headphonesAudioElement && headphonesStreamDestination) {
    headphonesAudioElement = document.createElement('audio')
    headphonesAudioElement.autoplay = true
    headphonesAudioElement.volume = 1.0 // Full volume
    headphonesAudioElement.muted = true // Start muted (will be unmuted when device is selected)
    headphonesAudioElement.srcObject = headphonesStreamDestination.stream
    document.body.appendChild(headphonesAudioElement)

    // Force play (handle autoplay policy)
    headphonesAudioElement.play().then(() => {
    }).catch(err => {
      console.warn('[Audio Outputs] Headphones autoplay blocked, will start on user interaction:', err)
    })
  }
}

// Get device label by ID
function getDeviceLabel(deviceId: string, defaultLabel: string): string {
  if (!deviceId) return defaultLabel
  const device = audioOutputs.value.find(d => d.deviceId === deviceId)
  return device?.label || `Audio Output ${deviceId.substring(0, 12)}...`
}

// Handle main output selection from modal
async function onMainOutputSelect(deviceId: string) {
  selectedMainOutput.value = deviceId
  await changeMainOutput()
}

// Handle headphones output selection from modal
async function onHeadphonesOutputSelect(deviceId: string) {
  selectedHeadphonesOutput.value = deviceId
  await changeHeadphonesOutput()
}

// Change main output device
async function changeMainOutput() {
  if (!mainAudioElement) return

  try {
    // Ensure main output is NEVER muted
    mainAudioElement.muted = false

    if (selectedMainOutput.value) {
      await (mainAudioElement as any).setSinkId(selectedMainOutput.value)
    } else {
      await (mainAudioElement as any).setSinkId('')
    }

    // Ensure playback continues after device change
    mainAudioElement.play().catch(err => {
      console.warn('[Main Output] Play after device change blocked:', err)
    })
  } catch (error) {
    console.error('[Main Output] Error changing device:', error)
  }
}

// Change headphones output device
async function changeHeadphonesOutput() {
  if (!headphonesAudioElement) return

  try {
    if (selectedHeadphonesOutput.value) {
      // Unmute and set to selected device
      headphonesAudioElement.muted = false
      await (headphonesAudioElement as any).setSinkId(selectedHeadphonesOutput.value)

      // Ensure playback starts
      headphonesAudioElement.play().catch(err => {
        console.warn('[Headphones Output] Play blocked:', err)
      })
    } else {
      // If "Off", mute the audio element to prevent output
      headphonesAudioElement.muted = true
    }
  } catch (error) {
    console.error('[Headphones Output] Error changing device:', error)
  }
}

// Ensure audio elements are playing (call this from track when user clicks play)
function ensureAudioPlaying() {
  console.log('[ensureAudioPlaying] Called. mainAudioElement:', mainAudioElement, 'paused?', mainAudioElement?.paused)
  
  // Check if mainStreamDestination has active audio tracks
  if (mainStreamDestination) {
    const streamTracks = mainStreamDestination.stream.getTracks()
    console.log('[ensureAudioPlaying] mainStreamDestination.stream tracks:', streamTracks.length, streamTracks.map(t => ({ kind: t.kind, enabled: t.enabled, readyState: t.readyState })))
  }
  
  console.log('[ensureAudioPlaying] headphonesAudioElement:', headphonesAudioElement, 'paused?', headphonesAudioElement?.paused)
  
  if (mainAudioElement && mainAudioElement.paused) {
    console.log('[ensureAudioPlaying] Main audio is paused, calling play()...')
    mainAudioElement.play().catch(err => {
      console.warn('[Audio] Main output play blocked:', err)
    })
  } else if (mainAudioElement) {
    console.log('[ensureAudioPlaying] Main audio already playing')
  } else {
    console.error('[ensureAudioPlaying] mainAudioElement is NULL!')
  }
  
  if (headphonesAudioElement && headphonesAudioElement.paused && !headphonesAudioElement.muted) {
    headphonesAudioElement.play().catch(err => {
      console.warn('[Audio] Headphones output play blocked:', err)
    })
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
  if (!mainStreamDestination || !Tone) {
    alert('Audio system not ready. Please try again.')
    return
  }

  try {
    // Clear previous recording
    recordedChunks = []

    // Create MediaRecorder from the master stream
    const stream = mainStreamDestination.stream

    // Try to use high quality audio codec
    let options: MediaRecorderOptions = { mimeType: 'audio/webm;codecs=opus' }

    if (options.mimeType && !MediaRecorder.isTypeSupported(options.mimeType)) {
      options = { mimeType: 'audio/webm' }
      if (options.mimeType && !MediaRecorder.isTypeSupported(options.mimeType)) {
        options = {} // Use default
      }
    }

    mediaRecorder = new MediaRecorder(stream, options)

    mediaRecorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        recordedChunks.push(event.data)
      }
    }

    mediaRecorder.onstop = () => {
      saveRecording()
    }

    mediaRecorder.start(1000) // Collect data every second
    isRecording.value = true
    recordingStartTime = Date.now()

    // Update recording time display
    recordingInterval = window.setInterval(() => {
      const elapsed = Math.floor((Date.now() - recordingStartTime) / 1000)
      const minutes = Math.floor(elapsed / 60)
      const seconds = elapsed % 60
      recordingTime.value = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
    }, 1000)

  } catch (error) {
    console.error('[Recording] Error starting recording:', error)
    alert('Error starting recording: ' + error)
  }
}

function stopRecording() {
  if (mediaRecorder && mediaRecorder.state !== 'inactive') {
    mediaRecorder.stop()
    isRecording.value = false

    if (recordingInterval) {
      clearInterval(recordingInterval)
      recordingInterval = null
    }
  }
}

function saveRecording() {
  if (recordedChunks.length === 0) {
    console.warn('[Recording] No data to save')
    return
  }

  // Determine MIME type and extension
  const mimeType = mediaRecorder?.mimeType || 'audio/webm'
  let extension = 'webm'

  if (mimeType.includes('mp4')) {
    extension = 'mp4'
  } else if (mimeType.includes('ogg')) {
    extension = 'ogg'
  }

  // Create blob from recorded chunks
  const blob = new Blob(recordedChunks, { type: mimeType })

  // Create download link
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.style.display = 'none'
  a.href = url

  // Generate filename with timestamp
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5)
  a.download = `master-recording-${timestamp}.${extension}`

  document.body.appendChild(a)
  a.click()

  // Cleanup
  setTimeout(() => {
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, 100)

  recordingTime.value = '00:00'
  recordedChunks = []

  console.log(`[Recording] Saved as ${a.download}`)
}

// Initialize master output
onMounted(async () => {
  const startTime = performance.now()

  // Get Tone.js from inject
  if (ToneRef?.value) {
    Tone = ToneRef.value
  } else {
    // Fallback: wait for it
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

  // Initialize master channel immediately so it's ready for tracks to connect
  initMasterChannel()

  // Start level monitoring
  startLevelMonitoring()

  // Listen for device changes
  navigator.mediaDevices.addEventListener('devicechange', enumerateAudioOutputs)
})

// Initialize master channel (called on first access)
function initMasterChannel() {
  if (masterChannel || !Tone) {
    return
  }

  // Use master channel from props if provided, otherwise create new one
  if (props.masterChannel) {
    masterChannel = toRaw(props.masterChannel)
  } else {
    // Create master channel with separate left/right control
    masterChannel = new Tone.Channel({
      volume: 0,
      pan: 0
    })
    
    // Configure master channel to maintain stereo (don't collapse to mono)
    const masterChannelIn = masterChannel.input as AudioNode
    masterChannelIn.channelCount = 2
    masterChannelIn.channelCountMode = 'explicit'
    masterChannelIn.channelInterpretation = 'discrete'
    
    const masterChannelOut = masterChannel.output as AudioNode
    masterChannelOut.channelCount = 2
    masterChannelOut.channelCountMode = 'explicit'
    masterChannelOut.channelInterpretation = 'discrete'
  }


  // Create meters for L/R
  leftMeter = new Tone.Meter()
  rightMeter = new Tone.Meter()

  // Split signal for metering (doesn't interrupt audio flow)
  splitNode = new Tone.Split()
  splitNode.connect(leftMeter, 0)
  splitNode.connect(rightMeter, 1)

  // Create headphones meter
  headphonesMeter = new Tone.Meter()

  // Create pre-limiter meters
  preLimiterSplit = new Tone.Split()
  preLimiterLeftMeter = new Tone.Meter()
  preLimiterRightMeter = new Tone.Meter()

  // Create analysis tap (stable connection point for spectrum meter)
  analysisTap = new Tone.Gain(1) // Unity gain, transparent
  
  // Configure analysisTap for stereo passthrough
  const analysisTapIn = analysisTap.input as GainNode
  analysisTapIn.channelCount = 2
  analysisTapIn.channelCountMode = 'explicit'
  analysisTapIn.channelInterpretation = 'discrete'
  
  const analysisTapOut = analysisTap.output as GainNode
  analysisTapOut.channelCount = 2
  analysisTapOut.channelCountMode = 'explicit'
  analysisTapOut.channelInterpretation = 'discrete'

  // Create FX chain output (after all effects, before routing split)
  // CRITICAL: Must maintain stereo from tracks
  fxChainOutput = new Tone.Gain(1)
  const fxChainOutputIn = fxChainOutput.input as GainNode
  fxChainOutputIn.channelCount = 2
  fxChainOutputIn.channelCountMode = 'explicit'
  fxChainOutputIn.channelInterpretation = 'discrete'
  
  const fxChainOutputOut = fxChainOutput.output as GainNode
  fxChainOutputOut.channelCount = 2
  fxChainOutputOut.channelCountMode = 'explicit'
  fxChainOutputOut.channelInterpretation = 'discrete'

  // Create master fader stereo controls (for main output only)
  masterFaderSplit = new Tone.Split()
  
  // Configure split input to receive stereo without down-mixing
  const splitInputNode = masterFaderSplit.input as AudioNode
  splitInputNode.channelCount = 2
  splitInputNode.channelCountMode = 'explicit'
  splitInputNode.channelInterpretation = 'discrete'
  
  masterFaderLeftGain = new Tone.Gain(1) // Start at unity
  masterFaderRightGain = new Tone.Gain(1) // Start at unity
  
  // Use native ChannelMerger instead of Tone.Merge (which collapses to mono)
  masterFaderMerge = Tone.context.createChannelMerger(2)
  
  // Wrap in Tone.Gain for compatibility with Tone.js
  masterFaderMergeWrapper = new Tone.Gain(1.0)
  const wrapperNode = masterFaderMergeWrapper.input as GainNode
  wrapperNode.channelCount = 2
  wrapperNode.channelCountMode = 'explicit'
  wrapperNode.channelInterpretation = 'discrete'

  // Initialize audio outputs
  initAudioOutputs()

  // Connect routing chain (with or without parametric EQ)
  connectMasterRouting()

  // Expose the master output and analysis tap
  masterOutput.value = masterChannel
  analysisOutput.value = analysisTap
}

// Expose current meter values for FX visualization
function getMeterValues() {
  if (!leftMeter || !rightMeter) return { left: -60, right: -60 }
  const leftValue = leftMeter.getValue() as number
  const rightValue = rightMeter.getValue() as number
  return {
    left: Math.max(-60, leftValue),
    right: Math.max(-60, rightValue)
  }
}

// Expose pre-limiter meter values for limiter visualization
function getPreLimiterValues() {
  if (!preLimiterLeftMeter || !preLimiterRightMeter) {
    return { left: -60, right: -60 }
  }

  const leftValue = preLimiterLeftMeter.getValue() as number
  const rightValue = preLimiterRightMeter.getValue() as number

  return {
    left: Math.max(-60, leftValue),
    right: Math.max(-60, rightValue)
  }
}

// Level monitoring
let levelMonitorInterval: number | null = null
let levelMonitorCount = 0
function startLevelMonitoring() {

  levelMonitorInterval = window.setInterval(() => {
    if (leftMeter && rightMeter && Tone) {
      const leftValue = leftMeter.getValue() as number
      const rightValue = rightMeter.getValue() as number

      // Tone.Meter already returns dB values (negative numbers)
      leftLevel.value = Math.max(-60, leftValue)
      rightLevel.value = Math.max(-60, rightValue)

      // Update headphones meter
      if (headphonesMeter) {
        const hpValue = headphonesMeter.getValue() as number
        headphonesLevel.value = Math.max(-60, hpValue)
      }

      levelMonitorCount++
    }
  }, 50)
}

// Link/unlink channels
function toggleLink() {
  isLinked.value = !isLinked.value
  if (isLinked.value) {
    rightVolume.value = leftVolume.value
  }
}

// Update master parameters
function updateMasterVolume() {
  if (!masterFaderLeftGain || !masterFaderRightGain || !Tone) {
    // Initialize on first volume change (user interaction)
    initMasterChannel()
    if (!masterFaderLeftGain || !masterFaderRightGain) return
  }

  // Apply separate volume to each channel
  const leftGainValue = Tone.dbToGain(leftVolume.value)
  const rightGainValue = Tone.dbToGain(rightVolume.value)
  
  // Direct assignment to avoid scheduling events
  masterFaderLeftGain.gain.value = leftGainValue
  masterFaderRightGain.gain.value = rightGainValue
}

function connectMasterRouting() {
  if (!masterChannel || !analysisTap || !Tone) return

  // Disconnect current routing BEFORE analysisTap
  // Non tocchiamo mai analysisTap -> split (rimane stabile per lo spectrum meter)
  try {
    masterChannel.disconnect()
  } catch (e) {
    // Ignore if already disconnected
  }

  // Disconnect EQ output if exists
  if (masterParametricEQ) {
    try {
      masterParametricEQ.output.disconnect()
    } catch (e) {
      // Ignore if already disconnected
    }
  }

  // Disconnect FX if they exist
  if (compressor) {
    try { compressor.disconnect() } catch (e) { }
  }
  if (reverb) {
    try { reverb.disconnect() } catch (e) { }
  }
  if (delay) {
    try { delay.disconnect() } catch (e) { }
  }
  if (limiter) {
    try { limiter.disconnect() } catch (e) { }
  }

  // Disconnect output gains
  if (mainOutputGain) {
    try { mainOutputGain.disconnect() } catch (e) { }
  }
  if (headphonesOutputGain) {
    try { headphonesOutputGain.disconnect() } catch (e) { }
  }
  if (masterFaderSplit) {
    try { masterFaderSplit.disconnect() } catch (e) { }
  }
  if (masterFaderLeftGain) {
    try { masterFaderLeftGain.disconnect() } catch (e) { }
  }
  if (masterFaderRightGain) {
    try { masterFaderRightGain.disconnect() } catch (e) { }
  }
  if (masterFaderMerge) {
    try { masterFaderMerge.disconnect() } catch (e) { }
  }
  if (masterFaderMergeWrapper) {
    try { masterFaderMergeWrapper.disconnect() } catch (e) { }
  }
  if (fxChainOutput) {
    try { 
      fxChainOutput.disconnect()
    } catch (e) { }
  }
  // Note: splitNode is never disconnected because it's always connected to meters
  // We only change what feeds INTO splitNode

  // Build chain: master -> [parametricEQ] -> [compressor] -> [reverb] -> [limiter] -> fxChainOutput
  // From fxChainOutput:
  //   -> masterFaderSplit -> L/R gain controls -> masterFaderMerge -> mainOutputGain -> speakers
  //   -> headphonesOutputGain (controlled by HP knob) -> headphones
  //   -> analysisTap -> spectrum analyzer
  let currentNode: any = masterChannel

  // 1. Parametric EQ (if exists)
  if (masterParametricEQ) {
    currentNode.connect(masterParametricEQ.input)
    currentNode = masterParametricEQ.output
  }

  // 2. Compressor (if enabled)
  if (compressorEnabled.value && compressor) {
    currentNode.connect(compressor)
    currentNode = compressor
  }

  // 3. Reverb (if enabled)
  if (reverbEnabled.value && reverb) {
    currentNode.connect(reverb)
    currentNode = reverb
  }

  // 4. Delay (if enabled)
  if (delayEnabled.value && delay) {
    currentNode.connect(delay)
    currentNode = delay
  }

  // 5. Limiter (if enabled)
  if (limiterEnabled.value && limiter) {
    // Connect pre-limiter meters
    currentNode.connect(preLimiterSplit!)
    preLimiterSplit!.connect(preLimiterLeftMeter!, 0)
    preLimiterSplit!.connect(preLimiterRightMeter!, 1)

    currentNode.connect(limiter)
    currentNode = limiter
  } else if (limiterEnabled.value) {
    console.warn('[MasterSection] ⚠️ Limiter enabled but node is null!')
  }

  // 6. Connect to FX chain output (end of effects)
  currentNode.connect(fxChainOutput)

  // 7. Connect analysis tap for spectrum analyzer
  fxChainOutput.connect(analysisTap)

  // 8. Route to audio outputs with INDEPENDENT volume controls
  // Main output: goes through stereo split -> L/R fader gains -> native merger -> wrapper
  if (mainOutputGain && mainStreamDestination && masterFaderSplit && masterFaderLeftGain && masterFaderRightGain && masterFaderMerge && masterFaderMergeWrapper && splitNode) {
    fxChainOutput.connect(masterFaderSplit)
    masterFaderSplit.connect(masterFaderLeftGain, 0) // Left channel
    masterFaderSplit.connect(masterFaderRightGain, 1) // Right channel
    
    // Connect L/R gains to native ChannelMerger inputs
    masterFaderLeftGain.connect(masterFaderMerge, 0, 0) // Left to left input
    masterFaderRightGain.connect(masterFaderMerge, 0, 1) // Right to right input
    
    // Connect native merger to wrapper (CRITICAL!)
    masterFaderMerge.connect(masterFaderMergeWrapper.input)
    
    // Connect metering from post-fader signal BEFORE sending to output
    masterFaderMergeWrapper.connect(splitNode)

    // Then send to output
    masterFaderMergeWrapper.connect(mainOutputGain)
    mainOutputGain.connect(mainStreamDestination as any)
  } else {
    console.error('[connectMasterRouting] Missing nodes for main output routing!', {
      mainOutputGain: !!mainOutputGain,
      mainStreamDestination: !!mainStreamDestination,
      masterFaderSplit: !!masterFaderSplit,
      masterFaderLeftGain: !!masterFaderLeftGain,
      masterFaderRightGain: !!masterFaderRightGain,
      masterFaderMerge: !!masterFaderMerge,
      masterFaderMergeWrapper: !!masterFaderMergeWrapper,
      splitNode: !!splitNode
    })
  }

  // Headphones output: bypasses master faders, uses only HP volume knob
  if (headphonesOutputGain && headphonesStreamDestination) {
    fxChainOutput.connect(headphonesOutputGain)

    // Connect headphones meter
    if (headphonesMeter) {
      headphonesOutputGain.connect(headphonesMeter)
    }

    headphonesOutputGain.connect(headphonesStreamDestination as any)
  }
}

// Watch for parameter changes
watch([leftVolume, rightVolume], updateMasterVolume)

// When linked, sync right to left
watch(leftVolume, (newVal) => {
  if (isLinked.value) {
    rightVolume.value = newVal
  }
})

// Watch headphones volume
watch(headphonesVolume, (newVol) => {
  if (headphonesOutputGain && Tone) {
    // Direct assignment to avoid scheduling events
    headphonesOutputGain.gain.value = Tone.dbToGain(newVol)
  }
})

// Format level for display
function formatLevel(level: number): string {
  if (level <= -60) return '-∞'
  return level >= 0 ? '0.0dB' : `${level.toFixed(1)}dB`
}

// Get color class based on level
function getLevelColor(level: number): string {
  if (level > -6) return 'text-red-400'
  if (level > -20) return 'text-yellow-400'
  return 'text-green-400'
}

function applyMasterParametricEQ(filters: any) {
  if (!filters || !Tone) return

  const filtersData = filters.filtersData || []
  
  // Save filters data for scene snapshots (only pure data, no Tone.js objects)
  masterEQFiltersData.value = filtersData.map((f: any) => ({
    type: f.type,
    frequency: f.frequency,
    gain: f.gain,
    Q: f.Q,
    color: f.color
  }))

  // Se non ci sono filtri, rimuovi l'EQ
  if (filtersData.length === 0) {
    if (masterParametricEQ) {
      try {
        masterParametricEQ.input?.disconnect()
        masterParametricEQ.output?.disconnect()
        if (masterParametricEQ.filters) {
          masterParametricEQ.filters.forEach((f: any) => {
            if (f && f.dispose) f.dispose()
          })
        }
      } catch (e) {
        console.warn('Error disposing master EQ:', e)
      }
      masterParametricEQ = null
    }
    connectMasterRouting()
    return
  }

  // Se l'EQ esiste già e il numero/tipo di filtri è lo stesso, aggiorna solo i parametri
  if (masterParametricEQ && masterParametricEQ.filters &&
    masterParametricEQ.filters.length === filtersData.length) {

    let needsRecreate = false

    // Controlla se i tipi sono cambiati
    for (let i = 0; i < filtersData.length; i++) {
      if (masterParametricEQ.filtersData[i]?.type !== filtersData[i].type) {
        needsRecreate = true
        break
      }
    }

    if (!needsRecreate) {
      // Direct assignment instead of rampTo to avoid event scheduling
      filtersData.forEach((filterData: any, index: number) => {
        const filter = masterParametricEQ.filters[index]
        if (filter) {
          filter.frequency.value = filterData.frequency
          filter.Q.value = filterData.Q
          if (filter.gain) {
            filter.gain.value = filterData.gain
          }
        }
      })
      masterParametricEQ.filtersData = filtersData
      return // Non ricreare tutto
    }
  }

  // Ricrea i filtri solo se necessario (primo avvio o cambio struttura)
  if (masterParametricEQ) {
    try {
      masterParametricEQ.input?.disconnect()
      masterParametricEQ.output?.disconnect()
      if (masterParametricEQ.filters) {
        masterParametricEQ.filters.forEach((f: any) => {
          if (f && f.dispose) f.dispose()
        })
      }
    } catch (e) {
      console.warn('Error disposing old master EQ:', e)
    }
  }

  try {
    const clonedFilters: any[] = []

    filtersData.forEach((filterData: any, index: number) => {
      const node = new Tone.Filter({
        type: filterData.type,
        frequency: filterData.frequency,
        Q: filterData.Q,
        gain: filterData.gain
      })

      clonedFilters.push(node)

      // Connetti in serie
      if (index > 0) {
        clonedFilters[index - 1].connect(node)
      }
    })

    // Crea l'oggetto con i filtri clonati
    if (clonedFilters.length > 0) {
      masterParametricEQ = {
        input: clonedFilters[0],
        output: clonedFilters[clonedFilters.length - 1],
        filters: clonedFilters,
        filtersData: filtersData
      }
    } else {
      masterParametricEQ = null
    }
  } catch (error) {
    console.error('Error cloning master EQ filters:', error)
    masterParametricEQ = null
  }

  initMasterChannel()
  connectMasterRouting()
}

// FX Control Methods
async function toggleCompressor(enabled: boolean, params?: { threshold: number, ratio: number, attack: number, release: number }) {
  if (!Tone) return

  compressorEnabled.value = enabled

  if (enabled && !compressor) {
    compressor = new Tone.Compressor({
      threshold: params?.threshold ?? -24,
      ratio: params?.ratio ?? 4,
      attack: params?.attack ?? 0.003,
      release: params?.release ?? 0.25
    })
    compressorNode.value = compressor
  }

  connectMasterRouting()
}

function updateCompressor(params: { threshold: number, ratio: number, attack: number, release: number }) {
  if (compressor && compressorEnabled.value) {
    // Direct assignment to avoid scheduling events
    compressor.threshold.value = params.threshold
    compressor.ratio.value = params.ratio
    compressor.attack.value = params.attack
    compressor.release.value = params.release
  }
}

async function toggleReverb(enabled: boolean, params?: { decay: number, preDelay: number, wet: number }) {
  if (!Tone) return

  reverbEnabled.value = enabled

  if (enabled && !reverb) {
    reverb = new Tone.Reverb({
      decay: params?.decay ?? 1.5,
      preDelay: params?.preDelay ?? 0.01,
      wet: params?.wet ?? 0.3
    })
    await reverb.generate()
    reverbNode.value = reverb
  }

  connectMasterRouting()
}

function updateReverb(params: { decay: number, preDelay: number, wet: number }) {
  if (reverb && reverbEnabled.value) {
    reverb.preDelay = params.preDelay
    // Direct assignment to avoid scheduling events
    reverb.wet.value = params.wet
    // Note: decay requires regeneration - toggle off/on to apply
  }
}

async function toggleDelay(enabled: boolean, params?: { delayTime: number, feedback: number, wet: number }) {
  if (!Tone) return

  delayEnabled.value = enabled

  if (enabled && !delay) {
    delay = new Tone.FeedbackDelay({
      delayTime: params?.delayTime ?? 0.25,
      feedback: params?.feedback ?? 0.5,
      wet: params?.wet ?? 0.3
    })
    delayNode.value = delay
  }

  connectMasterRouting()
}

function updateDelay(params: { delayTime: number, feedback: number, wet: number }) {
  if (delay && delayEnabled.value) {
    // Direct assignment to avoid scheduling events
    delay.delayTime.value = params.delayTime
    delay.feedback.value = params.feedback
    delay.wet.value = params.wet
  }
}

async function toggleLimiter(enabled: boolean, params?: { threshold: number }) {
  if (!Tone) return

  limiterEnabled.value = enabled

  if (enabled) {
    // Dispose old limiter chain if exists
    if (limiter) {
      try {
        limiterGain?.disconnect()
        limiterWaveShaper?.disconnect()
        limiter.disconnect()
        limiterGain?.dispose()
        limiterWaveShaper?.dispose()
        limiter.dispose()
      } catch (e) { }
    }

    // Clamp threshold to valid range
    let thresholdValue = params?.threshold ?? -1
    thresholdValue = Math.max(-20, Math.min(3, thresholdValue))

    try {
      // Simplified approach: single Gain node for testing
      const gainValue = Math.pow(10, thresholdValue / 20)
      limiter = new Tone.Gain(gainValue)
      limiterNode.value = limiter
    } catch (error) {
      console.error('[Limiter] Error creating limiter:', error)
      limiter = null
      limiterNode.value = null
      limiterEnabled.value = false
    }
  }

  connectMasterRouting()
}

function updateLimiter(params: { threshold: number }) {
  if (!limiter || !limiterEnabled.value) return

  // Debounce to avoid recreating too frequently
  if (limiterUpdateTimeout) {
    clearTimeout(limiterUpdateTimeout)
  }

  limiterUpdateTimeout = window.setTimeout(() => {
    if (limiter && limiterEnabled.value) {
      // Clamp threshold
      const thresholdValue = Math.max(-20, Math.min(3, params.threshold))

      try {
        // Dispose old chain
        limiter.disconnect()
        limiter.dispose()

        const gainValue = Math.pow(10, thresholdValue / 20)
        limiter = new Tone.Gain(gainValue)
        limiterNode.value = limiter

        connectMasterRouting()
      } catch (error) {
        console.error('[Limiter] Error updating limiter:', error)
      }
    }
    limiterUpdateTimeout = null
  }, 100)
}

// Expose master output for tracks to connect
defineExpose({
  getMasterOutput: () => {
    // Initialize master on first access from tracks
    if (!masterChannel) {
      initMasterChannel()
    }
    return masterOutput.value
  },
  masterOutput,
  analysisOutput,
  getMeterValues,
  getPreLimiterValues,
  applyMasterEQ: applyMasterParametricEQ,
  ensureAudioPlaying, // Expose for tracks to call when playing
  // FX control methods
  toggleCompressor,
  updateCompressor,
  toggleReverb,
  updateReverb,
  toggleDelay,
  updateDelay,
  toggleLimiter,
  updateLimiter,
  // FX nodes for visualization
  compressorNode,
  reverbNode,
  delayNode,
  limiterNode,
  
  // Scene management
  getSnapshot: () => {
    return {
      leftVolume: leftVolume.value,
      rightVolume: rightVolume.value,
      headphonesVolume: headphonesVolume.value,
      isLinked: isLinked.value,
      masterEQFilters: masterEQFiltersData.value.map(f => ({
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      })),
      compressorEnabled: compressorEnabled.value,
      reverbEnabled: reverbEnabled.value,
      delayEnabled: delayEnabled.value,
      limiterEnabled: limiterEnabled.value
    }
  },
  
  restoreFromSnapshot: (snapshot: any) => {
    // Restore volumes
    leftVolume.value = snapshot.leftVolume
    rightVolume.value = snapshot.rightVolume  
    headphonesVolume.value = snapshot.headphonesVolume
    isLinked.value = snapshot.isLinked
    
    // Restore EQ
    if (snapshot.masterEQFilters && snapshot.masterEQFilters.length > 0) {
      applyMasterParametricEQ({ filtersData: snapshot.masterEQFilters })
    }
    
    // Restore FX states
    toggleCompressor(snapshot.compressorEnabled)
    toggleReverb(snapshot.reverbEnabled)
    toggleDelay(snapshot.delayEnabled)
    toggleLimiter(snapshot.limiterEnabled)
  }
})

// Cleanup
onUnmounted(() => {
  if (masterChannel) masterChannel.dispose()
  if (leftMeter) leftMeter.dispose()
  if (rightMeter) rightMeter.dispose()
  if (splitNode) splitNode.dispose()
  if (analysisTap) analysisTap.dispose()
  if (fxChainOutput) fxChainOutput.dispose()
  if (masterFaderSplit) masterFaderSplit.dispose()
  if (masterFaderLeftGain) masterFaderLeftGain.dispose()
  if (masterFaderRightGain) masterFaderRightGain.dispose()
  // masterFaderMerge is a native ChannelMerger, no dispose needed
  if (masterFaderMergeWrapper) masterFaderMergeWrapper.dispose()

  // Cleanup FX
  if (compressor) compressor.dispose()
  if (reverb) reverb.dispose()
  if (limiter) limiter.dispose()

  // Stop recording if active
  if (isRecording.value) {
    stopRecording()
  }

  if (recordingInterval) {
    clearInterval(recordingInterval)
  }

  // Cleanup audio outputs
  if (mainOutputGain) mainOutputGain.dispose()
  if (headphonesOutputGain) headphonesOutputGain.dispose()

  // Remove audio elements
  if (mainAudioElement) {
    mainAudioElement.pause()
    mainAudioElement.srcObject = null
    mainAudioElement.remove()
  }
  if (headphonesAudioElement) {
    headphonesAudioElement.pause()
    headphonesAudioElement.srcObject = null
    headphonesAudioElement.remove()
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
