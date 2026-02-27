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
      <button v-if="isOscillator" @click="toggleFrequencySweep"
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
    <div class="flex flex-col h-full">
      <div class="text-[0.455rem] uppercase text-center">Volume</div>
      <div ref="faderContainer" class="flex-1 relative flex items-center justify-center gap-1 min-h-0">
        <!-- Routing Button -->
        <button 
          @click="toggleRouteToMaster" 
          :title="'Route to Master'"
          class="absolute left-[0.2rem] top-1/2 transform -translate-y-1/2 z-50 w-5 h-7 text-[8px] font-bold rounded transition-all flex items-center justify-center"
          :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-400'">
          M
        </button>
        
        <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />
        
        <TrackMeter 
          class="absolute right-[0.4rem] top-1/2 transform -translate-y-1/2 z-50 -mt-3"
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
import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue'
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

// Reactive state
const trackElement = ref<HTMLElement | null>(null)
const faderContainer = ref<HTMLElement | null>(null)
const faderHeight = ref(0)

const selectedSignal = ref<'sine' | 'square' | 'sawtooth' | 'triangle' | 'whiteNoise' | 'pinkNoise'>('sine')
const isPlaying = ref(false)
const isSweeping = ref(false)
const frequency = ref(1000) // Hz
const volume = ref(0.5) // 0-1 range
const pan = ref(0) // -1 to 1
const isMuted = ref(false)
const isSolo = ref(false)
const routeToMaster = ref(true)

// Meter levels (simulated for now)
const trackLevelL = ref(0)
const trackLevelR = ref(0)

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
function selectSignal(signal: typeof selectedSignal.value) {
  selectedSignal.value = signal
  console.log(`[Signal Track ${props.trackNumber}] Selected signal:`, signal)
  // TODO: Send to Rust engine
}

function toggleSignal() {
  isPlaying.value = !isPlaying.value
  console.log(`[Signal Track ${props.trackNumber}] ${isPlaying.value ? 'Started' : 'Stopped'}`)
  // TODO: Send to Rust engine to start/stop signal generation
}

function toggleFrequencySweep() {
  if (!isOscillator.value) return
  isSweeping.value = !isSweeping.value
  console.log(`[Signal Track ${props.trackNumber}] Sweep ${isSweeping.value ? 'enabled' : 'disabled'}`)
  // TODO: Send to Rust engine
}

function toggleMute() {
  isMuted.value = !isMuted.value
}

function toggleSolo() {
  isSolo.value = !isSolo.value
  emit('soloChange', { trackNumber: props.trackNumber, isSolo: isSolo.value })
}

function toggleRouteToMaster() {
  routeToMaster.value = !routeToMaster.value
}

// Watchers - Send changes to Rust engine
watch(selectedSignal, (signal) => {
  if (audioEngine?.state.value.isRunning && isPlaying.value) {
    // TODO: Send to Rust engine
    console.log(`[Signal Track ${props.trackNumber}] Signal changed to:`, signal)
  }
})

watch(frequency, (freq) => {
  if (audioEngine?.state.value.isRunning && isPlaying.value && isOscillator.value) {
    // TODO: Send to Rust engine
    console.log(`[Signal Track ${props.trackNumber}] Frequency changed to:`, freq, 'Hz')
  }
})

watch(volume, (newVolume) => {
  if (audioEngine?.state.value.isRunning) {
    // TODO: Send to Rust engine
    console.log(`[Signal Track ${props.trackNumber}] Volume changed to:`, newVolume)
  }
})

watch(isMuted, (muted) => {
  if (audioEngine?.state.value.isRunning) {
    // TODO: Send to Rust engine
    console.log(`[Signal Track ${props.trackNumber}] Mute:`, muted)
  }
})

// Fader height calculation
function updateFaderHeight() {
  if (faderContainer.value) {
    faderHeight.value = faderContainer.value.clientHeight
  }
}

// Lifecycle
onMounted(() => {
  const resizeObserver = new ResizeObserver(updateFaderHeight)
  if (faderContainer.value) {
    resizeObserver.observe(faderContainer.value)
  }
  updateFaderHeight()
})

onUnmounted(() => {
  // Cleanup if needed
})
</script>

<style scoped>
/* Add any component-specific styles here */
</style>
