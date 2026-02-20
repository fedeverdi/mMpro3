<template>
  <div
    class="track-channel relative bg-gray-800 rounded-lg border border-gray-700 p-1 flex flex-col items-center gap-1 h-full">
    <!-- Loading Overlay -->
    <div v-if="isLoading"
      class="absolute inset-0 bg-gray-900 bg-opacity-80 rounded-lg z-50 flex flex-col items-center justify-center gap-2">
      <svg class="animate-spin h-8 w-8 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none"
        viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
        </path>
      </svg>
      <span class="text-sm text-gray-300 font-medium">Loading...</span>
    </div>

    <!-- Track Header -->
    <div class="w-full flex flex-col gap-1">
      <div class="text-xs font-bold text-center text-gray-300">Track {{ trackNumber }}</div>

      <!-- Audio Source Selector -->
      <div class="w-full">
        <select v-model="audioSourceType" @change="handleSourceTypeChange"
          class="w-full text-xs bg-gray-700 text-gray-200 border border-gray-600 rounded px-1 py-1 focus:border-blue-500 focus:outline-none">
          <option value="file">📁 File</option>
          <option value="input">🎤 Audio Input</option>
        </select>
      </div>

      <!-- File Upload (shown when source is 'file') -->
      <div v-if="audioSourceType === 'file'" class="w-full">
        <input type="file" accept="audio/*" @change="handleFileUpload" ref="fileInput" class="hidden" />
        <button @click="($refs.fileInput as HTMLInputElement)?.click()"
          class="w-full px-2 truncate py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded border border-gray-600 transition-colors">
          {{ fileName || 'Load Audio' }}
        </button>
      </div>

      <!-- Audio Input Device Selector (shown when source is 'input') -->
      <div v-if="audioSourceType === 'input'" class="w-full">
        <select v-model="selectedAudioInput" @change="handleAudioInputChange"
          class="w-full text-xs bg-gray-800 text-gray-200 border border-gray-600 rounded px-1 py-1 focus:border-blue-500 focus:outline-none">
          <option value="">Select Input...</option>
          <option v-for="device in audioInputs" :key="device.deviceId" :value="device.deviceId">
            {{ device.label || `Input ${device.deviceId.substring(0, 12)}...` }}
          </option>
        </select>
      </div>

      <!-- Transport Controls -->
      <div class="flex gap-1 justify-center">
        <button @click="togglePlay"
          class="px-2 py-1 w-full text-xs rounded transition-colors flex items-center justify-center"
          :class="isPlaying ? 'bg-green-600 hover:bg-green-500 animate-pulse' : (audioLoaded ? 'bg-green-600 hover:bg-green-500' : 'bg-blue-600 hover:bg-blue-500')">
          <!-- Show microphone icon for audio input -->
          <svg v-if="audioSourceType === 'input'" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"
            class="w-3 h-3">
            <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
            <path
              d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
          </svg>
          <!-- Show play/pause for file playback -->
          <svg v-else-if="!isPlaying" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M8 5v14l11-7z" />
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M6 4h4v16H6zM14 4h4v16h-4z" />
          </svg>
        </button>
        <button @click="stopAudio"
          class="px-2 py-1 w-full text-xs rounded transition-colors bg-gray-600 hover:bg-gray-500 flex items-center justify-center">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M6 6h12v12H6z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Waveform Display -->
    <WaveformDisplay ref="waveformDisplayRef" :waveform-node="waveform" :audio-buffer="currentAudioBuffer"
      :is-playing="isPlaying" :current-time="currentPlaybackTime" />

    <!-- Gain Control -->
    <div class="w-full flex items-center justify-center h-[4rem]">
      <div class="scale-[0.65]">
        <Knob v-model="gain" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Gain" unit="dB" color="#8b5cf6" />
      </div>
    </div>

    <!-- FX Section -->
    <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
      <div class="flex gap-1">
        <TrackCompressor ref="trackCompressorRef" :track-number="trackNumber" :enabled="compressorEnabled"
          :compressor-node="compressor" :meter="meterL" @toggle="toggleCompressor" />
        <TrackReverb ref="trackReverbRef" :track-number="trackNumber" :enabled="reverbEnabled" :reverb-node="reverb"
          @toggle="toggleReverb" />
      </div>
    </div>


    <!-- EQ Section -->
    <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
      <div class="flex items-center justify-between px-2">
        <button @click="showEQ3Bands = !showEQ3Bands"
          class="flex items-center gap-1 hover:text-gray-200 transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor"
            class="w-3 h-3 text-gray-400 transition-transform" :class="showEQ3Bands ? 'rotate-90' : ''">
            <path fill-rule="evenodd"
              d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
              clip-rule="evenodd" />
          </svg>
          <div class="text-xs text-gray-400 uppercase tracking-wide">EQ</div>
        </button>
        <button @click="showParametricEQ = true"
          class="text-xs px-2 py-1 bg-blue-600 hover:bg-blue-500 text-white rounded transition-colors flex items-center justify-center"
          title="Open Parametric EQ">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="white" class="h-2.5 w-2.5">
            <path
              d="M487.4 315.7l-42.6-24.6c4.3-23.2 4.3-47 0-70.2l42.6-24.6c4.9-2.8 7.1-8.6 5.5-14-11.1-35.6-30-67.8-54.7-94.6-3.8-4.1-10-5.1-14.8-2.3L380.8 110c-17.9-15.4-38.5-27.3-60.8-35.1V25.8c0-5.6-3.9-10.5-9.4-11.7-36.7-8.2-74.3-7.8-109.2 0-5.5 1.2-9.4 6.1-9.4 11.7V75c-22.2 7.9-42.8 19.8-60.8 35.1L88.7 85.5c-4.9-2.8-11-1.9-14.8 2.3-24.7 26.7-43.6 58.9-54.7 94.6-1.7 5.4.6 11.2 5.5 14L67.3 221c-4.3 23.2-4.3 47 0 70.2l-42.6 24.6c-4.9 2.8-7.1 8.6-5.5 14 11.1 35.6 30 67.8 54.7 94.6 3.8 4.1 10 5.1 14.8 2.3l42.6-24.6c17.9 15.4 38.5 27.3 60.8 35.1v49.2c0 5.6 3.9 10.5 9.4 11.7 36.7 8.2 74.3 7.8 109.2 0 5.5-1.2 9.4-6.1 9.4-11.7v-49.2c22.2-7.9 42.8-19.8 60.8-35.1l42.6 24.6c4.9 2.8 11 1.9 14.8-2.3 24.7-26.7 43.6-58.9 54.7-94.6 1.5-5.5-.7-11.3-5.6-14.1zM256 336c-44.1 0-80-35.9-80-80s35.9-80 80-80 80 35.9 80 80-35.9 80-80 80z" />
          </svg>
        </button>
      </div>
      <!-- EQ Curve Thumbnail -->
      <EQThumbnail :filters="eqFiltersData" />

      <!-- 3-Band EQ Knobs (Accordion) -->
      <TrackEQ ref="trackEQRef" :eq3Node="eq3" :show="showEQ3Bands" />
    </div>

    <!-- Mute & Solo Buttons -->
    <div class="flex flex-row gap-1 w-full">
      <button @click="toggleMute" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isMuted ? 'bg-red-600 text-white animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
        M
      </button>
      <button @click="toggleSolo" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isSolo ? 'bg-yellow-500 text-gray-900 animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
        S
      </button>
    </div>

    <!-- Pan Knob -->
    <div class="flex justify-center  scale-[0.75]">
      <PanKnob class="" v-model="pan" label="Pan" />
    </div>

    <!-- Volume Fader and VU Meter -->
    <div class="flex flex-col h-full">
      <div class="text-[0.455rem] uppercase text-center">Volume</div>
      <div ref="faderContainer" class="flex-1 flex items-center justify-center gap-2  min-h-0">
        <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />
        <TrackMeter v-if="faderHeight > 0" :levelL="trackLevelL" :levelR="trackLevelR" :isStereo="isStereo"
          :height="faderHeight + 20" />
      </div>
    </div>

  </div>

  <!-- Parametric EQ Modal -->
  <ParametricEQModal v-model="showParametricEQ" :trackNumber="trackNumber" :eq-filters="eqFiltersData"
    @update="handleParametricEQUpdate" />
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed, toRaw, inject } from 'vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'
import type { TrackSnapshot } from '~/composables/useScenes'
import TrackFader from './TrackFader.vue'
import TrackMeter from './TrackMeter.vue'
import Knob from './core/Knob.vue'
import PanKnob from './PanKnob.vue'
import ParametricEQModal from './ParametricEQModal.vue'
import TrackCompressor from './audioTrack/TrackCompressor.vue'
import TrackReverb from './audioTrack/TrackReverb.vue'
import TrackEQ from './audioTrack/TrackEQ.vue'
import EQThumbnail from './audioTrack/EQThumbnail.vue'
import WaveformDisplay from './audioTrack/WaveformDisplay.vue'

