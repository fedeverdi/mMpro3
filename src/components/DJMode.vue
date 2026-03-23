<template>
  <div class="dj-wrap w-full h-full flex gap-2 p-2 select-none overflow-hidden bg-gradient-to-b from-gray-950 to-black">

    <!-- ══════════════════ DECK A ══════════════════ -->
    <Deck
      deck-name="A"
      theme-color="cyan"
      theme-rgb="34,211,238"
      :track-index="deckAIdx"
      :current-time="deckACurrentTime"
      :duration="deckADuration"
      :title="deckATitle"
      :artist="deckAArtist"
      :bpm="deckABpm"
      :has-file="deckAHasFile"
      :is-playing="deckAIsPlaying"
      :pitch="deckAPitch"
      :volume="deckAVolume"
      :level-l="deckALevelL"
      :level-r="deckALevelR"
      :rotation="vinylARotation"
      :audio-buffer="deckAAudioBuffer"
      :cue-point="deckACuePoint"
      :loop-active="deckALoopActive"
      :loop-fraction="deckALoopFraction"
      :loop-in-set="deckALoopInSet"
      :loop-out-set="deckALoopOutSet"
      :loop-start="deckALoopStart"
      :loop-end="deckALoopEnd"
      @play-pause="onPlayPauseA"
      @stop="onStopA"
      @cue-press="onCueAPress"
      @cue-release="onCueARelease"
      @load-file="onLoadA"
      @pitch-change="(v: any) => { deckAPitch = v; onPitchAChange() }"
      @volume-change="(v: any) => { deckAVolume = v; onDeckAVolume() }"
      @seek="onSeekA"
      @seek-release="onSeekReleaseA"
      @vinyl-mousedown="onVinylAMouseDown"
      @tap-tempo="onTapTempoA"
      @loop-speed="onLoopSpeedA"
      @loop-in="onLoopInA"
      @loop-out="onLoopOutA"
    />

    <!-- ══════════════════ CENTER STRIP ══════════════════ -->
    <div class="center-strip w-16 flex-shrink-0 flex flex-col items-center justify-between py-3">

      <div class="flex flex-col items-center gap-1">
        <span class="text-[10px] font-black text-cyan-400">A</span>
      </div>

      <div class="flex flex-col items-center gap-3">
        <!-- SYNC -->
        <button @click="syncDecks"
                class="px-2 py-1 text-[9px] font-bold rounded bg-gray-800 hover:bg-purple-900/50 text-gray-500 hover:text-purple-300 border border-gray-700 hover:border-purple-700/60 transition-all tracking-widest">
          SYNC
        </button>

        <!-- Crossfader -->
        <div class="flex flex-col items-center gap-1.5 w-full">
          <div class="text-[8px] text-gray-600 tracking-[0.2em]">X-FADE</div>
          <input type="range" min="0" max="1" step="0.01" v-model.number="crossfader"
                 class="w-full accent-purple-500 cursor-pointer"
                 @input="onCrossfader"/>
          <!-- Crossfader position indicator -->
          <div class="w-full flex justify-between px-0.5">
            <span class="text-[7px]" :class="crossfader < 0.4 ? 'text-cyan-400' : 'text-gray-700'">A</span>
            <span class="text-[7px]" :class="crossfader > 0.6 ? 'text-orange-400' : 'text-gray-700'">B</span>
          </div>
        </div>

        <!-- Master level indicator -->
        <div class="flex flex-col items-center gap-1">
          <span class="text-[8px] text-gray-700 tracking-widest">MST</span>
          <div class="flex gap-1 items-end" style="height: 40px">
            <div class="w-1.5 bg-gray-800 rounded-full overflow-hidden h-full relative">
              <div class="absolute bottom-0 left-0 right-0 rounded-full transition-[height] duration-75"
                   style="background: linear-gradient(to top, #10b981, #34d399)"
                   :style="{ height: levelPct(masterLevelL) }"/>
            </div>
            <div class="w-1.5 bg-gray-800 rounded-full overflow-hidden h-full relative">
              <div class="absolute bottom-0 left-0 right-0 rounded-full transition-[height] duration-75"
                   style="background: linear-gradient(to top, #10b981, #34d399)"
                   :style="{ height: levelPct(masterLevelR) }"/>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col items-center gap-1">
        <span class="text-[10px] font-black text-orange-400">B</span>
      </div>
    </div>

    <!-- ══════════════════ DECK B ══════════════════ -->
    <Deck
      deck-name="B"
      theme-color="orange"
      theme-rgb="251,146,60"
      :track-index="deckBIdx"
      :current-time="deckBCurrentTime"
      :duration="deckBDuration"
      :title="deckBTitle"
      :artist="deckBArtist"
      :bpm="deckBBpm"
      :has-file="deckBHasFile"
      :is-playing="deckBIsPlaying"
      :pitch="deckBPitch"
      :volume="deckBVolume"
      :level-l="deckBLevelL"
      :level-r="deckBLevelR"
      :rotation="vinylBRotation"
      :audio-buffer="deckBAudioBuffer"
      :cue-point="deckBCuePoint"
      :loop-active="deckBLoopActive"
      :loop-fraction="deckBLoopFraction"
      :loop-in-set="deckBLoopInSet"
      :loop-out-set="deckBLoopOutSet"
      :loop-start="deckBLoopStart"
      :loop-end="deckBLoopEnd"
      @play-pause="onPlayPauseB"
      @stop="onStopB"
      @cue-press="onCueBPress"
      @cue-release="onCueBRelease"
      @load-file="onLoadB"
      @pitch-change="(v) => { deckBPitch = v; onPitchBChange() }"
      @volume-change="(v) => { deckBVolume = v; onDeckBVolume() }"
      @seek="onSeekB"
      @seek-release="onSeekReleaseB"
      @vinyl-mousedown="onVinylBMouseDown"
      @tap-tempo="onTapTempoB"
      @loop-speed="onLoopSpeedB"
      @loop-in="onLoopInB"
      @loop-out="onLoopOutB"
    />

  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, onMounted, watch, onUnmounted, Ref } from 'vue'
