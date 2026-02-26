<template>
  <div
    ref="trackElement"
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
      <div class="w-full flex items-center justify-between gap-1 track-header cursor-move" 
        draggable="true"
        @dragstart="handleDragStart"
        title="Drag to reorder">
        <div class="flex items-center gap-1 flex-1 justify-center">
          <div class="text-xs font-bold text-gray-300">Track {{ trackNumber }}</div>
          <div v-if="isRecording" 
            class="px-1 py-0.5 bg-red-600 text-white text-[0.5rem] font-bold rounded animate-pulse">
            REC
          </div>
          <!-- BPM Display -->
          <BPMDisplay 
            :audio-buffer="currentAudioBuffer" 
            :audio-source-type="audioSourceType"
            :audio-loaded="audioLoaded" 
          />
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

      <!-- File Display (shown when source is 'file') -->
      <div v-if="audioSourceType === 'file'" class="w-full flex gap-1">
        <div class="flex-1 px-2 py-0.5 text-xs bg-gray-700 rounded border border-gray-600 overflow-hidden relative">
          <div v-if="isPlaying && fileName" class="animate-marquee whitespace-nowrap inline-block">
            {{ fileName }}&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;{{ fileName }}
          </div>
          <div v-else class="truncate">
            {{ fileName || 'No audio loaded' }}
          </div>
        </div>
        <!-- Library button - Small square icon button -->
        <button v-if="fileManagerAPI" @click="openLibrary"
          class="w-[1.6rem] h-[1.6rem] flex-shrink-0 bg-blue-700 hover:bg-blue-600 rounded border border-blue-600 transition-colors flex items-center justify-center"
          title="Open Audio Library">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </button>
      </div>

      <!-- Audio Input Device Selector (shown when source is 'input') -->
      <div v-if="audioSourceType === 'input'" class="w-full">
        <InputSelector
          icon="🎤"
          title="Select Audio Input"
          :devices="audioInputs"
          :selected-device-id="selectedAudioInput"
          default-label="No Input"
          default-description="Select an audio input device"
          default-icon="🎤"
          :show-file-option="false"
          @select="handleInputSelect"
        />
      </div>

      <!-- Transport Controls -->
      <div class="flex gap-1 justify-center">
        <button @click="togglePlay"
          class="px-2 py-1 flex-1 text-xs rounded transition-colors flex items-center justify-center"
          :class="currentPlaylist && isPlaying 
            ? 'bg-blue-600 hover:bg-blue-500 animate-pulse' 
            : isPlaying 
              ? 'bg-green-600 hover:bg-green-500 animate-pulse' 
              : audioLoaded 
                ? 'bg-green-600 hover:bg-green-500' 
                : 'bg-blue-600 hover:bg-blue-500'"
          :title="currentPlaylist && isPlaying ? 'Next track' : (isPlaying ? 'Pause' : 'Play')">
          <!-- Show microphone icon for audio input -->
          <svg v-if="audioSourceType === 'input'" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"
            class="w-3 h-3">
            <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
            <path
              d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
          </svg>
          <!-- Show play icon when not playing -->
          <svg v-else-if="!isPlaying" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M8 5v14l11-7z" />
          </svg>
          <!-- Show forward/next icon when playlist is playing (isPlaying is true at this point) -->
          <svg v-else-if="currentPlaylist" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M6 4l8 8-8 8V4zm10 0v16h2V4h-2z" />
          </svg>
          <!-- Show pause icon for single file playback -->
          <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M6 4h4v16H6zM14 4h4v16h-4z" />
          </svg>
        </button>
        <button @click="stopAudio"
          class="px-2 py-1 flex-1 text-xs rounded transition-colors bg-gray-600 hover:bg-gray-500 flex items-center justify-center">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M6 6h12v12H6z" />
          </svg>
        </button>
        <!-- Fade Out Button -->
        <FadeOutButton 
          :is-playing="isPlaying"
          :audio-source-type="audioSourceType"
          :volume="volume"
          @update:volume="volume = $event"
          @fade-complete="handleFadeComplete"
        />
      </div>
    </div>

    <!-- Content Wrapper with Transition -->
    <Transition name="fade" mode="out-in">
      <!-- Normal Track Content -->
      <div v-if="!showAuxPanel" key="normal" class="w-full flex-1 flex flex-col gap-2 min-h-0">
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
        @update-sends="auxSends.handleAuxSendsUpdate" @toggle-panel="showAuxPanel = !showAuxPanel" />

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
            class="absolute left-[0.2rem] bottom-[1.2rem] w-5 h-5 text-[0.4rem] font-bold rounded transition-all border"
            :class="isArmed 
              ? 'bg-red-700 border-red-500 text-white shadow-md shadow-red-500/50' 
              : 'bg-gray-800 border-gray-600 text-gray-400 hover:bg-gray-700 hover:border-gray-500'"
            title="Arm track for automation recording">
            A
          </button>
          
          <!-- Phase Invert Button -->
          <button @click="togglePhaseInvert" 
            class="absolute left-[0.2rem] bottom-[2.7rem] w-5 h-5 text-[0.65rem] font-bold rounded transition-all border"
            :class="phaseInverted 
              ? 'bg-purple-600 border-purple-400 text-white shadow-md shadow-purple-500/50' 
              : 'bg-gray-800 border-gray-600 text-gray-400 hover:bg-gray-700 hover:border-gray-500'"
            title="Phase Invert (180° polarity flip)">
            Ø
          </button>
          
          <!-- Routing Buttons -->
          <div class="flex flex-col gap-2 absolute left-[0.2rem] top-1/2 transform -translate-y-1/2 z-50">
            <button @click="routing.toggleRouteToMaster" :title="'Route to Master'"
              class="w-5 h-7 text-[8px] font-bold rounded transition-all flex items-center justify-center"
              :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
              M
            </button>
            <button v-if="props.allowSubgroupRouting" v-for="subgroup in props.subgroups" :key="subgroup.id" @click="routing.toggleRouteToSubgroup(subgroup.id)"
              :title="`Route to ${subgroup.name}`"
              class="w-5 h-7 text-[6px] font-bold rounded transition-all flex items-center justify-center"
              :class="routedSubgroups.has(subgroup.id) ? 'bg-orange-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
              S{{ subgroup.id }}
            </button>
          </div>
          <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />
          
          <!-- Phase Correlation LED (stereo only) - Above VU meters -->
          <div v-if="isStereo && faderHeight > 0" class="absolute right-[0.25rem] -top-8 z-50 flex flex-col items-center gap-0.5">
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
          
          <TrackMeter class="absolute right-[0.4rem] top-1/2 transform -translate-y-1/2 z-50 -mt-3"
            v-if="faderHeight > 0" :levelL="trackLevelL" :levelR="trackLevelR" :isStereo="isStereo"
            :height="faderHeight + 20" />
        </div>
      </div>
      </div>

      <!-- Aux Panel Content -->
      <div v-else key="aux" class="w-full flex-1 overflow-y-auto flex flex-col gap-2 aux-panel-scrollbar">
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
            @update-level="(val) => auxSends.updateLocalAuxSend(aux.id, 'level', val)"
            @toggle-pre-post="auxSends.toggleLocalPrePost(aux.id)" @toggle-mute="auxSends.toggleLocalMute(aux.id)" />
        </div>
      </div>
      </div>
    </Transition>

  </div>

  <!-- Parametric EQ Modal -->
  <ParametricEQModal v-model="showParametricEQ" :trackNumber="trackNumber" :eq-filters="eqFiltersData"
    :system-filters="systemFilters" @update="parametricEQ.handleParametricEQUpdate" />

  <!-- Phase Correlation Modal -->
  <PhaseCorrelationModal 
    v-model="showPhaseModal" 
    :correlation="phaseCorrelation" 
    :track-number="trackNumber"
    :audio-data-l="analyserDataL"
    :audio-data-r="analyserDataR"
    :is-stereo="isStereo"
  />
