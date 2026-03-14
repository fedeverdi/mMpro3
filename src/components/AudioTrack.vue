<template>
  <div ref="trackElement" :class="[
    'track-channel relative bg-gray-800 rounded-lg border p-1 flex flex-col items-center gap-1 h-full',
    'border-gray-700'
  ]">

    <!-- Track Header -->
    <div class="w-full flex flex-col gap-1">
      <div class="w-full flex items-center justify-between gap-1">
        <div class="flex items-center gap-1 flex-1 justify-center">
          <div class="text-xs font-bold text-gray-300">Track {{ trackNumber }}</div>
        </div>
        <button @click="$emit('remove')"
          class="w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
          title="Remove Track">
          ×
        </button>
      </div>

      <!-- Audio Source Selector -->
      <AudioSourceSelector v-model="audioSourceType" />

      <!-- Audio Input Device Selector -->
      <div v-if="audioSourceType === 'input'" class="w-full">
        <InputSelector icon="🎤" title="Select Audio Input" :devices="audioInputDevices"
          :selected-device-id="selectedAudioInput" default-label="No Input"
          default-description="Select an audio input device" default-icon="🎤" :show-file-option="false"
          @select="handleInputSelect" />
      </div>

      <!-- Audio File Selector -->
      <div v-if="audioSourceType === 'file'" class="w-full">
        <LibraryButton :file-name="selectedFileName" @click="openLibrary" />
      </div>

      <!-- Play/Stop Controls -->
      <PlayStopControls 
        :disabled="!selectedFileName"
        :is-playing="isPlaying"
        :is-playlist-mode="isPlaylistMode"
        @play="handlePlayFile"
        @stop="handleStopFile"
      />

      <!-- Waveform Display - Always visible -->
      <WaveformDisplay v-if="isLargeSize" :track-number="trackNumber - 1" :show-mode-buttons="false" mode="signal"
        :is-active="(audioSourceType === 'file' && isPlaying) || (audioSourceType === 'input' && selectedAudioInput !== '')" />
    </div>

    <!-- Main Content -->
    <div class="w-full flex-1 flex flex-col gap-2 min-h-0">

      <!-- Gain Control -->
      <div class="w-full flex items-center justify-center gap-2 h-[4rem]">
        <div class="flex flex-col gap-1 items-center justify-center pt-1">
          <PadButton v-model="padEnabled" />
          <HPFButton v-model="hpfEnabled" />
        </div>
        <div class="scale-[0.65]">
          <Knob v-model="gain" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Gain" unit="dB"
            color="#8b5cf6" @drag-start="isDraggingGain = true" @drag-end="isDraggingGain = false" />
        </div>
      </div>

      <!-- Effects Section -->
      <div class="w-full bg-gray-900 rounded p-1 border border-gray-700 grid grid-cols-2 gap-1">
        <TrackGate ref="trackGateRef" :track-number="trackNumber" :enabled="gateEnabled" :input-level-db="gateInputDb"
          :attenuation-db="gateAttenuationDb" @toggle="toggleGate" @update-params="handleGateParamsUpdate" />
        <TrackCompressor ref="trackCompressorRef" :track-number="trackNumber" :enabled="compressorEnabled"
          :input-level-db="compressorInputDb" :gain-reduction-db="compressorReductionDb" @toggle="toggleCompressor"
          @params-changed="handleCompressorParamsChanged" />
      </div>

      <!-- EQ Section -->
      <div class="w-full bg-gray-900 rounded p-1 border border-gray-700 relative">
        <div class="flex items-center justify-between px-2 gap-1">
          <button @click="showEQ3Bands = !showEQ3Bands"
            class="flex items-center gap-1 hover:text-gray-200 transition-colors flex-1">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor"
              class="w-3 h-3 text-gray-400 transition-transform" :class="showEQ3Bands ? 'rotate-90' : ''">
              <path fill-rule="evenodd"
                d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                clip-rule="evenodd" />
            </svg>
            <div class="text-xs text-gray-400 uppercase tracking-wide">EQ</div>
          </button>
          <button @click="showParametricEQ = true"
            class="px-2 py-0.5 text-[0.6rem] font-bold rounded bg-blue-600/20 hover:bg-blue-600/40 text-blue-300 border border-blue-500/30 transition-all">
            PEQ
          </button>
        </div>

        <!-- EQ Thumbnail (Frequency Response Curve) -->
        <EQThumbnail v-if="isLargeSize" :system-filters="eq4BandFilters" :filters="parametricEQFilters" :track-number="trackNumber" />

        <!-- 4-Band Parametric EQ - Absolute positioned -->
        <div class="absolute top-full left-0 right-0 z-[1000] mt-1">
          <TrackEQ :track-number="trackNumber" :show="showEQ3Bands" v-model:model-low="eqLow"
            v-model:model-low-mid="eqLowMid" v-model:model-high-mid="eqHighMid" v-model:model-high="eqHigh"
            v-model:model-enabled="eqEnabled" />
        </div>
      </div>

      <!-- Aux Sends -->
      <div v-if="props.auxBuses && props.auxBuses.length > 0" class="w-full">
        <TrackAuxSends :track-number="trackNumber" :aux-buses="props.auxBuses" :aux-sends-data="auxSendsData"
          @toggle-panel="showAuxSendsPanel = !showAuxSendsPanel" @update-sends="handleAuxSendsUpdate" />

        <!-- Aux Send Controls Grid (expandable) -->
        <div v-if="showAuxSendsPanel"
          class="w-full z-[200] absolute bg-gray-800 left-0 top-[6.8rem] max-h-[calc(100vh-12rem)] overflow-y-auto aux-scrollbar p-2">
          <button @click="showAuxSendsPanel = false"
            class="absolute right-2 top-2 w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all z-10"
            title="Close">
            ×
          </button>
          <div class="grid grid-cols-1 gap-2 pt-6">
            <AuxSendControl v-for="aux in props.auxBuses" :key="aux.id" :aux="aux"
              :aux-send-data="auxSendsData[aux.id]" @update-level="(val) => updateAuxSend(aux.id, val)"
              @toggle-pre-post="toggleAuxPrePost(aux.id)" @toggle-mute="toggleAuxMute(aux.id)" 
              @drag-start="handleAuxDragStart(aux.id)" @drag-end="handleAuxDragEnd(aux.id)" />
          </div>
        </div>
      </div>

        <!-- Mute & Solo Buttons -->
      <MuteSoloButtons 
        :is-muted="isMuted"
        :is-solo="isSolo"
        @toggle-mute="toggleMute"
        @toggle-solo="toggleSolo"
      />


      <!-- Pan Knob -->
      <div class="flex justify-center scale-[0.75]" :class="{ 'scale-[0.5] -mt-5' : !isLargeSize }">
        <PanKnob v-model="pan" label="Pan" @drag-start="isDraggingPan = true" @drag-end="isDraggingPan = false" />
      </div>
      <div class="text-[0.455rem] uppercase text-center mb-6" :class="{ 'mb-[2.5rem] -mt-5' : !isLargeSize }">Volume</div>

      <!-- Volume Fader and VU Meter -->
      <div class="flex flex-col flex-1 min-h-0 pb-[2rem] ">
        <div ref="faderContainer" class="flex-1 relative flex items-center justify-center gap-1 min-h-0">

          <!-- Routing and Phase Control Buttons -->
          <div class="flex flex-col gap-1 absolute left-[0.2rem] top-1/2 transform -translate-y-1/2 z-50">
            <button @click="toggleRouteToMaster" title="Route to Master"
              class="w-5 h-7 text-[7px] font-bold rounded transition-all flex items-center justify-center"
              :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
              M
            </button>
            <!-- Subgroup routing buttons -->
            <button v-for="subgroup in props.subgroups" :key="subgroup.id" @click="toggleSubgroupRoute(subgroup.id)"
              :title="`Route to ${subgroup.name}`"
              class="w-5 h-6 text-[7px] font-bold rounded transition-all flex items-center justify-center"
              :class="routedSubgroups.has(subgroup.id) ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
              {{ subgroup.id + 1 }}
            </button>
            <!-- Phase Invert Button -->
            <button @click="togglePhaseInvert"
              class="w-5 h-5 mt-4 text-[0.65rem] font-bold rounded transition-all border mt-1"
              :class="phaseInverted
                ? 'bg-purple-600 border-purple-400 text-white shadow-md shadow-purple-500/50'
                : 'bg-gray-800 border-gray-600 text-gray-400 hover:bg-gray-700 hover:border-gray-500'"
              title="Phase Invert">
              Ø
            </button>
            <!-- Phase Correlation Button -->
            <button @click="showPhaseCorrelationModal = true"
              class="w-5 h-5 text-[0.5rem] font-bold rounded border transition-all duration-300"
              :class="{
                'bg-red-900 border-red-600 text-red-300 animate-pulse': hasSignal && trackPhaseCorrelation < -0.2,
                'bg-yellow-900 border-yellow-600 text-yellow-300': hasSignal && trackPhaseCorrelation >= -0.2 && trackPhaseCorrelation < 0.2,
                'bg-green-900 border-green-600 text-green-300': hasSignal && trackPhaseCorrelation >= 0.2 && trackPhaseCorrelation < 0.85,
                'bg-blue-900 border-blue-600 text-blue-300': hasSignal && trackPhaseCorrelation >= 0.85,
                'bg-transparent border-gray-600 text-gray-400': !hasSignal,
                'hover:brightness-110': hasSignal
              }"
              :title="hasSignal ? `Phase Correlation: ${trackPhaseCorrelation.toFixed(2)}` : 'No signal'">
              <svg class="w-3 h-3 mx-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M12 2 L18 8 L12 14 L6 8 Z" />
                <line x1="12" y1="14" x2="12" y2="22" />
                <line x1="8" y1="18" x2="16" y2="18" />
              </svg>
            </button>
          </div>

          <TrackFader v-if="useFader && faderHeight > 0" v-model="volume" :trackHeight="faderHeight" 
            @drag-start="isDraggingVolume = true" @drag-end="isDraggingVolume = false" />
          
          <div v-else-if="!useFader" class="flex items-center justify-center flex-1">
            <KnobVolume v-model="volume" @drag-start="isDraggingVolume = true" @drag-end="isDraggingVolume = false" />
          </div>

          <TrackMeter class="absolute right-[0.4rem] top-1/2 transform -translate-y-1/2 z-50" v-if="faderHeight > 0"
            :levelL="trackLevelL" :levelR="trackLevelR" :isStereo="isStereo"
            :height="faderHeight + 20" />
        </div>
      </div>
    </div>
  </div>

  <!-- Parametric EQ Modal -->
  <ParametricEQModal v-model="showParametricEQ" :track-number="trackNumber - 1"
    :eq-filters="parametricEQFilters"
    :title="`Parametric EQ - Track ${trackNumber}`" @update="handleParametricEQUpdate" />
  
  <!-- Phase Correlation Modal -->
  <PhaseCorrelationModal 
    v-model="showPhaseCorrelationModal" 
    :track-number="trackNumber"
    :correlation="trackPhaseCorrelation"
    :audio-data-l="trackWaveformData.left"
    :audio-data-r="trackWaveformData.right"
    :is-stereo="isStereo"
  />
