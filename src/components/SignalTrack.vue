<template>
  <div
    class="track-channel relative bg-gray-700 rounded-lg border border-gray-900 p-1 flex flex-col items-center gap-1 h-full">
    
    <!-- Track Header -->
    <div class="text-xs font-bold text-center text-gray-300">Track {{ trackNumber }}</div>

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
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512" class="w-3 h-3" fill="currentColor"><path d="M216 288h-48c-8.84 0-16 7.16-16 16v192c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V304c0-8.84-7.16-16-16-16zM88 384H40c-8.84 0-16 7.16-16 16v96c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16v-96c0-8.84-7.16-16-16-16zm256-192h-48c-8.84 0-16 7.16-16 16v288c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V208c0-8.84-7.16-16-16-16zm128-96h-48c-8.84 0-16 7.16-16 16v384c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V112c0-8.84-7.16-16-16-16zM600 0h-48c-8.84 0-16 7.16-16 16v480c0 8.84 7.16 16 16 16h48c8.84 0 16-7.16 16-16V16c0-8.84-7.16-16-16-16z"/></svg>
      </button>
    </div>

    <!-- Waveform Display -->
    <WaveformDisplay ref="waveformDisplayRef" :showModeButtons="false" :waveform-node="waveform" :is-playing="isPlaying" />

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
      <div ref="faderContainer" class="flex-1 flex items-center justify-center gap-2 min-h-0">
        <TrackFader v-if="faderHeight > 0" v-model="volume" :trackHeight="faderHeight" />
        <TrackMeter v-if="faderHeight > 0" :levelL="trackLevelL" :levelR="trackLevelR" :isStereo="isStereo"
          :height="faderHeight + 20" />
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, inject, nextTick, computed, toRaw } from 'vue'
import FrequencyKnob from './core/FrequencyKnob.vue'
import TrackFader from './audioTrack/TrackFader.vue'
import TrackMeter from './audioTrack/TrackMeter.vue'
import PanKnob from './audioTrack/PanKnob.vue'
import WaveformDisplay from './audioTrack/WaveformDisplay.vue'

const ToneRef = inject<any>('Tone')
let Tone: any = null

interface Props {
  trackNumber: number
  masterChannel?: any
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'soloChange', value: { trackNumber: number, isSolo: boolean }): void
  (e: 'levelUpdate', value: { trackNumber: number, level: number }): void
}>()

type SignalType = 'sine' | 'square' | 'sawtooth' | 'triangle' | 'whiteNoise' | 'pinkNoise'

// Signal-specific state
const selectedSignal = ref<SignalType>('sine')
const isPlaying = ref(false)
const signalVolume = ref(0) // Generator output level (0 dB = unity gain)
const frequency = ref(1000)

// Frequency sweep state
const isSweeping = ref(false)
let sweepInterval: number | null = null
let sweepDirection = 1 // 1 = up, -1 = down

const isOscillator = computed(() => 
  ['sine', 'square', 'sawtooth', 'triangle'].includes(selectedSignal.value)
)

const signalTypeLabel = computed(() => {
  const labels: Record<SignalType, string> = {
    'sine': 'Sine Wave',
    'square': 'Square Wave',
    'sawtooth': 'Sawtooth Wave',
    'triangle': 'Triangle Wave',
    'whiteNoise': 'White Noise',
    'pinkNoise': 'Pink Noise'
  }
  return labels[selectedSignal.value]
})

// Track state (from Track.vue)
const faderContainer = ref<HTMLElement | null>(null)
const waveformDisplayRef = ref<any>(null)
const faderHeight = ref(0)
const isMuted = ref(false)
const isSolo = ref(false)

// Audio controls
const volume = ref(0)
const pan = ref(0)
const isStereo = ref(true)
const trackLevelL = ref(-60)
const trackLevelR = ref(-60)

// Tone.js nodes - Signal
let signalNode: any = null
let signalVolumeNode: any = null
let signalToStereo: any = null