defineOptions({
  inheritAttrs: false
})

// Inject Tone.js from App.vue (imported once for entire app)
const ToneRef = inject<any>('Tone')
let Tone: any = null

const { saveAudioFile, getAudioFile } = useAudioFileStorage()

interface Props {
  trackNumber: number
  masterChannel?: any
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'soloChange', value: { trackNumber: number, isSolo: boolean }): void
  (e: 'levelUpdate', value: { trackNumber: number, level: number }): void
}>()

// Audio state
const fileInput = ref<HTMLInputElement | null>(null)
const fileName = ref<string>('')
const fileId = ref<string>('') // IndexedDB file ID for scene persistence
const audioLoaded = ref(false)
const faderContainer = ref<HTMLElement | null>(null)
const faderHeight = ref(0)
const isPlaying = ref(false)
const isMuted = ref(false)
const isSolo = ref(false)
const isLoading = ref(false)
const showParametricEQ = ref(false)
const waveformDisplayRef = ref<any>(null)

// Audio source selection
const audioSourceType = ref<'file' | 'input'>('file')

// Audio inputs from shared composable
const { audioInputDevices, refreshAudioInputs } = useAudioDevices()
const audioInputs = audioInputDevices // Use the shared ref

const selectedAudioInput = ref<string>('')
let audioInputStream: MediaStream | null = null
let audioInputSource: MediaStreamAudioSourceNode | null = null