import Deck from './dj/Deck.vue'

interface Track {
  id: number
  type: string
  order: number
}

interface Props {
  deckATrack: Track | null
  deckBTrack: Track | null
}

const props = defineProps<Props>()
const audioEngine = inject<any>('audioEngine', null)

// Engine indices are 0-based (UI track IDs are 1-based)
const deckAIdx = computed(() => props.deckATrack != null ? props.deckATrack.id - 1 : null)
const deckBIdx = computed(() => props.deckBTrack != null ? props.deckBTrack.id - 1 : null)

// Local deck state
const deckAVolume = ref(1.0)
const deckBVolume = ref(1.0)
const deckAPitch = ref(0)
const deckBPitch = ref(0)
const crossfader = ref(0.5)

// CUE points for each deck
const deckACuePoint = ref(0)
const deckBCuePoint = ref(0)

// Loop state for each deck
const deckALoopActive = ref(false)
const deckALoopStart = ref(0)
const deckALoopEnd = ref(0)
const deckALoopOriginalEnd = ref(0) // Salva il loop end originale per calcolare le frazioni
const deckALoopFraction = ref(0)
const deckALoopInSet = ref(false)
const deckALoopOutSet = ref(false)
const deckBLoopActive = ref(false)
const deckBLoopStart = ref(0)
const deckBLoopEnd = ref(0)
const deckBLoopOriginalEnd = ref(0) // Salva il loop end originale per calcolare le frazioni
const deckBLoopFraction = ref(0)
const deckBLoopInSet = ref(false)
const deckBLoopOutSet = ref(false)

// Current playback time tracking (similar to AudioTrack)
const deckACurrentTime = ref(0)
const deckBCurrentTime = ref(0)
let deckAPlaybackStartTime = 0
let deckAPlaybackOffset = 0
let deckBPlaybackStartTime = 0
let deckBPlaybackOffset = 0

// Audio buffers for waveform visualization
const deckAAudioBuffer = ref<AudioBuffer | null>(null)
const deckBAudioBuffer = ref<AudioBuffer | null>(null)
const deckADuration = ref(0)
const deckBDuration = ref(0)

// Vinyl scratch state
const vinylAScratchActive = ref(false)
const vinylBScratchActive = ref(false)
const vinylARotation = ref(0)
const vinylBRotation = ref(0)
let vinylAScratchWasPlaying = false
let vinylAScratchLastAngle = 0
let vinylAScratchLastTime = 0
let vinylAScratchTarget: SVGElement | null = null
let vinylBScratchWasPlaying = false
let vinylBScratchLastAngle = 0
let vinylBScratchLastTime = 0
let vinylBScratchTarget: SVGElement | null = null

// Playback tracking interval
let playbackIntervalId: ReturnType<typeof setInterval> | null = null

// ══════════════════ GENERIC DECK UTILITIES ══════════════════

interface DeckState {
  idx: Ref<number | null>
  volume: Ref<number>
  pitch: Ref<number>
  cuePoint: Ref<number>
  currentTime: Ref<number>
  playbackStartTime: { value: number }
  playbackOffset: { value: number }
  audioBuffer: Ref<AudioBuffer | null>
  duration: Ref<number>
  scratchActive: Ref<boolean>
  rotation: Ref<number>
  scratchWasPlaying: { value: boolean }
  scratchLastAngle: { value: number }
  scratchLastTime: { value: number }
  scratchTarget: { value: SVGElement | null }
  cueHeld: { value: boolean }
}

// Mappa dei deck states
const deckStates: Record<'A' | 'B', DeckState> = {
  A: {
    idx: deckAIdx,
    volume: deckAVolume,
    pitch: deckAPitch,
    cuePoint: deckACuePoint,
    currentTime: deckACurrentTime,
    playbackStartTime: { get value() { return deckAPlaybackStartTime }, set value(v) { deckAPlaybackStartTime = v } },
    playbackOffset: { get value() { return deckAPlaybackOffset }, set value(v) { deckAPlaybackOffset = v } },
    audioBuffer: deckAAudioBuffer,
    duration: deckADuration,
    scratchActive: vinylAScratchActive,
    rotation: vinylARotation,
    scratchWasPlaying: { get value() { return vinylAScratchWasPlaying }, set value(v) { vinylAScratchWasPlaying = v } },
    scratchLastAngle: { get value() { return vinylAScratchLastAngle }, set value(v) { vinylAScratchLastAngle = v } },
    scratchLastTime: { get value() { return vinylAScratchLastTime }, set value(v) { vinylAScratchLastTime = v } },
    scratchTarget: { get value() { return vinylAScratchTarget }, set value(v) { vinylAScratchTarget = v } },
    cueHeld: { get value() { return deckACueHeld }, set value(v) { deckACueHeld = v } },
  },
  B: {
    idx: deckBIdx,
    volume: deckBVolume,
    pitch: deckBPitch,
    cuePoint: deckBCuePoint,
    currentTime: deckBCurrentTime,
    playbackStartTime: { get value() { return deckBPlaybackStartTime }, set value(v) { deckBPlaybackStartTime = v } },
    playbackOffset: { get value() { return deckBPlaybackOffset }, set value(v) { deckBPlaybackOffset = v } },
    audioBuffer: deckBAudioBuffer,
    duration: deckBDuration,
    scratchActive: vinylBScratchActive,
    rotation: vinylBRotation,
    scratchWasPlaying: { get value() { return vinylBScratchWasPlaying }, set value(v) { vinylBScratchWasPlaying = v } },
    scratchLastAngle: { get value() { return vinylBScratchLastAngle }, set value(v) { vinylBScratchLastAngle = v } },
    scratchLastTime: { get value() { return vinylBScratchLastTime }, set value(v) { vinylBScratchLastTime = v } },
    scratchTarget: { get value() { return vinylBScratchTarget }, set value(v) { vinylBScratchTarget = v } },
    cueHeld: { get value() { return deckBCueHeld }, set value(v) { deckBCueHeld = v } },
  }
}

