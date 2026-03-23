<template>
  <div class="deck flex-1 flex flex-col gap-1.5 bg-gray-900/80 rounded-xl p-3 min-w-0 overflow-hidden"
       :class="`border border-${themeColor}-500/20`">

    <!-- Header -->
    <div class="flex items-center justify-between flex-shrink-0">
      <div class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full transition-all"
              :class="[
                `bg-${themeColor}-400`,
                isPlaying ? `animate-pulse shadow-[0_0_8px_2px_rgba(${themeRgb},0.7)]` : 'opacity-50'
              ]"></span>
        <span class="text-[11px] font-black tracking-[0.3em] uppercase"
              :class="`text-${themeColor}-400`">
          Deck {{ deckName }}
        </span>
      </div>
      <span class="text-[10px] text-gray-500 font-mono tabular-nums">
        {{ bpm > 0 ? bpm.toFixed(1) + ' BPM' : '· · · BPM' }}
      </span>
    </div>

    <!-- Track Info Panel -->
    <div class="flex-shrink-0 bg-black/40 border border-gray-700 rounded-lg p-4 space-y-3 min-h-[200px]">
      
      <!-- Track title and artist -->
      <div class="text-center min-h-[2rem]">
        <div class="text-sm font-semibold text-white truncate leading-tight">{{ title }}</div>
        <div class="text-[10px] text-gray-400 truncate mt-0.5">{{ artist }}</div>
      </div>

      <!-- Waveform Monitor -->
      <div v-if="trackIndex !== null" class="px-1 py-1 h-24">
        <WaveformDisplay 
          :track-number="trackIndex"
          :audio-buffer="audioBuffer"
          :current-time="currentTime"
          :is-playing="isPlaying"
          :duration="duration"
          :show-mode-buttons="false"
          mode="waveform"
          waveform-color="#ffffff"
          :cue-point="cuePoint"
          @seek="handleSeek"
          @seek-release="emit('seek-release')"
          class="h-full"
        />
      </div>

      <!-- Timer -->
      <div class="flex items-center justify-between text-[10px] font-mono tabular-nums px-2">
        <span class="text-gray-400">{{ formatTime(currentTime) }}</span>
        <span class="text-gray-600">•</span>
        <span class="text-gray-400">{{ formatTime(duration) }}</span>
        <span class="text-gray-600 ml-auto pl-3">-{{ formatTime(duration - currentTime) }}</span>
      </div>
    </div>

    <!-- Vinyl + sliders row -->
    <div class="flex gap-2 flex-1 items-center justify-center min-h-0 overflow-hidden">

      <!-- Pitch slider (left) -->
      <div class="flex flex-col items-center gap-1.5 self-stretch w-10 flex-shrink-0">
        <!-- TAP button -->
        <button @click="emit('tap-tempo')"
                class="w-10 h-10 rounded text-[8px] font-bold transition-all tracking-wider border cursor-pointer flex items-center justify-center shadow bg-gray-800 hover:bg-gray-700 text-blue-400 border-blue-700 hover:border-blue-500">
          TAP
        </button>
        
        <span class="text-[8px] text-gray-400 font-mono tabular-nums leading-none font-bold">
          {{ pitch > 0 ? '+' : '' }}{{ pitch.toFixed(1) }}%
        </span>
        
        <!-- Pioneer-style pitch fader -->
        <div class="pioneer-fader flex-1 relative" @click="handlePitchClick">
          <!-- Graduated scale marks -->
          <div class="absolute left-0 top-0 h-full w-full flex flex-col justify-between pointer-events-none">
            <div v-for="mark in [8, 6, 4, 2, 0, -2, -4, -6, -8]" :key="mark" 
                 class="flex items-center w-full"
                 :class="mark === 0 ? 'h-0.5' : 'h-px'">
              <div class="w-1.5 h-full bg-gray-600" :class="mark === 0 ? 'bg-red-500/60 w-2' : ''"></div>
              <span class="text-[6px] ml-1 text-gray-600 font-mono leading-none"
                    :class="mark === 0 ? 'text-red-400/80 font-bold' : ''">
                {{ mark > 0 ? '+' + mark : mark }}
              </span>
            </div>
          </div>
          
          <!-- Track background -->
          <div class="fader-track absolute left-0 top-0 w-1.5 h-full bg-gradient-to-b from-gray-950 via-gray-900 to-gray-950 border-l border-r border-gray-800 shadow-inner">
            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-gray-800/20 to-transparent"></div>
          </div>
          
          <!-- Fader handle -->
          <div class="fader-handle absolute left-0 w-full pointer-events-none transition-all duration-100"
               :style="{ top: `calc(${pitchPercentage}% - 10px)` }">
            <div class="w-full h-5 rounded-sm relative"
                 :class="`bg-gradient-to-b from-gray-400 via-gray-500 to-gray-600 shadow-lg border border-gray-700`">
              <!-- Handle grip lines -->
              <div class="absolute inset-0 flex flex-col justify-center items-center gap-0.5 px-0.5">
                <div class="w-full h-px bg-gray-700/50"></div>
                <div class="w-full h-px bg-gray-300/30"></div>
                <div class="w-full h-px bg-gray-700/50"></div>
              </div>
              <!-- Side highlights -->
              <div class="absolute left-0 top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-gray-300/40 to-transparent"></div>
              <div class="absolute right-0 top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-gray-900/60 to-transparent"></div>
            </div>
          </div>
          
          <!-- Invisible input for keyboard/mouse -->
          <input type="range" min="-8" max="8" step="0.1" 
                 :value="pitch"
                 @input="handlePitchChange"
                 :disabled="!hasFile"
                 class="fader-input absolute inset-0 opacity-0 cursor-grab active:cursor-grabbing" />
        </div>
        
        <span class="text-[7px] text-gray-600 leading-none tracking-[0.2em] font-bold">TEMPO</span>
      </div>

      <!-- Vinyl record -->
      <div class="flex-1 flex items-center justify-center min-h-0 min-w-0 relative">
        <svg viewBox="0 0 200 200" preserveAspectRatio="xMidYMid meet"
             class="vinyl-disc cursor-grab active:cursor-grabbing"
             :style="{
               filter: `drop-shadow(0 0 ${glowIntensity * 14}px rgba(${themeRgb},${glowIntensity * 0.25}))`,
               transform: `rotate(${rotation}deg)`
             }"
             @mousedown="handleVinylMouseDown">
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
          <circle cx="100" cy="100" r="30" fill="none" :stroke="`rgba(${themeRgb},0.5)`" stroke-width="1"/>
          <circle cx="100" cy="100" r="26.5" fill="none" :stroke="`rgba(${themeRgb},0.12)`" stroke-width="0.4"/>
          <!-- Label text -->
          <text x="100" y="88" text-anchor="middle" :fill="`rgba(${themeRgb},0.35)`"
                font-size="4" font-family="monospace" letter-spacing="3">DECK {{ deckName }}</text>
          <text x="100" y="97" text-anchor="middle" :fill="`rgba(${themeRgb},0.95)`"
                font-size="5.8" font-family="monospace" font-weight="bold">
            {{ title.substring(0, 13) }}
          </text>
          <text x="100" y="106" text-anchor="middle" fill="rgba(255,255,255,0.4)"
                font-size="4" font-family="monospace">
            {{ artist.substring(0, 18) }}
          </text>
          <!-- Load instruction (shown when no file loaded) -->
          <text v-if="!hasFile" x="100" y="114" text-anchor="middle" :fill="`rgba(${themeRgb},0.5)`"
                font-size="3" font-family="monospace" letter-spacing="0.5" class="load-hint">
            CLICK TO LOAD
          </text>
          <!-- Label inner ring decoration -->
          <line x1="100" y1="70" x2="100" y2="75" :stroke="`rgba(${themeRgb},0.3)`" stroke-width="0.8"/>
          <!-- Center spindle -->
          <circle cx="100" cy="100" r="4" fill="#141414"/>
          <circle cx="100" cy="100" r="2.2" fill="#0d0d0d"/>
          <circle cx="100" cy="100" r="0.9" fill="#222"/>
          
          <!-- Clickable label area (for file loading) -->
          <circle cx="100" cy="100" r="31" fill="transparent" class="label-clickarea"/>
        </svg>
      </div>

      <!-- Volume fader + meter (right) -->
      <div ref="faderVolumeContainer" class="flex gap-2 items-stretch self-stretch flex-shrink-0">
        <!-- Volume fader -->
        <div class="flex flex-col items-center gap-1.5 self-stretch w-10 flex-shrink-0">
          <span class="text-[8px] text-gray-400 font-mono tabular-nums leading-none font-bold">
            {{ Math.round(volume * 100) }}%
          </span>
          
          <!-- Pioneer-style volume fader -->
        <div class="pioneer-fader flex-1 relative" @click="handleVolumeClick">
          <!-- Graduated scale marks -->
          <div class="absolute left-0 top-0 h-full w-full flex flex-col justify-between pointer-events-none">
            <div v-for="mark in [10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]" :key="mark" 
                 class="flex items-center w-full h-px">
              <div class="w-1.5 h-full bg-gray-600" :class="mark === 10 ? 'bg-amber-500/40' : ''"></div>
              <span class="text-[6px] ml-1 text-gray-600 font-mono leading-none"
                    :class="mark === 10 ? 'text-amber-400/60' : ''">
                {{ mark }}
              </span>
            </div>
          </div>
          
          <!-- Track background -->
          <div class="fader-track absolute left-0 top-0 w-1.5 h-full bg-gradient-to-b from-gray-950 via-gray-900 to-gray-950 border-l border-r border-gray-800 shadow-inner">
            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-gray-800/20 to-transparent"></div>
          </div>
          
          <!-- Fader handle -->
          <div class="fader-handle absolute left-0 w-full pointer-events-none transition-all duration-100"
               :style="{ top: `calc(${volumePercentage}% - 10px)` }">
            <div class="w-full h-5 rounded-sm relative"
                 :class="`bg-gradient-to-b from-gray-400 via-gray-500 to-gray-600 shadow-lg border border-gray-700`">
              <!-- Handle grip lines -->
              <div class="absolute inset-0 flex flex-col justify-center items-center gap-0.5 px-0.5">
                <div class="w-full h-px bg-gray-700/50"></div>
                <div class="w-full h-px bg-gray-300/30"></div>
                <div class="w-full h-px bg-gray-700/50"></div>
              </div>
              <!-- Side highlights -->
              <div class="absolute left-0 top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-gray-300/40 to-transparent"></div>
              <div class="absolute right-0 top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-gray-900/60 to-transparent"></div>
            </div>
          </div>
          
          <!-- Invisible input for keyboard/mouse -->
          <input type="range" min="0" max="1.5" step="0.01" 
                 :value="volume"
                 @input="handleVolumeChange"
                 class="fader-input absolute inset-0 opacity-0 cursor-grab active:cursor-grabbing" />
        </div>
        
          <span class="text-[7px] text-gray-600 leading-none tracking-[0.2em] font-bold">VOLUME</span>
        </div>
        
        <!-- Level meter -->
        <div class="flex flex-col items-center self-stretch justify-center gap-1">
          <div class="flex-1 flex items-center justify-center min-h-0">
            <TrackMeter 
              v-if="meterHeight > 0"
              :levelL="levelL"
              :levelR="levelR"
              :isStereo="true"
              :height="meterHeight - 40"
            />
          </div>
          <span class="text-[7px] text-gray-600 leading-none tracking-[0.2em] font-bold">METER</span>
        </div>
      </div>
    </div>

    <!-- Transport controls panel -->
    <div class="flex-shrink-0 mx-2 mb-2">
      <div class="bg-black/40 border border-gray-800 rounded-lg p-3">
        <div class="flex gap-3 justify-between items-center">
          
          <!-- Loop Sample buttons (left) -->
          <div class="flex gap-1.5">
            <button @click="emit('loop-sample', 1/2)"
                    :disabled="!hasFile"
                    class="w-11 h-11 rounded text-[9px] font-bold transition-all border cursor-pointer flex items-center justify-center shadow"
                    :class="!hasFile
                      ? 'bg-gray-900 text-gray-600 border-gray-800 cursor-not-allowed opacity-50'
                      : 'bg-gray-800 hover:bg-gray-700 text-purple-400 border-purple-700 hover:border-purple-500'">
              1/2
            </button>
            <button @click="emit('loop-sample', 1/4)"
                    :disabled="!hasFile"
                    class="w-11 h-11 rounded text-[9px] font-bold transition-all border cursor-pointer flex items-center justify-center shadow"
                    :class="!hasFile
                      ? 'bg-gray-900 text-gray-600 border-gray-800 cursor-not-allowed opacity-50'
                      : 'bg-gray-800 hover:bg-gray-700 text-purple-400 border-purple-700 hover:border-purple-500'">
              1/4
            </button>
            <button @click="emit('loop-sample', 1/8)"
                    :disabled="!hasFile"
                    class="w-11 h-11 rounded text-[9px] font-bold transition-all border cursor-pointer flex items-center justify-center shadow"
                    :class="!hasFile
                      ? 'bg-gray-900 text-gray-600 border-gray-800 cursor-not-allowed opacity-50'
                      : 'bg-gray-800 hover:bg-gray-700 text-purple-400 border-purple-700 hover:border-purple-500'">
              1/8
            </button>
            <button @click="emit('loop-sample', 1/16)"
                    :disabled="!hasFile"
                    class="w-11 h-11 rounded text-[9px] font-bold transition-all border cursor-pointer flex items-center justify-center shadow"
                    :class="!hasFile
                      ? 'bg-gray-900 text-gray-600 border-gray-800 cursor-not-allowed opacity-50'
                      : 'bg-gray-800 hover:bg-gray-700 text-purple-400 border-purple-700 hover:border-purple-500'">
              1/16
            </button>
          </div>
          
          <!-- CUE and Play buttons (right) -->
          <div class="flex gap-3">
            <!-- CUE button (rounded) -->
            <button @mousedown="emit('cue-press')" 
                    @mouseup="emit('cue-release')" 
                    @mouseleave="emit('cue-release')"
                    class="w-14 h-14 rounded-full text-[11px] font-bold transition-all tracking-wider border-2 cursor-pointer flex items-center justify-center shadow-lg"
                    :class="cuePoint > 0 
                      ? 'bg-gray-800 hover:bg-gray-700 text-orange-500 border-orange-500 cue-blink' 
                      : 'bg-gray-800 hover:bg-gray-700 text-orange-500 border-gray-700 hover:border-gray-500'">
              CUE
            </button>
            
            <!-- Play/Pause button -->
            <button @click="emit('play-pause')"
                    :disabled="!hasFile"
                    class="w-14 h-14 rounded-full font-bold transition-all flex items-center justify-center shadow-lg border-2"
                    :class="!hasFile
                      ? 'bg-gray-900 text-gray-600 border-gray-800 cursor-not-allowed opacity-50'
                      : isPlaying
                        ? 'bg-gray-800 text-green-500 hover:bg-gray-700 border-green-500 play-glow'
                        : 'bg-gray-800 text-green-500 border-green-500 hover:border-green-400'">
              <svg v-if="!isPlaying" class="w-5 h-5 ml-0.5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z"/>
              </svg>
              <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, inject, watch, onMounted, onUnmounted, type Ref } from 'vue'
