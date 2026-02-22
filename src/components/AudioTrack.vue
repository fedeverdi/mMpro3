<template>
  <div
    :class="[
      'track-channel relative bg-gray-800 rounded-lg border p-1 flex flex-col items-center gap-1 h-full',
      isRecording ? 'border-red-500 border-2' : 'border-gray-700'
    ]">
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
      <div class="w-full flex items-center justify-between gap-1">
        <div class="flex items-center gap-1 flex-1 justify-center">
          <div class="text-xs font-bold text-gray-300">Track {{ trackNumber }}</div>
          <div v-if="isRecording" 
            class="px-1 py-0.5 bg-red-600 text-white text-[0.5rem] font-bold rounded animate-pulse">
            REC
          </div>
        </div>
        <button @click="$emit('remove')"
          class="w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
          title="Remove Track">
          ×
        </button>
      </div>

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

    <!-- Normal Track Content -->
    <template v-if="!showAuxPanel">
      <!-- Waveform Display -->
      <WaveformDisplay ref="waveformDisplayRef" :waveform-node="waveform" :audio-buffer="currentAudioBuffer"
        :is-playing="isPlaying" :current-time="currentPlaybackTime" />

      <!-- Gain Control -->
      <div class="w-full flex items-center justify-center gap-2 h-[4rem]">
        <div class="flex flex-col gap-1 items-center justify-center pt-1">
          <PadButton v-model="padEnabled" />
          <HPFButton v-model="hpfEnabled" />
        </div>
        <div class="scale-[0.65]">
          <Knob v-model="gain" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Gain" unit="dB"
            color="#8b5cf6" />
        </div>
      </div>

      <!-- Gate Section -->
      <div class="w-full bg-gray-900 rounded p-1 border border-gray-700 grid grid-cols-2 gap-1">
        <TrackGate ref="trackGateRef" :track-number="trackNumber" :enabled="gateEnabled" :gate-node="gate"
          :meter="meterL" @toggle="toggleGate" @update-params="handleGateParamsUpdate" />
        <TrackCompressor ref="trackCompressorRef" :track-number="trackNumber" :enabled="compressorEnabled"
          :compressor-node="compressor" :meter="meterL" @toggle="toggleCompressor" />
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
        <EQThumbnail :filters="eqFiltersData" :system-filters="systemFilters" />

        <!-- 3-Band EQ Knobs (Accordion) -->
        <TrackEQ ref="trackEQRef" :eq3Node="eq3" :show="showEQ3Bands" />
      </div>

      <!-- Aux Sends Button -->
      <TrackAuxSends ref="auxSendsRef" :track-number="trackNumber" :aux-buses="auxBuses" :aux-sends-data="auxSendsData"
        @update-sends="handleAuxSendsUpdate" @toggle-panel="showAuxPanel = !showAuxPanel" />

      <!-- Mute & Solo Buttons -->
      <div class="flex flex-row gap-1 w-full">
        <button @click="toggleMute" class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all"
          :class="isMuted ? 'bg-red-600 text-white animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
          MUTE
        </button>
        <button @click="toggleSolo" class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all"
          :class="isSolo ? 'bg-yellow-500 text-gray-900 animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
          SOLO
        </button>
      </div>

      <!-- Aux Panel Content -->
      <div class="flex justify-center scale-[0.75]">
        <PanKnob class="" v-model="pan" label="Pan" />
      </div>

      <!-- Volume Fader and VU Meter -->
      <div class="flex flex-col h-full">
        <div class="text-[0.455rem] uppercase text-center">Volume</div>
        <div ref="faderContainer" class="flex-1 relative flex items-center justify-center gap-1 min-h-0">
          <!-- ARM Button -->
          <button @click="emit('toggle-arm')" 
            class="absolute -left-[1.9rem] bottom-[0.3rem] w-5 h-5 text-[0.4rem] font-bold rounded transition-all border"
            :class="isArmed 
              ? 'bg-red-700 border-red-500 text-white shadow-md shadow-red-500/50' 
              : 'bg-gray-800 border-gray-600 text-gray-400 hover:bg-gray-700 hover:border-gray-500'"
            title="Arm track for automation recording">
            A
          </button>
          
          <!-- Routing Buttons -->
          <div class="flex flex-col gap-4 absolute -left-[1.7rem] top-1/2 transform -translate-y-1/2 z-50">
            <button @click="toggleRouteToMaster" :title="'Route to Master'"
              class="w-5 h-7 text-[8px] font-bold rounded transition-all flex items-center justify-center"
              :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
              M
            </button>
            <button v-for="subgroup in props.subgroups" :key="subgroup.id" @click="toggleRouteToSubgroup(subgroup.id)"
              :title="`Route to ${subgroup.name}`"
              class="w-5 h-7 text-[6px] font-bold rounded transition-all flex items-center justify-center"
              :class="routedSubgroups.has(subgroup.id) ? 'bg-orange-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
              S{{ subgroup.id }}
            </button>
          </div>
          <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />
          
          <!-- Phase Correlation LED (stereo only) - Above VU meters -->
          <div v-if="isStereo && faderHeight > 0" class="absolute -right-[1.8rem] -top-7 z-50 flex flex-col items-center gap-0.5">
            <button
              @click="showPhaseModal = true"
              :class="[
                'w-3 h-3 rounded-full transition-all shadow-lg',
                phaseCorrelation < -0.2 ? 'bg-red-500 shadow-red-500/50 animate-pulse' :
                phaseCorrelation < 0.2 ? 'bg-yellow-500 shadow-yellow-500/50' :
                phaseCorrelation < 0.85 ? 'bg-green-500 shadow-green-500/50' :
                'bg-blue-500 shadow-blue-500/50'
              ]"
              title="Phase Correlation - Click for details"
            >
            </button>
            <span class="text-[0.35rem] text-gray-400 font-bold uppercase tracking-tighter">Phase</span>
          </div>
          
          <TrackMeter class="absolute -right-[1.6rem] top-1/2 transform -translate-y-1/2 z-50 -mt-3"
            v-if="faderHeight > 0" :levelL="trackLevelL" :levelR="trackLevelR" :isStereo="isStereo"
            :height="faderHeight + 20" />
        </div>
      </div>
    </template>

    <!-- Aux Panel Content -->
    <div v-else class="w-full flex-1 overflow-y-auto flex flex-col gap-2 aux-panel-scrollbar">
      <!-- Header -->
      <div class="sticky top-0 bg-gray-800 border-b border-teal-600/30 px-2 py-1 flex justify-between items-center">
        <span class="text-[0.65rem] font-bold text-teal-300">AUX SENDS</span>
        <button @click="showAuxPanel = false"
          class="w-4 h-4 pb-[0.08rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
          title="Close AUX Panel">
          ×
        </button>
      </div>

      <div class="px-2">
        <div v-if="auxBuses.length === 0" class="text-center py-4 text-gray-500 text-[0.6rem]">
          No aux buses
        </div>

        <div v-else class="flex flex-col gap-2">
          <AuxSendControl v-for="aux in auxBuses" :key="aux.id" :aux="aux" :aux-send-data="auxSendsData[aux.id]"
            @update-level="(val) => updateLocalAuxSend(aux.id, 'level', val)"
            @toggle-pre-post="toggleLocalPrePost(aux.id)" @toggle-mute="toggleLocalMute(aux.id)" />
        </div>
      </div>
    </div>

  </div>

  <!-- Parametric EQ Modal -->
  <ParametricEQModal v-model="showParametricEQ" :trackNumber="trackNumber" :eq-filters="eqFiltersData"
    :system-filters="systemFilters" @update="handleParametricEQUpdate" />

  <!-- Phase Correlation Modal -->
  <PhaseCorrelationModal 
    v-model="showPhaseModal" 
    :correlation="phaseCorrelation" 
    :track-number="trackNumber" 
  />
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed, toRaw, inject } from 'vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'
import TrackFader from './audioTrack/TrackFader.vue'
import TrackMeter from './audioTrack/TrackMeter.vue'
import PhaseCorrelationMeter from './audioTrack/PhaseCorrelationMeter.vue'
import PhaseCorrelationModal from './audioTrack/PhaseCorrelationModal.vue'
import Knob from './core/Knob.vue'
import PanKnob from './audioTrack/PanKnob.vue'
import ParametricEQModal from './master/ParametricEQModal.vue'
import TrackCompressor from './audioTrack/TrackCompressor.vue'
import TrackGate from './audioTrack/TrackGate.vue'
import TrackEQ from './audioTrack/TrackEQ.vue'
import EQThumbnail from './audioTrack/EQThumbnail.vue'
import WaveformDisplay from './audioTrack/WaveformDisplay.vue'
import TrackAuxSends from './audioTrack/TrackAuxSends.vue'
import AuxSendControl from './audioTrack/AuxSendControl.vue'
import PadButton from './audioTrack/PadButton.vue'
import HPFButton from './audioTrack/HPFButton.vue'

