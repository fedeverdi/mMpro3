<template>
  <div
    ref="trackElement"
    class="track-channel relative bg-gray-700 rounded-lg border border-gray-900 p-1 flex flex-col items-center gap-1 h-full">
    
    <!-- Track Header -->
    <div class="w-full flex items-center justify-between gap-1">
      <div class="text-xs font-bold text-gray-300 flex-1 text-center">Track {{ trackNumber }}</div>
      <button 
        @click="$emit('remove')" 
        class="w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
        title="Remove Track"
      >
        ×
      </button>
    </div>

    <!-- Signal Selector Buttons -->
    <div class="w-full flex flex-col gap-0.5">
      <button @click="selectSignal('sine')"
        :class="selectedSignal === 'sine' ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        class="w-full py-1 text-[0.65rem] rounded transition-colors">
        Sine
      </button>
      <button @click="selectSignal('square')"
        :class="selectedSignal === 'square' ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        class="w-full py-1 text-[0.65rem] rounded transition-colors">
        Square
      </button>
      <button @click="selectSignal('sawtooth')"
        :class="selectedSignal === 'sawtooth' ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        class="w-full py-1 text-[0.65rem] rounded transition-colors">
        Sawtooth
      </button>
      <button @click="selectSignal('triangle')"
        :class="selectedSignal === 'triangle' ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        class="w-full py-1 text-[0.65rem] rounded transition-colors">
        Triangle
      </button>
      <button @click="selectSignal('whiteNoise')"
        :class="selectedSignal === 'whiteNoise' ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        class="w-full py-1 text-[0.65rem] rounded transition-colors">
        White
      </button>
      <button @click="selectSignal('pinkNoise')"
        :class="selectedSignal === 'pinkNoise' ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        class="w-full py-1 text-[0.65rem] rounded transition-colors">
        Pink
      </button>
    </div>

    <!-- Transport Controls -->
    <div class="flex gap-1 justify-center w-full">
      <button @click="toggleSignal"
        class="px-2 py-1 w-full text-xs rounded transition-colors flex items-center justify-center bg-blue-600 hover:bg-blue-500"
        :class="isPlaying ? 'animate-pulse' : ''">
        <svg v-if="!isPlaying" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
          <path d="M8 5v14l11-7z" />
        </svg>
        <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
          <path d="M6 6h12v12H6z" />
        </svg>
      </button>
      <button v-if="isOscillator" @click.stop="toggleFrequencySweep"
        class="px-2 py-1 w-full text-xs rounded transition-colors flex items-center justify-center"
        :class="isSweeping ? 'bg-orange-600 hover:bg-orange-500 animate-pulse' : 'bg-gray-500 hover:bg-gray-600'">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512" class="w-3 h-3" fill="currentColor">
          <path d="M216 288h-48c-8.84 0-16 7.16-16 16v192c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V304c0-8.84-7.16-16-16-16zM88 384H40c-8.84 0-16 7.16-16 16v96c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16v-96c0-8.84-7.16-16-16-16zm256-192h-48c-8.84 0-16 7.16-16 16v288c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V208c0-8.84-7.16-16-16-16zm128-96h-48c-8.84 0-16 7.16-16 16v384c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V112c0-8.84-7.16-16-16-16zM600 0h-48c-8.84 0-16 7.16-16 16v480c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V16c0-8.84-7.16-16-16-16z"/>
        </svg>
      </button>
    </div>

    <!-- Display - Signal Controls -->
    <div v-if="isOscillator" class="w-full bg-gray-900 rounded p-2 border border-gray-700">
      <div class="text-xs text-center text-gray-400 mb-2">{{ signalTypeLabel }}</div>
      
      <!-- Frequency knob for oscillators -->
      <div class="scale-[0.7] flex justify-center">
        <FrequencyKnob v-model="frequency" :min="20" :max="20000" label="Frequency" color="#3b82f6" />
      </div>
    </div>

    <!-- Display for noise (no controls needed) -->
    <div v-else class="w-full bg-gray-900 rounded p-2 border border-gray-700">
      <div class="text-xs text-center text-gray-400">{{ signalTypeLabel }}</div>
    </div>

    <!-- Mute & Solo Buttons -->
    <div class="flex flex-row gap-1 w-full">
      <button @click="toggleMute" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isMuted ? 'bg-red-600 text-white animate-pulse' : 'bg-gray-500 hover:bg-gray-600 text-gray-300'">
        M
      </button>
      <button @click="toggleSolo" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isSolo ? 'bg-yellow-500 text-gray-900 animate-pulse' : 'bg-gray-500 hover:bg-gray-600 text-gray-300'">
        S
      </button>
    </div>

    <!-- Pan Knob -->
    <div class="flex justify-center scale-[0.75]">
      <PanKnob v-model="pan" label="Pan" />
    </div>

    <!-- Volume Fader and VU Meter -->
    <div class="flex flex-col flex-1 min-h-0 pb-6">
      <div class="text-[0.455rem] uppercase text-center mb-6">Volume</div>
      <div ref="faderContainer" class="flex-1 relative flex items-center justify-center gap-1 min-h-0">
        <!-- Routing Button -->
        <button 
          @click="toggleRouteToMaster" 
          :title="'Route to Master'"
          class="absolute -left-[1.8rem] top-1/2 transform -translate-y-1/2 z-50 w-5 h-7 text-[8px] font-bold rounded transition-all flex items-center justify-center"
          :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
          M
        </button>
        
        <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />
        
        <TrackMeter 
          class="absolute -right-[1.8rem] top-1/2 transform -translate-y-1/2 z-50 -mt-3"
          v-if="faderHeight > 0" 
          :levelL="trackLevelL" 
          :levelR="trackLevelR" 
          :isStereo="false"
          :height="faderHeight + 20" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, inject, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import FrequencyKnob from './core/FrequencyKnob.vue'