</template>

<script setup lang="ts">
import { computed, inject, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTrackAuxSends } from '~/composables/track/useTrackAuxSends'
import { useTrackCleanup } from '~/composables/track/useTrackCleanup'
import { useTrackDrag } from '~/composables/track/useTrackDrag'
import { useTrackLevelMonitoring } from '~/composables/track/useTrackLevelMonitoring'
import { useTrackLibraryLoader } from '~/composables/track/useTrackLibraryLoader'
import { useTrackAudioInput } from '~/composables/track/useTrackAudioInput'
import { useTrackCompressor } from '~/composables/track/useTrackCompressor'
import { useTrackGate } from '~/composables/track/useTrackGate'
import { useTrackParametricEQ } from '~/composables/track/useTrackParametricEQ'
import { useTrackPlaybackTime } from '~/composables/track/useTrackPlaybackTime'
import { useTrackRouting } from '~/composables/track/useTrackRouting'
import { useTrackSnapshot } from '~/composables/track/useTrackSnapshot'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'
import AuxSendControl from './audioTrack/AuxSendControl.vue'
import BPMDisplay from './audioTrack/BPMDisplay.vue'
import EQThumbnail from './audioTrack/EQThumbnail.vue'
import FadeOutButton from './audioTrack/FadeOutButton.vue'
import HPFButton from './audioTrack/HPFButton.vue'
import InputSelector from './audioTrack/InputSelector.vue'
import PadButton from './audioTrack/PadButton.vue'
import PanKnob from './audioTrack/PanKnob.vue'
import PhaseCorrelationModal from './audioTrack/PhaseCorrelationModal.vue'
import TrackAuxSends from './audioTrack/TrackAuxSends.vue'
import TrackCompressor from './audioTrack/TrackCompressor.vue'
import TrackEQ from './audioTrack/TrackEQ.vue'
import TrackFader from './audioTrack/TrackFader.vue'
import TrackGate from './audioTrack/TrackGate.vue'
import TrackMeter from './audioTrack/TrackMeter.vue'
import WaveformDisplay from './audioTrack/WaveformDisplay.vue'
import Knob from './core/Knob.vue'
import ParametricEQModal from './master/ParametricEQModal.vue'

