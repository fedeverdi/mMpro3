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
      <div class="w-full">
        <select v-model="audioSourceType"
          class="w-full text-xs bg-gray-700 text-gray-200 border border-gray-600 rounded px-1 py-1 focus:border-blue-500 focus:outline-none">
          <option value="input">🎤 Audio Input</option>
          <option value="file">📁 Audio File</option>
        </select>
      </div>

      <!-- Audio Input Device Selector -->
      <div v-if="audioSourceType === 'input'" class="w-full">
        <InputSelector icon="🎤" title="Select Audio Input" :devices="audioInputDevices"
          :selected-device-id="selectedAudioInput" default-label="No Input"
          default-description="Select an audio input device" default-icon="🎤" :show-file-option="false"
          @select="handleInputSelect" />
      </div>

      <!-- Audio File Selector -->
      <div v-if="audioSourceType === 'file'" class="w-full flex flex-col gap-1">
        <button @click="openLibrary"
          class="w-full text-xs bg-blue-600 hover:bg-blue-500 text-white border border-blue-500 rounded px-2 py-1 transition-all flex items-center gap-1 overflow-hidden">
          <span class="flex-shrink-0">📚</span>
          <span class="flex-1 min-w-0 overflow-hidden">
            <span class="block whitespace-nowrap animate-marquee">
              {{ selectedFileName || 'Load from Library' }}
            </span>
          </span>
        </button>
      </div>

      <!-- Play/Stop Controls -->
      <div class="flex gap-1">
        <button @click="handlePlayFile" :disabled="!selectedFileName"
          class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all flex items-center justify-center gap-1 disabled:opacity-50 disabled:cursor-not-allowed"
          :class="isPlaying ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
          ▶ PLAY
        </button>
        <button @click="handleStopFile" :disabled="!selectedFileName"
          class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all flex items-center justify-center gap-1 bg-gray-700 hover:bg-gray-600 text-gray-300 disabled:opacity-50 disabled:cursor-not-allowed">
          ■ STOP
        </button>
      </div>

      <!-- Waveform Display - Always visible -->
      <WaveformDisplay :track-number="trackNumber - 1" :show-mode-buttons="false" mode="signal"
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
            color="#8b5cf6" />
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
        <EQThumbnail :system-filters="eq4BandFilters" :filters="parametricEQFilters" />

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
          class="w-full z-[200] absolute bg-gray-800 left-0 top-[6.8rem] max-h-[50rem] overflow-y-auto custom-scrollbar">
          <button @click="showAuxSendsPanel = false"
            class="absolute right-2 top-2 w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
            title="Remove Track">
            ×
          </button>
          <div class="grid grid-cols-1 gap-2">
            <AuxSendControl v-for="aux in props.auxBuses.slice(0, 6)" :key="aux.id" :aux="aux"
              :aux-send-data="auxSendsData[aux.id]" @update-level="(val) => updateAuxSend(aux.id, val)"
              @toggle-pre-post="toggleAuxPrePost(aux.id)" @toggle-mute="toggleAuxMute(aux.id)" />
          </div>
        </div>
      </div>

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


      <!-- Pan Knob -->
      <div class="flex justify-center scale-[0.75]">
        <PanKnob v-model="pan" label="Pan" />
      </div>
      <div class="text-[0.455rem] uppercase text-center mb-6">Volume</div>

      <!-- Volume Fader and VU Meter -->
      <div class="flex flex-col flex-1 min-h-0 pb-6 ">
        <div ref="faderContainer" class="flex-1 relative flex items-center justify-center gap-1 min-h-0">

          <!-- Phase Invert Button -->
          <button @click="togglePhaseInvert"
            class="absolute left-[0.2rem] bottom-[2.7rem] w-5 h-5 text-[0.65rem] font-bold rounded transition-all border"
            :class="phaseInverted
              ? 'bg-purple-600 border-purple-400 text-white shadow-md shadow-purple-500/50'
              : 'bg-gray-800 border-gray-600 text-gray-400 hover:bg-gray-700 hover:border-gray-500'"
            title="Phase Invert">
            Ø
          </button>

          <!-- Routing Buttons -->
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
          </div>

          <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />

          <TrackMeter class="absolute right-[0.4rem] top-1/2 transform -translate-y-1/2 z-50" v-if="faderHeight > 0"
            :levelL="trackLevelL" :levelR="trackLevelR" :isStereo="audioSourceType === 'file'"
            :height="faderHeight + 20" />
        </div>
      </div>
    </div>
  </div>

  <!-- Parametric EQ Modal -->
  <ParametricEQModal v-model="showParametricEQ" :track-number="trackNumber"
    :title="`Parametric EQ - Track ${trackNumber + 1}`" @update="handleParametricEQUpdate" />
</template>