</template>

<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, ref, watch, nextTick, type Ref } from 'vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import AudioSourceSelector from './audioTrack/AudioSourceSelector.vue'
import HPFButton from './audioTrack/HPFButton.vue'
import InputSelector from './audioTrack/InputSelector.vue'
import LibraryButton from './audioTrack/LibraryButton.vue'
import MuteSoloButtons from './audioTrack/MuteSoloButtons.vue'
import PadButton from './audioTrack/PadButton.vue'
import PlayStopControls from './audioTrack/PlayStopControls.vue'
import PanKnob from './audioTrack/PanKnob.vue'
import TrackCompressor from './audioTrack/TrackCompressor.vue'
import TrackEQ from './audioTrack/TrackEQ.vue'
import TrackFader from './audioTrack/TrackFader.vue'
import KnobVolume from './audioTrack/KnobVolume.vue'
import TrackGate from './audioTrack/TrackGate.vue'
import TrackMeter from './audioTrack/TrackMeter.vue'
import TrackAuxSends from './audioTrack/TrackAuxSends.vue'
import AuxSendControl from './audioTrack/AuxSendControl.vue'
import Knob from './core/Knob.vue'
import ParametricEQModal from './master/ParametricEQModal.vue'
import PhaseCorrelationModal from './audioTrack/PhaseCorrelationModal.vue'
import WaveformDisplay from './audioTrack/WaveformDisplay.vue'
import EQThumbnail from './audioTrack/EQThumbnail.vue'

// Props
const props = defineProps<{
  trackNumber: number
  masterChannel?: any
  subgroups?: Array<{ id: number; name: string; channel?: any }>
  auxBuses?: Array<{ id: string | number; name: string; channel?: any }>
  allowSubgroupRouting?: boolean
}>()

// Import audio engine from context
const audioEngine = inject('audioEngine') as any

// Track window height reactively for responsive layout
const windowHeight = ref(window.innerHeight)

// Definiscom una computed isLargeSize per determinare l'altezza window disponibile in modo da avere spazio per la traccia
const isLargeSize = computed(() => {
  return windowHeight.value > 800
})

// Emits
const emit = defineEmits<{
  remove: []
  'toggle-arm': []
  'open-library': [trackNumber: number]
  'update:auxSends': [sends: Record<string, { level: number, preFader: boolean, muted: boolean }>]
}>()