defineOptions({
  inheritAttrs: false
})

// Inject Tone.js from App.vue (imported once for entire app)
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Inject automation system
const automation = inject<any>('automation', null)

// Inject file manager (Electron only)
const fileManagerAPI = inject<any>('fileManager', null)

const { getAudioFile } = useAudioFileStorage()

interface Props {
  trackNumber: number
  order: number
  masterChannel?: any
  subgroups?: Array<{ id: number, name: string, channel: any, ref: any }>
  auxBuses?: Array<{ id: string, name: string, volume: number, muted: boolean, soloed: boolean, routeToMaster: boolean, node?: any }>
  allowSubgroupRouting?: boolean
  isArmed?: boolean
  isDragging?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  subgroups: () => [],
  auxBuses: () => [],
  allowSubgroupRouting: true,
  isArmed: false,
  isDragging: false,
  order: 0
})

const emit = defineEmits<{
  (e: 'soloChange', value: { trackNumber: number, isSolo: boolean }): void
  (e: 'levelUpdate', value: { trackNumber: number, level: number }): void
  (e: 'remove'): void
  (e: 'toggle-arm'): void
  (e: 'drag-start'): void
}>()

// Initialize drag composable
const { trackElement, handleDragStart } = useTrackDrag({
  onDragStart: () => emit('drag-start')
})

// Initialize routing composable
const routing = useTrackRouting({
  getVolumeMerge: () => volumeMerge,
  getMasterChannel: () => props.masterChannel,
  getSubgroups: () => props.subgroups,
  getTone: () => Tone,
  getTrackNumber: () => props.trackNumber
})

const { routeToMaster, routedSubgroups } = routing

// Initialize parametric EQ composable
const parametricEQ = useTrackParametricEQ({
  getAudioNodes: () => ({
    eq3,
    channelSplit,
    waveform,
    compressor,
    balanceSplit
  }),
  getCompressorEnabled: () => compressorEnabled.value
})

const { eqFiltersData } = parametricEQ

// Initialize aux sends composable
const auxSends = useTrackAuxSends({
  getTone: () => Tone,
  getAudioNodes: () => ({ volumeMerge, balanceMerge }),
  getAuxBuses: () => props.auxBuses,
  getIsMuted: () => isMuted.value
})

const { auxSendsData } = auxSends

// Initialize level monitoring composable
const levelMonitoring = useTrackLevelMonitoring({
  getTone: () => Tone,
  getMeters: () => ({ meterL, meterR }),
  getAnalysers: () => ({ analyserL, analyserR }),
  getIsStereo: () => isStereo.value
})

const { trackLevelL, trackLevelR, phaseCorrelation, analyserBufferSize, analyserDataL, analyserDataR } = levelMonitoring

// Initialize playback time tracking composable
const playbackTime = useTrackPlaybackTime({
  getTone: () => Tone,
  getPlayer: () => player,
  getIsPlaying: () => isPlaying.value,
  getCurrentAudioBuffer: () => currentAudioBuffer
})

const { currentPlaybackTime } = playbackTime

// Initialize library loader composable
const libraryLoader = useTrackLibraryLoader({
  getTone: () => Tone,
  getAudioNodes: () => ({
    player,
    nextPlayer,
    nextBuffer,
    crossFade,
    padNode,
    currentAudioBuffer,
    gainNode,
    eq3,
    volumeMerge
  }),
  setAudioNodes: (updates) => {
    if (updates.player !== undefined) player = updates.player
    if (updates.nextPlayer !== undefined) nextPlayer = updates.nextPlayer
    if (updates.nextBuffer !== undefined) nextBuffer = updates.nextBuffer
    if (updates.crossFade !== undefined) crossFade = updates.crossFade
    if (updates.currentAudioBuffer !== undefined) currentAudioBuffer = updates.currentAudioBuffer
  },
  getState: () => ({
    fileName: fileName.value,
    fileId: fileId.value,
    isLoading: isLoading.value,
    audioLoaded: audioLoaded.value,
    isStereo: isStereo.value,
    isPlaying: isPlaying.value
  }),
  setState: (updates) => {
    if (updates.fileName !== undefined) fileName.value = updates.fileName
    if (updates.fileId !== undefined) fileId.value = updates.fileId
    if (updates.isLoading !== undefined) isLoading.value = updates.isLoading
    if (updates.audioLoaded !== undefined) audioLoaded.value = updates.audioLoaded
    if (updates.isStereo !== undefined) isStereo.value = updates.isStereo
    if (updates.isPlaying !== undefined) isPlaying.value = updates.isPlaying
  },
  getPlaylistState: () => ({
    currentPlaylist: currentPlaylist.value,
    playlistFiles,
    currentPlaylistIndex,
    nextTrack,
    manualStop,
    isPreloading
  }),
  setPlaylistState: (updates) => {
    if (updates.currentPlaylist !== undefined) currentPlaylist.value = updates.currentPlaylist
    if (updates.playlistFiles !== undefined) playlistFiles = updates.playlistFiles
    if (updates.currentPlaylistIndex !== undefined) currentPlaylistIndex = updates.currentPlaylistIndex
    if (updates.nextTrack !== undefined) nextTrack = updates.nextTrack
    if (updates.manualStop !== undefined) manualStop = updates.manualStop
    if (updates.isPreloading !== undefined) isPreloading = updates.isPreloading
  },
  getAudioFile: getAudioFile,
  initAudioNodes: initAudioNodes,
  getWaveformDisplayRef: () => waveformDisplayRef.value,
  getPlaybackTime: () => playbackTime
})

