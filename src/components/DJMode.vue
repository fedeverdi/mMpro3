<template>
  <div class="dj-wrap w-full h-full flex gap-2 p-2 select-none overflow-hidden bg-gradient-to-b from-gray-950 to-black">

    <!-- ══════════════════ DECK A ══════════════════ -->
    <div class="deck flex-1 flex flex-col gap-1.5 bg-gray-900/80 border border-cyan-500/20 rounded-xl p-3 min-w-0 overflow-hidden">

      <!-- Header -->
      <div class="flex items-center justify-between flex-shrink-0">
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-cyan-400 transition-all"
                :class="deckAIsPlaying ? 'animate-pulse shadow-[0_0_8px_2px_rgba(34,211,238,0.7)]' : 'opacity-50'"></span>
          <span class="text-[11px] font-black text-cyan-400 tracking-[0.3em] uppercase">Deck A</span>
        </div>
        <span class="text-[10px] text-gray-500 font-mono tabular-nums">
          {{ deckABpm > 0 ? deckABpm.toFixed(1) + ' BPM' : '· · · BPM' }}
        </span>
      </div>

      <!-- Track info -->
      <div class="text-center flex-shrink-0 min-h-[2rem]">
        <div class="text-xs font-semibold text-white truncate leading-tight">{{ deckATitle }}</div>
        <div class="text-[10px] text-gray-400 truncate">{{ deckAArtist }}</div>
      </div>

      <!-- Waveform Monitor -->
      <div v-if="deckAIdx !== null" class="flex-shrink-0 px-1 waveform-monitor">
        <WaveformDisplay 
          :track-number="deckAIdx"
          :audio-buffer="deckAAudioBuffer"
          :current-time="deckACurrentTime"
          :is-playing="deckAIsPlaying"
          :duration="deckADuration"
          :show-mode-buttons="false"
          mode="waveform"
          waveform-color="#ffffff"
          @seek="(time) => onSeekA(time)"
          @seek-release="onSeekReleaseA"
        />
      </div>

      <!-- Vinyl + sliders row (flex-1 takes remaining space) -->
      <div class="flex gap-2 flex-1 items-center justify-center min-h-0 overflow-hidden">

        <!-- Pitch slider (left) -->
        <div class="flex flex-col items-center gap-1 self-stretch w-7 flex-shrink-0">
          <span class="text-[8px] text-gray-500 font-mono tabular-nums leading-none">
            {{ deckAPitch > 0 ? '+' : '' }}{{ deckAPitch.toFixed(1) }}%
          </span>
          <input type="range" min="-8" max="8" step="0.1" v-model.number="deckAPitch"
                 @input="onPitchAChange"
                 :disabled="!deckAHasFile"
                 class="v-slider accent-cyan-400" />
          <span class="text-[8px] text-gray-600 leading-none tracking-widest">PITCH</span>
        </div>

        <!-- Vinyl record -->
        <div class="flex-1 flex items-center justify-center min-h-0 min-w-0">
          <svg viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet"
               class="vinyl-disc w-full h-full"
               :style="{
                 '--spin-dur': vinylSpeed(deckAPitch),
                 animationPlayState: deckAIsPlaying ? 'running' : 'paused',
                 filter: `drop-shadow(0 0 ${deckAGlow * 14}px rgba(34,211,238,${deckAGlow * 0.25}))`
               }">
            <!-- Disc body -->
            <circle cx="100" cy="100" r="99" fill="#080808"/>
            <circle cx="100" cy="100" r="98" fill="none" stroke="#1c1c1c" stroke-width="0.5"/>
            <!-- Grooves -->
            <circle v-for="i in 32" :key="i" cx="100" cy="100"
                    :r="94 - (i - 1) * 1.85" fill="none"
                    :stroke="`rgba(255,255,255,${0.035 + i * 0.0018})`" stroke-width="0.65"/>
            <!-- Sheen highlight -->
            <ellipse cx="72" cy="62" rx="24" ry="13" fill="rgba(255,255,255,0.022)"
                     transform="rotate(-25 72 62)"/>
            <ellipse cx="135" cy="148" rx="16" ry="8" fill="rgba(255,255,255,0.01)"
                     transform="rotate(-25 135 148)"/>
            <!-- Label background -->
            <circle cx="100" cy="100" r="31" fill="#071320"/>
            <circle cx="100" cy="100" r="30" fill="none" stroke="rgba(34,211,238,0.5)" stroke-width="1"/>
            <circle cx="100" cy="100" r="26.5" fill="none" stroke="rgba(34,211,238,0.12)" stroke-width="0.4"/>
            <!-- Label text -->
            <text x="100" y="88" text-anchor="middle" fill="rgba(34,211,238,0.35)"
                  font-size="4" font-family="monospace" letter-spacing="3">DECK A</text>
            <text x="100" y="97" text-anchor="middle" fill="rgba(34,211,238,0.95)"
                  font-size="5.8" font-family="monospace" font-weight="bold">
              {{ deckATitle.substring(0, 13) }}
            </text>
            <text x="100" y="106" text-anchor="middle" fill="rgba(255,255,255,0.4)"
                  font-size="4" font-family="monospace">
              {{ deckAArtist.substring(0, 18) }}
            </text>
            <!-- Label inner ring decoration -->
            <line x1="100" y1="70" x2="100" y2="75" stroke="rgba(34,211,238,0.3)" stroke-width="0.8"/>
            <!-- Center spindle -->
            <circle cx="100" cy="100" r="4" fill="#141414"/>
            <circle cx="100" cy="100" r="2.2" fill="#0d0d0d"/>
            <circle cx="100" cy="100" r="0.9" fill="#222"/>
          </svg>
        </div>

        <!-- Volume fader (right) -->
        <div class="flex flex-col items-center gap-1 self-stretch w-7 flex-shrink-0">
          <span class="text-[8px] text-gray-400 font-mono tabular-nums leading-none">
            {{ Math.round(deckAVolume * 100) }}%
          </span>
          <input type="range" min="0" max="1.5" step="0.01" v-model.number="deckAVolume"
                 class="v-slider accent-cyan-400" @input="onDeckAVolume"/>
          <span class="text-[8px] text-gray-600 leading-none tracking-widest">VOL</span>
        </div>
      </div>

      <!-- Level meters -->
      <div class="flex flex-col gap-0.5 flex-shrink-0">
        <div class="h-1.5 bg-gray-800/80 rounded-full overflow-hidden">
          <div class="h-full rounded-full level-bar transition-[width] duration-75"
               :style="{ width: levelPct(deckALevelL) }"/>
        </div>
        <div class="h-1.5 bg-gray-800/80 rounded-full overflow-hidden">
          <div class="h-full rounded-full level-bar transition-[width] duration-75"
               :style="{ width: levelPct(deckALevelR) }"/>
        </div>
      </div>

      <!-- Transport controls -->
      <div class="flex flex-col gap-1.5 flex-shrink-0">
        <div class="flex gap-1.5 justify-center items-center">
          <button @mousedown="onCueAPress" @mouseup="onCueARelease" @mouseleave="onCueARelease"
                  class="px-3 py-1.5 text-[10px] font-bold rounded transition-all tracking-wider bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-700 hover:border-gray-500 cursor-pointer">
            CUE
          </button>
          <button @click="onPlayPauseA"
                  :disabled="!deckAHasFile"
                  class="w-12 h-9 rounded-lg font-bold transition-all flex items-center justify-center"
                  :class="!deckAHasFile
                    ? 'bg-gray-900 text-gray-600 border border-gray-800 cursor-not-allowed opacity-50'
                    : deckAIsPlaying
                      ? 'bg-cyan-600 text-white shadow-[0_0_12px_rgba(34,211,238,0.4)] hover:bg-cyan-500'
                      : 'bg-gray-800 text-cyan-400 border border-cyan-800/60 hover:bg-cyan-900/30'">
            <svg v-if="!deckAIsPlaying" class="w-4 h-4 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z"/>
            </svg>
            <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
          </button>
          <button @click="onStopA"
                  :disabled="!deckAHasFile"
                  class="px-3 py-1.5 text-[11px] font-bold rounded transition-all"
                  :class="deckAHasFile
                    ? 'bg-gray-800 hover:bg-red-900/40 text-gray-300 hover:text-red-400 border border-gray-700 hover:border-red-800/60 cursor-pointer'
                    : 'bg-gray-900 text-gray-600 border border-gray-800 cursor-not-allowed opacity-50'">
            ■
          </button>
        </div>
        <button @click="onLoadA"
                class="w-full py-1.5 text-[10px] font-bold rounded bg-cyan-900/30 hover:bg-cyan-800/50 text-cyan-300 border border-cyan-700/40 hover:border-cyan-500/60 transition-all tracking-wider">
          LOAD FILE
        </button>
      </div>
    </div>

    <!-- ══════════════════ CENTER STRIP ══════════════════ -->
    <div class="center-strip w-16 flex-shrink-0 flex flex-col items-center justify-between py-3">

      <div class="flex flex-col items-center gap-1">
        <div class="w-px h-8 bg-gradient-to-b from-transparent to-cyan-500/40"></div>
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
        <div class="w-px h-8 bg-gradient-to-t from-transparent to-orange-500/40"></div>
      </div>
    </div>

    <!-- ══════════════════ DECK B ══════════════════ -->
    <div class="deck flex-1 flex flex-col gap-1.5 bg-gray-900/80 border border-orange-500/20 rounded-xl p-3 min-w-0 overflow-hidden">

      <!-- Header -->
      <div class="flex items-center justify-between flex-shrink-0">
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-orange-400 transition-all"
                :class="deckBIsPlaying ? 'animate-pulse shadow-[0_0_8px_2px_rgba(251,146,60,0.7)]' : 'opacity-50'"></span>
          <span class="text-[11px] font-black text-orange-400 tracking-[0.3em] uppercase">Deck B</span>
        </div>
        <span class="text-[10px] text-gray-500 font-mono tabular-nums">
          {{ deckBBpm > 0 ? deckBBpm.toFixed(1) + ' BPM' : '· · · BPM' }}
        </span>
      </div>

      <!-- Track info -->
      <div class="text-center flex-shrink-0 min-h-[2rem]">
        <div class="text-xs font-semibold text-white truncate leading-tight">{{ deckBTitle }}</div>
        <div class="text-[10px] text-gray-400 truncate">{{ deckBArtist }}</div>
      </div>

      <!-- Waveform Monitor -->
      <div v-if="deckBIdx !== null" class="flex-shrink-0 px-1 waveform-monitor">
        <WaveformDisplay 
          :track-number="deckBIdx"
          :audio-buffer="deckBAudioBuffer"
          :current-time="deckBCurrentTime"
          :is-playing="deckBIsPlaying"
          :duration="deckBDuration"
          :show-mode-buttons="false"
          mode="waveform"
          waveform-color="#ffffff"
          @seek="(time) => onSeekB(time)"
          @seek-release="onSeekReleaseB"
        />
      </div>

      <!-- Vinyl + sliders row -->
      <div class="flex gap-2 flex-1 items-center justify-center min-h-0 overflow-hidden">

        <!-- Volume fader (left for deck B — mirrored layout) -->
        <div class="flex flex-col items-center gap-1 self-stretch w-7 flex-shrink-0">
          <span class="text-[8px] text-gray-400 font-mono tabular-nums leading-none">
            {{ Math.round(deckBVolume * 100) }}%
          </span>
          <input type="range" min="0" max="1.5" step="0.01" v-model.number="deckBVolume"
                 class="v-slider accent-orange-400" @input="onDeckBVolume"/>
          <span class="text-[8px] text-gray-600 leading-none tracking-widest">VOL</span>
        </div>

        <!-- Vinyl record -->
        <div class="flex-1 flex items-center justify-center min-h-0 min-w-0">
          <svg viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet"
               class="vinyl-disc w-full h-full"
               :style="{
                 '--spin-dur': vinylSpeed(deckBPitch),
                 animationPlayState: deckBIsPlaying ? 'running' : 'paused',
                 filter: `drop-shadow(0 0 ${deckBGlow * 14}px rgba(251,146,60,${deckBGlow * 0.25}))`
               }">
            <circle cx="100" cy="100" r="99" fill="#080808"/>
            <circle cx="100" cy="100" r="98" fill="none" stroke="#1c1c1c" stroke-width="0.5"/>
            <circle v-for="i in 32" :key="i" cx="100" cy="100"
                    :r="94 - (i - 1) * 1.85" fill="none"
                    :stroke="`rgba(255,255,255,${0.035 + i * 0.0018})`" stroke-width="0.65"/>
            <ellipse cx="72" cy="62" rx="24" ry="13" fill="rgba(255,255,255,0.022)"
                     transform="rotate(-25 72 62)"/>
            <ellipse cx="135" cy="148" rx="16" ry="8" fill="rgba(255,255,255,0.01)"
                     transform="rotate(-25 135 148)"/>
            <!-- Label -->
            <circle cx="100" cy="100" r="31" fill="#1a0d04"/>
            <circle cx="100" cy="100" r="30" fill="none" stroke="rgba(251,146,60,0.5)" stroke-width="1"/>
            <circle cx="100" cy="100" r="26.5" fill="none" stroke="rgba(251,146,60,0.12)" stroke-width="0.4"/>
            <text x="100" y="88" text-anchor="middle" fill="rgba(251,146,60,0.35)"
                  font-size="4" font-family="monospace" letter-spacing="3">DECK B</text>
            <text x="100" y="97" text-anchor="middle" fill="rgba(251,146,60,0.95)"
                  font-size="5.8" font-family="monospace" font-weight="bold">
              {{ deckBTitle.substring(0, 13) }}
            </text>
            <text x="100" y="106" text-anchor="middle" fill="rgba(255,255,255,0.4)"
                  font-size="4" font-family="monospace">
              {{ deckBArtist.substring(0, 18) }}
            </text>
            <line x1="100" y1="70" x2="100" y2="75" stroke="rgba(251,146,60,0.3)" stroke-width="0.8"/>
            <circle cx="100" cy="100" r="4" fill="#141414"/>
            <circle cx="100" cy="100" r="2.2" fill="#0d0d0d"/>
            <circle cx="100" cy="100" r="0.9" fill="#222"/>
          </svg>
        </div>

        <!-- Pitch slider (right for deck B) -->
        <div class="flex flex-col items-center gap-1 self-stretch w-7 flex-shrink-0">
          <span class="text-[8px] text-gray-500 font-mono tabular-nums leading-none">
            {{ deckBPitch > 0 ? '+' : '' }}{{ deckBPitch.toFixed(1) }}%
          </span>
          <input type="range" min="-8" max="8" step="0.1" v-model.number="deckBPitch"
                 @input="onPitchBChange"
                 :disabled="!deckBHasFile"
                 class="v-slider accent-orange-400"/>
          <span class="text-[8px] text-gray-600 leading-none tracking-widest">PITCH</span>
        </div>
      </div>

      <!-- Level meters -->
      <div class="flex flex-col gap-0.5 flex-shrink-0">
        <div class="h-1.5 bg-gray-800/80 rounded-full overflow-hidden">
          <div class="h-full rounded-full level-bar transition-[width] duration-75"
               :style="{ width: levelPct(deckBLevelL) }"/>
        </div>
        <div class="h-1.5 bg-gray-800/80 rounded-full overflow-hidden">
          <div class="h-full rounded-full level-bar transition-[width] duration-75"
               :style="{ width: levelPct(deckBLevelR) }"/>
        </div>
      </div>

      <!-- Transport controls (mirrored order) -->
      <div class="flex flex-col gap-1.5 flex-shrink-0">
        <div class="flex gap-1.5 justify-center items-center">
          <button @click="onStopB"
                  :disabled="!deckBHasFile"
                  class="px-3 py-1.5 text-[11px] font-bold rounded transition-all"
                  :class="deckBHasFile
                    ? 'bg-gray-800 hover:bg-red-900/40 text-gray-300 hover:text-red-400 border border-gray-700 hover:border-red-800/60 cursor-pointer'
                    : 'bg-gray-900 text-gray-600 border border-gray-800 cursor-not-allowed opacity-50'">
            ■
          </button>
          <button @click="onPlayPauseB"
                  :disabled="!deckBHasFile"
                  class="w-12 h-9 rounded-lg font-bold transition-all flex items-center justify-center"
                  :class="!deckBHasFile
                    ? 'bg-gray-900 text-gray-600 border border-gray-800 cursor-not-allowed opacity-50'
                    : deckBIsPlaying
                      ? 'bg-orange-600 text-white shadow-[0_0_12px_rgba(251,146,60,0.4)] hover:bg-orange-500'
                      : 'bg-gray-800 text-orange-400 border border-orange-800/60 hover:bg-orange-900/30'">
            <svg v-if="!deckBIsPlaying" class="w-4 h-4 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z"/>
            </svg>
            <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
          </button>
          <button @mousedown="onCueBPress" @mouseup="onCueBRelease" @mouseleave="onCueBRelease"
                  class="px-3 py-1.5 text-[10px] font-bold rounded transition-all tracking-wider bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-700 hover:border-gray-500 cursor-pointer">
            CUE
        </button>
        </div>
        <button @click="onLoadB"
                class="w-full py-1.5 text-[10px] font-bold rounded bg-orange-900/30 hover:bg-orange-800/50 text-orange-300 border border-orange-700/40 hover:border-orange-500/60 transition-all tracking-wider">
          LOAD FILE
        </button>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, onMounted, watch, onUnmounted } from 'vue'