import PanKnob from './audioTrack/PanKnob.vue'
import TrackFader from './audioTrack/TrackFader.vue'
import TrackMeter from './audioTrack/TrackMeter.vue'

// Props
const props = defineProps<{
  trackNumber: number
  masterChannel?: any
  subgroups?: Array<{ id: number; name: string; channel: any }>
}>()

// Emits
const emit = defineEmits<{
  remove: []
  soloChange: [value: { trackNumber: number; isSolo: boolean }]
}>()

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// State to track if initialization has been done
const isInitialized = ref(false)

// Reactive state
const trackElement = ref<HTMLElement | null>(null)
const faderContainer = ref<HTMLElement | null>(null)
const faderHeight = ref(0)

const selectedSignal = ref<'sine' | 'square' | 'sawtooth' | 'triangle' | 'whiteNoise' | 'pinkNoise'>('sine')
const isPlaying = ref(false)
const isSweeping = ref(false)
const frequency = ref(1000) // Hz
const volume = ref(-6) // dB scale (-90 to +12), start at -6dB for reasonable level
const pan = ref(0) // -1 to 1
const isMuted = ref(false)
const isSolo = ref(false)
const routeToMaster = ref(true)

// Frequency sweep state
let sweepAnimationId: number | null = null
let sweepDirection = 1 // 1 for up, -1 for down
let sweepPosition = 0.5 // 0 to 1 (logarithmic position)
let isUpdatingFromSweep = false // Flag to prevent watch loops
let isTogglingFrequencySweep = false // Flag to prevent multiple rapid toggles

// Meter levels (in dB)
const trackLevelL = ref(-60)
const trackLevelR = ref(-60)

// Computed
const isOscillator = computed(() => 
  ['sine', 'square', 'sawtooth', 'triangle'].includes(selectedSignal.value)
)