defineOptions({
  inheritAttrs: false
})

// Inject Tone.js from App.vue (imported once for entire app)
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Inject automation system
const automation = inject<any>('automation', null)

const { saveAudioFile, getAudioFile } = useAudioFileStorage()

interface Props {
  trackNumber: number
  masterChannel?: any
  subgroups?: Array<{ id: number, name: string, channel: any, ref: any }>
  auxBuses?: Array<{ id: string, name: string, volume: number, muted: boolean, soloed: boolean, routeToMaster: boolean, node?: any }>
  isArmed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  subgroups: () => [],
  auxBuses: () => [],
  isArmed: false
})

const emit = defineEmits<{
  (e: 'soloChange', value: { trackNumber: number, isSolo: boolean }): void
  (e: 'levelUpdate', value: { trackNumber: number, level: number }): void
  (e: 'remove'): void
  (e: 'toggle-arm'): void
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
const showAuxPanel = ref(false)
const showPhaseModal = ref(false)
const auxSendsRef = ref<any>(null)
const waveformDisplayRef = ref<any>(null)

// Local aux sends state
const auxSendsData = ref<Record<string, { level: number, preFader: boolean, muted: boolean }>>({})

// Audio source selection
const audioSourceType = ref<'file' | 'input'>('file')

// Output routing selection (can route to both simultaneously)
const routeToMaster = ref(true)
const routedSubgroups = ref<Set<number>>(new Set()) // Set of subgroup IDs

// Audio inputs from shared composable
const { audioInputDevices, refreshAudioInputs } = useAudioDevices()
const audioInputs = audioInputDevices // Use the shared ref

const selectedAudioInput = ref<string>('')
let audioInputStream: MediaStream | null = null
let audioInputSource: MediaStreamAudioSourceNode | null = null

// FX state
const compressorEnabled = ref(false)
const gateEnabled = ref(false)

// Store EQ filters data for thumbnail
const eqFiltersData = ref<any[]>([])

// System filters (HPF) - separate from user filters, only for visualization
const systemFilters = computed(() => {
  const filters: any[] = []

  // Add HPF as a system filter when enabled
  if (hpfEnabled.value) {
    filters.push({
      id: 'hpf-system',
      type: 'highpass',
      frequency: 80,
      gain: 0,
      Q: 0.707,
      color: '#3b82f6',
      isSystem: true
    })
  }

  return filters
})

// EQ accordion state
const showEQ3Bands = ref(false)

// Audio controls
const volume = ref(0)
const gain = ref(0)
const padEnabled = ref(false) // PAD: -20dB attenuation
const hpfEnabled = ref(false) // HPF: 80Hz highpass filter
const pan = ref(0) // -1 (left) to +1 (right)
const isStereo = ref(true) // Track if source is stereo or mono (default stereo)
const trackLevelL = ref(-60) // Left/Mono level
const trackLevelR = ref(-60) // Right level (only for stereo)
const phaseCorrelation = ref(0) // Phase correlation: -1 (out of phase) to +1 (mono)

// Analyser nodes for phase correlation measurement
let analyserL: any = null
let analyserR: any = null
const analyserBufferSize = 2048
const analyserDataL = new Float32Array(analyserBufferSize)
const analyserDataR = new Float32Array(analyserBufferSize)

// Automation recording - track last recorded values to avoid redundant points
const lastRecordedVolume = ref<number | null>(null)
const lastRecordedPan = ref<number | null>(null)

// Check if this track is currently recording automation
const isRecording = computed(() => {
  if (!automation || !automation.isRecording.value) return false
  if (!automation.transport.value.isPlaying) return false
  if (!props.isArmed) return false // Only show REC if track is armed
  
  // Check if any lane for this track is in write mode
  return automation.automationLanes.value.some((lane: any) => 
    lane.trackId === props.trackNumber && 
    (lane.mode === 'write' || lane.mode === 'touch' || lane.mode === 'latch')
  )
})
const currentPlaybackTime = ref(0)

// Component refs
const trackEQRef = ref<InstanceType<typeof TrackEQ> | null>(null)
const trackCompressorRef = ref<InstanceType<typeof TrackCompressor> | null>(null)
const trackGateRef = ref<InstanceType<typeof TrackGate> | null>(null)

// Tone.js nodes
let player: any = null // Can be Tone.Player or Tone.UserMedia
let currentAudioBuffer: AudioBuffer | null = null // Store current audio buffer for player recreation
let padNode: any = null // PAD attenuation (-26dB when enabled) - PRE-GAIN
let gainNode: any = null
let hpfNode: any = null // High-pass filter at 80Hz
let gate: any = null // Custom noise gate (Gain node)
let eq3: any = null

// Gate parameters and state
let gateThreshold = -45 // dB
let gateAttack = 0.005 // seconds (5ms)
let gateRelease = 0.3 // seconds (300ms)
let gateRange = -30 // dB attenuation when closed
let gateIsOpen = false
let gateMonitoringId: number | null = null
let parametricEQFilters: any = null // Parametric EQ filter chain
let compressor: any = null
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
let gateMeter: any = null // Pre-gate meter for gate monitoring
let channelSplit: any = null // Split stereo to L/R for metering
let waveform: any = null // Waveform analyzer
let resizeObserver: ResizeObserver | null = null
let playbackTimeInterval: number | null = null
let playbackStartTime: number = 0
let audioContextStarted: boolean = false // Track if Tone.start() has been called

// Aux sends: Map of aux bus ID to { node, preFader }
const auxSendNodes = new Map<string, { node: any, preFader: boolean }>()

// Initialize aux sends data when aux buses change
watch(() => props.auxBuses, (newBuses) => {
  newBuses.forEach(aux => {
    if (!auxSendsData.value[aux.id]) {
      auxSendsData.value[aux.id] = {
        level: -60,
        preFader: false,
        muted: true
      }
    }
  })
}, { immediate: true, deep: true })

// Update aux sends when track mute changes
watch(isMuted, () => {
  // Re-apply aux sends to update gain based on mute state
  handleAuxSendsUpdate(auxSendsData.value)
})

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
  // PAD node (pre-gain attenuation)
  padNode = new Tone.Gain(1) // 1 = 0dB (no pad), will be set to lower value when enabled

  gainNode = new Tone.Gain(1) // 1 = 0dB (unity gain), not 0!

  // High-pass filter at 80Hz (bypassed when disabled)
  hpfNode = new Tone.Filter({
    type: 'highpass',
    frequency: 80,
    rolloff: -24 // 24dB/octave
  })

  // Custom noise gate using Gain node for proper attack/release/range control
  gate = new Tone.Gain(1) // Start at unity gain

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

  // Create analyser nodes for phase correlation measurement
  if (Tone.context) {
    analyserL = Tone.context.createAnalyser()
    analyserL.fftSize = analyserBufferSize
    analyserL.smoothingTimeConstant = 0 // No smoothing for accurate readings
    
    analyserR = Tone.context.createAnalyser()
    analyserR.fftSize = analyserBufferSize
    analyserR.smoothingTimeConstant = 0 // No smoothing for accurate readings
  }

  // Create meter for gate monitoring (pre-gate, post-gain)
  gateMeter = new Tone.Meter()

  // Create meter for gate monitoring (pre-gate, post-gain)
  gateMeter = new Tone.Meter()

  // Waveform analyzer (for visualization)
  waveform = new Tone.Waveform(512) // 512 samples for waveform display

  // Create FX nodes once (always present, bypassed when disabled)
  compressor = new Tone.Compressor({
    threshold: 0,     // Bypassed: 0dB threshold = no compression
    ratio: 1,         // Bypassed: 1:1 ratio = no compression
    attack: 0.1,
    release: 0.25
  })

  // Connect chain: pad -> hpf -> gain -> gate -> eq3 -> balance -> volume
  // PAD is pre-gain (professional mixer architecture)
  // HPF is pre-gain (removes rumble before amplification)
  // Gate is post-gain (eliminates noise from properly amplified signal)
  // Compressor is bypassed by default (not in chain)
  // HPF starts bypassed (direct connection pad -> gain)
  padNode.connect(gainNode) // Bypass HPF by default
  // hpfNode connections will be managed by updateHPF()
  // Gate starts bypassed (direct connection gain -> eq3)
  gainNode.connect(eq3) // Bypass gate by default
  // gate connections will be managed by toggleGate()
  // Connect gate meter to gainNode for pre-gate monitoring
  gainNode.connect(gateMeter)

  // Connect stereo metering to eq3 (before effects)
  eq3.connect(channelSplit)
  channelSplit.connect(meterL, 0) // Left channel to meterL
  channelSplit.connect(meterR, 1) // Right channel to meterR

  // Connect analyser nodes for phase correlation
  if (analyserL && analyserR) {
    channelSplit.connect(analyserL as any, 0) // Left channel to analyserL
    channelSplit.connect(analyserR as any, 1) // Right channel to analyserR
  }

  // Connect waveform analyzer to eq3 for visualization
  eq3.connect(waveform)

  // Initial FX chain: eq3 -> balanceSplit (dry path, compressor bypassed)
  // NOTE: Compressor is NOT connected by default to preserve stereo
  // Use toggleCompressor() to enable it
  eq3.connect(balanceSplit)

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

  // Restore aux sends if they were configured before nodes were created
  // (happens when restoring from scene)
  if (Object.keys(auxSendsData.value).length > 0) {
    nextTick(() => {
      handleAuxSendsUpdate(auxSendsData.value)
    })
  }
}