// FX state
const compressorEnabled = ref(false)
const reverbEnabled = ref(false)

// Store EQ filters data for thumbnail
const eqFiltersData = ref<any[]>([])

// EQ accordion state
const showEQ3Bands = ref(false)

// Audio controls
const volume = ref(0)
const gain = ref(0)
const pan = ref(0) // -1 (left) to +1 (right)
const isStereo = ref(true) // Track if source is stereo or mono (default stereo)
const trackLevelL = ref(-60) // Left/Mono level
const trackLevelR = ref(-60) // Right level (only for stereo)
const currentPlaybackTime = ref(0)

// Component refs
const trackEQRef = ref<InstanceType<typeof TrackEQ> | null>(null)
const trackCompressorRef = ref<InstanceType<typeof TrackCompressor> | null>(null)
const trackReverbRef = ref<InstanceType<typeof TrackReverb> | null>(null)

// Tone.js nodes
let player: any = null // Can be Tone.Player or Tone.UserMedia
let currentAudioBuffer: AudioBuffer | null = null // Store current audio buffer for player recreation
let gainNode: any = null
let eq3: any = null
let parametricEQFilters: any = null // Parametric EQ filter chain
let compressor: any = null
let reverb: any = null
// Balance control (stereo-preserving): Split → Gain L/R → Merge
let balanceSplit: any = null
let balanceLeft: any = null
let balanceRight: any = null
let balanceMerge: any = null
// Volume control (must preserve stereo)
let volumeNodeL: any = null
let volumeNodeR: any = null
let volumeSplit: any = null
let volumeMerge: any = null
let meterL: any = null // Left channel meter (or mono)
let meterR: any = null // Right channel meter (stereo only)
let channelSplit: any = null // Split stereo to L/R for metering
let waveform: any = null // Waveform analyzer
let resizeObserver: ResizeObserver | null = null
let playbackTimeInterval: number | null = null
let playbackStartTime: number = 0
let audioContextStarted: boolean = false // Track if Tone.start() has been called

// Calculate fader height based on container
function updateFaderHeight() {
  if (faderContainer.value) {
    const height = faderContainer.value.clientHeight
    // Subtract some padding for labels (about 60px for "Volume" label + value display)
    faderHeight.value = Math.max(100, height - 60)
  }
}

// Initialize audio chain
onMounted(async () => {
  // Wait for Tone.js to be loaded from App.vue
  if (ToneRef?.value) {
    Tone = ToneRef.value
  } else {
    // Fallback: wait for it to be injected
    const checkTone = setInterval(() => {
      if (ToneRef?.value) {
        Tone = ToneRef.value
        clearInterval(checkTone)
      }
    }, 100)
  }

  // Calculate initial fader height
  await nextTick()
  updateFaderHeight()

  // Watch for container size changes
  if (faderContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateFaderHeight()
    })
    resizeObserver.observe(faderContainer.value)
  }

  // Start level monitoring
  startLevelMonitoring()

  // Listen for device changes and refresh the shared list
  navigator.mediaDevices.addEventListener('devicechange', refreshAudioInputs)
})

// Initialize audio nodes (called on first use)
function initAudioNodes() {
  if (gainNode) return // Already initialized

  if (!Tone) {
    console.error('Tone.js not loaded')
    return
  }

  // Create audio nodes
  gainNode = new Tone.Gain(1) // 1 = 0dB (unity gain), not 0!

  eq3 = new Tone.EQ3({
    low: 0,
    mid: 0,
    high: 0
  })

  // Stereo-preserving balance control: Split → Gain L/R → Merge
  // Preserves stereo width while allowing L/R balance (pan) adjustment
  balanceSplit = new Tone.Split()
  balanceLeft = new Tone.Gain(1)   // Left channel gain
  balanceRight = new Tone.Gain(1)  // Right channel gain
  balanceMerge = new Tone.Merge()

  // Stereo-preserving volume control: Split → Gain L/R → Merge
  // Tone.Volume converts stereo to mono, so we use separate gains
  volumeSplit = new Tone.Split()
  volumeNodeL = new Tone.Gain(1)  // Left volume (0dB = unity)
  volumeNodeR = new Tone.Gain(1)  // Right volume (0dB = unity)
  volumeMerge = new Tone.Merge()

  // Create stereo metering: split channels and meter each separately
  channelSplit = new Tone.Split() // Split stereo into L/R
  meterL = new Tone.Meter() // Left channel (or mono)
  meterR = new Tone.Meter() // Right channel

  // Waveform analyzer (for visualization)
  waveform = new Tone.Waveform(512) // 512 samples for waveform display

  // Create FX nodes once (always present, bypassed when disabled)
  compressor = new Tone.Compressor({
    threshold: 0,     // Bypassed: 0dB threshold = no compression
    ratio: 1,         // Bypassed: 1:1 ratio = no compression
    attack: 0.1,
    release: 0.25
  })

  reverb = new Tone.Reverb({
    decay: 1.5,
    preDelay: 0.01,
    wet: 0            // Bypassed: 0% wet = no reverb
  })

  // Connect chain: gain -> eq3 -> reverb -> balance -> volume
  // Compressor is bypassed by default (not in chain)
  gainNode.connect(eq3)

  // Connect stereo metering to eq3 (before effects)
  eq3.connect(channelSplit)
  channelSplit.connect(meterL, 0) // Left channel to meterL
  channelSplit.connect(meterR, 1) // Right channel to meterR

  // Connect waveform analyzer to eq3 for visualization
  eq3.connect(waveform)

  // Initial FX chain: eq3 -> reverb -> balance (compressor bypassed)
  // NOTE: Compressor is NOT connected by default to preserve stereo
  // Use toggleCompressor() to enable it
  eq3.connect(reverb)

  reverb.connect(balanceSplit)

  // Balance control: Split → Gain L/R → Merge
  balanceSplit.connect(balanceLeft, 0)     // Left channel
  balanceSplit.connect(balanceRight, 1)    // Right channel
  balanceLeft.connect(balanceMerge, 0, 0)  // To left output
  balanceRight.connect(balanceMerge, 0, 1) // To right output

  // Volume control: Split → Gain L/R → Merge
  balanceMerge.connect(volumeSplit)
  volumeSplit.connect(volumeNodeL, 0)      // Left channel
  volumeSplit.connect(volumeNodeR, 1)      // Right channel
  volumeNodeL.connect(volumeMerge, 0, 0)   // To left output
  volumeNodeR.connect(volumeMerge, 0, 1)   // To right output

  // Volume to output (master or destination)
  connectToOutput()
}