const signalTypeLabel = computed(() => {
  const labels: Record<typeof selectedSignal.value, string> = {
    sine: 'Sine Wave',
    square: 'Square Wave',
    sawtooth: 'Sawtooth Wave',
    triangle: 'Triangle Wave',
    whiteNoise: 'White Noise',
    pinkNoise: 'Pink Noise'
  }
  return labels[selectedSignal.value]
})

// Handlers
async function selectSignal(signal: typeof selectedSignal.value) {
  selectedSignal.value = signal
  
  // Convert UI signal name to backend format
  const waveformMap: Record<typeof signal, string> = {
    sine: 'sine',
    square: 'square',
    sawtooth: 'sawtooth',
    triangle: 'triangle',
    whiteNoise: 'white',
    pinkNoise: 'pink'
  }
  
  if (audioEngine?.setSignalWaveform) {
    await audioEngine.setSignalWaveform(props.trackNumber - 1, waveformMap[signal])
  }
}

async function toggleSignal() {
  isPlaying.value = !isPlaying.value
  
  if (!audioEngine || !audioEngine.state.value.isRunning) {
    return
  }
  
  if (isPlaying.value) {
    const waveformMap: Record<typeof selectedSignal.value, string> = {
      sine: 'sine',
      square: 'square',
      sawtooth: 'sawtooth',
      triangle: 'triangle',
      whiteNoise: 'white',
      pinkNoise: 'pink'
    }
    await audioEngine.setTrackSourceSignal(
      props.trackNumber - 1,
      waveformMap[selectedSignal.value],
      frequency.value
    )
  } else {
    // STOP: Clear signal generator
    await audioEngine.clearTrackSource(props.trackNumber - 1)
  }
}

function toggleFrequencySweep() {
  // Prevent multiple rapid calls
  if (isTogglingFrequencySweep) return
  
  if (!isOscillator.value) return
  
  // Sweep only works when signal is playing
  if (!isPlaying.value) return
  
  isTogglingFrequencySweep = true
  
  isSweeping.value = !isSweeping.value
  
  if (isSweeping.value) {
    startFrequencySweep()
  } else {
    stopFrequencySweep()
  }
  
  // Reset flag after a short delay
  setTimeout(() => {
    isTogglingFrequencySweep = false
  }, 100)
}

function startFrequencySweep() {
  const minFreq = 20
  const maxFreq = 20000
  const logMin = Math.log(minFreq)
  const logMax = Math.log(maxFreq)
  
  // Calculate initial position based on current frequency
  const logFreq = Math.log(frequency.value)
  sweepPosition = (logFreq - logMin) / (logMax - logMin)
  
  const sweepSpeed = 0.0005 // Position change per frame (adjust for speed)
  
  function animate() {
    if (!isSweeping.value) {
      return
    }
    
    // Update position
    sweepPosition += sweepDirection * sweepSpeed
    
    // Reverse direction at boundaries
    if (sweepPosition >= 1.0) {
      sweepPosition = 1.0
      sweepDirection = -1
    } else if (sweepPosition <= 0.0) {
      sweepPosition = 0.0
      sweepDirection = 1
    }
    
    // Convert position to frequency (logarithmic)
    const logFreq = logMin + sweepPosition * (logMax - logMin)
    const newFreq = Math.round(Math.exp(logFreq))
    
    // Update frequency (with flag to prevent stopping sweep from watch)
    isUpdatingFromSweep = true
    frequency.value = newFreq
    
    // Reset flag after Vue's watch has been triggered
    nextTick(() => {
      isUpdatingFromSweep = false
    })
    
    // Continue animation
    sweepAnimationId = requestAnimationFrame(animate)
  }
  
  sweepAnimationId = requestAnimationFrame(animate)
}

function stopFrequencySweep() {
  if (sweepAnimationId !== null) {
    cancelAnimationFrame(sweepAnimationId)
    sweepAnimationId = null
  }
}

async function toggleMute() {
  isMuted.value = !isMuted.value
  
  if (audioEngine?.setTrackMute) {
    await audioEngine.setTrackMute(props.trackNumber - 1, isMuted.value)
  }
}