import WaveformDisplay from './audioTrack/WaveformDisplay.vue'

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

// Playback tracking interval
let playbackIntervalId: ReturnType<typeof setInterval> | null = null

// Start tracking playback position
function startPlaybackTracking() {
  if (playbackIntervalId !== null) return
  
  playbackIntervalId = setInterval(() => {
    // Track deck A
    if (deckAIsPlaying.value) {
      const elapsed = (Date.now() - deckAPlaybackStartTime) / 1000
      deckACurrentTime.value = deckAPlaybackOffset + elapsed
    }
    
    // Track deck B
    if (deckBIsPlaying.value) {
      const elapsed = (Date.now() - deckBPlaybackStartTime) / 1000
      deckBCurrentTime.value = deckBPlaybackOffset + elapsed
    }
  }, 50) // Update every 50ms
}

// Load waveform for deck A
async function loadWaveformA() {
  if (!audioEngine || deckAIdx.value === null) return
  
  try {
    const result = await audioEngine.getWaveformData(deckAIdx.value, 2000)
    
    if (result && result.data) {
      const useSampleRate = 8000
      const numFrames = Math.floor(result.duration * useSampleRate)
      
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
      deckAAudioBuffer.value = audioContext.createBuffer(1, numFrames, useSampleRate)
      
      const channelData = deckAAudioBuffer.value.getChannelData(0)
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
      
      deckADuration.value = result.duration
    }
  } catch (error) {
    console.error('[DJMode] Error loading waveform A:', error)
  }
}