// Audio devices
const { audioInputDevices } = useAudioDevices()

// Reactive state - UI Only
const trackElement = ref<HTMLElement | null>(null)
const faderContainer = ref<HTMLElement | null>(null)
const faderHeight = ref(0)
const selectedAudioFile = ref<string | null>(null)

// Height threshold for switching between fader and knob
const FADER_HEIGHT_THRESHOLD = 120
const useFader = computed(() => faderHeight.value >= FADER_HEIGHT_THRESHOLD)

const selectedFileName = ref<string | null>(null)

const audioSourceType = ref<'input' | 'file'>('input')
const selectedAudioInput = ref<string>('')

// Playlist state
const playlistFiles = ref<any[]>([])
const currentPlaylistIndex = ref(0)
const currentPlaylist = ref<any | null>(null)
const lastFileEndedDetected = ref(false) // Track to avoid duplicate playNextInPlaylist calls

// Control values
const volume = ref(0) // dB (-90 to +12)
const gain = ref(-12) // dB (default -12 dB for safe headroom)
const padEnabled = ref(false)
const hpfEnabled = ref(false)
const pan = ref(0) // -1 to 1
const isMuted = ref(false)
const isSolo = ref(false)
const phaseInverted = ref(false)
const showPhaseCorrelationModal = ref(false)
const routeToMaster = ref(true)
const routedSubgroups = ref<Set<number>>(new Set()) // Track which subgroups this track is routed to

// Computed - Check if we're in playlist mode
const isPlaylistMode = computed(() => {
  return currentPlaylist.value !== null && playlistFiles.value.length > 0
})

// Aux sends state - local ref-based state like volume/gain/pan
const auxSendsState = ref<Map<number, {
  level: number,
  muted: boolean,
  preFader: boolean
}>>(new Map())
const auxSendsIsDragging = ref<Map<number, boolean>>(new Map())

// Computed: Convert auxSendsState Map to Record format for component compatibility
const auxSendsData = computed(() => {
  const result: Record<string, { level: number, preFader: boolean, muted: boolean }> = {}
  auxSendsState.value.forEach((value, auxId) => {
    result[`aux-${auxId}`] = value
  })
  return result
})

const showAuxSendsPanel = ref(false)

// Effects state
const gateEnabled = ref(false)
const compressorEnabled = ref(false)
const showEQ3Bands = ref(false)
const showParametricEQ = ref(false)

// EQ values (4-band parametric EQ)
const eqLow = ref(0)       // -24 to +24 dB (80Hz Low Shelf)
const eqLowMid = ref(0)    // -24 to +24 dB (400Hz Peaking)
const eqHighMid = ref(0)   // -24 to +24 dB (2500Hz Peaking)
const eqHigh = ref(0)      // -24 to +24 dB (8000Hz High Shelf)
const eqEnabled = ref(true)

// Parametric EQ filters from modal
const parametricEQFilters = ref<any[]>([])

// Computed: Convert 4-band EQ values + HPF to filter format for EQThumbnail (system filters)
const eq4BandFilters = computed(() => {
  const filters = []

  // Add HPF (80Hz high-pass) if enabled
  if (hpfEnabled.value) {
    filters.push({
      type: 'highpass',
      frequency: 80,
      gain: 0,
      Q: 0.707
    })
  }

  // Add 4-band EQ filters if enabled
  if (eqEnabled.value) {
    const eqFilters = [
      {
        type: 'lowshelf',
        frequency: 80,
        gain: eqLow.value,
        Q: 0.707
      },
      {
        type: 'peaking',
        frequency: 400,
        gain: eqLowMid.value,
        Q: 0.707
      },
      {
        type: 'peaking',
        frequency: 2500,
        gain: eqHighMid.value,
        Q: 0.707
      },
      {
        type: 'highshelf',
        frequency: 8000,
        gain: eqHigh.value,
        Q: 0.707
      }
    ].filter(f => Math.abs(f.gain) > 0.1) // Only show bands with significant gain

    filters.push(...eqFilters)
  }

  return filters
})

// Watch when parametric EQ modal opens
watch(showParametricEQ, (isOpen) => {
  if (isOpen) {
  }
})

// Meter levels (simulated for now)
const trackLevelL = ref(-60)
const trackLevelR = ref(-60)

// Phase correlation from audio engine (smoothed)
const smoothedPhaseCorrelation = ref(0)
const rawPhaseCorrelation = computed(() => {
  const levels = audioEngine?.state.value.trackLevels.get(props.trackNumber - 1)
  return levels?.phaseCorrelation ?? 0
})

// Smooth phase correlation updates (reduce jitter)
watch(rawPhaseCorrelation, (newValue) => {
  const smoothingFactor = 0.2 // 0 = no smoothing, 1 = instant (20% lerp)
  smoothedPhaseCorrelation.value = smoothedPhaseCorrelation.value * (1 - smoothingFactor) + newValue * smoothingFactor
})

const trackPhaseCorrelation = computed(() => smoothedPhaseCorrelation.value)

// Check if track has signal (for phase correlation button visibility)
const hasSignal = computed(() => {
  return trackLevelL.value > -55 || trackLevelR.value > -55
})

// Check if track is stereo (from Rust file player state)
const isStereo = computed(() => {
  const params = audioEngine?.state.value.trackParameters.get(props.trackNumber - 1)
  return params?.isStereo ?? false
})

// Waveform data from audio engine
const trackWaveformData = computed(() => {
  const waveform = audioEngine?.state.value.trackWaveforms.get(props.trackNumber - 1)
  if (!waveform || waveform.length === 0) return { left: new Float32Array(0), right: new Float32Array(0) }
  
  // Convert mono waveform to stereo (split evenly)
  // In reality the waveform is already mixed L+R, so we just duplicate it
  const float32Array = new Float32Array(waveform)
  return {
    left: float32Array,
    right: float32Array
  }
})

// File playback state
const isPlaying = ref(false)

// Refs to child components
const trackEQRef = ref<InstanceType<typeof TrackEQ> | null>(null)
const trackCompressorRef = ref<InstanceType<typeof TrackCompressor> | null>(null)
const trackGateRef = ref<InstanceType<typeof TrackGate> | null>(null)