// Tone.js nodes - Track routing
let gainNode: any = null
let waveform: any = null // Waveform analyzer
let balanceSplit: any = null
let balanceLeft: any = null
let balanceRight: any = null
let balanceMerge: any = null
let volumeSplit: any = null
let volumeNodeL: any = null
let volumeNodeR: any = null
let volumeMerge: any = null
let meterL: any = null
let meterR: any = null
let channelSplit: any = null
let resizeObserver: ResizeObserver | null = null
let levelMonitorInterval: number | null = null

// Calculate fader height
function updateFaderHeight() {
  if (faderContainer.value) {
    const height = faderContainer.value.clientHeight
    faderHeight.value = Math.max(100, height - 60)
  }
}

// Initialize audio nodes
function initAudioNodes() {
  if (gainNode || !Tone) return

  // Create main gain node
  gainNode = new Tone.Gain(1)

  // Waveform analyzer (for visualization)
  waveform = new Tone.Waveform(512) // 512 samples for waveform display

  // Stereo-preserving balance control
  balanceSplit = new Tone.Split()
  balanceLeft = new Tone.Gain(1)
  balanceRight = new Tone.Gain(1)
  balanceMerge = new Tone.Merge()

  // Stereo-preserving volume control
  volumeSplit = new Tone.Split()
  volumeNodeL = new Tone.Gain(1)
  volumeNodeR = new Tone.Gain(1)
  volumeMerge = new Tone.Merge()

  // Stereo metering
  channelSplit = new Tone.Split()
  meterL = new Tone.Meter()
  meterR = new Tone.Meter()

  // Connect chain: gain -> balance -> volume
  gainNode.connect(balanceSplit)
  balanceSplit.connect(balanceLeft, 0)
  balanceSplit.connect(balanceRight, 1)
  balanceLeft.connect(balanceMerge, 0, 0)
  balanceRight.connect(balanceMerge, 0, 1)

  balanceMerge.connect(volumeSplit)
  volumeSplit.connect(volumeNodeL, 0)
  volumeSplit.connect(volumeNodeR, 1)
  volumeNodeL.connect(volumeMerge, 0, 0)
  volumeNodeR.connect(volumeMerge, 0, 1)

  // Connect waveform analyzer
  gainNode.connect(waveform)

  // Connect metering to gainNode (pre-volume)
  gainNode.connect(channelSplit)
  channelSplit.connect(meterL, 0)
  channelSplit.connect(meterR, 1)

  // Connect to master
  connectToOutput()
}

// Initialize signal nodes
function initSignalNodes() {
  if (signalVolumeNode || !Tone) return

  // Create volume node
  signalVolumeNode = new Tone.Volume(signalVolume.value)
  
  // Create mono-to-stereo converter
  // Connect the mono source to both L and R inputs of Merge
  signalToStereo = new Tone.Merge()
  
  if (gainNode) {
    // Connect mono output to both L (channel 0) and R (channel 1) inputs
    signalVolumeNode.connect(signalToStereo, 0, 0) // to left input
    signalVolumeNode.connect(signalToStereo, 0, 1) // to right input
    // Connect stereo output to gain node
    signalToStereo.connect(gainNode)
  }
}

// Create signal source
function createSignalSource() {
  // Dispose old signal node
  if (signalNode) {
    if (isPlaying.value && signalNode.stop) {
      try {
        signalNode.stop()
      } catch (e) {}
    }
    try {
      signalNode.disconnect()
      signalNode.dispose()
    } catch (e) {}
    signalNode = null
  }

  if (!Tone || !signalVolumeNode) return

  // Create new signal based on type
  switch (selectedSignal.value) {
    case 'sine':
    case 'square':
    case 'sawtooth':
    case 'triangle':
      signalNode = new Tone.Oscillator({
        type: selectedSignal.value,
        frequency: frequency.value
      })
      break
    case 'whiteNoise':
      signalNode = new Tone.Noise('white')
      break
    case 'pinkNoise':
      signalNode = new Tone.Noise('pink')
      break
  }

  if (signalNode) {
    signalNode.connect(signalVolumeNode)
    
    // If was playing, restart new signal
    if (isPlaying.value) {
      signalNode.start()
    }
  }
}

function selectSignal(type: SignalType) {
  selectedSignal.value = type
  handleSignalChange()
}

function handleSignalChange() {
  createSignalSource()
}

