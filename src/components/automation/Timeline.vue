<template>
  <div class="timeline-container bg-gray-900 border-t border-gray-700">
    <!-- Transport Controls -->
    <div class="transport-controls flex items-center gap-3 px-4 py-2 bg-gray-800 border-b border-gray-700">
      <!-- Play/Pause/Stop -->
      <div class="flex gap-2">
        <button @click="emit('stop')" 
          class="w-8 h-8 rounded bg-gray-700 hover:bg-gray-600 flex items-center justify-center transition-colors"
          title="Stop">
          <svg class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 20 20">
            <rect x="6" y="6" width="8" height="8" />
          </svg>
        </button>
        
        <button @click="togglePlayPause" 
          :class="[
            'w-8 h-8 rounded flex items-center justify-center transition-colors',
            transport.isPlaying 
              ? 'bg-orange-600 hover:bg-orange-500' 
              : 'bg-green-600 hover:bg-green-500'
          ]"
          :title="transport.isPlaying ? 'Pause' : 'Play'">
          <svg v-if="!transport.isPlaying" class="w-4 h-4 text-white ml-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
          </svg>
          <svg v-else class="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 20 20">
            <path d="M5.75 3a.75.75 0 00-.75.75v12.5c0 .414.336.75.75.75h1.5a.75.75 0 00.75-.75V3.75A.75.75 0 007.25 3h-1.5zM12.75 3a.75.75 0 00-.75.75v12.5c0 .414.336.75.75.75h1.5a.75.75 0 00.75-.75V3.75a.75.75 0 00-.75-.75h-1.5z" />
          </svg>
        </button>
        
        <button @click="emit('record')" 
          :class="[
            'w-8 h-8 rounded flex items-center justify-center transition-colors',
            isRecording 
              ? 'bg-red-600 animate-pulse' 
              : 'bg-gray-700 hover:bg-red-600/50'
          ]"
          title="Record Automation">
          <svg class="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
            <circle cx="10" cy="10" r="6" />
          </svg>
        </button>
      </div>

      <!-- Time Display -->
      <div class="flex items-center gap-2 px-3 py-1 bg-gray-900 rounded border border-gray-700 font-mono text-sm text-gray-300">
        <span>{{ formatTime(transport.currentTime) }}</span>
        <span class="text-gray-600">/</span>
        <span class="text-gray-500">{{ formatTime(transport.duration) }}</span>
      </div>

      <!-- BPM -->
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-500">BPM</span>
        <input 
          type="number" 
          :value="transport.bpm" 
          @input="updateBpm"
          class="w-16 px-2 py-1 bg-gray-900 border border-gray-700 rounded text-sm text-gray-300 focus:outline-none focus:border-teal-500"
          min="20" max="300" step="1"
        />
      </div>

      <!-- Time Signature -->
      <div class="flex items-center gap-1">
        <span class="text-xs text-gray-500">Time</span>
        <input 
          type="number" 
          :value="transport.timeSignature.numerator" 
          @input="updateTimeSignature($event, 'numerator')"
          class="w-10 px-2 py-1 bg-gray-900 border border-gray-700 rounded text-sm text-gray-300 text-center focus:outline-none focus:border-teal-500"
          min="1" max="16" step="1"
        />
        <span class="text-gray-600">/</span>
        <input 
          type="number" 
          :value="transport.timeSignature.denominator" 
          @input="updateTimeSignature($event, 'denominator')"
          class="w-10 px-2 py-1 bg-gray-900 border border-gray-700 rounded text-sm text-gray-300 text-center focus:outline-none focus:border-teal-500"
          min="1" max="16" step="1"
        />
      </div>

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- Loop Toggle -->
      <button 
        @click="emit('toggle-loop')"
        :class="[
          'px-3 py-1 rounded text-xs font-bold transition-colors',
          loopEnabled 
            ? 'bg-teal-600 text-white' 
            : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
        ]"
        title="Loop">
        LOOP
      </button>
    </div>

    <!-- Timeline Ruler -->
    <div class="timeline-ruler relative h-16 bg-gray-800" 
      ref="timelineRef"
      @mousedown="handleTimelineClick"
      @mousemove="handleTimelineHover">
      
      <!-- Grid Lines -->
      <div class="absolute inset-0 flex">
        <div 
          v-for="i in 20" 
          :key="i" 
          class="flex-1 border-r border-gray-700/30"
        ></div>
      </div>

      <!-- Time Markers -->
      <div class="absolute inset-x-0 top-0 h-6 flex items-end px-2">
        <div 
          v-for="marker in timeMarkers" 
          :key="marker.time"
          :style="{ left: `${marker.position * 100}%` }"
          class="absolute text-[0.65rem] text-gray-500 font-mono transform -translate-x-1/2"
        >
          {{ formatTime(marker.time) }}
        </div>
      </div>

      <!-- Playhead -->
      <div 
        :style="{ left: `${playheadPosition * 100}%` }"
        class="absolute top-0 bottom-0 w-0.5 bg-teal-400 pointer-events-none z-10"
      >
        <div class="absolute -top-1 left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-teal-400"></div>
      </div>

      <!-- Hover Time Indicator -->
      <div 
        v-if="hoverTime !== null"
        :style="{ left: `${hoverPosition * 100}%` }"
        class="absolute top-0 bottom-0 w-px bg-white/20 pointer-events-none"
      >
        <div class="absolute top-0 left-2 text-[0.6rem] text-white bg-black/70 px-1 rounded whitespace-nowrap">
          {{ formatTime(hoverTime) }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { TransportState } from '~/composables/useAutomation'

interface Props {
  transport: TransportState
  playheadPosition: number
  isRecording: boolean
  loopEnabled?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'play'): void
  (e: 'pause'): void
  (e: 'stop'): void
  (e: 'record'): void
  (e: 'seek', time: number): void
  (e: 'update-bpm', bpm: number): void
  (e: 'update-time-signature', numerator: number, denominator: number): void
  (e: 'toggle-loop'): void
}>()

