<template>
  <div class="track-channel relative bg-gray-800 rounded-lg border border-gray-700 p-1 flex flex-col items-center gap-1 h-full">
    <!-- Loading Overlay -->
    <div v-if="isLoading" class="absolute inset-0 bg-gray-900 bg-opacity-80 rounded-lg z-50 flex flex-col items-center justify-center gap-2">
      <svg class="animate-spin h-8 w-8 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      <span class="text-sm text-gray-300 font-medium">Loading...</span>
    </div>

    <!-- Track Header -->
    <div class="w-full flex flex-col gap-1">
      <div class="text-xs font-bold text-center text-gray-300">Track {{ trackNumber }}</div>

      <!-- Audio Source Selector -->
      <div class="w-full">
        <select 
          v-model="audioSourceType" 
          @change="handleSourceTypeChange"
          class="w-full text-xs bg-gray-700 text-gray-200 border border-gray-600 rounded px-1 py-1 focus:border-blue-500 focus:outline-none"
        >
          <option value="file">📁 File</option>
          <option value="input">🎤 Audio Input</option>
          <option value="youtube">🎵 YouTube</option>
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
        <select 
          v-model="selectedAudioInput" 
          @change="handleAudioInputChange"
          class="w-full text-xs bg-gray-800 text-gray-200 border border-gray-600 rounded px-1 py-1 focus:border-blue-500 focus:outline-none"
        >
          <option value="">Select Input...</option>
          <option v-for="device in audioInputs" :key="device.deviceId" :value="device.deviceId">
            {{ device.label || `Input ${device.deviceId.substring(0, 12)}...` }}
          </option>
        </select>
      </div>

      <!-- YouTube URL Input (shown when source is 'youtube') -->
      <div v-if="audioSourceType === 'youtube'" class="w-full flex gap-1 items-center">
        <input 
          v-model="audioUrl"
          type="text"
          placeholder="Paste YouTube URL..."
          class="flex-1 min-w-0 px-2 py-1 text-xs bg-gray-900 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
          @keyup.enter="handleUrlLoad"
        />
        <button 
          @click="handleUrlLoad"
          :disabled="!audioUrl || isLoading"
          class="flex-shrink-0 px-1.5 py-1 text-[10px] rounded transition-colors flex items-center justify-center"
          :class="audioLoaded ? 'bg-green-600 text-white cursor-default' : (audioUrl && !isLoading ? 'bg-blue-600 hover:bg-blue-500 text-white' : 'bg-gray-700 text-gray-500 cursor-not-allowed')"
        >
          <span v-if="isLoading" class="flex items-center gap-0.5">
            <svg class="animate-spin h-2.5 w-2.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </span>
          <span v-else>{{ audioLoaded ? 'Loaded' : 'Load' }}</span>
        </button>
      </div>


      <!-- Transport Controls -->
      <div class="flex gap-1 justify-center">
        <button @click="togglePlay"
          class="px-2 py-1 w-full text-xs rounded transition-colors flex items-center justify-center"
          :class="isPlaying ? 'bg-green-600 hover:bg-green-500 animate-pulse' : (audioLoaded ? 'bg-green-600 hover:bg-green-500' : 'bg-blue-600 hover:bg-blue-500')">
          <!-- Show microphone icon for audio input -->
          <svg v-if="audioSourceType === 'input'" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
            <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
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
    <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
      <div class="px-0.5 py-0.5">
        <canvas ref="waveformCanvas" class="w-full h-[30px] rounded border border-gray-700 bg-black"
          style="image-rendering: crisp-edges;"></canvas>
      </div>
    </div>

    <!-- Gain Control -->
    <div class="w-full flex items-center justify-center h-[4rem]">
      <div class="scale-[0.65]">
        <Knob v-model="gain" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Gain" unit="dB" color="#8b5cf6" />
      </div>
    </div>

    <!-- FX Section -->
    <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
      <div class="flex gap-1">
        <!-- Compressor Toggle -->
        <div @click="toggleCompressor" :class="[
          'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between',
          compressorEnabled ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]">
          <span>CO</span>
          <button :disabled="!compressorEnabled" @click.stop="showCompressorModal = true"
            class="p-0.5 rounded hover:bg-green-700">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>

        <!-- Reverb Toggle -->
        <div @click="toggleReverb" :class="[
          'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between',
          reverbEnabled ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]">
          <span>RE</span>
          <button :disabled="!reverbEnabled" @click.stop="showReverbModal = true"
            class="p-0.5 rounded hover:bg-green-700">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </div>


    <!-- EQ Section -->
    <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
      <div class="flex items-center justify-between px-2">
        <button @click="showEQ3Bands = !showEQ3Bands" class="flex items-center gap-1 hover:text-gray-200 transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-3 h-3 text-gray-400 transition-transform" :class="showEQ3Bands ? 'rotate-90' : ''">
            <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
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
      <div class="px-1 py-2">
        <canvas ref="eqThumbnail" class="w-full h-[50px] rounded border border-gray-700"
          style="image-rendering: crisp-edges;"></canvas>
      </div>
      <!-- 3-Band EQ Knobs (Accordion) -->
      <div v-show="showEQ3Bands" class="grid grid-cols-2 gap-2 items-center max-h-[10.5rem] -mt-4">
        <!-- Left column: Mid -->
        <div class="flex justify-center">
          <div class="scale-[0.75]">
            <Knob v-model="eqMid" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Mid" unit="dB" color="#f59e0b" />
          </div>
        </div>
        
        <!-- Right column: High and Low stacked -->
        <div class="flex flex-col -space-y-6">
          <div class="scale-[0.75]">
            <Knob v-model="eqHigh" :min="-12" :max="12" :step="0.5" :centerValue="0" label="High" unit="dB" color="#ef4444" />
          </div>
          <div class="scale-[0.75]">
            <Knob v-model="eqLow" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Low" unit="dB" color="#10b981" />
          </div>
        </div>
      </div>
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
        <Fader v-if="faderHeight > 0" v-model="volume" label="Volume" :trackHeight="faderHeight" color="blue" />
        <VuMeter v-if="faderHeight > 0" :level="trackLevel" :label="''" :height="faderHeight + 20" :width="12"
          :showValue="false" />
      </div>
    </div>

  </div>

  <!-- Parametric EQ Modal -->
  <ParametricEQModal v-model="showParametricEQ" :trackNumber="trackNumber" :eq-filters="eqFiltersData" @update="handleParametricEQUpdate" />

  <!-- FX Modals -->
  <Teleport to="body">
    <!-- Compressor Modal -->
    <div v-if="showCompressorModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      @click="showCompressorModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-xl font-bold text-green-300">Track {{ trackNumber }} - Compressor</h3>
          <button @click="showCompressorModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>
        <!-- Compression Curve Display -->
        <div class="mb-6 bg-black/50 rounded-lg p-4 border border-green-600/30">
          <p class="text-xs text-green-300 font-bold mb-2 text-center">COMPRESSION CURVE</p>
          <canvas ref="compressorCanvas" class="w-full rounded" style="height: 300px;"></canvas>
        </div>

        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <Knob v-model="compressorThreshold" :min="-60" :max="0" :step="0.5" label="Threshold" unit="dB"
            color="#10b981" @update:modelValue="updateCompressor" />
          <Knob v-model="compressorRatio" :min="1" :max="20" :step="0.1" label="Ratio" unit=":1" color="#10b981"
            @update:modelValue="updateCompressor" />
          <Knob v-model="compressorAttack" :min="0" :max="1" :step="0.01" label="Attack" unit="s" color="#10b981"
            @update:modelValue="updateCompressor" />
          <Knob v-model="compressorRelease" :min="0" :max="4" :step="0.01" label="Release" unit="s" color="#10b981"
            @update:modelValue="updateCompressor" />
        </div>
      </div>
    </div>

    <!-- Reverb Modal -->
    <div v-if="showReverbModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      @click="showReverbModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-xl font-bold text-green-300">Track {{ trackNumber }} - Reverb</h3>
          <button @click="showReverbModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>
        <div class="flex flex-wrap gap-4 justify-center">
          <Knob v-model="reverbDecay" :min="0.1" :max="10" :step="0.1" label="Decay" unit="s" color="#10b981"
            @update:modelValue="updateReverb" />
          <Knob v-model="reverbPreDelay" :min="0" :max="0.1" :step="0.001" label="Pre-Delay" unit="s" color="#f59e0b"
            @update:modelValue="updateReverb" />
          <Knob v-model="reverbWet" :min="0" :max="1" :step="0.01" label="Wet" unit="%" color="#06b6d4"
            @update:modelValue="updateReverb" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed, toRaw } from 'vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'