// Handlers
function handleInputSelect(deviceId: string | null) {
  selectedAudioInput.value = deviceId || ''
  selectedAudioFile.value = null
  selectedFileName.value = null

  if (audioEngine?.state.value.isRunning) {
    if (deviceId) {
      // Check if it's an aux return selection
      if (deviceId.startsWith('aux-return-')) {
        const auxIndex = parseInt(deviceId.replace('aux-return-', ''))
        console.log(`[Track ${props.trackNumber}] Setting aux return: Aux ${auxIndex + 1}`)
        audioEngine.setTrackSourceAuxReturn(props.trackNumber - 1, auxIndex)
      } else {
        // Find the device to get its name
        const device = audioInputDevices.value.find(d => d.id === deviceId)
        if (device) {
          // Extract device name (remove "- Channel X" suffix if present)
          const deviceName = device.name.replace(/ - Channel \d+$/, '')
          // Extract channel indices from composite ID if present (format: "deviceId:channelIndex")
          const channelMatch = deviceId.match(/:(\d+)$/)
          const channelIndex = channelMatch ? parseInt(channelMatch[1]) : 0
                    
          // For stereo: use channel and channel+1
          audioEngine.setTrackSourceInput(props.trackNumber - 1, channelIndex, channelIndex + 1, deviceName)
        }
      }
    } else {
      // Clear input (deviceId is null = "No Input" selected)
      console.log(`[Track ${props.trackNumber}] Clearing audio input`)
      audioEngine.setTrackSourceInput(props.trackNumber - 1, 0, 1, null)
    }
  }
}

function openLibrary() {
  emit('open-library', props.trackNumber)
}

// File playback controls
function handlePlayFile() {
  // If in playlist mode and already playing, go to next track
  if (isPlaylistMode.value && isPlaying.value) {
    playNextInPlaylist()
    return
  }
  
  // Otherwise, play the current file (file is already loaded in Rust)
  if (audioEngine?.state.value.isRunning && selectedFileName.value) {

    // Don't pass file_id - just play the already loaded file
    audioEngine.playFile(props.trackNumber - 1)
    // isPlaying will be updated via Rust engine broadcast
  }
}

function handleStopFile() {
  if (audioEngine?.state.value.isRunning && selectedFileName.value) {
    audioEngine.stopFile(props.trackNumber - 1)
    // isPlaying will be updated via Rust engine broadcast
  }
}

// Method to load file from library by ID or file object
async function loadFileFromLibrary(fileIdOrObject: string | any, autoPlay = false, fromPlaylist = false) {
  try {
    // If string ID is passed, fetch file data from library
    let fileData: any
    
    if (typeof fileIdOrObject === 'string') {
      fileData = await window.audioEngine.getLibraryFile(fileIdOrObject)
      if (!fileData) {
        throw new Error(`File not found in library: ${fileIdOrObject}`)
      }
    } else {
      // File object passed directly
      fileData = fileIdOrObject
    }
    
    // Reset playlist mode if NOT loading from playlist
    if (!fromPlaylist) {
      currentPlaylist.value = null
      playlistFiles.value = []
      currentPlaylistIndex.value = 0
    }
    
    selectedAudioFile.value = fileData.id
    selectedFileName.value = fileData.title || fileData.fileName
    audioSourceType.value = 'file'
    
    // Reset file ended detection flag when loading a new file
    lastFileEndedDetected.value = false

    // Use file path directly from library (no need for temp file)
    if (audioEngine?.state.value.isRunning && fileData.filePath) {
      // Pass playlist info if loading from playlist
      const playlistId = fromPlaylist ? currentPlaylist.value?.id : null
      const playlistName = fromPlaylist ? currentPlaylist.value?.name : null
      const playlistIndex = fromPlaylist ? currentPlaylistIndex.value : null
      
      audioEngine.setTrackSourceFile(props.trackNumber - 1, fileData.filePath, fileData.artist, fileData.title, playlistId, playlistName, playlistIndex)
      
      // Auto-play the file only if requested
      if (autoPlay) {
        audioEngine.playFile(props.trackNumber - 1)
        // isPlaying will be updated via Rust engine broadcast
      }
    } else {
      console.warn(`[Track ${props.trackNumber}] Cannot load file - engine not running or filePath missing`, {
        engineRunning: audioEngine?.state.value.isRunning,
        hasFilePath: !!fileData.filePath
      })
    }
  } catch (error) {
    console.error(`[Track ${props.trackNumber}] Error loading file from library:`, error)
  }
}

// Method to load playlist from library (called from parent)
async function loadPlaylistFromLibrary(playlist: any) {
  try {
    // Import usePlaylist to get files
    const { usePlaylist } = await import('~/composables/usePlaylist')
    const { getPlaylistFiles } = usePlaylist()
    const files = await getPlaylistFiles(playlist.id)
        
    if (files.length === 0) {
      return
    }

    // Store playlist state
    currentPlaylist.value = playlist
    playlistFiles.value = files
    currentPlaylistIndex.value = 0
    
    // Load and play first file
    const firstFile = files[0]
    const trackName = firstFile.title || firstFile.fileName
    const trackDisplay = firstFile.artist ? `${firstFile.artist} - ${trackName}` : trackName
    
    selectedFileName.value = `${playlist.name} (1/${files.length}) - ${trackDisplay}`
    audioSourceType.value = 'file'
    
    // Load first file from playlist (fromPlaylist = true, autoPlay = false - user must press play)
    await loadFileFromLibrary(firstFile, false, true)
  } catch (error) {
    console.error('Error loading playlist:', error)
  }
}

// Play next file in playlist
async function playNextInPlaylist() {
  if (!currentPlaylist.value || playlistFiles.value.length === 0) return
  
  // Save the current playing state before stopping
  const wasPlaying = isPlaying.value

  // Stop current playback
  if (audioEngine?.state.value.isRunning) {
    audioEngine.stopFile(props.trackNumber - 1)
    // isPlaying will be updated via Rust engine broadcast
  }

  // Wait a bit for the audio buffer to clear
  await new Promise(resolve => setTimeout(resolve, 100))
  
  const nextIndex = (currentPlaylistIndex.value + 1) % playlistFiles.value.length
  currentPlaylistIndex.value = nextIndex
  
  const nextFile = playlistFiles.value[nextIndex]
  if (!nextFile) return
  
  const trackName = nextFile.title || nextFile.fileName
  const trackDisplay = nextFile.artist ? `${nextFile.artist} - ${trackName}` : trackName
  selectedFileName.value = `${currentPlaylist.value.name} (${nextIndex + 1}/${playlistFiles.value.length}) - ${trackDisplay}`
  
  // Load and auto-play if it was playing before (fromPlaylist = true)
  await loadFileFromLibrary(nextFile, wasPlaying, true)
}

function toggleMute() {
  isMuted.value = !isMuted.value
}

function toggleSolo() {
  isSolo.value = !isSolo.value
}

function togglePhaseInvert() {
  phaseInverted.value = !phaseInverted.value
}

function toggleRouteToMaster() {
  routeToMaster.value = !routeToMaster.value
}

function toggleSubgroupRoute(subgroupId: number) {
  // Create a new Set to trigger Vue reactivity
  const newRoutes = new Set(routedSubgroups.value)
  
  if (newRoutes.has(subgroupId)) {
    newRoutes.delete(subgroupId)
    routedSubgroups.value = newRoutes
    // Send to backend
    if (audioEngine?.state.value.isRunning) {
      audioEngine.setTrackRouteToSubgroup(props.trackNumber - 1, subgroupId, false)
    }
  } else {
    newRoutes.add(subgroupId)
    routedSubgroups.value = newRoutes
    // Send to backend
    if (audioEngine?.state.value.isRunning) {
      audioEngine.setTrackRouteToSubgroup(props.trackNumber - 1, subgroupId, true)
    }
  }
}