let deckACueHeld = false
let deckBCueHeld = false
let deckACurrentRate = 1.0
let deckBCurrentRate = 1.0

// Tap tempo state
const deckATapTimes = ref<number[]>([])
const deckBTapTimes = ref<number[]>([])
const deckATappedBpm = ref<number>(0)
const deckBTappedBpm = ref<number>(0)
const TAP_TIMEOUT = 2000 // Reset after 2 seconds of inactivity
const MIN_TAPS = 2 // Minimum taps to calculate BPM
const MAX_TAPS = 4 // Average over last 4 taps

// ══════════════════ GENERIC DECK FUNCTIONS ══════════════════

// Calcola il tempo base di rotazione per un deck
function getBaseRotationTime(deck: 'A' | 'B'): number {
  return 1.8 / (1 + deckStates[deck].pitch.value / 100)
}

// Generic waveform loading
async function loadWaveform(deck: 'A' | 'B') {
  const state = deckStates[deck]
  if (!audioEngine || state.idx.value === null) return
  
  try {
    const result = await audioEngine.getWaveformData(state.idx.value, 2000)
    
    if (result && result.data) {
      const useSampleRate = 8000
      const numFrames = Math.floor(result.duration * useSampleRate)
      
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
      state.audioBuffer.value = audioContext.createBuffer(1, numFrames, useSampleRate)
      
      const channelData = state.audioBuffer.value.getChannelData(0)
      const sourceData = result.data
      
      // Normalize
      let maxAbs = 0
      for (let i = 0; i < sourceData.length; i++) {
        maxAbs = Math.max(maxAbs, Math.abs(sourceData[i]))
      }
      const normalizeFactor = maxAbs > 0 ? 0.7 / maxAbs : 1.0
      
      // Interpolate
      for (let i = 0; i < numFrames; i++) {
        const sourcePos = (i / numFrames) * sourceData.length
        const sourceIndex = Math.floor(sourcePos)
        const frac = sourcePos - sourceIndex
        
        const sample1 = sourceData[Math.min(sourceIndex, sourceData.length - 1)] || 0
        const sample2 = sourceData[Math.min(sourceIndex + 1, sourceData.length - 1)] || 0
        const interpolated = sample1 + (sample2 - sample1) * frac
        
        channelData[i] = interpolated * normalizeFactor
      }
      
      state.duration.value = result.duration
    }
  } catch (error) {
    console.error(`[DJMode] Error loading waveform ${deck}:`, error)
  }
}

// Generic seek handler
function onSeek(deck: 'A' | 'B', time: number) {
  const state = deckStates[deck]
  if (!audioEngine || state.idx.value === null) return
  
  state.currentTime.value = time
  state.playbackOffset.value = time
  audioEngine.seekFile(state.idx.value, time)
  
  const baseRotationTime = getBaseRotationTime(deck)
  const rotations = time / baseRotationTime
  state.rotation.value = rotations * 360
  
  const isPlaying = deck === 'A' ? deckAIsPlaying.value : deckBIsPlaying.value
  if (isPlaying) {
    state.playbackStartTime.value = Date.now()
  }
}

// Generic vinyl scratch handlers
function getMouseAngle(event: MouseEvent, element: SVGElement): number {
  const rect = element.getBoundingClientRect()
  const centerX = rect.left + rect.width / 2
  const centerY = rect.top + rect.height / 2
  const deltaX = event.clientX - centerX
  const deltaY = event.clientY - centerY
  return Math.atan2(deltaY, deltaX) * (180 / Math.PI)
}

function onVinylMouseDown(deck: 'A' | 'B', event: MouseEvent) {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  if (!hasFile || !audioEngine || state.idx.value === null) return
  event.preventDefault()
  
  state.scratchActive.value = true
  state.scratchTarget.value = event.currentTarget as SVGElement
  state.scratchLastAngle.value = getMouseAngle(event, state.scratchTarget.value)
  state.scratchLastTime.value = Date.now()
  
  const isPlaying = deck === 'A' ? deckAIsPlaying.value : deckBIsPlaying.value
  state.scratchWasPlaying.value = isPlaying
  
  // Se è in pausa, attiviamo temporaneamente il playback per lo scratching
  if (!state.scratchWasPlaying.value) {
    audioEngine.playFile(state.idx.value)
  }
  
  // Impostiamo il rate a 0 per fermare il movimento mentre teniamo il disco
  audioEngine.setFilePlaybackRate(state.idx.value, 0)
  
  const moveHandler = (e: MouseEvent) => onVinylDocumentMouseMove(deck, e)
  const upHandler = () => onVinylDocumentMouseUp(deck)
  
  // Store handlers for cleanup
  if (deck === 'A') {
    document.addEventListener('mousemove', onVinylADocumentMouseMove)
    document.addEventListener('mouseup', onVinylADocumentMouseUp)
  } else {
    document.addEventListener('mousemove', onVinylBDocumentMouseMove)
    document.addEventListener('mouseup', onVinylBDocumentMouseUp)
  }
}