import type { TrackSnapshot } from '~/composables/useScenes'
import { PeakingFilter } from '~/lib/filters/peaking.class'
import { LowShelvingFilter } from '~/lib/filters/lowShelving.class'
import { HighShelvingFilter } from '~/lib/filters/highShelving.class'
import Fader from './Fader.vue'
import Knob from './Knob.vue'
import PanKnob from './PanKnob.vue'
import ParametricEQModal from './ParametricEQModal.vue'
import VuMeter from './VuMeter.vue'

defineOptions({
  inheritAttrs: false
})

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
const audioUrl = ref<string>('')
const isLoading = ref(false)
const showParametricEQ = ref(false)
const eqThumbnail = ref<HTMLCanvasElement | null>(null)
const waveformCanvas = ref<HTMLCanvasElement | null>(null)

// Audio source selection
const audioSourceType = ref<'file' | 'input' | 'youtube'>('file')

// Audio inputs from shared composable
const { audioInputDevices, refreshAudioInputs } = useAudioDevices()
const audioInputs = audioInputDevices // Use the shared ref

const selectedAudioInput = ref<string>('')
let audioInputStream: MediaStream | null = null
let audioInputSource: MediaStreamAudioSourceNode | null = null

// FX state
const showCompressorModal = ref(false)
const showReverbModal = ref(false)
const compressorEnabled = ref(false)
const reverbEnabled = ref(false)
const compressorThreshold = ref(-20)
const compressorRatio = ref(4)
const compressorAttack = ref(0.1)
const compressorRelease = ref(0.25)
const reverbDecay = ref(1.5)
const reverbPreDelay = ref(0.01)
const reverbWet = ref(0.3)

// Compressor visualization
const compressorCanvas = ref<HTMLCanvasElement | null>(null)
const currentInputLevel = ref(-60)
const currentGainReduction = ref(0)
let compressorAnimationId: number | null = null

// Filter calculators for thumbnail
const peakingCalculator = new PeakingFilter()
const lowShelvingCalculator = new LowShelvingFilter()
const highShelvingCalculator = new HighShelvingFilter()

// Store EQ filters data for thumbnail
const eqFiltersData = ref<any[]>([])

// EQ accordion state
const showEQ3Bands = ref(false)

// Audio controls
const volume = ref(0)
const gain = ref(0)
const pan = ref(0) // -1 (left) to +1 (right)
const eqLow = ref(0)
const eqMid = ref(0)
const eqHigh = ref(0)
const trackLevel = ref(-60)

// Tone.js nodes
let player: any = null // Can be Tone.Player or Tone.UserMedia
let currentAudioBuffer: AudioBuffer | null = null // Store current audio buffer for player recreation
let gainNode: any = null
let monoConverter: any = null // Convert stereo to mono before panning
let postFxMono: any = null // Ensure mono signal after effects (before pan split)
let eq3: any = null
let parametricEQFilters: any = null // Parametric EQ filter chain
let compressor: any = null
let reverb: any = null
let panLeftGain: any = null // Left channel gain for manual panning
let panRightGain: any = null // Right channel gain for manual panning
let nativeMerger: any = null // Native Web Audio API ChannelMerger
let panMerge: any = null // Tone.Gain wrapper for stereo output
let volumeNode: any = null
let meter: any = null
let waveform: any = null // Waveform analyzer
let waveformAnimationId: number | null = null
let resizeObserver: ResizeObserver | null = null

// DEBUG: Track player lifecycle
let playerCreatedCount = 0
let playerDisposedCount = 0

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
  // Import Tone.js dynamically on client side only
  Tone = await import('tone')

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

  // Draw initial empty EQ thumbnail
  drawEQThumbnail()
  
  // Initialize waveform display
  stopWaveformDrawing()

  // Start level monitoring
  startLevelMonitoring()

  // Listen for device changes and refresh the shared list
  navigator.mediaDevices.addEventListener('devicechange', refreshAudioInputs)
})