function toggleGate() {
  gateEnabled.value = !gateEnabled.value
}

function toggleCompressor() {
  compressorEnabled.value = !compressorEnabled.value
}

function handleEQParamsChanged(params: { low: number; mid: number; high: number }) {
  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackEQ(
      props.trackNumber - 1,
      params.low,
      params.mid,
      params.high
    )
  }
}

function handleCompressorParamsChanged(params: { threshold: number; ratio: number; attack: number; release: number }) {
  // Send to Rust engine
  if (audioEngine?.state.value.isRunning && compressorEnabled.value) {
    audioEngine.setTrackCompressor(
      props.trackNumber - 1,
      true,
      params.threshold,
      params.ratio,
      params.attack * 1000,   // Convert seconds to milliseconds
      params.release * 1000   // Convert seconds to milliseconds
    )
  }
}

function handleGateParamsUpdate(params: { threshold: number; attack: number; release: number; range: number }) {
  // Send to Rust engine - convert attack/release from seconds to milliseconds
  if (audioEngine?.state.value.isRunning && gateEnabled.value) {
    audioEngine.setTrackGate(
      props.trackNumber - 1,
      true,
      params.threshold,
      params.range,
      params.attack * 1000,   // Convert seconds to milliseconds
      params.release * 1000   // Convert seconds to milliseconds
    )
  }
}

// Handle aux sends updates
function handleAuxSendsUpdate(sends: Record<string, { level: number, preFader: boolean, muted: boolean }>) {
  // Update local state Map
  Object.entries(sends).forEach(([auxId, send]) => {
    const auxIndex = parseInt(auxId.replace(/\D/g, ''))
    auxSendsState.value.set(auxIndex, {
      level: send.level,
      muted: send.muted,
      preFader: send.preFader
    })
  })

  // Send to Rust engine for each aux
  if (audioEngine?.state.value.isRunning) {
    Object.entries(sends).forEach(([auxId, send]) => {
      // Extract numeric index from aux ID (aux-0, aux-1, etc. - already 0-based)
      const auxIndex = parseInt(auxId.replace(/\D/g, ''))
      // Convert dB to linear gain
      const linearGain = Math.pow(10, send.level / 20)
      audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader ?? false, send.muted)
    })
  }
}

// Update individual aux send level
function updateAuxSend(auxId: string | number, level: number) {
  // Extract numeric index from aux ID (aux-0, aux-1, etc.)
  const auxIndex = typeof auxId === 'number' ? auxId : parseInt(auxId.replace(/\D/g, ''))
  
  // Get existing state or create default
  const existingState = auxSendsState.value.get(auxIndex) || {
    level: -60,
    preFader: false,
    muted: true
  }

  // Update state with new level
  auxSendsState.value.set(auxIndex, {
    ...existingState,
    level: level,
    // Auto-unmute if level > -60
    muted: level > -60 ? false : existingState.muted
  })

  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    const send = auxSendsState.value.get(auxIndex)!
    const linearGain = Math.pow(10, send.level / 20)
    audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader ?? false, send.muted)
  }
}

// Toggle aux send pre/post fader
function toggleAuxPrePost(auxId: string | number) {
  // Extract numeric index from aux ID
  const auxIndex = typeof auxId === 'number' ? auxId : parseInt(auxId.replace(/\D/g, ''))
  
  // Get existing state or create default
  const existingState = auxSendsState.value.get(auxIndex) || {
    level: -60,
    preFader: false,
    muted: true
  }

  // Toggle preFader
  auxSendsState.value.set(auxIndex, {
    ...existingState,
    preFader: !existingState.preFader
  })

  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    const send = auxSendsState.value.get(auxIndex)!
    const linearGain = Math.pow(10, send.level / 20)
    audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader ?? false, send.muted)
  }
}

// Toggle aux send mute
function toggleAuxMute(auxId: string | number) {
  // Extract numeric index from aux ID
  const auxIndex = typeof auxId === 'number' ? auxId : parseInt(auxId.replace(/\D/g, ''))
  
  // Get existing state or create default
  const existingState = auxSendsState.value.get(auxIndex) || {
    level: -60,
    preFader: false,
    muted: true
  }

  // Toggle muted
  auxSendsState.value.set(auxIndex, {
    ...existingState,
    muted: !existingState.muted
  })

  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    const send = auxSendsState.value.get(auxIndex)!
    const linearGain = Math.pow(10, send.level / 20)
    audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader ?? false, send.muted)
  }
}

// Handle aux send drag start - set isDragging flag
function handleAuxDragStart(auxId: string | number) {
  const auxIndex = typeof auxId === 'number' ? auxId : parseInt(String(auxId).replace(/\D/g, ''))
  auxSendsIsDragging.value.set(auxIndex, true)
}

// Handle aux send drag end - clear isDragging flag
function handleAuxDragEnd(auxId: string | number) {
  const auxIndex = typeof auxId === 'number' ? auxId : parseInt(String(auxId).replace(/\D/g, ''))
  auxSendsIsDragging.value.set(auxIndex, false)
}

function handleParametricEQUpdate(filters: any) {
  // Set flag to prevent watch loop
  isUpdatingParametricFromUser.value = true
  
  // Save filters for EQThumbnail display
  if (filters.filtersData) {
    parametricEQFilters.value = filters.filtersData.map((f: any) => ({
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      Q: f.Q
    }))
  }

  // Convert filtersData to the format expected by Rust engine
  if (filters.filtersData && audioEngine?.state.value.isRunning) {
    const rustFilters = filters.filtersData.map((f: any) => ({
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      q: f.Q
    }))

    audioEngine.setParametricEQFilters(props.trackNumber - 1, rustFilters)
  }
  
  // Reset flag after a short delay to allow Rust broadcast to complete
  setTimeout(() => {
    isUpdatingParametricFromUser.value = false
  }, 100)
}

// Flag to prevent watch loops when updating from engine
const isUpdatingFromEngine = ref(false)
const isUpdatingParametricFromUser = ref(false)
const isDraggingVolume = ref(false)
const isDraggingGain = ref(false)
const isDraggingPan = ref(false)

// Watchers - Send changes to Rust engine
watch(volume, (newVolume) => {
  if (isUpdatingFromEngine.value) return
  // Convert dB to linear gain: gain = 10^(dB/20)
  // volume is in dB range (-90 to +12)
  let gainValue: number
  if (newVolume <= -90) {
    gainValue = 0.0 // Mute
  } else {
    gainValue = Math.pow(10, newVolume / 20)
  }

  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackVolume(props.trackNumber - 1, gainValue)
  }
})