// Handle parametric EQ update
function handleParametricEQUpdate(filters: any) {
  if (!filters) return

  // Store the latest filter chain
  parametricEQFilters = filters

  // Store filter data for thumbnail
  if (filters.filtersData) {
    // Convert Vue reactive proxy to raw array
    const rawFiltersData = toRaw(filters.filtersData)

    eqFiltersData.value = rawFiltersData.map((f: any) => ({
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      Q: f.Q
    }))
  }

  // Apply the parametric EQ to the audio chain
  applyParametricEQ()
}

// Insert or remove parametric EQ from the chain with minimal disconnections
function applyParametricEQ() {
  if (!eq3 || !reverb) return

  // Disconnect only eq3 (meters and waveform stay connected)
  try {
    eq3.disconnect()
  } catch (e) {
    // Ignore disconnection errors
  }

  // Reconnect meters and waveform to eq3
  if (channelSplit) eq3.connect(channelSplit)
  if (waveform) eq3.connect(waveform)

  // Determine next node in chain (compressor if enabled, reverb if not)
  const nextNode = (compressorEnabled.value && compressor) ? compressor : reverb

  // Insert parametric EQ between eq3 and next node if present
  if (parametricEQFilters && parametricEQFilters.input && parametricEQFilters.output) {
    eq3.connect(parametricEQFilters.input)

    // Disconnect old parametric output if needed
    try {
      parametricEQFilters.output.disconnect()
    } catch (e) { }

    parametricEQFilters.output.connect(nextNode)
  } else {
    // No parametric EQ: connect eq3 directly to next node
    eq3.connect(nextNode)
  }
}

// Connect to output (ONLY to master, NEVER to destination)
function connectToOutput() {
  if (!volumeMerge || !Tone) return false
  volumeMerge.connect(toRaw(props.masterChannel))
}

// Level monitoring for stereo/mono
let levelMonitorInterval: number | null = null
function startLevelMonitoring() {
  levelMonitorInterval = window.setInterval(() => {
    if (meterL && Tone) {
      const levelL = meterL.getValue() as number
      trackLevelL.value = Math.max(-60, levelL)

      // For stereo, also read right channel
      if (isStereo.value && meterR) {
        const levelR = meterR.getValue() as number
        trackLevelR.value = Math.max(-60, levelR)
      } else {
        // Mono: copy left to right for consistency
        trackLevelR.value = trackLevelL.value
      }
    }
  }, 50)
}

// Playback time tracking
function startPlaybackTimeTracking() {
  if (playbackTimeInterval !== null) return

  if (!Tone) return

  // Record start time
  playbackStartTime = Tone.now()

  playbackTimeInterval = window.setInterval(() => {
    if (player && isPlaying.value && Tone) {
      // Calculate elapsed time since start
      const elapsed = Tone.now() - playbackStartTime

      // For looping player, use modulo of duration
      if (currentAudioBuffer && currentAudioBuffer.duration) {
        currentPlaybackTime.value = elapsed % currentAudioBuffer.duration
      }
    }
  }, 50) // Update every 50ms for smooth animation
}

function stopPlaybackTimeTracking() {
  if (playbackTimeInterval !== null) {
    clearInterval(playbackTimeInterval)
    playbackTimeInterval = null
  }
  currentPlaybackTime.value = 0
  playbackStartTime = 0
}