// Load waveform for deck B
async function loadWaveformB() {
  if (!audioEngine || deckBIdx.value === null) return
  
  try {
    const result = await audioEngine.getWaveformData(deckBIdx.value, 2000)
    
    if (result && result.data) {
      const useSampleRate = 8000
      const numFrames = Math.floor(result.duration * useSampleRate)
      
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
      deckBAudioBuffer.value = audioContext.createBuffer(1, numFrames, useSampleRate)
      
      const channelData = deckBAudioBuffer.value.getChannelData(0)
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
      
      deckBDuration.value = result.duration
    }
  } catch (error) {
    console.error('[DJMode] Error loading waveform B:', error)
  }
}

// Seek handlers (from waveform drag)
function onSeekA(time: number) {
  if (!audioEngine || deckAIdx.value === null) return
  
  // Update position
  deckACurrentTime.value = time
  deckAPlaybackOffset = time
  audioEngine.seekFile(deckAIdx.value, time)
  
  // If playing, update playback start time for smooth continuation
  if (deckAIsPlaying.value) {
    deckAPlaybackStartTime = Date.now()
  }
}

function onSeekB(time: number) {
  if (!audioEngine || deckBIdx.value === null) return
  
  // Update position
  deckBCurrentTime.value = time
  deckBPlaybackOffset = time
  audioEngine.seekFile(deckBIdx.value, time)
  
  // If playing, update playback start time for smooth continuation
  if (deckBIsPlaying.value) {
    deckBPlaybackStartTime = Date.now()
  }
}