import WaveformDisplay from '../audioTrack/WaveformDisplay.vue'
import TrackMeter from '../audioTrack/TrackMeter.vue'

// Refs for dynamic height calculation
const faderVolumeContainer = ref<HTMLElement | null>(null)
const meterHeight = ref(0)

interface Props {
  deckName: string
  themeColor: 'cyan' | 'orange'
  themeRgb: string
  trackIndex: number | null
  currentTime: number
  duration: number
  title: string
  artist: string
  bpm: number
  hasFile: boolean
  isPlaying: boolean
  pitch: number
  volume: number
  levelL: number
  levelR: number
  rotation: number
  audioBuffer: AudioBuffer | null
  cuePoint: number
}

const props = withDefaults(defineProps<Props>(), {
  title: 'No Track',
  artist: '',
  bpm: 0,
  hasFile: false,
  isPlaying: false,
  pitch: 0,
  volume: 1,
  levelL: -60,
  levelR: -60,
  rotation: 0,
  audioBuffer: null,
  cuePoint: 0,
})

const emit = defineEmits<{
  'play-pause': []
  'stop': []
  'cue-press': []
  'cue-release': []
  'load-file': []
  'pitch-change': [value: number]
  'volume-change': [value: number]
  'seek': [time: number]
  'seek-release': []
  'vinyl-mousedown': [event: MouseEvent]
  'tap-tempo': []
  'loop-sample': [fraction: number]
}>()