async function toggleSignal() {
  if (!signalNode || !Tone) return

  if (!isPlaying.value) {
    // Warning per segnali diversi da rumore
    const isNoise = selectedSignal.value === 'whiteNoise' || selectedSignal.value === 'pinkNoise'
    
    if (!isNoise) {
      const signalWarnings = {
        'square': '⚠️ ATTENZIONE: Le onde quadre contengono armoniche molto intense che possono DANNEGGIARE gli altoparlanti e l\'udito!',
        'sawtooth': '⚠️ ATTENZIONE: Le onde a dente di sega contengono molte armoniche che possono essere pericolose!',
        'triangle': '⚠️ ATTENZIONE: Le onde triangolari possono raggiungere livelli elevati!',
        'sine': '⚠️ ATTENZIONE: Controlla sempre i livelli prima di riprodurre!'
      }
      
      const warning = signalWarnings[selectedSignal.value as keyof typeof signalWarnings] || '⚠️ ATTENZIONE!'
      
      const confirmed = confirm(
        `${warning}\n\n` +
        `VERIFICA CHE:\n` +
        `• Il fader della traccia sia al minimo\n` +
        `• Il volume master sia al minimo\n` +
        `• Le cuffie siano scollegate o a volume basso\n\n` +
        `Alza gradualmente il volume SOLO dopo aver verificato i livelli.\n\n` +
        `Vuoi continuare?`
      )
      
      if (!confirmed) return
    }
    
    await Tone.start()
    signalNode.start()
    isPlaying.value = true
  } else {
    signalNode.stop()
    isPlaying.value = false
    // Ricrea il nodo per poterlo riavviare
    createSignalSource()
  }
}

function toggleFrequencySweep() {
  if (isSweeping.value) {
    // Stop sweep
    if (sweepInterval) clearInterval(sweepInterval)
    sweepInterval = null
    isSweeping.value = false
  } else {
    // Start sweep
    isSweeping.value = true
    const logMin = Math.log(20)
    const logMax = Math.log(20000)
    const step = (logMax - logMin) / 200 // 200 steps for smooth sweep
    
    sweepInterval = window.setInterval(() => {
      const currentLog = Math.log(frequency.value)
      const newLog = currentLog + (step * sweepDirection)
      
      // Reverse direction at boundaries
      if (newLog >= logMax) {
        sweepDirection = -1
        frequency.value = 20000
      } else if (newLog <= logMin) {
        sweepDirection = 1
        frequency.value = 20
      } else {
        frequency.value = Math.round(Math.exp(newLog))
      }
    }, 50) // 50ms = 20fps, total sweep ~10 seconds per direction
  }
}

// Connect to master output
function connectToOutput() {
  if (!volumeMerge || !props.masterChannel || !Tone) return
  volumeMerge.connect(toRaw(props.masterChannel))
}

// Level monitoring
function startLevelMonitoring() {
  levelMonitorInterval = window.setInterval(() => {
    if (meterL && Tone) {
      const levelL = meterL.getValue() as number
      trackLevelL.value = Math.max(-60, levelL)

      if (isStereo.value && meterR) {
        const levelR = meterR.getValue() as number
        trackLevelR.value = Math.max(-60, levelR)
      } else {
        trackLevelR.value = trackLevelL.value
      }
    }
  }, 50)
}

// Update volume
function updateVolume() {
  if (!volumeNodeL || !volumeNodeR || !Tone) return

  if (isMuted.value) {
    volumeNodeL.gain.value = 0
    volumeNodeR.gain.value = 0
  } else {
    const gainValue = Tone.dbToGain(volume.value)
    volumeNodeL.gain.value = gainValue
    volumeNodeR.gain.value = gainValue
  }
}

// Update pan
function updatePan() {
  if (!balanceLeft || !balanceRight || !Tone) return
  
  // Linear pan for signal generators (better for testing/calibration)
  // Pan range: -1 (full left) to +1 (full right)
  // Center (0): both channels at 1.0 for unity gain
  const leftGain = 1 - Math.max(0, pan.value)
  const rightGain = 1 + Math.min(0, pan.value)
  
  balanceLeft.gain.value = leftGain
  balanceRight.gain.value = rightGain
}

function toggleMute() {
  isMuted.value = !isMuted.value
  updateVolume()
}