const { loadFileFromLibrary, loadPlaylistFromLibrary, loadFileFromIndexedDB } = libraryLoader

// Initialize audio input composable
const audioInput = useTrackAudioInput({
  getTone: () => Tone,
  getAudioNodes: () => ({
    player,
    padNode,
    currentAudioBuffer
  }),
  setAudioNodes: (updates) => {
    if (updates.player !== undefined) player = updates.player
    if (updates.currentAudioBuffer !== undefined) currentAudioBuffer = updates.currentAudioBuffer
    if (updates.audioInputSource !== undefined) audioInputSource = updates.audioInputSource
    if (updates.channelSplitter !== undefined) channelSplitter = updates.channelSplitter
    if (updates.audioInputStream !== undefined) audioInputStream = updates.audioInputStream
  },
  getAudioInputNodes: () => ({
    audioInputSource,
    channelSplitter,
    audioInputStream
  }),
  getState: () => ({
    selectedAudioInput: selectedAudioInput.value,
    audioLoaded: audioLoaded.value,
    isPlaying: isPlaying.value,
    fileName: fileName.value,
    fileId: fileId.value,
    isStereo: isStereo.value,
    audioContextStarted
  }),
  setState: (updates) => {
    if (updates.selectedAudioInput !== undefined) selectedAudioInput.value = updates.selectedAudioInput
    if (updates.audioLoaded !== undefined) audioLoaded.value = updates.audioLoaded
    if (updates.isPlaying !== undefined) isPlaying.value = updates.isPlaying
    if (updates.fileName !== undefined) fileName.value = updates.fileName
    if (updates.fileId !== undefined) fileId.value = updates.fileId
    if (updates.isStereo !== undefined) isStereo.value = updates.isStereo
    if (updates.audioContextStarted !== undefined) audioContextStarted = updates.audioContextStarted
  },
  getPlaylistState: () => ({
    currentPlaylist: currentPlaylist.value,
    playlistFiles,
    currentPlaylistIndex
  }),
  setPlaylistState: (updates) => {
    if (updates.currentPlaylist !== undefined) currentPlaylist.value = updates.currentPlaylist
    if (updates.playlistFiles !== undefined) playlistFiles = updates.playlistFiles
    if (updates.currentPlaylistIndex !== undefined) currentPlaylistIndex = updates.currentPlaylistIndex
  },
  getAudioInputs: () => audioInputs.value,
  initAudioNodes: initAudioNodes,
  stopAudio: stopAudio,
  getWaveformDisplayRef: () => waveformDisplayRef.value,
  getRouting: () => routing,
  getMasterChannel: () => props.masterChannel,
  getTrackNumber: () => props.trackNumber
})

const { handleSourceTypeChange, handleAudioInputChange, handleInputSelect } = audioInput

// Initialize compressor composable
const compressorControl = useTrackCompressor({
  getCompressorEnabled: () => compressorEnabled.value,
  setCompressorEnabled: (enabled) => { compressorEnabled.value = enabled },
  getAudioNodes: () => ({
    eq3,
    compressor,
    balanceSplit
  }),
  getTrackCompressorRef: () => trackCompressorRef.value
})

const { toggleCompressor } = compressorControl

// Initialize gate composable
const gateControl = useTrackGate({
  getGateEnabled: () => gateEnabled.value,
  setGateEnabled: (enabled) => { gateEnabled.value = enabled },
  getAudioNodes: () => ({
    gainNode,
    gate,
    eq3,
    gateMeter
  }),
  getTrackGateRef: () => trackGateRef.value,
  getGateState: () => ({
    gateThreshold,
    gateAttack,
    gateRelease,
    gateRange,
    gateIsOpen,
    gateMonitoringId
  }),
  setGateState: (updates) => {
    if (updates.gateThreshold !== undefined) gateThreshold = updates.gateThreshold
    if (updates.gateAttack !== undefined) gateAttack = updates.gateAttack
    if (updates.gateRelease !== undefined) gateRelease = updates.gateRelease
    if (updates.gateRange !== undefined) gateRange = updates.gateRange
    if (updates.gateIsOpen !== undefined) gateIsOpen = updates.gateIsOpen
    if (updates.gateMonitoringId !== undefined) gateMonitoringId = updates.gateMonitoringId
  }
})