// File upload handler
async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]

  if (!file || !Tone) {
    console.error('Cannot load file - missing file or Tone.js')
    return
  }

  // Initialize audio nodes on first use
  initAudioNodes()

  fileName.value = file.name
  isLoading.value = true

  try {
    // Save file to IndexedDB for scene persistence
    const savedFileId = await saveAudioFile(file)
    fileId.value = savedFileId

    // Create buffer from file
    const arrayBuffer = await file.arrayBuffer()
    const audioBuffer = await Tone.context.decodeAudioData(arrayBuffer)

    // Check if we need to create a new player or just swap buffer
    if (player && typeof player.stop === 'function' && 'buffer' in player) {
      // It's already a Tone.Player - just swap the buffer

      // CRITICAL: Stop player first to reset internal state
      try {
        player.stop()
      } catch (e) { }

      // Dispose old buffer
      if (player.buffer && typeof player.buffer.dispose === 'function') {
        try {
          player.buffer.dispose()
        } catch (e) { }
      }

      // Dispose old currentAudioBuffer if different
      if (currentAudioBuffer && currentAudioBuffer !== player.buffer) {
        try {
          if (typeof (currentAudioBuffer as any).dispose === 'function') {
            (currentAudioBuffer as any).dispose()
          }
        } catch (e) { }
      }

      // Assign new buffer
      player.buffer = audioBuffer

    } else {
      // First time or was audio input - create new Tone.Player

      if (player) {
        // Cleanup old player (was Gain node for mic input)
        try {
          player.disconnect()
          player.dispose()
        } catch (e) { }
      }

      // Dispose old currentAudioBuffer
      if (currentAudioBuffer) {
        try {
          if (typeof (currentAudioBuffer as any).dispose === 'function') {
            (currentAudioBuffer as any).dispose()
          }
        } catch (e) { }
      }

      // Create new Tone.Player
      player = new Tone.Player({
        url: audioBuffer,
        loop: true,
        playbackRate: 1.0,
      })
      player.connect(gainNode)
    }

    // Update current buffer reference for waveform
    currentAudioBuffer = audioBuffer

    // Detect if stereo or mono
    isStereo.value = audioBuffer.numberOfChannels === 2

    // Verify audio chain is connected
    if (!gainNode || !eq3 || !volumeMerge) {
      alert('Audio system not ready. Please refresh the page.')
      isLoading.value = false
      return
    }

    audioLoaded.value = true
    isLoading.value = false

    // Force DOM update
    await nextTick()

    // CRITICAL: Reset file input to allow reloading the same file
    // Without this, selecting the same file again won't trigger onChange
    target.value = ''
  } catch (error) {
    console.error('❌ Error loading audio file:', error)
    alert('Error loading audio file: ' + error)
    isLoading.value = false
    // Reset input even on error
    target.value = ''
  }
}

// Load audio file from IndexedDB (for restoring from scene)
async function loadFileFromIndexedDB(savedFileId: string) {
  if (!Tone) {
    console.error('Tone.js not loaded')
    return
  }

  // Initialize audio nodes on first use
  initAudioNodes()

  isLoading.value = true

  try {
    // Retrieve file from IndexedDB
    const storedFile = await getAudioFile(savedFileId)

    if (!storedFile) {
      console.error('File not found in IndexedDB')
      alert('Could not restore audio file from scene. File may have been deleted.')
      isLoading.value = false
      return
    }

    // Decode audio buffer
    const audioBuffer = await Tone.context.decodeAudioData(storedFile.arrayBuffer)

    // Check if we need to create a new player or just swap buffer
    if (player && typeof player.stop === 'function' && 'buffer' in player) {
      // It's already a Tone.Player - just swap the buffer

      // CRITICAL: Stop player first to reset internal state
      try {
        player.stop()
      } catch (e) { }

      if (player.buffer && typeof player.buffer.dispose === 'function') {
        try {
          player.buffer.dispose()
        } catch (e) { }
      }

      if (currentAudioBuffer && currentAudioBuffer !== player.buffer) {
        try {
          if (typeof (currentAudioBuffer as any).dispose === 'function') {
            (currentAudioBuffer as any).dispose()
          }
        } catch (e) { }
      }

      // Assign new buffer
      player.buffer = audioBuffer

      // CRITICAL: Reset playback rate to 1.0
      player.playbackRate = 1.0

    } else {
      // First time or was audio input - create new Tone.Player

      if (player) {
        try {
          player.disconnect()
          player.dispose()
        } catch (e) { }
      }

      if (currentAudioBuffer) {
        try {
          if (typeof (currentAudioBuffer as any).dispose === 'function') {
            (currentAudioBuffer as any).dispose()
          }
        } catch (e) { }
      }

      // Create new Tone.Player
      player = new Tone.Player({
        url: audioBuffer,
        loop: true,
        playbackRate: 1.0,
        fadeIn: 0.01,   // Prevent resampling artifacts
        fadeOut: 0.01   // Prevent clicks on stop
      })
      player.connect(gainNode)
    }

    // Update current buffer reference for waveform
    currentAudioBuffer = audioBuffer

    // Detect if stereo or mono
    isStereo.value = audioBuffer.numberOfChannels === 2

    // Verify audio chain is connected
    if (!gainNode || !eq3 || !volumeMerge) {
      alert('Audio system not ready. Please refresh the page.')
      isLoading.value = false
      return
    }

    audioLoaded.value = true
    isLoading.value = false

    // Force DOM update
    await nextTick()
  } catch (error) {
    console.error('❌ Error loading audio file from IndexedDB:', error)
    alert('Error loading audio file from scene: ' + error)
    isLoading.value = false
  }
}