// Computed glow intensity based on audio levels
const glowIntensity = computed(() =>
  Math.max(0, Math.min(1, (Math.max(props.levelL, props.levelR) + 60) / 60))
)

// Format time from seconds to MM:SS
function formatTime(seconds: number): string {
  if (!seconds || !isFinite(seconds)) return '0:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

// Pitch fader position (0% = top/+8, 50% = center/0, 100% = bottom/-8)
const pitchPercentage = computed(() => {
  return ((8 - props.pitch) / 16) * 100
})

// Volume fader position (0% = top/max, 100% = bottom/min)
const volumePercentage = computed(() => {
  return (1 - (props.volume / 1.5)) * 100
})

// Meter height calculation
let updateMeterHeightTimeout: ReturnType<typeof setTimeout> | null = null
function updateMeterHeight() {
  // Throttle resize calculations
  if (updateMeterHeightTimeout) return

  updateMeterHeightTimeout = setTimeout(() => {
    if (faderVolumeContainer.value) {
      meterHeight.value = faderVolumeContainer.value.clientHeight
    }
    updateMeterHeightTimeout = null
  }, 16) // ~60fps
}

// Watch for resize trigger from parent
const resizeTrigger = inject<Ref<number>>('resizeTrigger', ref(0))

onMounted(() => {
  watch(resizeTrigger, () => {
    updateMeterHeight()
  })
  updateMeterHeight()
})

onUnmounted(() => {
  if (updateMeterHeightTimeout) {
    clearTimeout(updateMeterHeightTimeout)
  }
})

// Event handlers
function handlePitchChange(event: Event) {
  const target = event.target as HTMLInputElement
  emit('pitch-change', parseFloat(target.value))
}

function handlePitchClick(event: MouseEvent) {
  if (!props.hasFile) return
  const target = event.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const y = event.clientY - rect.top
  const percentage = y / rect.height
  const newPitch = 8 - (percentage * 16)
  emit('pitch-change', Math.max(-8, Math.min(8, newPitch)))
}

function handleVolumeChange(event: Event) {
  const target = event.target as HTMLInputElement
  emit('volume-change', parseFloat(target.value))
}

function handleVolumeClick(event: MouseEvent) {
  const target = event.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const y = event.clientY - rect.top
  const percentage = y / rect.height
  const newVolume = 1.5 - (percentage * 1.5)
  emit('volume-change', Math.max(0, Math.min(1.5, newVolume)))
}

function handleSeek(time: number) {
  emit('seek', time)
}

function handleVinylMouseDown(event: MouseEvent) {
  const target = event.currentTarget as SVGElement
  const rect = target.getBoundingClientRect()
  
  // Calcola le coordinate relative al centro del disco
  const centerX = rect.left + rect.width / 2
  const centerY = rect.top + rect.height / 2
  const clickX = event.clientX - centerX
  const clickY = event.clientY - centerY
  
  // Calcola la distanza dal centro (normalizzata rispetto al raggio del disco)
  const distance = Math.sqrt(clickX * clickX + clickY * clickY)
  const diskRadius = rect.width / 2
  const normalizedDistance = (distance / diskRadius) * 100 // Normalizza a viewBox 200x200 (raggio 100)
  
  // Se il click è nella zona dell'etichetta (raggio 31), carica il file
  if (normalizedDistance <= 31) {
    emit('load-file')
    return
  }
  
  // Altrimenti, inizia lo scratch
  emit('vinyl-mousedown', event)
}
</script>

<style scoped>
.vinyl-disc {
  user-select: none;
  -webkit-user-drag: none;
  aspect-ratio: 1 / 1;
  max-width: 100%;
  max-height: 100%;
  width: auto;
  height: auto;
}

/* Label click area - hover effect for file loading */
.label-clickarea {
  cursor: pointer;
  transition: fill 0.2s ease;
}

.label-clickarea:hover {
  fill: rgba(255, 255, 255, 0.08);
}

/* Load hint text animation */
.load-hint {
  animation: pulse-hint 2s ease-in-out infinite;
}

@keyframes pulse-hint {
  0%, 100% { opacity: 0.5; }
  50% { opacity: 1; }
}

/* Pioneer-style fader container */
.pioneer-fader {
  position: relative;
  width: 100%;
  min-height: 0;
  background: linear-gradient(to right, 
    rgba(20, 20, 20, 0.8) 0%,
    rgba(30, 30, 30, 0.9) 50%,
    rgba(20, 20, 20, 0.8) 100%
  );
  border-radius: 3px;
  padding: 8px 4px;
  box-shadow: 
    inset 0 2px 4px rgba(0, 0, 0, 0.8),
    inset 0 -1px 2px rgba(255, 255, 255, 0.05);
}

/* Fader track (the slot where the handle moves) */
.fader-track {
  box-shadow: 
    inset 0 1px 3px rgba(0, 0, 0, 0.9),
    inset 0 -1px 1px rgba(255, 255, 255, 0.03);
}

/* Fader handle (the part you drag) */
.fader-handle {
  z-index: 2;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.5));
}