const { toggleGate, handleGateParamsUpdate } = gateControl

// Initialize snapshot composable
const snapshot = useTrackSnapshot({
  // Props
  getTrackNumber: () => props.trackNumber,
  getOrder: () => props.order,
  getSubgroups: () => props.subgroups || [],
  
  // State getters
  getVolume: () => volume.value,
  getGain: () => gain.value,
  getPadEnabled: () => padEnabled.value,
  getHpfEnabled: () => hpfEnabled.value,
  getPhaseInverted: () => phaseInverted.value,
  getPan: () => pan.value,
  getIsMuted: () => isMuted.value,
  getIsSolo: () => isSolo.value,
  getRouteToMaster: () => routeToMaster.value,
  getRoutedSubgroups: () => routedSubgroups.value,
  getAudioSourceType: () => audioSourceType.value,
  getSelectedAudioInput: () => selectedAudioInput.value,
  getFileName: () => fileName.value,
  getFileId: () => fileId.value,
  getIsPlaying: () => isPlaying.value,
  getCurrentPlaybackTime: () => currentPlaybackTime.value,
  getCurrentAudioBuffer: () => currentAudioBuffer,
  getEqFiltersData: () => eqFiltersData.value,
  getCompressorEnabled: () => compressorEnabled.value,
  getGateEnabled: () => gateEnabled.value,
  getAuxSendsData: () => auxSendsData.value,
  
  // State setters
  setVolume: (value) => { volume.value = value },
  setGain: (value) => { gain.value = value },
  setPadEnabled: (value) => { padEnabled.value = value },
  setHpfEnabled: (value) => { hpfEnabled.value = value },
  setPhaseInverted: (value) => { phaseInverted.value = value },
  setPan: (value) => { pan.value = value },
  setIsMuted: (value) => { isMuted.value = value },
  setIsSolo: (value) => { isSolo.value = value },
  setRouteToMaster: (value) => { routeToMaster.value = value },
  setRoutedSubgroups: (value) => { routedSubgroups.value = value },
  setAudioSourceType: (value) => { audioSourceType.value = value },
  setSelectedAudioInput: (value) => { selectedAudioInput.value = value },
  setFileName: (value) => { fileName.value = value },
  setFileId: (value) => { fileId.value = value },
  setEqFiltersData: (value) => { eqFiltersData.value = value },
  setCompressorEnabled: (value) => { compressorEnabled.value = value },
  setGateEnabled: (value) => { gateEnabled.value = value },
  setAuxSendsData: (value) => { auxSendsData.value = value },
  setAudioLoaded: (value) => { audioLoaded.value = value },
  
  // Component refs
  getTrackEQRef: () => trackEQRef.value,
  getTrackCompressorRef: () => trackCompressorRef.value,
  getTrackGateRef: () => trackGateRef.value,
  
  // Audio nodes
  getPhaseInvertNode: () => phaseInvertNode,
  getPlayer: () => player,
  
  // Functions
  initAudioNodes,
  updatePad,
  updateHPF,
  handleAudioInputChange,
  loadFileFromIndexedDB,
  toggleCompressor,
  toggleGate,
  
  // Composables
  routingConnectToOutput: () => routing.connectToOutput(),
  auxSendsHandleUpdate: (data) => auxSends.handleAuxSendsUpdate(data),
  parametricEQHandleUpdate: (data) => parametricEQ.handleParametricEQUpdate(data),
  parametricEQGetFilters: () => parametricEQ.getParametricEQFilters(),
  parametricEQSetFilters: (filters) => parametricEQ.setParametricEQFilters(filters)
})

const { getSnapshot, restoreFromSnapshot, resetToDefaults } = snapshot

// Audio state
const fileName = ref<string>('')
const fileId = ref<string>('') // IndexedDB file ID for scene persistence
const audioLoaded = ref(false)
const faderContainer = ref<HTMLElement | null>(null)
const faderHeight = ref(0)
const isPlaying = ref(false)
const isMuted = ref(false)
const isSolo = ref(false)
const phaseInverted = ref(false)
const isLoading = ref(false)
const showParametricEQ = ref(false)
const showAuxPanel = ref(false)
const showPhaseModal = ref(false)
const waveformDisplayRef = ref<any>(null)

// Audio source selection
const audioSourceType = ref<'file' | 'input'>('file')

// Audio inputs from shared composable
const { audioInputDevices, refreshAudioInputs } = useAudioDevices()
const audioInputs = audioInputDevices // Use the shared ref

const selectedAudioInput = ref<string>('')
let audioInputStream: MediaStream | null = null
let audioInputSource: MediaStreamAudioSourceNode | null = null
let channelSplitter: ChannelSplitterNode | null = null

// FX state
const compressorEnabled = ref(false)
const gateEnabled = ref(false)

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

// Analyser nodes for phase correlation measurement
let analyserL: any = null
let analyserR: any = null