<script setup lang="ts">
import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import HPFButton from './audioTrack/HPFButton.vue'
import InputSelector from './audioTrack/InputSelector.vue'
import PadButton from './audioTrack/PadButton.vue'
import PanKnob from './audioTrack/PanKnob.vue'
import TrackCompressor from './audioTrack/TrackCompressor.vue'
import TrackEQ from './audioTrack/TrackEQ.vue'
import TrackFader from './audioTrack/TrackFader.vue'
import TrackGate from './audioTrack/TrackGate.vue'
import TrackMeter from './audioTrack/TrackMeter.vue'
import TrackAuxSends from './audioTrack/TrackAuxSends.vue'
import AuxSendControl from './audioTrack/AuxSendControl.vue'
import Knob from './core/Knob.vue'
import ParametricEQModal from './master/ParametricEQModal.vue'
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

// Emits
const emit = defineEmits<{
  remove: []
  'toggle-arm': []
  'open-library': [trackNumber: number]
}>()

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Audio devices
const { audioInputDevices } = useAudioDevices()

// Reactive state - UI Only
const trackElement = ref<HTMLElement | null>(null)
const faderContainer = ref<HTMLElement | null>(null)
const faderHeight = ref(0)
const selectedAudioFile = ref<string | null>(null)
const selectedFileName = ref<string | null>(null)

const audioSourceType = ref<'input' | 'file'>('input')
const selectedAudioInput = ref<string>('')

// Control values
const volume = ref(0) // dB (-90 to +12)
const gain = ref(0) // dB
const padEnabled = ref(false)
const hpfEnabled = ref(false)
const pan = ref(0) // -1 to 1
const isMuted = ref(false)
const isSolo = ref(false)
const phaseInverted = ref(false)
const routeToMaster = ref(true)
const routedSubgroups = ref<Set<number>>(new Set()) // Track which subgroups this track is routed to

// Aux sends state
const auxSendsData = ref<Record<string, { level: number, preFader: boolean, muted: boolean }>>({})
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
    console.log(`[Track ${props.trackNumber}] Parametric EQ modal opened`)
  }
})

// Meter levels (simulated for now)
const trackLevelL = ref(-60)
const trackLevelR = ref(-60)

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

  // Send to Rust engine with device name
  if (audioEngine?.state.value.isRunning && deviceId) {
    // Find the device to get its name
    const device = audioInputDevices.value.find(d => d.id === deviceId)
    if (device) {
      // Extract device name (remove "- Channel X" suffix if present)
      const deviceName = device.name.replace(/ - Channel \d+$/, '')
      // Extract channel indices from composite ID if present (format: "deviceId:channelIndex")
      const channelMatch = deviceId.match(/:(\d+)$/)
      const channelIndex = channelMatch ? parseInt(channelMatch[1]) : 0
      
      console.log(`[Track ${props.trackNumber}] Selected device: ${deviceName}, channel: ${channelIndex}`)
      
      // For stereo: use channel and channel+1
      audioEngine.setTrackSourceInput(props.trackNumber - 1, channelIndex, channelIndex + 1, deviceName)
    }
  }
}

function openLibrary() {
  emit('open-library', props.trackNumber)
}

// File playback controls
function handlePlayFile() {
  if (audioEngine?.state.value.isRunning && selectedAudioFile.value) {
    audioEngine.playFile(props.trackNumber - 1)
    isPlaying.value = true
  }
}

function handleStopFile() {
  if (audioEngine?.state.value.isRunning && selectedAudioFile.value) {
    audioEngine.stopFile(props.trackNumber - 1)
    isPlaying.value = false
  }
}

// Method to load file from library (called from parent)
async function loadFileFromLibrary(storedFile: any) {
  try {
    selectedAudioFile.value = storedFile.id
    selectedFileName.value = storedFile.title || storedFile.fileName
    audioSourceType.value = 'file'

    // Save ArrayBuffer to temp file and get the file path
    const tempFilePath = await window.audioEngine.saveTempAudioFile(
      storedFile.arrayBuffer,
      storedFile.fileName
    )

    // Set track source to file in Rust engine
    if (audioEngine?.state.value.isRunning) {
      await audioEngine.setTrackSourceFile(props.trackNumber - 1, tempFilePath)
      // Auto-play the file
      await audioEngine.playFile(props.trackNumber - 1)
      isPlaying.value = true
    }
  } catch (error) {
    console.error(`[Track ${props.trackNumber}] Error loading file from library:`, error)
  }
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
  if (routedSubgroups.value.has(subgroupId)) {
    routedSubgroups.value.delete(subgroupId)
    // Send to backend
    if (audioEngine?.state.value.isRunning) {
      audioEngine.setTrackRouteToSubgroup(props.trackNumber - 1, subgroupId, false)
    }
  } else {
    routedSubgroups.value.add(subgroupId)
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
      params.attack,
      params.release
    )
  }
}

function handleGateParamsUpdate(params: { threshold: number; attack: number; release: number; range: number }) {
  // Send to Rust engine
  if (audioEngine?.state.value.isRunning && gateEnabled.value) {
    audioEngine.setTrackGate(
      props.trackNumber - 1,
      true,
      params.threshold,
      params.range,
      params.attack,
      params.release
    )
  }
}