// Handle source type change
function handleSourceTypeChange() {
  // Stop any current playback
  stopAudio()

  // Stop waveform visualization
  waveformDisplayRef.value?.stop()

  // Clean up current source before switching
  if (player) {
    try {
      if (typeof player.stop === 'function') {
        player.stop()
      }

      // Dispose old buffer to free memory
      if (player.buffer && typeof player.buffer.dispose === 'function') {
        try {
          player.buffer.dispose()
        } catch (e) { }
      }

      player.disconnect()
      player.dispose()
    } catch (e) { }
    player = null
  }

  // Dispose and clear audio buffer reference
  if (currentAudioBuffer) {
    try {
      if (typeof (currentAudioBuffer as any).dispose === 'function') {
        (currentAudioBuffer as any).dispose()
      }
    } catch (e) { }
  }
  currentAudioBuffer = null

  // Disconnect and clean up audio input source
  if (audioInputSource) {
    try {
      audioInputSource.disconnect()
    } catch (e) { }
    audioInputSource = null
  }

  if (audioInputStream) {
    audioInputStream.getTracks().forEach(track => track.stop())
    audioInputStream = null
  }

  // DON'T destroy audio nodes - they're reusable!
  // Just reset state
  audioLoaded.value = false
  isPlaying.value = false
  fileName.value = ''
  fileId.value = ''
  selectedAudioInput.value = ''
}

// Handle audio input device change
async function handleAudioInputChange() {
  if (!selectedAudioInput.value || !Tone) return

  // Initialize audio nodes if needed
  initAudioNodes()

  try {
    // Start audio context once (required by browsers for user interaction)
    if (!audioContextStarted) {
      await Tone.start()
      audioContextStarted = true
    }

    // Stop previous input stream if any
    if (audioInputStream) {
      audioInputStream.getTracks().forEach(track => track.stop())
    }

    // Disconnect old audio input source
    if (audioInputSource) {
      try {
        audioInputSource.disconnect()
      } catch (e) { }
      audioInputSource = null
    }

    // Dispose old player if it exists and is not already a Gain (input wrapper)
    if (player && typeof player.stop === 'function') {
      // It's a Tone.Player, dispose it
      try {
        player.stop()
        player.disconnect()
        player.dispose()
      } catch (e) { }
      player = null
    }

    // Get audio stream from selected device
    audioInputStream = await navigator.mediaDevices.getUserMedia({
      audio: {
        deviceId: { exact: selectedAudioInput.value },
        echoCancellation: false,
        noiseSuppression: false,
        autoGainControl: false
      }
    })

    // Create MediaStreamSource from the stream (native Web Audio API node)
    audioInputSource = Tone.context.createMediaStreamSource(audioInputStream)

    // Detect if stereo or mono based on audio tracks
    const audioTracks = audioInputStream.getAudioTracks()
    const channelCount = audioTracks.length > 0 ? audioTracks[0].getSettings().channelCount || 1 : 1
    isStereo.value = channelCount === 2

    // Reuse player if it's already a Gain node, otherwise create new
    if (!player || typeof player.stop === 'function') {
      player = new Tone.Gain(1)
      player.connect(gainNode!)
    }

    // Connect the native media stream source to player input
    if (audioInputSource) {
      audioInputSource.connect(player.input)
    }

    // CRITICAL: Ensure volume node is connected to master output
    connectToOutput()

    // Find device name for display
    const device = audioInputs.value.find(d => d.deviceId === selectedAudioInput.value)
    fileName.value = device?.label || 'Audio Input'

    audioLoaded.value = true
    isPlaying.value = true // Input is always "playing"

    // Ensure master audio elements are playing (critical for output!)
    if (props.masterChannel?.ensureAudioPlaying) {
      props.masterChannel.ensureAudioPlaying()
    }

    // Start waveform visualization for audio input
    waveformDisplayRef.value?.start()

    // Note: No playback time tracking for audio input (it's live)

  } catch (error) {
    console.error(`[Track ${props.trackNumber}] Error connecting audio input:`, error)
    alert('Error accessing audio input. Please check permissions and try again.')
    audioLoaded.value = false
    isPlaying.value = false
  }
}