// Automation recording - track last recorded values to avoid redundant points
const lastRecordedVolume = ref<number | null>(null)
const lastRecordedTime = ref<number>(0)
let automationRecordingInterval: any = null

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

// Component refs
const trackEQRef = ref<InstanceType<typeof TrackEQ> | null>(null)
const trackCompressorRef = ref<InstanceType<typeof TrackCompressor> | null>(null)
const trackGateRef = ref<InstanceType<typeof TrackGate> | null>(null)

// Tone.js nodes
let player: any = null // Can be Tone.Player or Tone.UserMedia
let nextPlayer: any = null // Pre-loaded next player for seamless playback
let nextBuffer: AudioBuffer | null = null // Pre-loaded next track buffer
let crossFade: any = null // Tone.CrossFade for seamless transitions
let currentAudioBuffer: AudioBuffer | null = null // Store current audio buffer for player recreation
let padNode: any = null // PAD attenuation (-26dB when enabled) - PRE-GAIN
let phaseInvertNode: any = null // Phase invert (polarity flip) - PRE-GAIN
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
let audioContextStarted: boolean = false // Track if Tone.start() has been called

// Playlist queue management (using refs for reactivity in template)
const currentPlaylist = ref<any>(null)
let playlistFiles: any[] = []
let currentPlaylistIndex = 0
let nextTrack: any = null // Next track to play (null for single files, set for playlists)
let manualStop = false // Flag to distinguish manual stop from natural track end
let isPreloading = false // Flag to track if next track is being pre-loaded

// Cleanup composable
const cleanup = useTrackCleanup({
  // Automation
  getAutomationRecordingInterval: () => automationRecordingInterval,
  setAutomationRecordingInterval: (v: any) => { automationRecordingInterval = v },
  
  // Audio input
  getAudioInputSource: () => audioInputSource,
  setAudioInputSource: (v: any) => { audioInputSource = v },
  getChannelSplitter: () => channelSplitter,
  setChannelSplitter: (v: any) => { channelSplitter = v },
  getAudioInputStream: () => audioInputStream,
  setAudioInputStream: (v: any) => { audioInputStream = v },
  
  // Players and buffers
  getPlayer: () => player,
  getNextPlayer: () => nextPlayer,
  getCurrentAudioBuffer: () => currentAudioBuffer,
  setCurrentAudioBuffer: (v: any) => { currentAudioBuffer = v },
  getNextBuffer: () => nextBuffer,
  setNextBuffer: (v: any) => { nextBuffer = v },
  
  // Audio nodes
  getCrossFade: () => crossFade,
  getGainNode: () => gainNode,
  getEq3: () => eq3,
  getBalanceSplit: () => balanceSplit,
  getBalanceLeft: () => balanceLeft,
  getBalanceRight: () => balanceRight,
  getBalanceMerge: () => balanceMerge,
  getVolumeSplit: () => volumeSplit,
  getVolumeNodeL: () => volumeNodeL,
  getVolumeNodeR: () => volumeNodeR,
  getVolumeMerge: () => volumeMerge,
  getMeterL: () => meterL,
  getMeterR: () => meterR,
  getGateMeter: () => gateMeter,
  getChannelSplit: () => channelSplit,
  getWaveform: () => waveform,
  getCompressor: () => compressor,
  
  // Composable cleanups
  stopGateMonitoring: () => gateControl.stopGateMonitoring(),
  auxSendsCleanup: () => auxSends.cleanup(),
  parametricEQCleanup: () => parametricEQ.cleanup(),
  levelMonitoringCleanup: () => levelMonitoring.cleanup(),
  playbackTimeCleanup: () => playbackTime.cleanup(),
  
  // UI
  getResizeObserver: () => resizeObserver,
  getWaveformDisplayRef: () => waveformDisplayRef.value,
  
  // Functions
  refreshAudioInputs
})

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
  auxSends.handleAuxSendsUpdate(auxSendsData.value)
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
  levelMonitoring.startLevelMonitoring()

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

  // Phase invert node (polarity flip)
  phaseInvertNode = new Tone.Gain(1) // 1 = normal polarity, -1 = inverted polarity

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

  // Create CrossFade node for seamless playlist transitions
  // Input A (0): current player, Input B (1): next player
  crossFade = new Tone.CrossFade(0) // Start with input A (current player)
  crossFade.connect(padNode) // CrossFade output goes to PAD input

  // Connect chain: player -> crossFade -> pad -> phaseInvert -> hpf -> gain -> gate -> eq3 -> balance -> volume
  // PAD is pre-gain (professional mixer architecture)
  // Phase invert is pre-gain (inverts polarity before amplification)
  // HPF is pre-gain (removes rumble before amplification)
  // Gate is post-gain (eliminates noise from properly amplified signal)
  // Compressor is bypassed by default (not in chain)
  // HPF starts bypassed (direct connection phaseInvert -> gain)
  padNode.connect(phaseInvertNode)
  phaseInvertNode.connect(gainNode) // Bypass HPF by default
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
  routing.connectToOutput()

  // Restore aux sends if they were configured before nodes were created
  // (happens when restoring from scene)
  if (Object.keys(auxSendsData.value).length > 0) {
    nextTick(() => {
      auxSends.handleAuxSendsUpdate(auxSendsData.value)
    })
  }
}