.fader-handle:hover > div {
  background: linear-gradient(to bottom, 
    rgb(160, 160, 160),
    rgb(140, 145, 150),
    rgb(120, 130, 140)
  );
}

/* Hidden range input for functionality */
.fader-input {
  writing-mode: vertical-lr;
  direction: rtl;
  -webkit-appearance: slider-vertical;
  appearance: slider-vertical;
  z-index: 3;
}

.fader-input:disabled {
  cursor: not-allowed;
}

.fader-input:focus {
  outline: none;
}

/* CUE button blink animation - border only */
@keyframes cue-blink {
  0%, 100% { border-color: rgb(249, 115, 22); } /* orange-500 */
  50% { border-color: rgba(249, 115, 22, 0.3); }
}

.cue-blink {
  animation: cue-blink 1.5s ease-in-out infinite;
}

/* Play button glow animation - white shadow */
@keyframes play-glow {
  0%, 100% { box-shadow: 0 0 15px rgba(255, 255, 255, 0.6); }
  50% { box-shadow: 0 0 5px rgba(255, 255, 255, 0.2); }
}

.play-glow {
  animation: play-glow 1.5s ease-in-out infinite;
}

.waveform-monitor {
  height: 120px;
  min-height: 120px;
}

.waveform-monitor :deep(canvas) {
  height: 80px !important;
  min-height: 80px !important;
}
</style>