// Playback controls
async function togglePlay() {
  if (!Tone) return

  // For audio input, toggle mute instead
  if (audioSourceType.value === 'input') {
    if (audioLoaded.value) {
      toggleMute()
    }
    return
  }

  // For file playback
  if (!player || !audioLoaded.value) {
    console.warn('Cannot play: audio not loaded yet')
    return
  }

  if (!isPlaying.value) {
    // Start audio context once (required by browsers for user interaction)
    if (!audioContextStarted) {
      await Tone.start()
      audioContextStarted = true
    }

    // Ensure master audio elements are playing
    if (props.masterChannel?.ensureAudioPlaying) {
      props.masterChannel.ensureAudioPlaying()
    }

    // CRITICAL FIX: Recreate player after stop to avoid resampling issues
    // AudioBufferSourceNode cannot be reused after stop in Web Audio API
    if (player && currentAudioBuffer) {
      try {
        player.disconnect()
        player.dispose()
      } catch (e) { }

      // Small delay to ensure cleanup
      await new Promise(resolve => setTimeout(resolve, 10))

      // Recreate player with same buffer
      player = new Tone.Player({
        url: currentAudioBuffer,
        loop: true,
        playbackRate: 1.0,
        fadeIn: 0.01,
        fadeOut: 0.01
      })

      // Reconnect to audio chain
      player.connect(gainNode)
    }

    // Start with future time for more stable playback
    const startTime = Tone.now() + 0.05  // 50ms in future
    player.start(startTime)
    isPlaying.value = true

    // Start waveform
    waveformDisplayRef.value?.start()

    // Start playback time tracking
    startPlaybackTimeTracking()
  } else {
    // Stop playback
    player.stop()
    isPlaying.value = false

    // Stop waveform
    waveformDisplayRef.value?.stop()

    // Stop playback time tracking
    stopPlaybackTimeTracking()
  }
}

function stopAudio() {
  // For audio input, mute instead
  if (audioSourceType.value === 'input') {
    if (!isMuted.value) {
      toggleMute()
    }
    return
  }

  // For file playback
  if (!player || !audioLoaded.value) return

  // Stop player
  player.stop()
  isPlaying.value = false

  // Reset playback position to start
  currentPlaybackTime.value = 0
  stopPlaybackTimeTracking()

  // Stop waveform
  waveformDisplayRef.value?.stop()
}

function toggleMute() {
  isMuted.value = !isMuted.value
  updateVolume()
}

function toggleSolo() {
  isSolo.value = !isSolo.value
  emit('soloChange', { trackNumber: props.trackNumber, isSolo: isSolo.value })
}

// Update audio parameters
function updateVolume() {
  if (!volumeNodeL || !volumeNodeR || !Tone) return

  if (isMuted.value) {
    volumeNodeL.gain.value = 0  // Mute = 0 gain
    volumeNodeR.gain.value = 0
  } else {
    // Convert dB to linear gain
    const gainValue = Tone.dbToGain(volume.value)
    volumeNodeL.gain.value = gainValue
    volumeNodeR.gain.value = gainValue
  }
}

function updateGain() {
  if (!gainNode || !Tone) return
  gainNode.gain.value = Tone.dbToGain(gain.value)
}

// Update pan value (constant power panning for stereo preservation)
function updatePan() {
  if (!balanceLeft || !balanceRight || !Tone) return
  
  // Constant power panning formula
  // Pan: -1 (left) to +1 (right)
  const panRadians = (pan.value * Math.PI) / 4  // Map -1..+1 to -π/4..+π/4
  
  balanceLeft.gain.value = Math.cos(panRadians + Math.PI / 4)
  balanceRight.gain.value = Math.sin(panRadians + Math.PI / 4)
}

// Watch for parameter changes
watch(volume, updateVolume)
watch(gain, updateGain)
watch(pan, updatePan)

// Expose methods for external control
defineExpose({
  setMuted: (muted: boolean) => {
    isMuted.value = muted
    updateVolume()
  },
  isSolo: () => isSolo.value,

  getSnapshot: () => {
    return {
      trackNumber: props.trackNumber,
      volume: volume.value,
      pan: pan.value,
      muted: isMuted.value,
      soloed: isSolo.value,
      sourceType: audioSourceType.value,
      selectedInputDevice: audioSourceType.value === 'input' ? selectedAudioInput.value : undefined,
      fileName: audioSourceType.value === 'file' ? fileName.value : undefined,
      fileId: audioSourceType.value === 'file' ? fileId.value : undefined,
      eq3: trackEQRef.value?.getParams(),
      parametricEQFilters: eqFiltersData.value.map(f => ({
        id: f.id,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      })),
      compressorEnabled: compressorEnabled.value,
      compressor: trackCompressorRef.value?.getParams(),
      reverbEnabled: reverbEnabled.value,
      reverb: trackReverbRef.value?.getParams()
    }
  },

  restoreFromSnapshot: (snapshot: any) => {
    // Restore volume and pan
    volume.value = snapshot.volume
    pan.value = snapshot.pan
    isMuted.value = snapshot.muted
    isSolo.value = snapshot.soloed

    // Restore source type and related data
    audioSourceType.value = snapshot.sourceType || 'file'
    if (snapshot.selectedInputDevice) {
      selectedAudioInput.value = snapshot.selectedInputDevice
      nextTick(() => {
        handleAudioInputChange()
      })
    }
    if (snapshot.fileName && snapshot.fileId) {
      // Restore audio file from IndexedDB
      fileName.value = snapshot.fileName
      fileId.value = snapshot.fileId
      nextTick(async () => {
        await loadFileFromIndexedDB(snapshot.fileId!)
      })
    }

    // Restore 3-band EQ
    if (snapshot.eq3) {
      trackEQRef.value?.setParams(snapshot.eq3)
    }

    // Restore parametric EQ
    if (snapshot.parametricEQFilters && snapshot.parametricEQFilters.length > 0) {
      eqFiltersData.value = snapshot.parametricEQFilters.map((f: any) => ({
        id: f.id,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      }))
      // Apply EQ filters via the update handler
      handleParametricEQUpdate({ filtersData: eqFiltersData.value })
    }

    // Restore compressor
    const shouldEnableCompressor = snapshot.compressorEnabled || false
    if (snapshot.compressor) {
      trackCompressorRef.value?.setParams(snapshot.compressor)
    }
    if (shouldEnableCompressor !== compressorEnabled.value) {
      toggleCompressor()
    }

    // Restore reverb
    const shouldEnableReverb = snapshot.reverbEnabled || false
    if (snapshot.reverb) {
      trackReverbRef.value?.setParams(snapshot.reverb)
    }
    if (shouldEnableReverb !== reverbEnabled.value) {
      toggleReverb()
    }
  }
})