// Draw EQ curve thumbnail
function drawEQThumbnail() {
  if (!eqThumbnail.value) return

  const canvas = eqThumbnail.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()

  canvas.width = rect.width * dpr
  canvas.height = 40 * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = 40

  // Clear canvas
  ctx.fillStyle = '#111827' // bg-gray-900
  ctx.fillRect(0, 0, width, height)

  // Draw grid
  ctx.strokeStyle = 'rgba(75, 85, 99, 0.3)' // gray-600 with opacity
  ctx.lineWidth = 0.5

  // Horizontal lines (0dB center line more visible)
  for (let i = 0; i <= 2; i++) {
    const y = (height / 2) * i
    ctx.globalAlpha = i === 1 ? 0.5 : 0.3
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(width, y)
    ctx.stroke()
  }
  ctx.globalAlpha = 1

  // If no filters, just show flat line
  if (eqFiltersData.value.length === 0) {
    ctx.strokeStyle = '#6B7280' // gray-500
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()
    return
  }

  // Calculate convolution curve
  const points = 200
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)

  ctx.strokeStyle = '#FFFFFF'
  ctx.lineWidth = 1.5
  ctx.beginPath()

  for (let i = 0; i < points; i++) {
    const x = (i / points) * width
    const logFreq = minFreq + (i / points) * (maxFreq - minFreq)
    const freq = Math.pow(10, logFreq)

    // Calculate total gain at this frequency
    let totalGain = 0

    eqFiltersData.value.forEach(filter => {
      let gain = 0

      if (filter.type === 'peaking') {
        gain = peakingCalculator.computeResponseAtFrequency(
          freq,
          filter.frequency,
          filter.gain,
          filter.Q
        )
      } else if (filter.type === 'lowshelf') {
        gain = lowShelvingCalculator.computeResponseAtFrequency(
          freq,
          filter.frequency,
          filter.gain,
          filter.Q
        )
      } else if (filter.type === 'highshelf') {
        gain = highShelvingCalculator.computeResponseAtFrequency(
          freq,
          filter.frequency,
          filter.gain,
          filter.Q
        )
      } else if (filter.type === 'lowpass' || filter.type === 'highpass') {
        // For lowpass/highpass, we don't have calculators so skip for now
        gain = 0
      }

      totalGain += gain
    })

    // Map gain to y position (scale: -24dB to +24dB)
    const y = height / 2 - (totalGain * (height / 48))

    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }

  ctx.stroke()
}

// Draw waveform visualization
function drawWaveform() {
  if (!waveformCanvas.value || !waveform) {
    return
  }

  const canvas = waveformCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()

  // Set canvas size accounting for device pixel ratio
  canvas.width = rect.width * dpr
  canvas.height = 50 * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = 50

  // Clear canvas
  ctx.fillStyle = '#000000' // black background
  ctx.fillRect(0, 0, width, height)

  // Get waveform data
  const values = waveform.getValue()
  
  if (!values || values.length === 0) {
    return
  }

  // Draw waveform
  ctx.strokeStyle = '#22d3ee' // cyan-400
  ctx.lineWidth = 1.5
  ctx.beginPath()

  const sliceWidth = width / values.length
  let x = 0

  for (let i = 0; i < values.length; i++) {
    // Normalize value from -1,1 to 0,height
    const v = (values[i] + 1) / 2
    const y = v * height

    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }

    x += sliceWidth
  }

  ctx.stroke()

  // Draw center line
  ctx.strokeStyle = 'rgba(100, 116, 139, 0.3)' // gray-500 with opacity
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, height / 2)
  ctx.lineTo(width, height / 2)
  ctx.stroke()
}

// Start waveform animation loop
function startWaveformDrawing() {
 
  if (waveformAnimationId !== null) return

  function animate() {
    drawWaveform()
    waveformAnimationId = requestAnimationFrame(animate)
  }

  waveformAnimationId = requestAnimationFrame(animate)
}

// Stop waveform animation loop
function stopWaveformDrawing() {
  if (waveformAnimationId !== null) {
    cancelAnimationFrame(waveformAnimationId)
    waveformAnimationId = null
  }
  
  // Clear the canvas
  if (waveformCanvas.value) {
    const canvas = waveformCanvas.value
    const ctx = canvas.getContext('2d')
    if (ctx) {
      const rect = canvas.getBoundingClientRect()
      ctx.fillStyle = '#000000'
      ctx.fillRect(0, 0, rect.width, 50)
      
      // Draw center line on empty canvas
      ctx.strokeStyle = 'rgba(100, 116, 139, 0.3)'
      ctx.lineWidth = 1
      ctx.beginPath()
      ctx.moveTo(0, 25)
      ctx.lineTo(rect.width, 25)
      ctx.stroke()
    }
  }
}

// Initialize audio nodes (called on first use)
function initAudioNodes() {
  if (gainNode) return // Already initialized

  if (!Tone) {
    console.error('Tone.js not loaded')
    return
  }

  // Create audio nodes
  gainNode = new Tone.Gain(1) // 1 = 0dB (unity gain), not 0!
  
  // Convert stereo to mono (pan will create stereo from mono)
  monoConverter = new Tone.Mono()
  
  // Second mono converter AFTER effects (ensures mono before pan split)
  postFxMono = new Tone.Mono()
  const postFxMonoNode = postFxMono.output as AudioNode
  postFxMonoNode.channelCount = 1
  postFxMonoNode.channelCountMode = 'explicit'
  
  eq3 = new Tone.EQ3({
    low: 0,
    mid: 0,
    high: 0
  })
  
  // CRITICAL: Force EQ3 output to be MONO (1 channel)
  // This ensures when we duplicate to L/R gains, they get identical MONO signals
  const eq3OutputNode = eq3.output as AudioNode
  eq3OutputNode.channelCount = 1
  eq3OutputNode.channelCountMode = 'explicit'
  
  // Manual pan: duplicate mono signal to L/R gains, then merge to stereo
  panLeftGain = new Tone.Gain(0.707) // ~-3dB (equal power at center)
  panRightGain = new Tone.Gain(0.707) // ~-3dB (equal power at center)
  
  // CRITICAL: Force these gains to be MONO throughout (input AND output)
  // This ensures ChannelMerger receives true mono inputs on each channel
  const leftGainInputNode = panLeftGain.input as GainNode
  leftGainInputNode.channelCount = 1
  leftGainInputNode.channelCountMode = 'explicit'
  
  const leftGainOutputNode = panLeftGain.output as GainNode
  leftGainOutputNode.channelCount = 1
  leftGainOutputNode.channelCountMode = 'explicit'
  
  const rightGainInputNode = panRightGain.input as GainNode
  rightGainInputNode.channelCount = 1
  rightGainInputNode.channelCountMode = 'explicit'
  
  const rightGainOutputNode = panRightGain.output as GainNode
  rightGainOutputNode.channelCount = 1
  rightGainOutputNode.channelCountMode = 'explicit'
  
  // Use native Web Audio API ChannelMerger (Tone.Merge collapses to mono)
  // ChannelMerger with 2 inputs automatically produces 2 channel output
  nativeMerger = Tone.context.createChannelMerger(2)
  
  // Wrap in Tone.Gain to interface with Tone.js ecosystem
  // CRITICAL: Must configure to preserve stereo from ChannelMerger
  panMerge = new Tone.Gain(1.0)
  const panMergeNode = panMerge.input as GainNode
  panMergeNode.channelCount = 2
  panMergeNode.channelCountMode = 'explicit'
  panMergeNode.channelInterpretation = 'discrete'
  
  volumeNode = new Tone.Volume(0)
  const volumeInputNode = volumeNode.input as GainNode
  volumeInputNode.channelCount = 2
  volumeInputNode.channelCountMode = 'explicit'
  volumeInputNode.channelInterpretation = 'discrete'
  
  meter = new Tone.Meter()
  
  // Waveform analyzer (for visualization)
  waveform = new Tone.Waveform(512) // 512 samples for waveform display

  // Connect chain: gain -> mono -> eq3 -> meter (pre-pan) -> postFxMono -> [dup to L/R gains] -> nativeMerger -> panMerge -> volume
  gainNode.connect(monoConverter)
  monoConverter.connect(eq3)
  
  // Connect meter to eq3 to measure signal BEFORE panning
  eq3.connect(meter)
  
  // Connect waveform analyzer to eq3 for visualization
  eq3.connect(waveform)
  
  // Initial pan connection (will be rebuilt when effects are added)
  // Always go through postFxMono to ensure mono before split
  eq3.connect(postFxMono)
  postFxMono.connect(panLeftGain)
  postFxMono.connect(panRightGain)
  panLeftGain.connect(nativeMerger, 0, 0)
  panRightGain.connect(nativeMerger, 0, 1)
  
  // Connect native merger to Tone.Gain wrapper
  nativeMerger.connect(panMerge.input)
  
  // panMerge to volume
  panMerge.connect(volumeNode)

  // Volume to output (master or destination)
  connectToOutput()
}