function onVinylDocumentMouseMove(deck: 'A' | 'B', event: MouseEvent) {
  const state = deckStates[deck]
  if (!state.scratchActive.value || !audioEngine || state.idx.value === null || !state.scratchTarget.value) return
  
  const currentAngle = getMouseAngle(event, state.scratchTarget.value)
  const currentTime = Date.now()
  const deltaAngle = currentAngle - state.scratchLastAngle.value
  const deltaTime = Math.max(1, currentTime - state.scratchLastTime.value)
  const angularVelocity = (deltaAngle / deltaTime) * 1000
  const normalAngularVelocity = 360 / getBaseRotationTime(deck)
  const scratchRate = angularVelocity / normalAngularVelocity
  
  // Applica il rate per il suono in tempo reale
  audioEngine.setFilePlaybackRate(state.idx.value, scratchRate)
  
  // Calcola il movimento nel tempo basato sull'angolo
  const baseRotationTime = getBaseRotationTime(deck)
  const deltaTimeInAudio = (deltaAngle / 360) * baseRotationTime
  
  // Aggiorna la posizione nel brano
  const newTime = Math.max(0, Math.min(state.duration.value, state.currentTime.value + deltaTimeInAudio))
  state.currentTime.value = newTime
  state.playbackOffset.value = newTime
  
  // Fa seek per muovere davvero la posizione
  audioEngine.seekFile(state.idx.value, newTime)
  
  // RICALCOLA la rotazione DIRETTAMENTE dal currentTime (sempre sincronizzato)
  const rotations = state.currentTime.value / baseRotationTime
  state.rotation.value = rotations * 360
  
  state.scratchLastAngle.value = currentAngle
  state.scratchLastTime.value = currentTime
}

function onVinylDocumentMouseUp(deck: 'A' | 'B') {
  const state = deckStates[deck]
  
  if (deck === 'A') {
    document.removeEventListener('mousemove', onVinylADocumentMouseMove)
    document.removeEventListener('mouseup', onVinylADocumentMouseUp)
  } else {
    document.removeEventListener('mousemove', onVinylBDocumentMouseMove)
    document.removeEventListener('mouseup', onVinylBDocumentMouseUp)
  }
  
  if (!state.scratchActive.value || !audioEngine || state.idx.value === null) return
  
  // Calcola il rate normale considerando il pitch
  const normalRate = 1 + state.pitch.value / 100
  
  // Aggiorna il rate tracciato
  if (deck === 'A') deckACurrentRate = normalRate
  else deckBCurrentRate = normalRate
  
  if (state.scratchWasPlaying.value) {
    // Era in riproduzione, ripristina il rate con il pitch applicato
    audioEngine.setFilePlaybackRate(state.idx.value, normalRate)
    state.playbackStartTime.value = Date.now()
    state.playbackOffset.value = state.currentTime.value
  } else {
    // Era in pausa, ripristina il rate normale e metti in pausa
    audioEngine.setFilePlaybackRate(state.idx.value, normalRate)
    audioEngine.pauseFile(state.idx.value)
  }
  
  state.scratchActive.value = false
  state.scratchTarget.value = null
}

// Transport controls
function onPlayPause(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  const isPlaying = deck === 'A' ? deckAIsPlaying.value : deckBIsPlaying.value
  if (!audioEngine || state.idx.value === null || !hasFile) return
  
  if (isPlaying) {
    audioEngine.pauseFile(state.idx.value)
  } else {
    // Quando si preme play, assicurati che il rate corretto sia applicato
    const rate = 1 + state.pitch.value / 100
    if (deck === 'A') deckACurrentRate = rate
    else deckBCurrentRate = rate
    audioEngine.setFilePlaybackRate(state.idx.value, rate)
    audioEngine.playFile(state.idx.value)
  }
}

function onStop(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  if (!audioEngine || state.idx.value === null || !hasFile) return
  audioEngine.stopFile(state.idx.value)
  state.currentTime.value = 0
  state.playbackOffset.value = 0
  state.playbackStartTime.value = Date.now()
  state.rotation.value = 0
  // Disattiva il loop
  if (deck === 'A') {
    deckALoopActive.value = false
    deckALoopFraction.value = 0
  } else {
    deckBLoopActive.value = false
    deckBLoopFraction.value = 0
  }
  // Ripristina il rate corretto dopo lo stop
  const rate = 1 + state.pitch.value / 100
  if (deck === 'A') deckACurrentRate = rate
  else deckBCurrentRate = rate
  audioEngine.setFilePlaybackRate(state.idx.value, rate)
}