// No action needed on release - just continue playing from current position
function onSeekReleaseA() {
  // Track continues from where it was dragged
}

function onSeekReleaseB() {
  // Track continues from where it was dragged
}

// Sync initial volume from engine on mount
onMounted(() => {
  if (deckAIdx.value !== null) {
    const p = audioEngine?.state.value.trackParameters.get(deckAIdx.value)
    if (p?.volume != null) deckAVolume.value = p.volume
  }
  if (deckBIdx.value !== null) {
    const p = audioEngine?.state.value.trackParameters.get(deckBIdx.value)
    if (p?.volume != null) deckBVolume.value = p.volume
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
  if (isPlaying) {
    deckAPlaybackStartTime = Date.now()
    deckAPlaybackOffset = deckACurrentTime.value
    startPlaybackTracking()
  } else {
    deckAPlaybackOffset = deckACurrentTime.value
  }
}, { immediate: false })

watch(() => deckBIsPlaying.value, (isPlaying) => {
  if (isPlaying) {
    deckBPlaybackStartTime = Date.now()
    deckBPlaybackOffset = deckBCurrentTime.value
    startPlaybackTracking()
  } else {
    deckBPlaybackOffset = deckBCurrentTime.value
  }
}, { immediate: false })

// Reset time when files change
watch(() => deckAParams.value?.fileName, (newFileName, oldFileName) => {
  if (newFileName !== undefined) {
    // Stop playback tracking when file changes
    if (playbackIntervalId !== null) {
      clearInterval(playbackIntervalId)
      playbackIntervalId = null
    }
    deckACurrentTime.value = 0
    deckACuePoint.value = 0
    deckAPlaybackStartTime = 0
    deckAPlaybackOffset = 0
    
    // Load waveform if file changed
    if (newFileName && newFileName !== oldFileName) {
      loadWaveformA()
    }
  }
}, { flush: 'post' })

watch(() => deckBParams.value?.fileName, (newFileName, oldFileName) => {
  if (newFileName !== undefined) {
    // Stop playback tracking when file changes
    if (playbackIntervalId !== null) {
      clearInterval(playbackIntervalId)
      playbackIntervalId = null
    }
    deckBCurrentTime.value = 0
    deckBCuePoint.value = 0
    deckBPlaybackStartTime = 0
    deckBPlaybackOffset = 0
    
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

const deckABpm = computed(() => deckALevels.value?.bpm ?? 0)
const deckBBpm = computed(() => deckBLevels.value?.bpm ?? 0)

const deckALevelL = computed(() => deckALevels.value?.left ?? -60)
const deckALevelR = computed(() => deckALevels.value?.right ?? -60)
const deckBLevelL = computed(() => deckBLevels.value?.left ?? -60)
const deckBLevelR = computed(() => deckBLevels.value?.right ?? -60)

const masterLevelL = computed(() => audioEngine?.state.value.masterLevels?.left ?? -60)
const masterLevelR = computed(() => audioEngine?.state.value.masterLevels?.right ?? -60)

// Level glow for vinyl halo (0–1 from audio levels)
const deckAGlow = computed(() =>
  Math.max(0, Math.min(1, (Math.max(deckALevelL.value, deckALevelR.value) + 60) / 60))
)
const deckBGlow = computed(() =>
  Math.max(0, Math.min(1, (Math.max(deckBLevelL.value, deckBLevelR.value) + 60) / 60))
)

// Helpers
function levelPct(db: number): string {
  return Math.max(0, Math.min(100, (db + 60) / 60 * 100)).toFixed(1) + '%'
}

// Vinyl rotation speed based on pitch (33 1/3 RPM = 1.8s/rotation)
function vinylSpeed(pitch: number): string {
  return (1.8 / (1 + pitch / 100)).toFixed(3) + 's'
}

// Standard DJ crossfader curve (A side)
function cfFactorA(cf: number): number {
  return cf <= 0.5 ? 1.0 : 1.0 - (cf - 0.5) * 2.0
}
// Standard DJ crossfader curve (B side)
function cfFactorB(cf: number): number {
  return cf >= 0.5 ? 1.0 : cf * 2.0
}

// Volume handlers (apply crossfader gain)
function onDeckAVolume() {
  if (!audioEngine || deckAIdx.value === null) return
  audioEngine.setTrackVolume(deckAIdx.value, deckAVolume.value * cfFactorA(crossfader.value))
}
function onDeckBVolume() {
  if (!audioEngine || deckBIdx.value === null) return
  audioEngine.setTrackVolume(deckBIdx.value, deckBVolume.value * cfFactorB(crossfader.value))
}
function onCrossfader() {
  if (!audioEngine) return
  const cf = crossfader.value
  if (deckAIdx.value !== null)
    audioEngine.setTrackVolume(deckAIdx.value, deckAVolume.value * cfFactorA(cf))
  if (deckBIdx.value !== null)
    audioEngine.setTrackVolume(deckBIdx.value, deckBVolume.value * cfFactorB(cf))
}

// Transport
function onPlayPauseA() {
  if (!audioEngine || deckAIdx.value === null || !deckAHasFile.value) return
  deckAIsPlaying.value ? audioEngine.pauseFile(deckAIdx.value) : audioEngine.playFile(deckAIdx.value)
}
function onPlayPauseB() {
  if (!audioEngine || deckBIdx.value === null || !deckBHasFile.value) return
  deckBIsPlaying.value ? audioEngine.pauseFile(deckBIdx.value) : audioEngine.playFile(deckBIdx.value)
}

// CUE functionality: Press = set point, Hold = preview from cue, Release = return to cue
let deckACueHeld = false
let deckBCueHeld = false

function onCueAPress() {
  if (!audioEngine || deckAIdx.value === null || !deckAHasFile.value) return
  
  if (deckAIsPlaying.value) {
    // If playing: set CUE point at current position and pause
    deckACuePoint.value = deckACurrentTime.value
    audioEngine.pauseFile(deckAIdx.value)
    console.log('[DJMode] CUE A set at:', deckACuePoint.value)
  } else {
    // If paused: start preview from CUE point (hold to play)
    deckACueHeld = true
    deckACurrentTime.value = deckACuePoint.value
    deckAPlaybackOffset = deckACuePoint.value
    audioEngine.seekFile(deckAIdx.value, deckACuePoint.value)
    audioEngine.playFile(deckAIdx.value)
    console.log('[DJMode] CUE A preview from:', deckACuePoint.value)
  }
}

function onCueARelease() {
  if (!audioEngine || deckAIdx.value === null || !deckAHasFile.value) return
  
  if (deckACueHeld) {
    // Release CUE: return to cue point and pause
    deckACueHeld = false
    audioEngine.pauseFile(deckAIdx.value)
    deckACurrentTime.value = deckACuePoint.value
    deckAPlaybackOffset = deckACuePoint.value
    audioEngine.seekFile(deckAIdx.value, deckACuePoint.value)
    console.log('[DJMode] CUE A released, return to:', deckACuePoint.value)
  }
}

function onCueBPress() {
  if (!audioEngine || deckBIdx.value === null || !deckBHasFile.value) return
  
  if (deckBIsPlaying.value) {
    // If playing: set CUE point at current position and pause
    deckBCuePoint.value = deckBCurrentTime.value
    audioEngine.pauseFile(deckBIdx.value)
    console.log('[DJMode] CUE B set at:', deckBCuePoint.value)
  } else {
    // If paused: start preview from CUE point (hold to play)
    deckBCueHeld = true
    deckBCurrentTime.value = deckBCuePoint.value
    deckBPlaybackOffset = deckBCuePoint.value
    audioEngine.seekFile(deckBIdx.value, deckBCuePoint.value)
    audioEngine.playFile(deckBIdx.value)
    console.log('[DJMode] CUE B preview from:', deckBCuePoint.value)
  }
}

function onCueBRelease() {
  if (!audioEngine || deckBIdx.value === null || !deckBHasFile.value) return
  
  if (deckBCueHeld) {
    // Release CUE: return to cue point and pause
    deckBCueHeld = false
    audioEngine.pauseFile(deckBIdx.value)
    deckBCurrentTime.value = deckBCuePoint.value
    deckBPlaybackOffset = deckBCuePoint.value
    audioEngine.seekFile(deckBIdx.value, deckBCuePoint.value)
    console.log('[DJMode] CUE B released, return to:', deckBCuePoint.value)
  }
}
function onStopA() {
  if (!audioEngine || deckAIdx.value === null || !deckAHasFile.value) return
  audioEngine.stopFile(deckAIdx.value)
}
function onStopB() {
  if (!audioEngine || deckBIdx.value === null || !deckBHasFile.value) return
  audioEngine.stopFile(deckBIdx.value)
}

// SYNC: adjust deck B pitch to match deck A BPM
function syncDecks() {
  if (deckABpm.value > 0 && deckBBpm.value > 0) {
    deckBPitch.value = Math.max(-8, Math.min(8, (deckABpm.value / deckBBpm.value - 1) * 100))
  }
}

// Pitch change handlers (Note: Playback speed control not yet implemented in backend)
function onPitchAChange() {
  // TODO: When backend implements playback speed control, call:
  // audioEngine.setTrackPlaybackRate(deckAIdx.value, 1 + deckAPitch.value / 100)
  console.log('[DJMode] Pitch A changed to:', deckAPitch.value, '% (backend API pending)')
}

function onPitchBChange() {
  // TODO: When backend implements playback speed control, call:
  // audioEngine.setTrackPlaybackRate(deckBIdx.value, 1 + deckBPitch.value / 100)
  console.log('[DJMode] Pitch B changed to:', deckBPitch.value, '% (backend API pending)')
}

// Load file handlers
const emit = defineEmits<{
  'open-library': [trackId: number]
}>()

function onLoadA() {
  if (deckAIdx.value !== null) {
    emit('open-library', deckAIdx.value + 1) // Convert back to 1-based track ID
  }
}

function onLoadB() {
  if (deckBIdx.value !== null) {
    emit('open-library', deckBIdx.value + 1) // Convert back to 1-based track ID
  }
}
</script>

<style scoped>
@keyframes spin-vinyl {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.vinyl-disc {
  animation: spin-vinyl var(--spin-dur, 1.8s) linear infinite;
  animation-play-state: paused; /* overridden by :style binding when playing */
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