// Cleanup
onUnmounted(() => {
  // Disconnect audio input source if active
  if (audioInputSource) {
    try {
      audioInputSource.disconnect()
    } catch (e) { }
    audioInputSource = null
  }

  // Stop audio input stream if active
  if (audioInputStream) {
    audioInputStream.getTracks().forEach(track => track.stop())
    audioInputStream = null
  }

  // Cleanup player and free buffer memory
  if (player) {
    // Dispose buffer first to free memory
    if (player.buffer && typeof player.buffer.dispose === 'function') {
      try {
        player.buffer.dispose()
      } catch (e) { }
    }
    player.dispose()
  }

  // Dispose and clear audio buffer reference
  if (currentAudioBuffer) {
    try {
      if (typeof (currentAudioBuffer as any).dispose === 'function') {
        (currentAudioBuffer as any).dispose()
      }
    } catch (e) { }
  }
  currentAudioBuffer = null

  if (gainNode) gainNode.dispose()
  if (eq3) eq3.dispose()
  if (balanceSplit) balanceSplit.dispose()
  if (balanceLeft) balanceLeft.dispose()
  if (balanceRight) balanceRight.dispose()
  if (balanceMerge) balanceMerge.dispose()
  if (volumeSplit) volumeSplit.dispose()
  if (volumeNodeL) volumeNodeL.dispose()
  if (volumeNodeR) volumeNodeR.dispose()
  if (volumeMerge) volumeMerge.dispose()
  if (meterL) meterL.dispose()
  if (meterR) meterR.dispose()
  if (channelSplit) channelSplit.dispose()
  if (waveform) waveform.dispose()
  if (compressor) compressor.dispose()
  if (reverb) reverb.dispose()

  // Cleanup parametric EQ filters if present
  if (parametricEQFilters) {
    try {
      if (parametricEQFilters.dispose) {
        parametricEQFilters.dispose()
      }
    } catch (e) { }
    parametricEQFilters = null
  }

  if (resizeObserver) {
    resizeObserver.disconnect()
  }

  if (levelMonitorInterval) {
    clearInterval(levelMonitorInterval)
  }

  if (playbackTimeInterval) {
    clearInterval(playbackTimeInterval)
  }

  // Stop waveform drawing
  waveformDisplayRef.value?.stop()

  // Remove device change listener
  navigator.mediaDevices.removeEventListener('devicechange', refreshAudioInputs)
})

// FX Functions - Physically reconnect chain to preserve stereo
function toggleCompressor() {
  compressorEnabled.value = !compressorEnabled.value

  if (!eq3 || !compressor || !reverb) return

  // Disconnect eq3 from everything
  try {
    eq3.disconnect(compressor)
  } catch (e) { }
  try {
    eq3.disconnect(reverb)
  } catch (e) { }

  // Disconnect compressor from reverb
  try {
    compressor.disconnect(reverb)
  } catch (e) { }

  if (compressorEnabled.value) {
    // Apply real parameters from component
    const params = trackCompressorRef.value?.getParams() || {
      threshold: -20,
      ratio: 4,
      attack: 0.1,
      release: 0.25
    }
    compressor.threshold.value = params.threshold
    compressor.ratio.value = params.ratio
    compressor.attack.value = params.attack
    compressor.release.value = params.release

    // Chain: eq3 → compressor → reverb
    eq3.connect(compressor)
    compressor.connect(reverb)
  } else {
    // Bypass compressor completely: eq3 → reverb (skip compressor)
    eq3.connect(reverb)
  }
}

function toggleReverb() {
  reverbEnabled.value = !reverbEnabled.value

  if (!reverb) return

  if (reverbEnabled.value) {
    // Apply real parameters from component
    const params = trackReverbRef.value?.getParams() || {
      decay: 1.5,
      preDelay: 0.01,
      wet: 0.3
    }
    reverb.decay = params.decay
    reverb.preDelay = params.preDelay
    // Direct assignment instead of rampTo
    reverb.wet.value = params.wet
  } else {
    // Bypass: wet=0 = no reverb
    reverb.wet.value = 0
  }
}
</script>