function applyParametricEQChain() {
  if (!parametricEQFilters || !eq3 || !panMerge || !meter) return

  // Use rebuildAudioChain to properly handle FX chain
  rebuildAudioChain()
}

// Handle parametric EQ update
function handleParametricEQUpdate(filters: any) {
  if (!filters) return

  // Store the latest filter chain
  parametricEQFilters = filters

  // Try to apply immediately (may no-op if audio chain not ready yet)
  applyParametricEQChain()

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
    drawEQThumbnail()
  }
}

// Connect to output (ONLY to master, not destination)
function connectToOutput() {
  if (!volumeNode || !Tone) return

  // Extract the masterOutput from the masterSection ref
  let master = props.masterChannel?.masterOutput?.value || props.masterChannel?.masterOutput || props.masterChannel?.value || props.masterChannel

  // Extract raw object from Vue Proxy (important for Tone.js!)
  if (master) {
    master = toRaw(master)
  }

  if (master) {
    try {
      // First disconnect from everything
      try {
        volumeNode.disconnect()
      } catch (e) {
        // Ignore
      }

      // Reconnect to meter
      volumeNode.connect(meter!)

      // Connect to master (now using raw object)
      volumeNode.connect(master)
      return true
    } catch (e) {
      console.error(`Track ${props.trackNumber}: Error connecting to master:`, e)
      // Fallback to destination if master fails
      volumeNode.connect(meter!)
      volumeNode.toDestination()
      return false
    }
  } else {
    // Temporary connection to destination until master is ready
    volumeNode.toDestination()
    console.warn(`Track ${props.trackNumber}: ⚠️ No master available, connected to DESTINATION temporarily`)
    return false
  }
}

// Computed to track the actual master value
const masterValue = computed(() => {
  const mc = props.masterChannel
  return mc?.masterOutput?.value || mc?.masterOutput || mc?.value || mc
})

// Watch for master value changes
watch(masterValue, (newMaster) => {
  if (newMaster && volumeNode && Tone) {
    connectToOutput()
  }
})

// Level monitoring
let levelMonitorInterval: number | null = null
function startLevelMonitoring() {
  levelMonitorInterval = window.setInterval(() => {
    if (meter && Tone) {
      const level = meter.getValue() as number
      trackLevel.value = Math.max(-60, level)
    }
  }, 50)
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

    // Stop and COMPLETELY disconnect old player
    if (player) {
      
      isPlaying.value = false
      stopWaveformDrawing()
      
      // CRITICAL: Only stop if it's a Tone.Player (has stop method)
      // For audio input, player is a Tone.Gain which doesn't have stop()
      if (typeof player.stop === 'function') {
        player.loop = false
        player.stop()
      }
      
      // Unsync from transport if synced
      if (player.sync && typeof player.unsync === 'function') {
        try {
          player.unsync()
        } catch (e) {}
      }
      
      // Disconnect completely
      player.disconnect()
      
      // Dispose player
      player.dispose()
      playerDisposedCount++
      player = null   
    }
    
    // CRITICAL: Instead of just recreating gainNode, REBUILD ENTIRE AUDIO CHAIN
    // This prevents any lingering connections or state issues
    if (gainNode) {
      
      // Disconnect and dispose ALL nodes
      try {
        gainNode.disconnect()
        gainNode.dispose()
      } catch (e) {}
      
      try {
        monoConverter.disconnect()
        monoConverter.dispose()
      } catch (e) {}
      
      try {
        eq3.disconnect()
        eq3.dispose()
      } catch (e) {}
      
      try {
        postFxMono.disconnect()  
        postFxMono.dispose()
      } catch (e) {}
      
      try {
        panLeftGain.disconnect()
        panLeftGain.dispose()
      } catch (e) {}
      
      try {
        panRightGain.disconnect()
        panRightGain.dispose()
      } catch (e) {}
      
      try {
        panMerge.disconnect()
        panMerge.dispose()
      } catch (e) {}
      
      // Recreate all nodes
      gainNode = new Tone.Gain(1)
      monoConverter = new Tone.Mono()
      
      postFxMono = new Tone.Mono()
      const postFxMonoNode = postFxMono.output as AudioNode
      postFxMonoNode.channelCount = 1
      postFxMonoNode.channelCountMode = 'explicit'
      
      eq3 = new Tone.EQ3({ low: 0, mid: 0, high: 0 })
      const eq3OutputNode = eq3.output as AudioNode
      eq3OutputNode.channelCount = 1
      eq3OutputNode.channelCountMode = 'explicit'
      
      panLeftGain = new Tone.Gain(0.707)
      panRightGain = new Tone.Gain(0.707)
      
      const leftGainInputNode = panLeftGain.input as GainNode
      leftGainInputNode.channelCount = 1
      leftGainInputNode.channelCountMode = 'explicit'
      const leftGainOutputNode = panLeftGain.output as GainNode
      leftGainOutputNode.channelCount = 1
      leftGainOutputNode.channelCountMode = 'explicit'
      
      const rightGainInputNode = panRightGain.input as GainNode
      rightGainInputNode.channelCount = 1
      rightGainInputNode.channelCountMode = 'explicit'
      const rightGainOutputNode = panRightGain.output as GainNode
      rightGainOutputNode.channelCount = 1
      rightGainOutputNode.channelCountMode = 'explicit'
      
      // Native merger doesn't need to be recreated (it's raw Web Audio API)
      // But let's do it anyway
      nativeMerger = Tone.context.createChannelMerger(2)
      
      panMerge = new Tone.Gain(1.0)
      const panMergeNode = panMerge.input as GainNode
      panMergeNode.channelCount = 2
      panMergeNode.channelCountMode = 'explicit'
      panMergeNode.channelInterpretation = 'discrete'
      
      // Reconnect the entire chain
      gainNode.connect(monoConverter)
      monoConverter.connect(eq3)
      eq3.connect(meter)
      eq3.connect(waveform)
      eq3.connect(postFxMono)
      postFxMono.connect(panLeftGain)
      postFxMono.connect(panRightGain)
      panLeftGain.connect(nativeMerger, 0, 0)
      panRightGain.connect(nativeMerger, 0, 1)
      nativeMerger.connect(panMerge.input)
      panMerge.connect(volumeNode)
      
      // Reconnect to master/destination
      connectToOutput()
    }

    // Longer delay to ensure Tone.js internals are completely cleaned
    await new Promise(resolve => setTimeout(resolve, 150))
    
    // Create buffer from file
    const arrayBuffer = await file.arrayBuffer()
    const audioBuffer = await Tone.context.decodeAudioData(arrayBuffer)
    
    // Store buffer and create new player
    currentAudioBuffer = audioBuffer
    player = new Tone.Player(audioBuffer)

    // Verify audio chain is connected
    if (!gainNode || !eq3 || !volumeNode) {
      alert('Audio system not ready. Please refresh the page.')
      isLoading.value = false
      return
    }

    player.connect(gainNode)
    player.loop = true
    playerCreatedCount++
    
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

    // Stop and dispose old player if exists
    if (player) {
      isPlaying.value = false
      stopWaveformDrawing()
      if (typeof player.stop === 'function') {
        player.stop()
      }
      player.disconnect()
      player.dispose()
      player = null
    }
    
    currentAudioBuffer = null

    // Small delay to ensure clean state
    await new Promise(resolve => setTimeout(resolve, 100))

    // Decode audio buffer
    const audioBuffer = await Tone.context.decodeAudioData(storedFile.arrayBuffer)
    
    // Store buffer and create new player
    currentAudioBuffer = audioBuffer
    player = new Tone.Player(audioBuffer)

    // Verify audio chain is connected
    if (!gainNode || !eq3 || !volumeNode) {
      alert('Audio system not ready. Please refresh the page.')
      isLoading.value = false
      return
    }

    player.connect(gainNode)
    player.loop = true

    audioLoaded.value = true
    isLoading.value = false

    // Rebuild audio chain only if parametric EQ filters exist (from saved scene)
    if (parametricEQFilters) {
      applyParametricEQChain()
    }

    // Force DOM update
    await nextTick()
  } catch (error) {
    console.error('❌ Error loading audio file from IndexedDB:', error)
    alert('Error loading audio file from scene: ' + error)
    isLoading.value = false
  }
}