function onCuePress(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  const isPlaying = deck === 'A' ? deckAIsPlaying.value : deckBIsPlaying.value
  if (!audioEngine || state.idx.value === null || !hasFile) return
  
  // Se un loop è attivo, premere CUE lo disattiva
  const loopActive = deck === 'A' ? deckALoopActive.value : deckBLoopActive.value
  if (loopActive) {
    if (deck === 'A') {
      deckALoopActive.value = false
      deckALoopFraction.value = 0
    } else {
      deckBLoopActive.value = false
      deckBLoopFraction.value = 0
    }
    return
  }
  
  if (isPlaying) {
    // Durante la riproduzione, salva solo il cue point SENZA fermare l'audio
    state.cuePoint.value = state.currentTime.value
  } else {
    // In pausa, attiva il cue hold per preview
    state.cueHeld.value = true
    state.currentTime.value = state.cuePoint.value
    state.playbackOffset.value = state.cuePoint.value
    audioEngine.seekFile(state.idx.value, state.cuePoint.value)
    // Applica il rate corretto prima di fare play
    const rate = 1 + state.pitch.value / 100
    audioEngine.setFilePlaybackRate(state.idx.value, rate)
    audioEngine.playFile(state.idx.value)
  }
}

function onCueRelease(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  if (!audioEngine || state.idx.value === null || !hasFile) return
  
  if (state.cueHeld.value) {
    state.cueHeld.value = false
    audioEngine.pauseFile(state.idx.value)
    state.currentTime.value = state.cuePoint.value
    state.playbackOffset.value = state.cuePoint.value
    audioEngine.seekFile(state.idx.value, state.cuePoint.value)
    // Ripristina il rate corretto anche quando si rilascia il CUE
    const rate = 1 + state.pitch.value / 100
    audioEngine.setFilePlaybackRate(state.idx.value, rate)
  }
}

function onDeckVolume(deck: 'A' | 'B') {
  const state = deckStates[deck]
  if (!audioEngine || state.idx.value === null) return
  const cfFactor = deck === 'A' ? cfFactorA(crossfader.value) : cfFactorB(crossfader.value)
  audioEngine.setTrackVolume(state.idx.value, state.volume.value * cfFactor)
}

function onLoad(deck: 'A' | 'B') {
  const state = deckStates[deck]
  if (state.idx.value !== null) {
    emit('open-library', state.idx.value + 1)
  }
}

function onPitchChange(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const isPlaying = deck === 'A' ? deckAIsPlaying.value : deckBIsPlaying.value
  if (!audioEngine || state.idx.value === null) return
  
  // Calcola il rate dal pitch (pitch è una percentuale, es: +8% = 1.08, -8% = 0.92)
  const newRate = 1 + state.pitch.value / 100
  const oldRate = deck === 'A' ? deckACurrentRate : deckBCurrentRate
  
  // Applica il rate SOLO se il file sta suonando E non stiamo facendo scratching
  if (isPlaying && !state.scratchActive.value) {
    // Reset dei tempi SOLO se il rate è effettivamente cambiato
    if (Math.abs(newRate - oldRate) > 0.001) {
      state.playbackOffset.value = state.currentTime.value
      state.playbackStartTime.value = Date.now()
      
      if (deck === 'A') deckACurrentRate = newRate
      else deckBCurrentRate = newRate
    }
    
    audioEngine.setFilePlaybackRate(state.idx.value, newRate)
  }
}

// ══════════════════ DECK-SPECIFIC WRAPPERS ══════════════════

// Deck A wrappers
const loadWaveformA = () => loadWaveform('A')
const loadWaveformB = () => loadWaveform('B')
const onSeekA = (time: number) => onSeek('A', time)
const onSeekB = (time: number) => onSeek('B', time)
const onSeekReleaseA = () => {}
const onSeekReleaseB = () => {}
const onVinylAMouseDown = (event: MouseEvent) => onVinylMouseDown('A', event)
const onVinylBMouseDown = (event: MouseEvent) => onVinylMouseDown('B', event)
const onVinylADocumentMouseMove = (event: MouseEvent) => onVinylDocumentMouseMove('A', event)
const onVinylBDocumentMouseMove = (event: MouseEvent) => onVinylDocumentMouseMove('B', event)
const onVinylADocumentMouseUp = () => onVinylDocumentMouseUp('A')
const onVinylBDocumentMouseUp = () => onVinylDocumentMouseUp('B')
const onPlayPauseA = () => onPlayPause('A')
const onPlayPauseB = () => onPlayPause('B')
const onStopA = () => onStop('A')
const onStopB = () => onStop('B')
const onCueAPress = () => onCuePress('A')
const onCueBPress = () => onCuePress('B')
const onCueARelease = () => onCueRelease('A')
const onCueBRelease = () => onCueRelease('B')
const onDeckAVolume = () => onDeckVolume('A')
const onDeckBVolume = () => onDeckVolume('B')
const onLoadA = () => onLoad('A')
const onLoadB = () => onLoad('B')
const onPitchAChange = () => onPitchChange('A')
const onPitchBChange = () => onPitchChange('B')

// Tap tempo handlers
function onTapTempo(deck: 'A' | 'B') {
  const tapTimes = deck === 'A' ? deckATapTimes : deckBTapTimes
  const now = Date.now()
  
  // Reset if last tap was too long ago
  if (tapTimes.value.length > 0 && now - tapTimes.value[tapTimes.value.length - 1] > TAP_TIMEOUT) {
    tapTimes.value = []
  }
  
  // Add current tap
  tapTimes.value.push(now)
  
  // Keep only last MAX_TAPS
  if (tapTimes.value.length > MAX_TAPS) {
    tapTimes.value.shift()
  }
  
  // Calculate BPM if we have enough taps
  if (tapTimes.value.length >= MIN_TAPS) {
    const intervals: number[] = []
    for (let i = 1; i < tapTimes.value.length; i++) {
      intervals.push(tapTimes.value[i] - tapTimes.value[i - 1])
    }
    
    // Average interval in milliseconds
    const avgInterval = intervals.reduce((a, b) => a + b, 0) / intervals.length
    
    // Convert to BPM (60000 ms per minute / interval)
    const bpm = Math.round(60000 / avgInterval)
    
    // Update tapped BPM
    if (deck === 'A') {
      deckATappedBpm.value = bpm
    } else {
      deckBTappedBpm.value = bpm
    }
    
    console.log(`Deck ${deck} TAP BPM: ${bpm}`)
  }
}