function toggleSolo() {
  isSolo.value = !isSolo.value
  emit('soloChange', { trackNumber: props.trackNumber, isSolo: isSolo.value })
}

// Update frequency for oscillators
watch(frequency, (newFreq) => {
  if (signalNode && typeof signalNode.frequency !== 'undefined') {
    signalNode.frequency.value = newFreq
  }
})

// Update signal volume
watch(signalVolume, (newVolume) => {
  if (signalVolumeNode) {
    signalVolumeNode.volume.value = newVolume
  }
})

// Watch for changes
watch(volume, updateVolume)
watch(pan, updatePan)
watch(() => props.masterChannel, connectToOutput)

onMounted(async () => {
  if (ToneRef?.value) {
    Tone = ToneRef.value
  }

  await nextTick()
  updateFaderHeight()

  if (faderContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateFaderHeight()
    })
    resizeObserver.observe(faderContainer.value)
  }

  initAudioNodes()
  initSignalNodes()
  createSignalSource()
  startLevelMonitoring()
})

onUnmounted(() => {
  // Clean up sweep interval
  if (sweepInterval) {
    clearInterval(sweepInterval)
    sweepInterval = null
  }
  
  // Dispose signal nodes
  if (signalNode) {
    if (isPlaying.value && signalNode.stop) {
      try {
        signalNode.stop()
      } catch (e) {}
    }
    try {
      signalNode.disconnect()
      signalNode.dispose()
    } catch (e) {}
  }
  
  if (signalVolumeNode) {
    signalVolumeNode.dispose()
  }
  
  if (signalToStereo) {
    signalToStereo.dispose()
  }

  // Dispose track nodes
  if (gainNode) gainNode.dispose()
  if (waveform) waveform.dispose()
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

  if (resizeObserver) {
    resizeObserver.disconnect()
  }

  if (levelMonitorInterval) {
    clearInterval(levelMonitorInterval)
  }
})

// Expose methods for scene management
defineExpose({
  getSnapshot: () => {
    return {
      trackType: 'signal' as const,
      selectedSignal: selectedSignal.value,
      signalVolume: signalVolume.value,
      frequency: frequency.value,
      isPlaying: isPlaying.value,
      volume: volume.value,
      pan: pan.value,
      isMuted: isMuted.value,
      isSolo: isSolo.value
    }
  },

  restoreFromSnapshot: (snapshot: any) => {
    if (snapshot.selectedSignal) {
      selectedSignal.value = snapshot.selectedSignal
      handleSignalChange()
    }
    if (snapshot.signalVolume !== undefined) {
      signalVolume.value = snapshot.signalVolume
    }
    if (snapshot.frequency !== undefined) {
      frequency.value = snapshot.frequency
    }
    if (snapshot.volume !== undefined) {
      volume.value = snapshot.volume
    }
    if (snapshot.pan !== undefined) {
      pan.value = snapshot.pan
    }
    if (snapshot.isMuted !== undefined) {
      isMuted.value = snapshot.isMuted
      updateVolume()
    }
    if (snapshot.isSolo !== undefined) {
      isSolo.value = snapshot.isSolo
    }
    if (snapshot.isPlaying && !isPlaying.value) {
      toggleSignal()
    }
  },

  getInputNode: () => gainNode,
  setMuted: (muted: boolean) => {
    isMuted.value = muted
    updateVolume()
  },
  isSolo: () => isSolo.value,
  getVolume: () => volume.value,
  setVolume: (val: number) => volume.value = val,
  getPan: () => pan.value,
  setPan: (val: number) => pan.value = val,
  getMute: () => isMuted.value,
  getSolo: () => isSolo.value,
  
  // Frequency control methods
  getFrequency: () => frequency.value,
  setFrequency: (val: number) => {
    frequency.value = val
  },
  isOscillator: () => isOscillator.value,
  randomizeFrequency: () => {
    // Random frequency between 20Hz and 20kHz on logarithmic scale
    const logMin = Math.log(20)
    const logMax = Math.log(20000)
    const randomLog = logMin + Math.random() * (logMax - logMin)
    frequency.value = Math.round(Math.exp(randomLog))
  }
})
</script>