// Helper to check if URL is YouTube
function isYouTubeUrl(url: string): boolean {
  return /(?:youtube\.com|youtu\.be)/.test(url)
}

// URL handler (supports YouTube and direct audio URLs)
async function handleUrlLoad() {
  if (!audioUrl.value || !Tone) return

  // Initialize audio nodes on first use
  initAudioNodes()

  isLoading.value = true

  try {
    // Stop and dispose old player if exists
    if (player) {
      isPlaying.value = false
      stopWaveformDrawing()
      if (typeof player.stop === 'function') {
        player.stop()
      }
      player.disconnect()
      player.dispose()
      player = null
    }
    
    currentAudioBuffer = null

    // Small delay to ensure clean state
    await new Promise(resolve => setTimeout(resolve, 100))

    let finalUrl = audioUrl.value
    let audioTitle = 'Audio from URL'

    // Check if it's a YouTube URL
    if (isYouTubeUrl(audioUrl.value)) {
      try {
        // Get YouTube info
        const infoResponse = await $fetch<any>('/api/youtube', {
          query: { url: audioUrl.value }
        })

        if (infoResponse.success) {
          audioTitle = infoResponse.title || 'YouTube Audio'
          
          // Use the server as a proxy - the server will stream the audio from YouTube
          finalUrl = `/api/youtube-stream?url=${encodeURIComponent(audioUrl.value)}`
        } else {
          throw new Error('YouTube API failed')
        }
      } catch (ytError: any) {
        console.error('[YouTube Client] Loading failed:', ytError)
        alert('Failed to load YouTube video: ' + (ytError.message || ytError))
        isLoading.value = false
        audioLoaded.value = false
        return
      }
    }

    // Create new player from URL
    player = new Tone.Player({
      url: finalUrl,
      onload: () => {
        audioLoaded.value = true
        fileName.value = audioTitle
        isLoading.value = false
      },
      onerror: (error: any) => {
        console.error('Error loading audio:', error)
        alert('Error loading audio. Make sure the URL is accessible and in a supported format (MP3, WAV, OGG).')
        isLoading.value = false
        audioLoaded.value = false
      }
    }).connect(gainNode!)

    player.loop = true

  } catch (error: any) {
    console.error('URL load error:', error)
    alert('Failed to load audio. Please check the URL and try again.')
    isLoading.value = false
    audioLoaded.value = false
  }
}

// Handle source type change
function handleSourceTypeChange() {
  // Stop any current playback
  stopAudio()
  
  // Stop waveform visualization
  stopWaveformDrawing()
  
  // Clean up current source
  if (player) {
    try {
      player.disconnect()
      player.dispose()
    } catch (e) {}
    player = null
  }
  
  // Clear audio buffer
  currentAudioBuffer = null
  
  // Disconnect and clean up audio input source
  if (audioInputSource) {
    try {
      audioInputSource.disconnect()
    } catch (e) {}
    audioInputSource = null
  }
  
  if (audioInputStream) {
    audioInputStream.getTracks().forEach(track => track.stop())
    audioInputStream = null
  }
  
  // CRITICAL: Reset all audio nodes to ensure clean state when switching source types
  // This prevents issues when switching from microphone to file or vice versa
  if (gainNode) {
    try {
      gainNode.disconnect()
      gainNode.dispose()
    } catch (e) {}
    gainNode = null
  }
  
  if (monoConverter) {
    try {
      monoConverter.disconnect()
      monoConverter.dispose()
    } catch (e) {}
    monoConverter = null
  }
  
  if (eq3) {
    try {
      eq3.disconnect()
      eq3.dispose()
    } catch (e) {}
    eq3 = null
  }
  
  if (postFxMono) {
    try {
      postFxMono.disconnect()
      postFxMono.dispose()
    } catch (e) {}
    postFxMono = null
  }
  
  if (panLeftGain) {
    try {
      panLeftGain.disconnect()
      panLeftGain.dispose()
    } catch (e) {}
    panLeftGain = null
  }
  
  if (panRightGain) {
    try {
      panRightGain.disconnect()
      panRightGain.dispose()
    } catch (e) {}
    panRightGain = null
  }
  
  if (panMerge) {
    try {
      panMerge.disconnect()
      panMerge.dispose()
    } catch (e) {}
    panMerge = null
  }
  
  if (volumeNode) {
    try {
      volumeNode.disconnect()
      volumeNode.dispose()
    } catch (e) {}
    volumeNode = null
  }
  
  if (meter) {
    try {
      meter.disconnect()
      meter.dispose()
    } catch (e) {}
    meter = null
  }
  
  if (waveform) {
    try {
      waveform.disconnect()
      waveform.dispose()
    } catch (e) {}
    waveform = null
  }
  
  // Reset FX nodes
  if (compressor) {
    try {
      compressor.disconnect()
      compressor.dispose()
    } catch (e) {}
    compressor = null
  }
  
  if (reverb) {
    try {
      reverb.disconnect()
      reverb.dispose()
    } catch (e) {}
    reverb = null
  }
  
  if (parametricEQFilters) {
    try {
      parametricEQFilters.disconnect()
      parametricEQFilters.dispose()
    } catch (e) {}
    parametricEQFilters = null
  }
  
  audioLoaded.value = false
  isPlaying.value = false
  fileName.value = ''
  fileId.value = ''
  selectedAudioInput.value = ''
}