watch(gain, (newGain) => {
  if (isUpdatingFromEngine.value) return
  // Convert dB to linear gain: gain = 10^(dB/20)
  // gain knob is in dB range (-12 to +12)
  const gainValue = Math.pow(10, newGain / 20)

  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackGain(props.trackNumber - 1, gainValue)
  }
})

watch(padEnabled, (enabled) => {
  if (props.allowSubgroupRouting && audioEngine?.state.value.isRunning) {
    audioEngine.setTrackPad(props.trackNumber - 1, enabled)
  }
})

watch(hpfEnabled, (enabled) => {
  if (isUpdatingFromEngine.value) return
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackHPF(props.trackNumber - 1, enabled)
  }
})

watch(phaseInverted, (enabled) => {
  if (isUpdatingFromEngine.value) return
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackPhaseInvert(props.trackNumber - 1, enabled)
  }
})

watch(isMuted, (muted) => {
  if (isUpdatingFromEngine.value) return
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackMute(props.trackNumber - 1, muted)
  }
})

watch(isSolo, (solo) => {
  if (isUpdatingFromEngine.value) return
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackSolo(props.trackNumber - 1, solo)
  }
})

watch(routeToMaster, (route) => {
  if (isUpdatingFromEngine.value) return
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackRouteToMaster(props.trackNumber - 1, route)
  }
})

watch(compressorEnabled, (enabled) => {
  if (audioEngine?.state.value.isRunning) {
    const params = trackCompressorRef.value?.getParams()
    if (params) {
      audioEngine.setTrackCompressor(
        props.trackNumber - 1,
        enabled,
        params.threshold,
        params.ratio,
        params.attack,
        params.release
      )
    }
  }
})

watch(gateEnabled, (enabled) => {
  if (audioEngine?.state.value.isRunning) {
    const params = trackGateRef.value?.getParams()
    if (params) {
      audioEngine.setTrackGate(
        props.trackNumber - 1,
        enabled,
        params.threshold,
        params.range,
        params.attack * 1000,   // Convert seconds to milliseconds
        params.release * 1000   // Convert seconds to milliseconds
      )
    }
  }
})

watch(pan, (newPan) => {
  if (isUpdatingFromEngine.value) return
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackPan(props.trackNumber - 1, newPan)
  }
})

// Compressor visualization data
const compressorInputDb = ref(-90)
const compressorReductionDb = ref(0)

// Gate visualization data
const gateInputDb = ref(-90)
const gateAttenuationDb = ref(0)

// Watch for meter level updates from audio engine
watch(
  () => audioEngine?.state.value.trackLevels.get(props.trackNumber - 1),
  (levels) => {
    if (levels) {
      // Convert linear (0-1) to dB (-60 to 0)
      // dB = 20 * log10(linear)
      trackLevelL.value = levels.left > 0 ? 20 * Math.log10(levels.left) : -60
      trackLevelR.value = levels.right > 0 ? 20 * Math.log10(levels.right) : -60

      // Update compressor visualization data
      compressorInputDb.value = levels.compressorInputDb || -90
      compressorReductionDb.value = levels.compressorReductionDb || 0

      // Update gate visualization data
      gateInputDb.value = levels.gateInputDb || -90
      gateAttenuationDb.value = levels.gateAttenuationDb || 0
      
      // Check if file ended naturally (for playlist auto-advance)
      if (levels.fileEnded && currentPlaylist.value && playlistFiles.value.length > 0) {
        // Avoid duplicate calls (fileEnded stays true until new file is loaded)
        if (!lastFileEndedDetected.value) {
          lastFileEndedDetected.value = true
          playNextInPlaylist()
        }
      } else if (!levels.fileEnded) {
        // Reset flag when file is playing (fileEnded = false)
        lastFileEndedDetected.value = false
      }
    }
  },
  { deep: true }
)

// Fader height calculation
let updateFaderHeightTimeout: ReturnType<typeof setTimeout> | null = null
function updateFaderHeight() {
  // Throttle resize calculations to prevent blocking during window animations
  if (updateFaderHeightTimeout) return
  
  updateFaderHeightTimeout = setTimeout(() => {
    if (faderContainer.value) {
      faderHeight.value = faderContainer.value.clientHeight
    }
    updateFaderHeightTimeout = null
  }, 16) // ~60fps
}

// Use centralized resize trigger from parent instead of local ResizeObserver
const resizeTrigger = inject<Ref<number>>('resizeTrigger', ref(0))