// Handle aux sends updates
function handleAuxSendsUpdate(sends: Record<string, { level: number, preFader: boolean, muted: boolean }>) {
  auxSendsData.value = sends

  // Send to Rust engine for each aux
  if (audioEngine?.state.value.isRunning) {
    Object.entries(sends).forEach(([auxId, send]) => {
      // Extract numeric index from aux ID (handles "aux1", "aux-1", etc.)
      const auxIndex = parseInt(auxId.replace(/\D/g, '')) - 1
      // Convert dB to linear gain
      const linearGain = Math.pow(10, send.level / 20)
      audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader, send.muted)
    })
  }
}

// Update individual aux send level
function updateAuxSend(auxId: string | number, level: number) {
  const auxKey = typeof auxId === 'number' ? `aux${auxId}` : auxId
  if (!auxSendsData.value[auxKey]) {
    auxSendsData.value[auxKey] = {
      level: -60,
      preFader: false,
      muted: true
    }
  }

  auxSendsData.value[auxKey].level = level

  // Auto-unmute if level > -60
  if (level > -60 && auxSendsData.value[auxKey].muted) {
    auxSendsData.value[auxKey].muted = false
  }

  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    const send = auxSendsData.value[auxKey]
    // Convert aux ID to numeric index (0-based)
    const auxIndex = typeof auxId === 'number' ? auxId - 1 : parseInt(auxId.replace(/\D/g, '')) - 1
    const linearGain = Math.pow(10, send.level / 20)
    audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader, send.muted)
  }
}

// Toggle aux send pre/post fader
function toggleAuxPrePost(auxId: string | number) {
  const auxKey = typeof auxId === 'number' ? `aux${auxId}` : auxId
  if (!auxSendsData.value[auxKey]) {
    auxSendsData.value[auxKey] = {
      level: -60,
      preFader: false,
      muted: true
    }
  }

  auxSendsData.value[auxKey].preFader = !auxSendsData.value[auxKey].preFader

  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    const send = auxSendsData.value[auxKey]
    const auxIndex = typeof auxId === 'number' ? auxId - 1 : parseInt(auxId.replace(/\D/g, '')) - 1
    const linearGain = Math.pow(10, send.level / 20)
    audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader, send.muted)
  }
}

// Toggle aux send mute
function toggleAuxMute(auxId: string | number) {
  const auxKey = typeof auxId === 'number' ? `aux${auxId}` : auxId
  if (!auxSendsData.value[auxKey]) {
    auxSendsData.value[auxKey] = {
      level: -60,
      preFader: false,
      muted: true
    }
  }

  auxSendsData.value[auxKey].muted = !auxSendsData.value[auxKey].muted

  // Send to Rust engine
  if (audioEngine?.state.value.isRunning) {
    const send = auxSendsData.value[auxKey]
    const auxIndex = typeof auxId === 'number' ? auxId - 1 : parseInt(auxId.replace(/\D/g, '')) - 1
    const linearGain = Math.pow(10, send.level / 20)
    audioEngine.setTrackAuxSend(props.trackNumber - 1, auxIndex, linearGain, send.preFader, send.muted)
  }
}

function handleParametricEQUpdate(filters: any) {
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
}

// Watchers - Send changes to Rust engine
watch(volume, (newVolume) => {
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
  // Convert dB to linear gain: gain = 10^(dB/20)
  // gain knob is in dB range (-12 to +12)
  const gainValue = Math.pow(10, newGain / 20)

  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackGain(props.trackNumber - 1, gainValue)
  }
})

watch(padEnabled, (enabled) => {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackPad(props.trackNumber - 1, enabled)
  }
})

watch(hpfEnabled, (enabled) => {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackHPF(props.trackNumber - 1, enabled)
  }
})

watch(isMuted, (muted) => {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackMute(props.trackNumber - 1, muted)
  }
})

watch(routeToMaster, (route) => {
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
        params.attack,
        params.release
      )
    }
  }
})

watch(pan, (newPan) => {
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
    }
  },
  { deep: true }
)

// Fader height calculation
function updateFaderHeight() {
  if (faderContainer.value) {
    faderHeight.value = faderContainer.value.clientHeight
  }
}

// Lifecycle
onMounted(async () => {
  // Audio input devices are already enumerated during app initialization
  // No need to refresh them here

  // Set up resize observer for fader
  const resizeObserver = new ResizeObserver(updateFaderHeight)
  if (faderContainer.value) {
    resizeObserver.observe(faderContainer.value)
  }

  updateFaderHeight()
})

onUnmounted(() => {
  // Cleanup if needed
})

// Expose methods to parent
defineExpose({
  loadFileFromLibrary
})
</script>

<style scoped>
/* Marquee animation for long file names */
@keyframes marquee {
  0% {
    transform: translateX(0);
  }

  100% {
    transform: translateX(-50%);
  }
}

.animate-marquee {
  display: inline-block;
  padding-right: 100%;
  animation: marquee 10s linear infinite;
}

.animate-marquee:hover {
  animation-play-state: paused;
}
</style>