function toggleSolo() {
  isSolo.value = !isSolo.value
  emit('soloChange', { trackNumber: props.trackNumber, isSolo: isSolo.value })
}

async function toggleRouteToMaster() {
  routeToMaster.value = !routeToMaster.value
  
  if (audioEngine?.setTrackRouteToMaster) {
    await audioEngine.setTrackRouteToMaster(props.trackNumber - 1, routeToMaster.value)
  }
}

// Watchers - Send changes to Rust engine
watch(frequency, async (freq) => {
  // Capture the flag value immediately (before any await)
  const wasUpdatingFromSweep = isUpdatingFromSweep
    
  if (audioEngine?.state.value.isRunning && audioEngine?.setSignalFrequency) {
    await audioEngine.setSignalFrequency(props.trackNumber - 1, freq)
  }
  
  // Stop sweep if user manually changed frequency (not from sweep animation)
  if (!wasUpdatingFromSweep && isSweeping.value) {
    isSweeping.value = false
    stopFrequencySweep()
  }
})

// Stop sweep when signal is stopped
watch(isPlaying, (playing) => {
  if (!playing && isSweeping.value) {
    isSweeping.value = false
    stopFrequencySweep()
  }
})

watch(volume, async (newVolume) => {
  if (audioEngine?.state.value.isRunning && audioEngine?.setTrackVolume) {
    // Convert dB to linear: linear = 10^(dB/20)
    const linearVolume = newVolume <= -85 ? 0 : Math.pow(10, newVolume / 20)
    await audioEngine.setTrackVolume(props.trackNumber - 1, linearVolume)
  }
})

watch(pan, async (newPan) => {
  if (audioEngine?.state.value.isRunning && audioEngine?.setTrackPan) {
    await audioEngine.setTrackPan(props.trackNumber - 1, newPan)
  }
})

// Watch for meter level updates from audio engine
watch(
  () => audioEngine?.state.value.trackLevels.get(props.trackNumber - 1),
  (levels) => {
    if (levels) {
      // Convert linear (0-1) to dB (-60 to 0)
      trackLevelL.value = levels.left > 0 ? 20 * Math.log10(levels.left) : -60
      trackLevelR.value = levels.right > 0 ? 20 * Math.log10(levels.right) : -60
    }
  },
  { deep: true }
)

// Watch for audio engine to become ready and initialize
watch(
  () => audioEngine?.state.value.isRunning,
  async (isRunning) => {
    if (isRunning && !isInitialized.value) {
      isInitialized.value = true
      
      // Set initial parameters (volume, pan, routing)
      // Don't create signal generator yet - wait for user to press play
      const linearVolume = volume.value <= -85 ? 0 : Math.pow(10, volume.value / 20)

      await audioEngine.setTrackVolume(props.trackNumber - 1, linearVolume)
      await audioEngine.setTrackPan(props.trackNumber - 1, pan.value)
      await audioEngine.setTrackRouteToMaster(props.trackNumber - 1, routeToMaster.value)      
    }
  },
  { immediate: true }
)

// Fader height calculation
function updateFaderHeight() {
  if (faderContainer.value) {
    faderHeight.value = faderContainer.value.clientHeight
  }
}

// Lifecycle
onMounted(async () => { 
  const resizeObserver = new ResizeObserver(updateFaderHeight)
  if (faderContainer.value) {
    resizeObserver.observe(faderContainer.value)
  }
  updateFaderHeight()
  
  // Note: Signal generator initialization is handled by the audioEngine running watch
})

onUnmounted(() => {
  // Stop frequency sweep
  stopFrequencySweep()
  
  // Cleanup: mute the track
  if (audioEngine?.setTrackMute) {
    audioEngine.setTrackMute(props.trackNumber - 1, true)
  }
})
</script>

<style scoped>
/* Add any component-specific styles here */
</style>