const onTapTempoA = () => onTapTempo('A')
const onTapTempoB = () => onTapTempo('B')

// Loop speed handlers - cambia la velocità di riproduzione del loop manuale
function onLoopSpeed(deck: 'A' | 'B', speedFraction: number) {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  const loopInSet = deck === 'A' ? deckALoopInSet.value : deckBLoopInSet.value
  const loopOutSet = deck === 'A' ? deckALoopOutSet.value : deckBLoopOutSet.value
  const loopStart = deck === 'A' ? deckALoopStart.value : deckBLoopStart.value
  const originalLoopEnd = deck === 'A' ? deckALoopOriginalEnd.value : deckBLoopOriginalEnd.value
  
  if (!audioEngine || state.idx.value === null || !hasFile || !loopInSet || !loopOutSet) return
  
  // Calcola la durata originale del loop
  const originalLoopDuration = originalLoopEnd - loopStart
  
  // Calcola la nuova durata del loop in base alla frazione
  const newLoopDuration = originalLoopDuration * speedFraction
  const newLoopEnd = loopStart + newLoopDuration
  
  // Aggiorna il loop con la nuova durata
  if (deck === 'A') {
    deckALoopEnd.value = newLoopEnd
    deckALoopFraction.value = speedFraction
    deckALoopActive.value = true // Assicurati che il loop sia attivo
  } else {
    deckBLoopEnd.value = newLoopEnd
    deckBLoopFraction.value = speedFraction
    deckBLoopActive.value = true // Assicurati che il loop sia attivo
  }
  
  console.log(`Deck ${deck} Loop Speed: ${speedFraction} (durata: ${newLoopDuration.toFixed(2)}s)`)
}

const onLoopSpeedA = (fraction: number) => onLoopSpeed('A', fraction)
const onLoopSpeedB = (fraction: number) => onLoopSpeed('B', fraction)

// Manual Loop In/Out handlers
function onLoopIn(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  
  if (!audioEngine || state.idx.value === null || !hasFile) return
  
  // Imposta il punto di inizio del loop alla posizione corrente
  const loopStart = state.currentTime.value
  
  if (deck === 'A') {
    deckALoopStart.value = loopStart
    deckALoopInSet.value = true
    deckACuePoint.value = loopStart // Aggiorna anche il cue point
    // Se era già attivo un loop, disattivalo e resetta il loop OUT
    if (deckALoopActive.value) {
      deckALoopActive.value = false
      deckALoopOutSet.value = false
      deckALoopOriginalEnd.value = 0
      deckALoopFraction.value = 0
    }
  } else {
    deckBLoopStart.value = loopStart
    deckBLoopInSet.value = true
    deckBCuePoint.value = loopStart // Aggiorna anche il cue point
    // Se era già attivo un loop, disattivalo e resetta il loop OUT
    if (deckBLoopActive.value) {
      deckBLoopActive.value = false
      deckBLoopOutSet.value = false
      deckBLoopOriginalEnd.value = 0
      deckBLoopFraction.value = 0
    }
  }
  
  console.log(`Deck ${deck} Loop IN impostato a: ${loopStart.toFixed(2)}s`)
}

function onLoopOut(deck: 'A' | 'B') {
  const state = deckStates[deck]
  const hasFile = deck === 'A' ? deckAHasFile.value : deckBHasFile.value
  const loopInSet = deck === 'A' ? deckALoopInSet.value : deckBLoopInSet.value
  const loopStart = deck === 'A' ? deckALoopStart.value : deckBLoopStart.value
  
  if (!audioEngine || state.idx.value === null || !hasFile || !loopInSet) return
  
  // Imposta il punto di fine del loop alla posizione corrente
  const loopEnd = state.currentTime.value
  
  // Verifica che il punto di fine sia dopo il punto di inizio
  if (loopEnd <= loopStart) {
    console.warn(`Deck ${deck} Loop OUT (${loopEnd.toFixed(2)}s) deve essere dopo Loop IN (${loopStart.toFixed(2)}s)`)
    return
  }
  
  if (deck === 'A') {
    deckALoopEnd.value = loopEnd
    deckALoopOriginalEnd.value = loopEnd // Salva il loop end originale
    deckALoopOutSet.value = true
    deckALoopFraction.value = 1 // Frazione 1 = loop completo (resettato quando si usa loop speed)
    // Attiva automaticamente il loop
    deckALoopActive.value = true
  } else {
    deckBLoopEnd.value = loopEnd
    deckBLoopOriginalEnd.value = loopEnd // Salva il loop end originale
    deckBLoopOutSet.value = true
    deckBLoopFraction.value = 1 // Frazione 1 = loop completo (resettato quando si usa loop speed)
    // Attiva automaticamente il loop
    deckBLoopActive.value = true
  }
  
  console.log(`Deck ${deck} Loop OUT impostato a: ${loopEnd.toFixed(2)}s (durata: ${(loopEnd - loopStart).toFixed(2)}s) - Loop ATTIVATO`)
}