// Handle audio input device change
async function handleAudioInputChange() {
  if (!selectedAudioInput.value || !Tone) return
  
  console.log(`[Track ${props.trackNumber}] handleAudioInputChange called for device:`, selectedAudioInput.value)
  
  // Initialize audio nodes if needed
  initAudioNodes()
  
  if (!gainNode) {
    console.error(`[Track ${props.trackNumber}] gainNode failed to initialize!`)
    return
  }
  
  try {
    // Ensure audio context is running
    await Tone.start()
    
    // Stop previous input stream if any
    if (audioInputStream) {
      audioInputStream.getTracks().forEach(track => track.stop())
    }
    
    // Dispose old player/source
    if (player) {
      try {
        player.disconnect()
        player.dispose()
      } catch (e) {}
      player = null
    }
    
    // Disconnect old audio input source
    if (audioInputSource) {
      try {
        audioInputSource.disconnect()
      } catch (e) {}
      audioInputSource = null
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
    
    // Wrap it in a Tone.js Gain node so we can use Tone's connect methods
    player = new Tone.Gain(1)
    
    // Connect the native media stream source to Tone's input node
    // player.input is the underlying Web Audio API node that Tone wraps
    if (audioInputSource) {
      audioInputSource.connect(player.input)
    }
    
    // Connect to audio chain
    player.connect(gainNode!)
    
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
    startWaveformDrawing()
    
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
  
  // For audio input, play/pause doesn't make sense - it's always live
  if (audioSourceType.value === 'input') {
    if (audioLoaded.value) {
      // Toggle mute instead
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
    await Tone.start()

    
    // Ensure master audio elements are playing
    if (props.masterChannel?.ensureAudioPlaying) {
      props.masterChannel.ensureAudioPlaying()
    }
    
    // CRITICAL: If player is already started (from previous loop), stop it first
    if (player.state === 'started' && typeof player.stop === 'function') {
      player.stop()
      // Small delay to ensure clean stop
      await new Promise(resolve => setTimeout(resolve, 50))
    }
    
    
    if (typeof player.start === 'function') {
      player.start()
    }
    isPlaying.value = true
    
    // Start waveform (disabled)
    startWaveformDrawing()
  } else {
    if (typeof player.stop === 'function') {
      player.stop()
    }
    isPlaying.value = false
    
    // Stop waveform (disabled)
    stopWaveformDrawing()
  }
}

function stopAudio() {
  // For audio input, we can't really "stop" - mute instead
  if (audioSourceType.value === 'input') {
    if (!isMuted.value) {
      toggleMute()
    }
    return
  }
  
  // For file playback
  if (!player || !audioLoaded.value) return
    
  if (typeof player.stop === 'function') {
    player.stop()
  }
  isPlaying.value = false
    
  // Stop waveform (disabled)
  stopWaveformDrawing()
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
  if (!volumeNode) return

  if (isMuted.value) {
    volumeNode.volume.value = -Infinity
    // Also mute the pan gains
    if (panLeftGain) panLeftGain.gain.value = 0
    if (panRightGain) panRightGain.gain.value = 0
  } else {
    volumeNode.volume.value = volume.value
    // Restore pan
    updatePan()
  }
}

function updateGain() {
  if (!gainNode || !Tone) return
  gainNode.gain.value = Tone.dbToGain(gain.value)
}

function updateEQ() {
  if (!eq3) return
  eq3.low.value = eqLow.value
  eq3.mid.value = eqMid.value
  eq3.high.value = eqHigh.value
}

function updatePan() {
  if (!panLeftGain || !panRightGain) return
  
  if (isMuted.value) {
    panLeftGain.gain.value = 0
    panRightGain.gain.value = 0
    return
  }
  
  const panValue = pan.value // -1 to +1
  
  // Constant power panning
  // -1 = full left: L=1, R=0
  //  0 = center: L=0.707, R=0.707 (equal power)
  // +1 = full right: L=0, R=1
  const theta = (panValue + 1) * Math.PI / 4 // Map -1..1 to 0..PI/2
  const leftGain = Math.cos(theta)
  const rightGain = Math.sin(theta)
  
  panLeftGain.gain.value = leftGain
  panRightGain.gain.value = rightGain
}

// Watch for parameter changes
watch(volume, updateVolume)
watch(gain, updateGain)
watch(pan, updatePan)
watch([eqLow, eqMid, eqHigh], updateEQ)

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
      youtubeURL: audioSourceType.value === 'youtube' ? audioUrl.value : undefined,
      selectedInputDevice: audioSourceType.value === 'input' ? selectedAudioInput.value : undefined,
      fileName: audioSourceType.value === 'file' ? fileName.value : undefined,
      fileId: audioSourceType.value === 'file' ? fileId.value : undefined,
      parametricEQFilters: eqFiltersData.value.map(f => ({
        id: f.id,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      })),
      compressorEnabled: compressorEnabled.value,
      compressorThreshold: compressorThreshold.value,
      compressorRatio: compressorRatio.value,
      compressorAttack: compressorAttack.value,
      compressorRelease: compressorRelease.value,
      reverbEnabled: reverbEnabled.value,
      reverbDecay: reverbDecay.value,
      reverbPreDelay: reverbPreDelay.value,
      reverbWet: reverbWet.value
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
    if (snapshot.youtubeURL) {
      audioUrl.value = snapshot.youtubeURL
      // Auto-reload YouTube URL
      nextTick(() => {
        handleUrlLoad()
      })
    }
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
    compressorThreshold.value = snapshot.compressorThreshold || -20
    compressorRatio.value = snapshot.compressorRatio || 4
    compressorAttack.value = snapshot.compressorAttack || 0.1
    compressorRelease.value = snapshot.compressorRelease || 0.25
    if (shouldEnableCompressor !== compressorEnabled.value) {
      toggleCompressor()
    }
    
    // Restore reverb
    const shouldEnableReverb = snapshot.reverbEnabled || false
    reverbDecay.value = snapshot.reverbDecay || 1.5
    reverbPreDelay.value = snapshot.reverbPreDelay || 0.01
    reverbWet.value = snapshot.reverbWet || 0.3
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
    } catch (e) {}
    audioInputSource = null
  }
  
  // Stop audio input stream if active
  if (audioInputStream) {
    audioInputStream.getTracks().forEach(track => track.stop())
    audioInputStream = null
  }
  
  // Cleanup player
  if (player) {
    player.dispose()
  }
  if (gainNode) gainNode.dispose()
  if (monoConverter) monoConverter.dispose()
  if (postFxMono) postFxMono.dispose()
  if (eq3) eq3.dispose()
  if (panLeftGain) panLeftGain.dispose()
  if (panRightGain) panRightGain.dispose()
  if (panMerge) panMerge.dispose()
  // nativeMerger is Web Audio API node, not Tone.js - no dispose needed
  if (volumeNode) volumeNode.dispose()
  if (meter) meter.dispose()
  if (waveform) waveform.dispose()
  if (compressor) compressor.dispose()
  if (reverb) reverb.dispose()

  if (resizeObserver) {
    resizeObserver.disconnect()
  }

  if (levelMonitorInterval) {
    clearInterval(levelMonitorInterval)
  }

  // Stop compressor monitoring
  stopCompressorMonitoring()
  
  // Stop waveform drawing
  stopWaveformDrawing()
  
  // Remove device change listener
  navigator.mediaDevices.removeEventListener('devicechange', refreshAudioInputs)
})