// Open file manager library (Electron only)
function openLibrary() {
  if (fileManagerAPI?.openFileManager) {
    fileManagerAPI.openFileManager(props.trackNumber)
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
        loop: nextTrack ? false : true,
        playbackRate: 1.0,
        fadeIn: 0.01,
        fadeOut: 0.01,
        onstop: () => {
          if (!manualStop && nextTrack) {
            loadFileFromLibrary(nextTrack, true).then(() => {
              if (player) {
                manualStop = false
                player.start()
                isPlaying.value = true
                waveformDisplayRef.value?.start()
                playbackTime.startPlaybackTimeTracking()
              }
            })
          } else {
            manualStop = false
          }
        },
      })

      // Reconnect to audio chain
      player.connect(padNode)
    }

    // Start with future time for more stable playback
    const startTime = Tone.now() + 0.05  // 50ms in future
    manualStop = false
    player.start(startTime)
    isPlaying.value = true

    // Start waveform
    waveformDisplayRef.value?.start()

    // Start playback time tracking
    playbackTime.startPlaybackTimeTracking()
  } else {
    // If playlist is playing, skip to next track instead of stopping
    if (currentPlaylist.value && playlistFiles.length > 0) {
      // Stop current track
      manualStop = true
      player.stop()
      isPlaying.value = false
      playbackTime.stopPlaybackTimeTracking()
      
      // Move to next track
      currentPlaylistIndex++
      if (currentPlaylistIndex >= playlistFiles.length) {
        currentPlaylistIndex = 0 // Loop back to start
      }
      
      // Load and play next track
      const nextFile = playlistFiles[currentPlaylistIndex]
      
      await loadFileFromLibrary(nextFile, true) // preserve playlist state
      const trackName = nextFile.title || nextFile.fileName
      const trackDisplay = nextFile.artist ? `${nextFile.artist} - ${trackName}` : trackName
      fileName.value = `${currentPlaylist.value.name} (${currentPlaylistIndex + 1}/${playlistFiles.length}) - ${trackDisplay}`
      
      // Auto-start playback
      await nextTick()
      if (player && typeof player.start === 'function') {
        const startTime = Tone.now() + 0.05
        manualStop = false
        player.start(startTime)
        isPlaying.value = true
        waveformDisplayRef.value?.start()
        playbackTime.startPlaybackTimeTracking()
      }
    } else {
      // No playlist: stop playback normally
      manualStop = true
      player.stop()
      isPlaying.value = false

      // Stop waveform
      waveformDisplayRef.value?.stop()

      // Stop playback time tracking
      playbackTime.stopPlaybackTimeTracking()
    }
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
  manualStop = true
  player.stop()
  isPlaying.value = false

  // Reset playback position to start
  currentPlaybackTime.value = 0
  playbackTime.stopPlaybackTimeTracking()

  // Stop waveform
  waveformDisplayRef.value?.stop()
}