// Handle parametric EQ update
function handleParametricEQUpdate(filters: any) {
  if (!filters) return

  // Store the latest filter chain
  const previousFilters = parametricEQFilters
  parametricEQFilters = filters

  // Store filter data for thumbnail
  if (filters.filtersData) {
    // Convert Vue reactive proxy to raw array
    const rawFiltersData = toRaw(filters.filtersData)

    // Map to plain objects for storage
    eqFiltersData.value = rawFiltersData.map((f: any) => ({
      id: f.id,
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      Q: f.Q,
      color: f.color
    }))
  }

  // Only reconnect the audio chain if the filter chain structure changed
  // (e.g., filters added/removed, or first time initialization)
  // If it's just parameter updates (dragging), the nodes are already connected
  const shouldReconnect = !previousFilters ||
    !previousFilters.input ||
    !previousFilters.output ||
    previousFilters.input !== filters.input ||
    previousFilters.output !== filters.output

  if (shouldReconnect) {
    applyParametricEQ()
  }
}

// Insert or remove parametric EQ from the chain with minimal disconnections
function applyParametricEQ() {
  if (!eq3) return

  // Disconnect only eq3 (meters and waveform stay connected)
  try {
    eq3.disconnect()
  } catch (e) {
    // Ignore disconnection errors
  }

  // Reconnect meters and waveform to eq3
  if (channelSplit) eq3.connect(channelSplit)
  if (waveform) eq3.connect(waveform)

  // Determine next node in chain (compressor if enabled, balanceSplit if not)
  const nextNode = (compressorEnabled.value && compressor) ? compressor : balanceSplit

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

// Connect to output (can connect to master and/or multiple subgroups)
function connectToOutput() {
  if (!volumeMerge || !Tone) return false

  // Disconnect only from master and subgroups (preserve aux sends)
  if (props.masterChannel) {
    try {
      volumeMerge.disconnect(toRaw(props.masterChannel))
    } catch (e) {
      // Ignore if not connected
    }
  }

  // Disconnect from all subgroups
  props.subgroups?.forEach(subgroup => {
    if (subgroup?.channel) {
      try {
        volumeMerge.disconnect(toRaw(subgroup.channel))
      } catch (e) {
        // Ignore if not connected
      }
    }
  })

  // Connect to master if enabled
  if (routeToMaster.value && props.masterChannel) {
    volumeMerge.connect(toRaw(props.masterChannel))
  }

  // Connect to each enabled subgroup
  routedSubgroups.value.forEach(subgroupId => {
    const subgroup = props.subgroups?.find(s => s.id === subgroupId)
    if (subgroup?.channel) {
      volumeMerge.connect(toRaw(subgroup.channel))
    }
  })

  // Warn if no output is selected
  if (!routeToMaster.value && routedSubgroups.value.size === 0) {
    console.warn(`[Track ${props.trackNumber}] No output destination selected`)
  }
}

// Toggle routing buttons
function toggleRouteToMaster() {
  routeToMaster.value = !routeToMaster.value
  connectToOutput()
}

function toggleRouteToSubgroup(subgroupId: number) {
  if (routedSubgroups.value.has(subgroupId)) {
    routedSubgroups.value.delete(subgroupId)
  } else {
    routedSubgroups.value.add(subgroupId)
  }
  routedSubgroups.value = new Set(routedSubgroups.value) // Trigger reactivity
  connectToOutput()
}

// Disconnect from a specific subgroup (called when subgroup is removed)
function disconnectFromSubgroup(subgroupId: number) {
  if (routedSubgroups.value.has(subgroupId)) {
    routedSubgroups.value.delete(subgroupId)
    routedSubgroups.value = new Set(routedSubgroups.value)
    connectToOutput()
  }
}

// Local aux send control functions
function updateLocalAuxSend(auxId: string, property: 'level', value: number) {
  if (!auxSendsData.value[auxId]) {
    auxSendsData.value[auxId] = {
      level: -60,
      preFader: false,
      muted: true
    }
  }

  auxSendsData.value[auxId].level = value

  // If level is increased from minimum, unmute automatically
  if (value > -60 && auxSendsData.value[auxId].muted) {
    auxSendsData.value[auxId].muted = false
  }

  // Trigger audio routing update
  handleAuxSendsUpdate(auxSendsData.value)
}

function toggleLocalPrePost(auxId: string) {
  if (!auxSendsData.value[auxId]) return

  auxSendsData.value[auxId].preFader = !auxSendsData.value[auxId].preFader
  handleAuxSendsUpdate(auxSendsData.value)
}

function toggleLocalMute(auxId: string) {
  if (!auxSendsData.value[auxId]) return

  auxSendsData.value[auxId].muted = !auxSendsData.value[auxId].muted
  handleAuxSendsUpdate(auxSendsData.value)
}

// Handle aux sends update
function handleAuxSendsUpdate(sends: Record<string, any>) {
  if (!Tone || !volumeMerge || !balanceMerge) return

  // Get all aux IDs from the sends object
  const sendIds = Object.keys(sends)

  // Remove sends that are no longer in the update or are inactive
  auxSendNodes.forEach((sendInfo, auxId) => {
    const send = sends[auxId]
    const isActive = send && send.level > -60 && !send.muted

    if (!send || !isActive) {
      // Disconnect and dispose the send node
      try {
        sendInfo.node.disconnect()
        sendInfo.node.dispose()
      } catch (e) {
        console.warn('Error disposing aux send node:', e)
      }
      auxSendNodes.delete(auxId)
    }
  })

  // Update or create active sends
  sendIds.forEach(auxId => {
    const send = sends[auxId]
    const isActive = send && send.level > -60 && !send.muted

    if (!isActive) return

    // Find the corresponding aux bus
    const auxBus = props.auxBuses?.find(bus => bus.id === auxId)
    if (!auxBus || !auxBus.node) {
      console.warn(`Aux bus ${auxId} not found or has no node`)
      return
    }

    // Get existing send info
    let sendInfo = auxSendNodes.get(auxId)
    let sendNode: any
    let needsNewNode = false

    // Check if preFader changed - need to recreate node to avoid double connections
    if (sendInfo && sendInfo.preFader !== send.preFader) {
      // Dispose old node completely
      try {
        sendInfo.node.disconnect()
        sendInfo.node.dispose()
      } catch (e) {
        console.warn('Error disposing old send node:', e)
      }
      auxSendNodes.delete(auxId)
      sendInfo = undefined
      needsNewNode = true
    }

    // Create new node if needed
    if (!sendInfo) {
      sendNode = new Tone.Gain(0)
      sendInfo = { node: sendNode, preFader: send.preFader }
      auxSendNodes.set(auxId, sendInfo)
      needsNewNode = true
    } else {
      sendNode = sendInfo.node
    }

    // Update send level (convert dB to gain)
    // If track is muted, force send to 0 regardless of send level
    const gainValue = isMuted.value ? 0 : Tone.dbToGain(send.level)
    sendNode.gain.value = gainValue

    // Connect new nodes
    if (needsNewNode) {
      try {
        // Choose source based on pre/post fader
        const source = send.preFader ? balanceMerge : volumeMerge
        source.connect(sendNode)
        sendNode.connect(toRaw(auxBus.node))

      } catch (e) {
        console.error('Error connecting aux send:', e)
      }
    }
  })
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
        
        // Calculate phase correlation for stereo tracks
        if (analyserL && analyserR) {
          // Get time-domain data from both channels
          analyserL.getFloatTimeDomainData(analyserDataL)
          analyserR.getFloatTimeDomainData(analyserDataR)
          
          // Calculate correlation: correlation = sum(L*R) / sqrt(sum(L²) * sum(R²))
          let sumLR = 0
          let sumL2 = 0
          let sumR2 = 0
          
          for (let i = 0; i < analyserBufferSize; i++) {
            const l = analyserDataL[i]
            const r = analyserDataR[i]
            sumLR += l * r
            sumL2 += l * l
            sumR2 += r * r
          }
          
          // Prevent division by zero and check for valid signal
          const denominator = Math.sqrt(sumL2 * sumR2)
          if (denominator > 0.00001) {
            const newCorrelation = sumLR / denominator
            // Smooth the correlation value slightly to reduce jitter
            phaseCorrelation.value = phaseCorrelation.value * 0.7 + newCorrelation * 0.3
          } else {
            // Silence: default to neutral correlation
            phaseCorrelation.value = 0
          }
        }
      } else {
        // Mono: copy left to right for consistency
        trackLevelR.value = trackLevelL.value
        phaseCorrelation.value = 1 // Mono is always perfectly correlated
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
      player.connect(padNode)
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
async function loadFileFromIndexedDB(savedFileId: string, silent: boolean = false) {
  if (!Tone) {
    console.error('Tone.js not loaded')
    return
  }

  // Initialize audio nodes on first use
  initAudioNodes()

  // Don't show spinner when restoring from scene (silent mode)
  if (!silent) {
    isLoading.value = true
  }

  try {
    // Retrieve file from IndexedDB
    const storedFile = await getAudioFile(savedFileId)

    if (!storedFile) {
      console.error('File not found in IndexedDB')
      alert('Could not restore audio file from scene. File may have been deleted.')
      if (!silent) {
        isLoading.value = false
      }
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
      player.connect(padNode)
    }

    // Update current buffer reference for waveform
    currentAudioBuffer = audioBuffer

    // Detect if stereo or mono
    isStereo.value = audioBuffer.numberOfChannels === 2

    // Verify audio chain is connected
    if (!gainNode || !eq3 || !volumeMerge) {
      alert('Audio system not ready. Please refresh the page.')
      if (!silent) {
        isLoading.value = false
      }
      return
    }

    audioLoaded.value = true
    if (!silent) {
      isLoading.value = false
    }

    // Force DOM update
    await nextTick()
  } catch (error) {
    console.error('❌ Error loading audio file from IndexedDB:', error)
    alert('Error loading audio file from scene: ' + error)
    if (!silent) {
      isLoading.value = false
    }
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
      player.connect(padNode!)
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
      player.connect(padNode)
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
    volumeNodeL.gain.rampTo(0, 0.01)  // Mute with short ramp
    volumeNodeR.gain.rampTo(0, 0.01)
  } else {
    // Convert dB to linear gain
    const gainValue = Tone.dbToGain(volume.value)
    volumeNodeL.gain.rampTo(gainValue, 0.01)
    volumeNodeR.gain.rampTo(gainValue, 0.01)
  }
}

function updateGain() {
  if (!gainNode || !Tone) return
  gainNode.gain.rampTo(Tone.dbToGain(gain.value), 0.01)
}

// Update PAD state (pre-gain attenuation)
function updatePad() {
  if (!padNode || !Tone) return
  // PAD: -26dB attenuation when enabled, 0dB (unity) when disabled
  padNode.gain.rampTo(padEnabled.value ? Tone.dbToGain(-26) : 1, 0.01)
}

// Update HPF state
function updateHPF() {
  if (!hpfNode || !padNode || !gainNode) return

  // Safer approach: use disconnect() without arguments to disconnect all
  // Then rebuild the chain
  try {
    padNode.disconnect()
    hpfNode.disconnect()
  } catch (e) { }

  if (hpfEnabled.value) {
    // Enable HPF: pad -> hpf -> gain
    padNode.connect(hpfNode)
    hpfNode.connect(gainNode)
  } else {
    // Disable HPF: pad -> gain (direct bypass)
    padNode.connect(gainNode)
  }
}

// Update pan value (constant power panning for stereo preservation)
function updatePan() {
  if (!balanceLeft || !balanceRight || !Tone) return

  // Constant power panning formula
  // Pan: -1 (left) to +1 (right)
  const panRadians = (pan.value * Math.PI) / 4  // Map -1..+1 to -π/4..+π/4

  balanceLeft.gain.rampTo(Math.cos(panRadians + Math.PI / 4), 0.01)
  balanceRight.gain.rampTo(Math.sin(panRadians + Math.PI / 4), 0.01)
}

// Watch for parameter changes
watch(volume, updateVolume)
watch(gain, updateGain)
watch(padEnabled, updatePad)
watch(hpfEnabled, updateHPF)
watch(pan, updatePan)

// Automation recording - watch for parameter changes
watch(volume, (newValue) => {
  if (!automation || !automation.transport.value.isPlaying) return
  
  // Check if there's a volume lane for this track in write/touch/latch mode
  const volumeLane = automation.automationLanes.value.find((lane: any) => 
    lane.trackId === props.trackNumber && 
    lane.parameter === 'volume' &&
    (lane.mode === 'write' || lane.mode === 'touch' || lane.mode === 'latch')
  )
  
  if (!volumeLane) return
  
  // Only record if value changed significantly (>= 0.1 dB)
  if (lastRecordedVolume.value === null || Math.abs(newValue - lastRecordedVolume.value) >= 0.1) {
    const currentTime = automation.transport.value.currentTime
    automation.addPoint(props.trackNumber, 'volume', currentTime, newValue)
    lastRecordedVolume.value = newValue
  }
})

watch(pan, (newValue) => {
  if (!automation || !automation.transport.value.isPlaying) return
  
  // Check if there's a pan lane for this track in write/touch/latch mode
  const panLane = automation.automationLanes.value.find((lane: any) => 
    lane.trackId === props.trackNumber && 
    lane.parameter === 'pan' &&
    (lane.mode === 'write' || lane.mode === 'touch' || lane.mode === 'latch')
  )
  
  if (!panLane) return
  
  // Only record if value changed significantly (>= 0.01)
  if (lastRecordedPan.value === null || Math.abs(newValue - lastRecordedPan.value) >= 0.01) {
    const currentTime = automation.transport.value.currentTime
    automation.addPoint(props.trackNumber, 'pan', currentTime, newValue)
    lastRecordedPan.value = newValue
  }
})

// Expose methods for external control
defineExpose({
  setMuted: (muted: boolean) => {
    isMuted.value = muted
    updateVolume()
  },
  isSolo: () => isSolo.value,
  disconnectFromSubgroup, // Expose for cleanup when subgroup is removed
  
  // Automation control - set values without triggering recording
  setVolume: (value: number, skipRecording = false) => {
    if (skipRecording) {
      // Temporarily update last recorded value to prevent recording
      lastRecordedVolume.value = value
    }
    volume.value = value
  },
  
  setPan: (value: number, skipRecording = false) => {
    if (skipRecording) {
      // Temporarily update last recorded value to prevent recording
      lastRecordedPan.value = value
    }
    pan.value = value
  },

  getSnapshot: () => {
    return {
      trackNumber: props.trackNumber,
      volume: volume.value,
      gain: gain.value,
      padEnabled: padEnabled.value,
      hpfEnabled: hpfEnabled.value,
      pan: pan.value,
      muted: isMuted.value,
      soloed: isSolo.value,
      routeToMaster: routeToMaster.value,
      routedSubgroups: Array.from(routedSubgroups.value), // Convert Set to Array for serialization
      sourceType: audioSourceType.value,
      selectedInputDevice: audioSourceType.value === 'input' ? selectedAudioInput.value : undefined,
      fileName: audioSourceType.value === 'file' ? fileName.value : undefined,
      fileId: audioSourceType.value === 'file' ? fileId.value : undefined,
      isPlaying: isPlaying.value,
      currentTime: currentPlaybackTime.value,
      duration: currentAudioBuffer?.duration || 0,
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
      gateEnabled: gateEnabled.value,
      gate: trackGateRef.value?.getParams(),
      auxSends: { ...auxSendsData.value }
    }
  },

  restoreFromSnapshot: (snapshot: any) => {
    // Initialize audio nodes if not already done
    initAudioNodes()

    // Restore volume, gain, pad, hpf, and pan
    volume.value = snapshot.volume
    if (snapshot.gain !== undefined) {
      gain.value = snapshot.gain
    }
    if (snapshot.padEnabled !== undefined) {
      padEnabled.value = snapshot.padEnabled
    }
    if (snapshot.hpfEnabled !== undefined) {
      hpfEnabled.value = snapshot.hpfEnabled
    }
    pan.value = snapshot.pan
    isMuted.value = snapshot.muted
    isSolo.value = snapshot.soloed

    // Ensure PAD and HPF audio nodes are properly connected after restore
    nextTick(() => {
      updatePad()
      updateHPF()
    })

    // Restore output routing
    if (snapshot.routeToMaster !== undefined) {
      routeToMaster.value = snapshot.routeToMaster
    }

    // Restore routed subgroups (support both old and new format)
    if (snapshot.routedSubgroups && Array.isArray(snapshot.routedSubgroups)) {
      routedSubgroups.value = new Set(snapshot.routedSubgroups)
    } else if (snapshot.routeToSubgroup) {
      // Legacy format - assume routing to first subgroup if exists
      if (props.subgroups && props.subgroups.length > 0) {
        routedSubgroups.value = new Set([props.subgroups[0].id])
      }
    }

    // Reconnect to correct destination(s)
    nextTick(() => {
      connectToOutput()
    })

    // Restore aux sends
    if (snapshot.auxSends) {
      auxSendsData.value = { ...snapshot.auxSends }
      // Trigger aux sends update to reconnect nodes
      nextTick(() => {
        handleAuxSendsUpdate(auxSendsData.value)
      })
    }

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
        // silent=true to avoid showing spinner during scene animation
        await loadFileFromIndexedDB(snapshot.fileId!, true)
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

    // Restore gate
    const shouldEnableGate = snapshot.gateEnabled || false
    if (snapshot.gate) {
      trackGateRef.value?.setParams(snapshot.gate)
    }
    if (shouldEnableGate !== gateEnabled.value) {
      toggleGate()
    }
  },

  resetToDefaults: () => {
    // Reset volume and pan
    volume.value = 0
    pan.value = 0
    isMuted.value = false
    isSolo.value = false

    // Reset routing
    routeToMaster.value = true
    routedSubgroups.value = new Set()

    // Reset audio source
    audioSourceType.value = 'file'
    selectedAudioInput.value = ''
    fileName.value = ''
    fileId.value = ''
    audioLoaded.value = false

    // Stop player if active
    if (player && typeof player.stop === 'function') {
      try {
        player.stop()
      } catch (e) { }
    }

    // Reset 3-band EQ to defaults
    if (trackEQRef.value?.setParams) {
      trackEQRef.value.setParams({
        low: 0,
        mid: 0,
        high: 0
      })
    }

    // Clear parametric EQ
    eqFiltersData.value = []
    if (parametricEQFilters) {
      try {
        parametricEQFilters.input?.disconnect()
        parametricEQFilters.output?.disconnect()
      } catch (e) { }
      parametricEQFilters = null
    }

    // Disable and reset compressor
    if (compressorEnabled.value) {
      toggleCompressor()
    }
    if (trackCompressorRef.value?.setParams) {
      trackCompressorRef.value.setParams({
        threshold: -24,
        ratio: 4,
        attack: 0.003,
        release: 0.25
      })
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
  if (gateMeter) gateMeter.dispose()
  if (channelSplit) channelSplit.dispose()

  // Stop gate monitoring
  stopGateMonitoring()

  // Cleanup aux send nodes
  auxSendNodes.forEach((sendInfo) => {
    try {
      sendInfo.node.disconnect()
      sendInfo.node.dispose()
    } catch (e) { }
  })
  auxSendNodes.clear()
  if (waveform) waveform.dispose()
  if (compressor) compressor.dispose()

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

  if (!eq3 || !compressor) return

  // Disconnect eq3 from everything
  try {
    eq3.disconnect(compressor)
  } catch (e) { }
  try {
    eq3.disconnect(balanceSplit)
  } catch (e) { }

  // Disconnect compressor from balanceSplit
  try {
    compressor.disconnect(balanceSplit)
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

    // Chain: eq3 → compressor → balanceSplit
    eq3.connect(compressor)
    compressor.connect(balanceSplit)
  } else {
    // Bypass compressor completely: eq3 → balanceSplit (skip compressor)
    eq3.connect(balanceSplit)
  }
}

function toggleGate() {
  gateEnabled.value = !gateEnabled.value

  if (!gainNode || !gate || !eq3) return

  // Disconnect gainNode from eq3 and gate
  try {
    gainNode.disconnect(eq3)
  } catch (e) { }
  try {
    gainNode.disconnect(gate)
  } catch (e) { }

  // Disconnect gate from eq3
  try {
    gate.disconnect(eq3)
  } catch (e) { }

  if (gateEnabled.value) {
    // Apply real parameters from component
    const params = trackGateRef.value?.getParams() || {
      threshold: -45,
      attack: 0.005,
      release: 0.3,
      range: -30
    }
    gateThreshold = params.threshold
    gateAttack = params.attack
    gateRelease = params.release
    gateRange = params.range

    // Reset gate to open state
    gate.gain.value = 1
    gateIsOpen = true

    // Chain: gainNode → gate → eq3
    gainNode.connect(gate)
    gate.connect(eq3)

    // Ensure gateMeter stays connected for monitoring
    try {
      gainNode.connect(gateMeter)
    } catch (e) { }

    // Start gate monitoring
    startGateMonitoring()
  } else {
    // Stop gate monitoring
    stopGateMonitoring()

    // Bypass gate: gainNode → eq3 (skip gate)
    gainNode.connect(eq3)

    // Ensure gateMeter stays connected for monitoring
    try {
      gainNode.connect(gateMeter)
    } catch (e) { }
  }
}

function handleGateParamsUpdate(params: { threshold: number, attack: number, release: number, range: number }) {
  if (!gate) return

  // Update custom gate parameters
  gateThreshold = params.threshold
  gateAttack = params.attack
  gateRelease = params.release
  gateRange = params.range
}

// Custom gate monitoring functions
function startGateMonitoring() {
  if (gateMonitoringId || !gateMeter || !gate) return

  function updateGate() {
    if (!gateEnabled.value || !gateMeter || !gate) {
      gateMonitoringId = null
      return
    }

    try {
      // Get current level from gateMeter (already in dB)
      const currentLevel = gateMeter.getValue()

      // Determine if gate should be open
      const shouldBeOpen = currentLevel > gateThreshold

      if (shouldBeOpen !== gateIsOpen) {
        gateIsOpen = shouldBeOpen

        if (shouldBeOpen) {
          // Open gate: ramp to unity gain (0dB = 1)
          gate.gain.rampTo(1, gateAttack)
        } else {
          // Close gate: ramp to range attenuation
          // Convert dB to linear gain: gain = 10^(dB/20)
          const targetGain = Math.pow(10, gateRange / 20)
          gate.gain.rampTo(targetGain, gateRelease)
        }
      }
    } catch (error) {
      console.error('[Gate] Monitoring error:', error)
    }

    gateMonitoringId = requestAnimationFrame(updateGate)
  }

  updateGate()
}

function stopGateMonitoring() {
  if (gateMonitoringId) {
    cancelAnimationFrame(gateMonitoringId)
    gateMonitoringId = null
  }

  // Reset gate to unity gain when stopping
  if (gate && gate.gain) {
    gate.gain.rampTo(1, 0.05)
  }
  gateIsOpen = false
}

</script>

<style scoped>
/* Custom lightweight scrollbar for AUX panel */
.aux-panel-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.aux-panel-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.aux-panel-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(20, 184, 166, 0.3);
  border-radius: 2px;
}

.aux-panel-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(20, 184, 166, 0.5);
}
</style>