// Lifecycle
onMounted(async () => {
  // Audio input devices are already enumerated during app initialization
  // No need to refresh them here

  // Watch for centralized resize trigger instead of using ResizeObserver
  watch(resizeTrigger, () => {
    updateFaderHeight()
    windowHeight.value = window.innerHeight // Update window height on resize
  })

  updateFaderHeight()
  
  // Load initial state from trackParameters if available
  const initialParams = audioEngine?.state.value.trackParameters?.get(props.trackNumber - 1)
  if (initialParams?.routeToSubgroups) {
    routedSubgroups.value = new Set(initialParams.routeToSubgroups)
  }
  
  // Load initial aux sends state (critical for remote browsers)
  if (initialParams?.auxSends && Array.isArray(initialParams.auxSends)) {
    initialParams.auxSends.forEach((send: any, auxId: number) => {
      // Convert linear to dB, use -60 as default for uninitialized (0.0) sends
      const levelDb = send.level > 0 ? 20 * Math.log10(send.level) : -60
      auxSendsState.value.set(auxId, {
        level: levelDb,
        muted: send.muted ?? false,
        preFader: send.pre_fader ?? false
      })
      // Initialize isDragging flag
      auxSendsIsDragging.value.set(auxId, false)
    })
  }
  
  // Watch for track parameter updates from Rust engine (incoming sync)
  watch(() => audioEngine?.state.value.trackParameters?.get(props.trackNumber - 1), (params) => {
    if (!params) return
    
    isUpdatingFromEngine.value = true
    
    // Convert linear gain to dB for volume (skip if dragging)
    if (params.volume !== undefined && !isDraggingVolume.value) {
      volume.value = params.volume > 0 ? 20 * Math.log10(params.volume) : -90
    }
    
    // Convert linear gain to dB for gain (skip if dragging)
    if (params.gain !== undefined && !isDraggingGain.value) {
      gain.value = params.gain > 0 ? 20 * Math.log10(params.gain) : -12
    }
    
    // Update other parameters
    if (params.mute !== undefined) isMuted.value = params.mute
    if (params.pan !== undefined && !isDraggingPan.value) pan.value = params.pan
    if (params.routeToMaster !== undefined) routeToMaster.value = params.routeToMaster
    if (params.routeToSubgroups !== undefined) {
      routedSubgroups.value = new Set(params.routeToSubgroups)
    }
    if (params.padEnabled !== undefined) padEnabled.value = params.padEnabled
    if (params.hpfEnabled !== undefined) hpfEnabled.value = params.hpfEnabled
    if (params.phaseInverted !== undefined) phaseInverted.value = params.phaseInverted
    
    // Update compressor state
    if (params.compressor) {
      compressorEnabled.value = params.compressor.enabled
      // Sync compressor params using the component's setParams method
      if (trackCompressorRef.value && params.compressor.enabled) {
        trackCompressorRef.value.setParams({
          threshold: params.compressor.thresholdDb,
          ratio: params.compressor.ratio,
          attack: params.compressor.attackMs / 1000,  // Convert ms to seconds
          release: params.compressor.releaseMs / 1000  // Convert ms to seconds
        })
      }
    }
    
    // Update gate state  
    if (params.gate) {
      gateEnabled.value = params.gate.enabled
      // Sync gate params using the component's setParams method
      if (trackGateRef.value && params.gate.enabled) {
        trackGateRef.value.setParams({
          threshold: params.gate.thresholdDb,
          range: params.gate.rangeDb,
          attack: params.gate.attackMs / 1000,  // Convert ms to seconds
          release: params.gate.releaseMs / 1000  // Convert ms to seconds
        })
      }
    }
    
    // Update EQ enabled states
    if (params.eqEnabled !== undefined) eqEnabled.value = params.eqEnabled
    if (params.parametricEqEnabled !== undefined) {
      // Note: parametricEqEnabled from backend doesn't directly map to showParametricEQ (which is modal visibility)
      // We just sync the enabled state, modal visibility is controlled by user
    }
    
    // Update EQ band values
    if (params.eqLow !== undefined) eqLow.value = params.eqLow
    if (params.eqLowMid !== undefined) eqLowMid.value = params.eqLowMid
    if (params.eqHighMid !== undefined) eqHighMid.value = params.eqHighMid
    if (params.eqHigh !== undefined) eqHigh.value = params.eqHigh
    
    // Update file player state (sync fileName from backend)
    // Watch fileArtist and fileTitle - update label whenever they change
    if (params.fileArtist !== undefined || params.fileTitle !== undefined || (params.fileName && params.fileName.trim() !== '')) {
      let displayName: string | null = null
      if (params.fileArtist && params.fileTitle) {
        displayName = `${params.fileArtist} - ${params.fileTitle}`
      } else if (params.fileTitle) {
        displayName = params.fileTitle
      } else if (params.fileName && params.fileName.trim() !== '') {
        displayName = params.fileName
      }
      
      if (displayName) {
        selectedFileName.value = displayName
        audioSourceType.value = 'file'
      }
    }
    
    // Sync play state from Rust engine
    if (params.isPlaying !== undefined) {
      isPlaying.value = params.isPlaying
    }
    
    // Sync playlist state from Rust engine (server-side truth)
    // IMPORTANTE: Il backend viene usato solo per RIPRISTINARE lo stato dopo un reload
    // Lo stato locale ha SEMPRE la precedenza durante l'uso normale
    if (params.playlistId && params.playlistName) {
      // Se la playlist dal backend è diversa da quella locale, ripristina lo stato
      if (!currentPlaylist.value || currentPlaylist.value.id !== params.playlistId) {
        // Ricarica la playlist dal backend
        (async () => {
          try {
            const { usePlaylist } = await import('~/composables/usePlaylist')
            const { getPlaylistFiles } = usePlaylist()
            const files = await getPlaylistFiles(params.playlistId!)
          
            if (files && files.length > 0) {
              currentPlaylist.value = { id: params.playlistId, name: params.playlistName }
              playlistFiles.value = files
              currentPlaylistIndex.value = params.playlistCurrentIndex ?? 0
              
              // Update display name to show playlist info
              const currentFile = files[params.playlistCurrentIndex ?? 0]
              if (currentFile) {
                const trackName = currentFile.title || currentFile.fileName
                const trackDisplay = currentFile.artist ? `${currentFile.artist} - ${trackName}` : trackName
                selectedFileName.value = `${params.playlistName} (${(params.playlistCurrentIndex ?? 0) + 1}/${files.length}) - ${trackDisplay}`
              }
            } else {
              console.warn('[AudioTrack] No files found for playlist restore')
            }
          } catch (error) {
            console.error('Error restoring playlist from backend:', error)
          }
        })()
      } else if (params.playlistCurrentIndex !== undefined && params.playlistCurrentIndex !== currentPlaylistIndex.value) {
        // La playlist è la stessa ma l'indice è cambiato (es: next track)
        currentPlaylistIndex.value = params.playlistCurrentIndex
        
        // Update display name
        if (playlistFiles.value.length > 0) {
          const currentFile = playlistFiles.value[params.playlistCurrentIndex]
          if (currentFile) {
            const trackName = currentFile.title || currentFile.fileName
            const trackDisplay = currentFile.artist ? `${currentFile.artist} - ${trackName}` : trackName
            selectedFileName.value = `${params.playlistName} (${params.playlistCurrentIndex + 1}/${playlistFiles.value.length}) - ${trackDisplay}`
          }
        }
      }
    }
    // NON cancelliamo mai lo stato locale della playlist basandoci sui params dal backend
    // Lo stato locale ha sempre la precedenza, il backend serve solo per il ripristino
    
    // Sync aux sends from Rust engine (only when not dragging)
    if (params.auxSends && Array.isArray(params.auxSends)) {
      params.auxSends.forEach((send: any, auxId: number) => {
        // Check if this particular aux send is being dragged
        const isDragging = auxSendsIsDragging.value.get(auxId)
        if (!isDragging) {
          // Only sync if backend has a meaningful value (level > 0)
          // or if we don't have existing state yet
          const existingState = auxSendsState.value.get(auxId)
          
          if (send.level > 0 || !existingState) {
            // Convert linear to dB for level, use -60 as default for uninitialized (0.0) sends
            const levelDb = send.level > 0 ? 20 * Math.log10(send.level) : -60
            
            // Update or create the aux send state
            auxSendsState.value.set(auxId, {
              level: levelDb,
              muted: send.muted ?? false,
              preFader: send.pre_fader ?? false
            })
          }
        }
      })
    }
    
    // Re-enable watches after Vue reactivity cycle completes
    nextTick(() => { isUpdatingFromEngine.value = false })
  }, { deep: true, immediate: true })
  
  // Watch for EQ filter updates from Rust engine
  watch(() => audioEngine?.state.value.trackEQFilters, (trackEQMap) => {
    if (!trackEQMap) return
    
    // Don't update if we just sent the changes (prevent loop)
    if (isUpdatingParametricFromUser.value) return
    
    // NOTE: Rust uses 0-indexed tracks, but frontend trackNumber is 1-indexed
    const filters = trackEQMap.get(props.trackNumber - 1)
    if (!filters) return
    
    // Convert q (lowercase) to Q (uppercase) for frontend
    // The modal will handle its own blocking during drag via isDragging flag
    parametricEQFilters.value = filters.map((f: any) => ({
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      Q: f.q  // Rust uses lowercase 'q', frontend uses uppercase 'Q'
    }))
  }, { deep: true })
})