const onLoopInA = () => onLoopIn('A')
const onLoopOutA = () => onLoopOut('A')
const onLoopInB = () => onLoopIn('B')
const onLoopOutB = () => onLoopOut('B')

// Start tracking playback position
function startPlaybackTracking() {
  if (playbackIntervalId !== null) return
  
  playbackIntervalId = setInterval(() => {
    for (const deck of ['A', 'B'] as const) {
      const state = deckStates[deck]
      const isPlaying = deck === 'A' ? deckAIsPlaying.value : deckBIsPlaying.value
      
      if (isPlaying && !state.scratchActive.value) {
        // Usa il rate tracciato, non ricalcolarlo dal pitch ogni volta
        const currentRate = deck === 'A' ? deckACurrentRate : deckBCurrentRate
        const elapsed = (Date.now() - state.playbackStartTime.value) / 1000
        state.currentTime.value = state.playbackOffset.value + (elapsed * currentRate)
        
        // Gestisci loop se attivo
        const loopActive = deck === 'A' ? deckALoopActive.value : deckBLoopActive.value
        if (loopActive) {
          const loopStart = deck === 'A' ? deckALoopStart.value : deckBLoopStart.value
          const loopEnd = deck === 'A' ? deckALoopEnd.value : deckBLoopEnd.value
          
          if (state.currentTime.value >= loopEnd) {
            // Riporta al punto di inizio del loop
            state.currentTime.value = loopStart
            state.playbackOffset.value = loopStart
            state.playbackStartTime.value = Date.now()
            audioEngine.seekFile(state.idx.value, loopStart)
          }
        }
        
        // Update rotation - baseRotationTime già tiene conto del pitch!
        const baseRotationTime = getBaseRotationTime(deck)
        const rotations = state.currentTime.value / baseRotationTime
        state.rotation.value = rotations * 360
      }
    }
  }, 16)
}

// Sync initial volume from engine on mount
onMounted(() => {
  for (const deck of ['A', 'B'] as const) {
    const state = deckStates[deck]
    if (state.idx.value !== null) {
      const p = audioEngine?.state.value.trackParameters.get(state.idx.value)
      if (p?.volume != null) state.volume.value = p.volume
    }
  }
})

// Engine state reads
const deckAParams = computed(() =>
  deckAIdx.value !== null ? audioEngine?.state.value.trackParameters.get(deckAIdx.value) : null
)
const deckBParams = computed(() =>
  deckBIdx.value !== null ? audioEngine?.state.value.trackParameters.get(deckBIdx.value) : null
)
const deckALevels = computed(() =>
  deckAIdx.value !== null ? audioEngine?.state.value.trackLevels.get(deckAIdx.value) : null
)
const deckBLevels = computed(() =>
  deckBIdx.value !== null ? audioEngine?.state.value.trackLevels.get(deckBIdx.value) : null
)

const deckAIsPlaying = computed(() => deckAParams.value?.isPlaying ?? false)
const deckBIsPlaying = computed(() => deckBParams.value?.isPlaying ?? false)

// Watch for playback state changes to update tracking (MUST be after deckAIsPlaying/deckBIsPlaying definitions)
watch(() => deckAIsPlaying.value, (isPlaying) => {
  const state = deckStates.A
  if (isPlaying) {
    state.playbackStartTime.value = Date.now()
    state.playbackOffset.value = state.currentTime.value
    startPlaybackTracking()
  } else {
    state.playbackOffset.value = state.currentTime.value
  }
}, { immediate: false })

watch(() => deckBIsPlaying.value, (isPlaying) => {
  const state = deckStates.B
  if (isPlaying) {
    state.playbackStartTime.value = Date.now()
    state.playbackOffset.value = state.currentTime.value
    startPlaybackTracking()
  } else {
    state.playbackOffset.value = state.currentTime.value
  }
}, { immediate: false })

// Reset time when files change
watch(() => deckAParams.value?.fileName, (newFileName, oldFileName) => {
  if (newFileName !== undefined) {
    const state = deckStates.A
    // Stop playback tracking when file changes
    if (playbackIntervalId !== null) {
      clearInterval(playbackIntervalId)
      playbackIntervalId = null
    }
    state.currentTime.value = 0
    state.cuePoint.value = 0
    state.playbackStartTime.value = 0
    state.playbackOffset.value = 0
    state.rotation.value = 0
    
    // Disattiva il loop
    deckALoopActive.value = false
    deckALoopFraction.value = 0
    deckALoopInSet.value = false
    deckALoopOutSet.value = false
    deckALoopOriginalEnd.value = 0
    
    // Inizializza il playback rate al valore del pitch corrente
    if (newFileName && state.idx.value !== null && audioEngine) {
      const rate = 1 + state.pitch.value / 100
      audioEngine.setFilePlaybackRate(state.idx.value, rate)
    }
    
    // Load waveform if file changed
    if (newFileName && newFileName !== oldFileName) {
      loadWaveformA()
    }
  }
}, { flush: 'post' })