// FX Functions
function toggleCompressor() {
  compressorEnabled.value = !compressorEnabled.value

  if (compressorEnabled.value) {
    createCompressor()
  } else {
    removeCompressor()
  }
}

function toggleReverb() {
  reverbEnabled.value = !reverbEnabled.value

  if (reverbEnabled.value) {
    createReverb()
  } else {
    removeReverb()
  }
}

function createCompressor() {
  if (!Tone || compressor) return

  compressor = new Tone.Compressor({
    threshold: compressorThreshold.value,
    ratio: compressorRatio.value,
    attack: compressorAttack.value,
    release: compressorRelease.value
  })

  rebuildAudioChain()
}

function removeCompressor() {
  if (compressor) {
    compressor.disconnect()
    compressor.dispose()
    compressor = null
    rebuildAudioChain()
  }
}

function createReverb() {
  if (!Tone || reverb) return

  reverb = new Tone.Reverb({
    decay: reverbDecay.value,
    preDelay: reverbPreDelay.value,
    wet: reverbWet.value
  })

  rebuildAudioChain()
}

function removeReverb() {
  if (reverb) {
    reverb.disconnect()
    reverb.dispose()
    reverb = null
    rebuildAudioChain()
  }
}

function updateCompressor() {
  if (!compressor) return

  // Use parameter ramping to prevent audio spikes
  const rampTime = 0.05 // 50ms smooth transition
  compressor.threshold.rampTo(compressorThreshold.value, rampTime)
  compressor.ratio.rampTo(compressorRatio.value, rampTime)
  compressor.attack.rampTo(compressorAttack.value, rampTime)
  compressor.release.rampTo(compressorRelease.value, rampTime)

  // Redraw curve if modal is open
  if (showCompressorModal.value) {
    nextTick(() => drawTrackCompressionCurve())
  }
}

function updateReverb() {
  if (!reverb) return

  // Use parameter ramping for smooth changes
  const rampTime = 0.05 // 50ms
  reverb.decay = reverbDecay.value // Decay can't be ramped, it's a constructor property
  reverb.preDelay = reverbPreDelay.value // PreDelay can't be ramped either
  reverb.wet.rampTo(reverbWet.value, rampTime)
}

// Track compressor visualization
watch(showCompressorModal, (isOpen) => {
  if (isOpen) {
    startCompressorMonitoring()
    nextTick(() => drawTrackCompressionCurve())
  } else {
    stopCompressorMonitoring()
  }
})

function startCompressorMonitoring() {
  if (compressorAnimationId) return

  function updateTrackLevels() {
    if (!showCompressorModal.value || !meter) {
      compressorAnimationId = null
      return
    }

    try {
      // Get track-specific meter values
      const leftLevel = meter.getLevel('left')
      const rightLevel = meter.getLevel('right')
      const avgLevel = (leftLevel + rightLevel) / 2
      currentInputLevel.value = Math.max(-60, Math.min(0, avgLevel))
    } catch (error) {
      currentInputLevel.value = -60
    }

    // Calculate gain reduction for this track
    const inputDb = currentInputLevel.value
    if (inputDb > compressorThreshold.value && compressorEnabled.value) {
      const excess = inputDb - compressorThreshold.value
      const reducedExcess = excess / compressorRatio.value
      currentGainReduction.value = excess - reducedExcess
    } else {
      currentGainReduction.value = 0
    }

    drawTrackCompressionCurve()
    compressorAnimationId = requestAnimationFrame(updateTrackLevels)
  }

  updateTrackLevels()
}

function stopCompressorMonitoring() {
  if (compressorAnimationId) {
    cancelAnimationFrame(compressorAnimationId)
    compressorAnimationId = null
  }
}