onUnmounted(() => {
  if (updateFaderHeightTimeout) {
    clearTimeout(updateFaderHeightTimeout)
  }
})

// Expose methods to parent component
defineExpose({
  loadFileFromLibrary,
  loadPlaylistFromLibrary,
  getState: () => ({
    // Audio source (single file or playlist)
    audioFile: selectedAudioFile.value && !currentPlaylist.value ? {
      id: selectedAudioFile.value,
      fileName: selectedFileName.value || ''
    } : undefined,
    
    // Playlist state
    playlist: currentPlaylist.value ? {
      id: currentPlaylist.value.id,
      name: currentPlaylist.value.name,
      currentIndex: currentPlaylistIndex.value,
      files: playlistFiles.value.map(f => ({
        id: f.id,
        fileName: f.fileName,
        title: f.title,
        artist: f.artist
      }))
    } : undefined,
    
    // Basic controls
    gain: gain.value,
    volume: volume.value,
    pan: pan.value,
    mute: isMuted.value,
    solo: isSolo.value,
    phaseInvert: phaseInverted.value,
    padEnabled: padEnabled.value,
    hpfEnabled: hpfEnabled.value,
    
    // Routing
    routeToMaster: routeToMaster.value,
    routedSubgroups: Array.from(routedSubgroups.value),
    
    // Aux Sends
    auxSends: auxSendsData.value,
    
    // Effects enabled state (detailed params are in Rust)
    gateEnabled: gateEnabled.value,
    compressorEnabled: compressorEnabled.value,
    
    // EQ (4-band)
    eqEnabled: eqEnabled.value,
    eqLow: eqLow.value,
    eqLowMid: eqLowMid.value,
    eqHighMid: eqHighMid.value,
    eqHigh: eqHigh.value,
    
    // Parametric EQ
    parametricEQFilters: parametricEQFilters.value
  }),
  setState: async (state: any) => {
    // Restore playlist or single file
    if (state.playlist) {
      // Restore playlist state
      try {
        currentPlaylist.value = {
          id: state.playlist.id,
          name: state.playlist.name
        }
        playlistFiles.value = state.playlist.files || []
        currentPlaylistIndex.value = state.playlist.currentIndex || 0
        
        // Load current file from playlist
        if (playlistFiles.value.length > 0) {
          const currentIndex = currentPlaylistIndex.value
          const currentFile = playlistFiles.value[currentIndex]
          
          if (currentFile) {
            const trackName = currentFile.title || currentFile.fileName
            const trackDisplay = currentFile.artist ? `${currentFile.artist} - ${trackName}` : trackName
            selectedFileName.value = `${currentPlaylist.value.name} (${currentIndex + 1}/${playlistFiles.value.length}) - ${trackDisplay}`
            audioSourceType.value = 'file'
            
            // Load the file without auto-playing (fromPlaylist = true)
            await loadFileFromLibrary(currentFile, false, true)
          }
        }
      } catch (error) {
        console.error('[AudioTrack] Error loading playlist from scene:', error)
      }
    } else if (state.audioFile && state.audioFile.id) {
      // Load single audio file from library by ID (without auto-playing)
      try {
        await loadFileFromLibrary(state.audioFile.id, false)
      } catch (error) {
        console.error('[AudioTrack] Error loading file from scene:', error)
        // Fallback: just set the UI state
        selectedAudioFile.value = state.audioFile.id
        selectedFileName.value = state.audioFile.fileName
        audioSourceType.value = 'file'
      }
    }
    
    // Basic controls
    gain.value = state.gain ?? -12
    volume.value = state.volume ?? 0
    pan.value = state.pan ?? 0
    isMuted.value = state.mute ?? false
    isSolo.value = state.solo ?? false
    phaseInverted.value = state.phaseInvert ?? false
    padEnabled.value = state.padEnabled ?? false
    hpfEnabled.value = state.hpfEnabled ?? false
    
    // Routing
    routeToMaster.value = state.routeToMaster ?? true
    routedSubgroups.value = new Set(state.routedSubgroups ?? [])
    
    // Apply basic controls and routing to backend
    if (audioEngine?.state.value.isRunning) {
      // Apply route to master
      audioEngine.setTrackRouteToMaster(props.trackNumber - 1, routeToMaster.value)
      
      // Apply routes to subgroups
      if (state.routedSubgroups && Array.isArray(state.routedSubgroups)) {
        for (const subgroupId of state.routedSubgroups) {
          audioEngine.setTrackRouteToSubgroup(props.trackNumber - 1, subgroupId, true)
        }
      }
    }
    
    // Aux Sends - populate local state Map and apply to backend
    if (state.auxSends) {
      for (const [auxKey, sendData] of Object.entries(state.auxSends)) {
        const send = sendData as { level: number, preFader: boolean, muted: boolean }
        // Extract numeric index from aux key (aux-0, aux-1, etc.)
        const auxIndex = typeof auxKey === 'string' ? parseInt(auxKey.replace(/\D/g, '')) : auxKey
        
        // Update local state
        auxSendsState.value.set(auxIndex, {
          level: send.level,
          preFader: send.preFader,
          muted: send.muted
        })
        
        // Apply to backend
        if (audioEngine?.state.value.isRunning) {
          const linearGain = Math.pow(10, send.level / 20)
          audioEngine.setTrackAuxSend(
            props.trackNumber - 1,
            auxIndex,
            linearGain,
            send.preFader,
            send.muted
          )
        }
      }
    }
    
    // Effects
    gateEnabled.value = state.gateEnabled ?? false
    compressorEnabled.value = state.compressorEnabled ?? false
    
    // EQ
    eqEnabled.value = state.eqEnabled ?? false
    eqLow.value = state.eqLow ?? 0
    eqLowMid.value = state.eqLowMid ?? 0
    eqHighMid.value = state.eqHighMid ?? 0
    eqHigh.value = state.eqHigh ?? 0
    
    // Parametric EQ
    parametricEQFilters.value = state.parametricEQFilters ?? []
  }
})
</script>

<style scoped>
/* Thin scrollbar for aux sends panel */
.aux-scrollbar::-webkit-scrollbar {
  width: 3px;
}

.aux-scrollbar::-webkit-scrollbar-track {
  background: #1f2937;
  border-radius: 2px;
}

.aux-scrollbar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 2px;
  transition: background 0.2s;
}

.aux-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Firefox scrollbar */
.aux-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}
</style>