watch(() => deckBParams.value?.fileName, (newFileName, oldFileName) => {
  if (newFileName !== undefined) {
    const state = deckStates.B
    // Stop playback tracking when file changes
    if (playbackIntervalId !== null) {
      clearInterval(playbackIntervalId)
      playbackIntervalId = null
    }
    state.currentTime.value = 0
    state.cuePoint.value = 0
    state.playbackStartTime.value = 0
    state.playbackOffset.value = 0
    state.rotation.value = 0
    
    // Disattiva il loop
    deckBLoopActive.value = false
    deckBLoopFraction.value = 0
    deckBLoopInSet.value = false
    deckBLoopOutSet.value = false
    deckBLoopOriginalEnd.value = 0
    
    // Inizializza il playback rate al valore del pitch corrente
    if (newFileName && state.idx.value !== null && audioEngine) {
      const rate = 1 + state.pitch.value / 100
      audioEngine.setFilePlaybackRate(state.idx.value, rate)
    }
    
    // Load waveform if file changed
    if (newFileName && newFileName !== oldFileName) {
      loadWaveformB()
    }
  }
}, { flush: 'post' })

// Cleanup on unmount
onUnmounted(() => {
  if (playbackIntervalId !== null) {
    clearInterval(playbackIntervalId)
    playbackIntervalId = null
  }
  
  document.removeEventListener('mousemove', onVinylADocumentMouseMove)
  document.removeEventListener('mouseup', onVinylADocumentMouseUp)
  document.removeEventListener('mousemove', onVinylBDocumentMouseMove)
  document.removeEventListener('mouseup', onVinylBDocumentMouseUp)
})

// Check if decks have files loaded
const deckAHasFile = computed(() => {
  const p = deckAParams.value
  const hasFile = p?.fileName && p.fileName.length > 0
  return hasFile
})
const deckBHasFile = computed(() => {
  const p = deckBParams.value
  const hasFile = p?.fileName && p.fileName.length > 0
  return hasFile
})

const deckATitle = computed(() => {
  const p = deckAParams.value
  if (!p) return props.deckATrack ? 'EMPTY' : 'NO TRACK'
  return p.fileTitle || p.fileName?.split('/').pop()?.replace(/\.[^.]+$/, '') || 'NO FILE'
})
const deckAArtist = computed(() => deckAParams.value?.fileArtist || '---')

const deckBTitle = computed(() => {
  const p = deckBParams.value
  if (!p) return props.deckBTrack ? 'EMPTY' : 'NO TRACK'
  return p.fileTitle || p.fileName?.split('/').pop()?.replace(/\.[^.]+$/, '') || 'NO FILE'
})
const deckBArtist = computed(() => deckBParams.value?.fileArtist || '---')

// BPM with tap tempo override
const deckABpm = computed(() => deckATappedBpm.value > 0 ? deckATappedBpm.value : (deckALevels.value?.bpm ?? 0))
const deckBBpm = computed(() => deckBTappedBpm.value > 0 ? deckBTappedBpm.value : (deckBLevels.value?.bpm ?? 0))

// Convert linear amplitude levels to dB (same as AudioTrack)
const deckALevelL = computed(() => {
  const level = deckALevels.value?.left
  return level && level > 0 ? 20 * Math.log10(level) : -60
})
const deckALevelR = computed(() => {
  const level = deckALevels.value?.right
  return level && level > 0 ? 20 * Math.log10(level) : -60
})
const deckBLevelL = computed(() => {
  const level = deckBLevels.value?.left
  return level && level > 0 ? 20 * Math.log10(level) : -60
})
const deckBLevelR = computed(() => {
  const level = deckBLevels.value?.right
  return level && level > 0 ? 20 * Math.log10(level) : -60
})

const masterLevelL = computed(() => {
  const level = audioEngine?.state.value.masterLevels?.left
  return level && level > 0 ? 20 * Math.log10(level) : -60
})
const masterLevelR = computed(() => {
  const level = audioEngine?.state.value.masterLevels?.right
  return level && level > 0 ? 20 * Math.log10(level) : -60
})

// Helper for level meters
function levelPct(db: number): string {
  return Math.max(0, Math.min(100, (db + 60) / 60 * 100)).toFixed(1) + '%'
}

// Standard DJ crossfader curve
// Standard DJ crossfader curve
function cfFactorA(cf: number): number {
  return cf <= 0.5 ? 1.0 : 1.0 - (cf - 0.5) * 2.0
}

function cfFactorB(cf: number): number {
  return cf >= 0.5 ? 1.0 : cf * 2.0
}

function onCrossfader() {
  if (!audioEngine) return
  const cf = crossfader.value
  if (deckAIdx.value !== null)
    audioEngine.setTrackVolume(deckAIdx.value, deckAVolume.value * cfFactorA(cf))
  if (deckBIdx.value !== null)
    audioEngine.setTrackVolume(deckBIdx.value, deckBVolume.value * cfFactorB(cf))
}

function syncDecks() {
  if (deckABpm.value > 0 && deckBBpm.value > 0) {
    deckBPitch.value = Math.max(-8, Math.min(8, (deckABpm.value / deckBBpm.value - 1) * 100))
  }
}

const emit = defineEmits<{
  'open-library': [trackId: number]
}>()
</script>

<style scoped>
.vinyl-disc {
  user-select: none;
  will-change: transform; /* GPU acceleration for smooth rotation */
}

/* Vertical range input (writing-mode approach — solid cross-browser support) */
.v-slider {
  writing-mode: vertical-lr;
  direction: rtl;
  -webkit-appearance: slider-vertical;
  appearance: slider-vertical;
  cursor: pointer;
  /* stretch to fill flex column */
  flex: 1 1 0;
  width: 20px;
  min-height: 50px;
}

.v-slider:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.level-bar {
  background: linear-gradient(to right, #10b981 0%, #facc15 65%, #ef4444 100%);
}

/* Waveform monitor styling - higher */
.waveform-monitor :deep(canvas) {
  height: 90px !important;
}
</style>