function drawTrackCompressionCurve() {
  if (!compressorCanvas.value || !showCompressorModal.value) return

  const canvas = compressorCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // Get display size and scale for high DPI
  const rect = canvas.getBoundingClientRect()
  const width = rect.width
  const height = rect.height
  const dpr = window.devicePixelRatio || 1

  canvas.width = width * dpr
  canvas.height = height * dpr
  ctx.scale(dpr, dpr)

  // Clear canvas
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)

  const padding = 40
  const graphWidth = width - padding * 2
  const graphHeight = height - padding * 2

  // Helper functions for coordinate mapping
  const dbToX = (db: number) => padding + ((db + 60) / 60) * graphWidth
  const dbToY = (db: number) => height - padding - ((db + 60) / 60) * graphHeight

  // Draw grid
  ctx.strokeStyle = '#333333'
  ctx.lineWidth = 1
  for (let db = -60; db <= 0; db += 10) {
    // Vertical lines
    ctx.beginPath()
    ctx.moveTo(dbToX(db), padding)
    ctx.lineTo(dbToX(db), height - padding)
    ctx.stroke()

    // Horizontal lines
    ctx.beginPath()
    ctx.moveTo(padding, dbToY(db))
    ctx.lineTo(width - padding, dbToY(db))
    ctx.stroke()
  }

  // Draw axes
  ctx.strokeStyle = '#666666'
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(padding, padding)
  ctx.lineTo(padding, height - padding)
  ctx.lineTo(width - padding, height - padding)
  ctx.stroke()

  // Axis labels
  ctx.fillStyle = '#999999'
  ctx.font = '10px monospace'
  ctx.textAlign = 'center'

  // X-axis labels
  for (let db = -60; db <= 0; db += 20) {
    ctx.fillText(`${db}`, dbToX(db), height - padding + 15)
  }
  ctx.fillText('Input (dB)', width / 2, height - 5)

  // Y-axis labels
  ctx.textAlign = 'right'
  for (let db = -60; db <= 0; db += 20) {
    ctx.fillText(`${db}`, padding - 5, dbToY(db) + 3)
  }
  ctx.save()
  ctx.translate(12, height / 2)
  ctx.rotate(-Math.PI / 2)
  ctx.textAlign = 'center'
  ctx.fillText('Output (dB)', 0, 0)
  ctx.restore()

  // Draw 1:1 reference line (no compression)
  ctx.strokeStyle = '#555555'
  ctx.lineWidth = 1
  ctx.setLineDash([5, 5])
  ctx.beginPath()
  ctx.moveTo(dbToX(-60), dbToY(-60))
  ctx.lineTo(dbToX(0), dbToY(0))
  ctx.stroke()
  ctx.setLineDash([])

  // Draw compression curve
  ctx.strokeStyle = '#10b981'
  ctx.lineWidth = 3
  ctx.beginPath()

  const knee = 6
  const thresholdValue = compressorThreshold.value
  const ratioValue = compressorRatio.value

  for (let inputDb = -60; inputDb <= 0; inputDb += 0.5) {
    let outputDb: number

    if (inputDb < thresholdValue - knee / 2) {
      outputDb = inputDb
    } else if (inputDb > thresholdValue + knee / 2) {
      const excess = inputDb - thresholdValue
      outputDb = thresholdValue + excess / ratioValue
    } else {
      const kneeInput = inputDb - thresholdValue + knee / 2
      const kneeRatio = kneeInput / knee
      const excess = inputDb - thresholdValue
      outputDb = inputDb + ((excess / ratioValue) - excess) * kneeRatio
    }

    const x = dbToX(inputDb)
    const y = dbToY(outputDb)

    if (inputDb === -60) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }

  ctx.stroke()

  // Draw threshold line
  ctx.strokeStyle = '#ef4444'
  ctx.lineWidth = 2
  ctx.setLineDash([3, 3])
  ctx.beginPath()
  ctx.moveTo(dbToX(thresholdValue), padding)
  ctx.lineTo(dbToX(thresholdValue), height - padding)
  ctx.stroke()
  ctx.setLineDash([])

  // Threshold and ratio labels
  ctx.fillStyle = '#ef4444'
  ctx.font = 'bold 11px sans-serif'
  ctx.textAlign = 'center'
  ctx.fillText(`Threshold: ${thresholdValue}dB`, dbToX(thresholdValue), padding - 10)

  ctx.fillStyle = '#10b981'
  ctx.textAlign = 'left'
  ctx.fillText(`Ratio: ${ratioValue.toFixed(1)}:1`, width - padding - 80, padding + 15)

  // Draw realtime signal indicator
  if (currentInputLevel.value > -60) {
    const inputDb = currentInputLevel.value

    // Calculate output level with compression
    let outputDb = inputDb
    if (compressorEnabled.value) {
      if (inputDb > thresholdValue - knee / 2) {
        if (inputDb > thresholdValue + knee / 2) {
          const excess = inputDb - thresholdValue
          outputDb = thresholdValue + excess / ratioValue
        } else {
          const kneeInput = inputDb - thresholdValue + knee / 2
          const kneeRatio = kneeInput / knee
          const excess = inputDb - thresholdValue
          outputDb = inputDb + ((excess / ratioValue) - excess) * kneeRatio
        }
      }
    }

    // Draw input level marker
    ctx.strokeStyle = '#fbbf24'
    ctx.lineWidth = 2
    ctx.setLineDash([2, 2])
    ctx.beginPath()
    ctx.moveTo(dbToX(inputDb), padding)
    ctx.lineTo(dbToX(inputDb), height - padding)
    ctx.stroke()
    ctx.setLineDash([])

    // Draw signal point on curve
    ctx.fillStyle = '#fbbf24'
    ctx.beginPath()
    ctx.arc(dbToX(inputDb), dbToY(outputDb), 5, 0, 2 * Math.PI)
    ctx.fill()

    // Add glow
    ctx.shadowBlur = 15
    ctx.shadowColor = '#fbbf24'
    ctx.beginPath()
    ctx.arc(dbToX(inputDb), dbToY(outputDb), 5, 0, 2 * Math.PI)
    ctx.fill()
    ctx.shadowBlur = 0

    // Gain reduction display
    if (compressorEnabled.value && currentGainReduction.value > 0.1) {
      ctx.fillStyle = '#fbbf24'
      ctx.font = 'bold 12px sans-serif'
      ctx.textAlign = 'right'
      ctx.fillText(`GR: -${currentGainReduction.value.toFixed(1)}dB`, width - padding - 10, padding + 35)
    }
  }
}

// Watch for parameter changes to redraw curve
watch([compressorThreshold, compressorRatio, compressorAttack, compressorRelease], () => {
  if (showCompressorModal.value) {
    nextTick(() => drawTrackCompressionCurve())
  }
})

function rebuildAudioChain() {
  if (!eq3 || !panLeftGain || !panRightGain || !panMerge || !volumeNode || !postFxMono || !nativeMerger) return

  // Disconnect everything
  try {
    eq3.disconnect()
    if (parametricEQFilters) parametricEQFilters.output.disconnect()
    if (compressor) compressor.disconnect()
    if (reverb) reverb.disconnect()
    if (postFxMono) postFxMono.disconnect()
    panLeftGain.disconnect()
    panRightGain.disconnect()
  } catch (e) {
    // Ignore disconnection errors
  }

  // Build new chain: eq3 -> [parametricEQ] -> [compressor] -> [reverb] -> postFxMono -> [dup to L/R gains] -> merge -> volume
  let currentNode: any = eq3

  // CRITICAL: Reconnect meter and waveform to eq3 (they were disconnected above)
  if (meter) eq3.connect(meter)
  if (waveform) eq3.connect(waveform)

  // Parametric EQ
  if (parametricEQFilters && parametricEQFilters.input && parametricEQFilters.output) {
    currentNode.connect(parametricEQFilters.input)
    currentNode = parametricEQFilters.output
  }

  // Compressor
  if (compressor) {
    currentNode.connect(compressor)
    currentNode = compressor
  }

  // Reverb
  if (reverb) {
    currentNode.connect(reverb)
    currentNode = reverb
  }
  
  // CRITICAL: Force mono AFTER all effects (effects might produce stereo)
  // This ensures we split a true MONO signal to L/R gains
  currentNode.connect(postFxMono)
  currentNode = postFxMono

  // Manual pan: duplicate MONO to L/R gains -> native merger -> wrapper
  currentNode.connect(panLeftGain)
  currentNode.connect(panRightGain)
  panLeftGain.connect(nativeMerger, 0, 0)
  panRightGain.connect(nativeMerger, 0, 1)

  // panMerge to volume (already connected in initAudioNodes)
  // connectToOutput() already called in initAudioNodes
}
</script>