const timelineRef = ref<HTMLElement | null>(null)
const hoverTime = ref<number | null>(null)
const hoverPosition = ref(0)

// Time markers for ruler (every 10 seconds by default)
const timeMarkers = computed(() => {
  const markers = []
  const step = Math.max(10, Math.ceil(props.transport.duration / 10 / 10) * 10)
  
  for (let time = 0; time <= props.transport.duration; time += step) {
    markers.push({
      time,
      position: time / props.transport.duration
    })
  }
  
  return markers
})

function togglePlayPause() {
  if (props.transport.isPlaying) {
    emit('pause')
  } else {
    emit('play')
  }
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  const ms = Math.floor((seconds % 1) * 100)
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`
}

function handleTimelineClick(event: MouseEvent) {
  if (!timelineRef.value) return
  
  const rect = timelineRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const position = Math.max(0, Math.min(1, x / rect.width))
  const time = position * props.transport.duration
  
  emit('seek', time)
}

function handleTimelineHover(event: MouseEvent) {
  if (!timelineRef.value) return
  
  const rect = timelineRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  
  if (x >= 0 && x <= rect.width) {
    hoverPosition.value = x / rect.width
    hoverTime.value = hoverPosition.value * props.transport.duration
  } else {
    hoverTime.value = null
  }
}

function updateBpm(event: Event) {
  const target = event.target as HTMLInputElement
  const bpm = parseInt(target.value)
  if (!isNaN(bpm) && bpm >= 20 && bpm <= 300) {
    emit('update-bpm', bpm)
  }
}

function updateTimeSignature(event: Event, type: 'numerator' | 'denominator') {
  const target = event.target as HTMLInputElement
  const value = parseInt(target.value)
  if (!isNaN(value) && value >= 1 && value <= 16) {
    if (type === 'numerator') {
      emit('update-time-signature', value, props.transport.timeSignature.denominator)
    } else {
      emit('update-time-signature', props.transport.timeSignature.numerator, value)
    }
  }
}
</script>

<style scoped>
.timeline-container {
  user-select: none;
}

.timeline-ruler {
  cursor: pointer;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.animate-pulse {
  animation: pulse 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>