// Handle fade-out completion
function handleFadeComplete() {
  stopAudio()
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
  if (!hpfNode || !padNode || !phaseInvertNode || !gainNode) return

  // Safer approach: use disconnect() without arguments to disconnect all
  // Then rebuild the chain
  try {
    phaseInvertNode.disconnect()
    hpfNode.disconnect()
  } catch (e) { }

  if (hpfEnabled.value) {
    // Enable HPF: phaseInvert -> hpf -> gain
    phaseInvertNode.connect(hpfNode)
    hpfNode.connect(gainNode)
  } else {
    // Disable HPF: phaseInvert -> gain (direct bypass)
    phaseInvertNode.connect(gainNode)
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

// Toggle phase inversion (180° polarity flip)
function togglePhaseInvert() {
  phaseInverted.value = !phaseInverted.value
  
  if (!phaseInvertNode) return
  
  // Use rampTo to avoid clicks (10ms transition)
  phaseInvertNode.gain.rampTo(phaseInverted.value ? -1 : 1, 0.01)
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
  
  const currentTime = automation.transport.value.currentTime
  
  // In WRITE mode: always record to capture all movements
  // In TOUCH/LATCH: only record if value changed significantly
  const shouldRecord = volumeLane.mode === 'write' ||
    lastRecordedVolume.value === null ||
    Math.abs(newValue - lastRecordedVolume.value) >= 0.05 // Reduced threshold for smoother curves
  
  if (shouldRecord) {
    automation.addPoint(props.trackNumber, 'volume', currentTime, newValue)
    lastRecordedVolume.value = newValue
  }
})

// Start continuous automation recording when in WRITE mode
watch([() => automation?.isRecording.value, () => automation?.transport.value.isPlaying], ([isRec, isPlaying]) => {
  // Stop any existing interval
  if (automationRecordingInterval) {
    clearInterval(automationRecordingInterval)
    automationRecordingInterval = null
  }
  
  if (!automation || !isRec || !isPlaying) return
  
  // Check if any lane for this track is in WRITE mode
  const hasWriteLane = automation.automationLanes.value.some((lane: any) => 
    lane.trackId === props.trackNumber && lane.mode === 'write'
  )
  
  if (!hasWriteLane) return
  
  // Record current values every 50ms to ensure continuous data capture
  automationRecordingInterval = setInterval(() => {
    if (!automation || !automation.transport.value.isPlaying) {
      clearInterval(automationRecordingInterval)
      automationRecordingInterval = null
      return
    }
    
    const currentTime = automation.transport.value.currentTime
    
    // Avoid recording multiple points at the exact same time
    if (Math.abs(currentTime - lastRecordedTime.value) < 0.01) return
    
    lastRecordedTime.value = currentTime
    
    // Record volume if there's a WRITE volume lane
    const volumeLane = automation.automationLanes.value.find((lane: any) => 
      lane.trackId === props.trackNumber && 
      lane.parameter === 'volume' &&
      lane.mode === 'write'
    )
    if (volumeLane) {
      automation.addPoint(props.trackNumber, 'volume', currentTime, volume.value)
      lastRecordedVolume.value = volume.value
    }
  }, 50) // 20 points per second for smooth curves
})

// Expose methods for external control
defineExpose({
  setMuted: (muted: boolean) => {
    isMuted.value = muted
    updateVolume()
  },
  isSolo: () => isSolo.value,
  disconnectFromSubgroup: routing.disconnectFromSubgroup, // Expose for cleanup when subgroup is removed
  loadFileFromLibrary, // Expose for file manager integration
  loadPlaylistFromLibrary, // Expose for playlist integration
  isAudioLoaded: () => audioLoaded.value && audioSourceType.value === 'file', // Check if track has a file loaded
  
  // Automation control - set values without triggering recording
  setVolume: (value: number, skipRecording = false) => {
    if (skipRecording) {
      // Temporarily update last recorded value to prevent recording
      lastRecordedVolume.value = value
    }
    volume.value = value
  },
  
  setPan: (value: number, skipRecording = false) => {
    // Pan automation playback only (recording disabled)
    pan.value = value
  },

  // Playback control methods for automation transport synchronization
  startPlayback: async () => {
    if (!Tone) return
    if (isPlaying.value) return // Already playing

    // For audio input, unmute instead
    if (audioSourceType.value === 'input') {
      if (audioLoaded.value && isMuted.value) {
        toggleMute()
      }
      return
    }

    // For file playback
    if (!player || !audioLoaded.value) {
      return // Cannot play: audio not loaded
    }

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
    if (player && currentAudioBuffer) {
      try {
        player.disconnect()
        player.dispose()
      } catch (e) { }

      await new Promise(resolve => setTimeout(resolve, 10))

      player = new Tone.Player({
        url: currentAudioBuffer,
        loop: nextTrack ? false : true,
        playbackRate: 1.0,
        fadeIn: 0.01,
        fadeOut: 0.01,
        onstop: () => {
          if (!manualStop && nextTrack) {
            loadFileFromLibrary(nextTrack, true).then(() => {
              if (player) {
                manualStop = false
                player.start()
                isPlaying.value = true
                waveformDisplayRef.value?.start()
                playbackTime.startPlaybackTimeTracking()
              }
            })
          } else {
            manualStop = false
          }
        },
      })

      player.connect(padNode)
    }

    const startTime = Tone.now() + 0.05
    manualStop = false
    player.start(startTime)
    isPlaying.value = true

    waveformDisplayRef.value?.start()
    playbackTime.startPlaybackTimeTracking()
  },

  pausePlayback: () => {
    if (!isPlaying.value) return // Already paused

    // For audio input, mute instead
    if (audioSourceType.value === 'input') {
      if (audioLoaded.value && !isMuted.value) {
        toggleMute()
      }
      return
    }

    // For file playback
    if (!player || !audioLoaded.value) return

    manualStop = true
    player.stop()
    isPlaying.value = false
    waveformDisplayRef.value?.stop()
    playbackTime.stopPlaybackTimeTracking()
  },

  stopPlayback: () => {
    // For audio input, mute instead
    if (audioSourceType.value === 'input') {
      if (audioLoaded.value && !isMuted.value) {
        toggleMute()
      }
      return
    }

    // For file playback
    if (!player || !audioLoaded.value) return

    manualStop = true
    player.stop()
    isPlaying.value = false
    currentPlaybackTime.value = 0
    playbackTime.stopPlaybackTimeTracking()
    waveformDisplayRef.value?.stop()
  },

  getSnapshot,
  restoreFromSnapshot,
  resetToDefaults
})

// Cleanup
onUnmounted(() => {
  cleanup.cleanup()
})

</script>