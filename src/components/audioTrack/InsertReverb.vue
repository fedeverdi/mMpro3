<template>
  <!-- Reverb Button -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between gap-1',
    enabled ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <div class="flex items-center gap-1">
      <div class="drag-handle cursor-move text-gray-400 hover:text-gray-200" @click.stop>
        <svg class="w-2 h-2" fill="currentColor" viewBox="0 0 24 24">
          <path d="M9 3h2v2H9V3zm0 4h2v2H9V7zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm0 4h2v2H9v-2zM13 3h2v2h-2V3zm0 4h2v2h-2V7zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2z"/>
        </svg>
      </div>
      <span>RV</span>
    </div>
    <div class="flex items-center gap-0.5">
      <button :disabled="!enabled" @click.stop="showModal = true"
        class="p-0.5 rounded hover:bg-green-700 disabled:opacity-50">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>
      <button @click.stop="$emit('remove')" class="p-0.5 rounded hover:bg-red-600 transition-all" title="Remove">
        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Reverb Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[1000]"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold text-green-300">Track {{ trackNumber }} - Insert #{{ insertId }}</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>
        
        <!-- PCM70-style label -->
        <div class="text-xs text-green-400 mb-3 font-mono">LEXICON PCM70 • DIGITAL REVERBERATOR</div>
        
        <!-- LCD Preset Selector -->
        <div class="bg-gradient-to-b from-blue-900 to-blue-950 border-2 border-blue-800 rounded-md p-3 mb-4 shadow-inner">
          <div class="text-[0.65rem] text-cyan-500/70 mb-2">PRESET PROGRAM</div>
          <div class="flex items-center justify-between gap-2">
            <button @click="previousPreset" 
              class="text-cyan-300 hover:text-cyan-100 text-lg font-bold px-2 py-1 hover:bg-blue-800/30 rounded transition-colors">
              ◀
            </button>
            <div class="flex-1 text-center">
              <div class="text-sm font-bold text-cyan-300">{{ currentPreset.name }}</div>
              <div class="text-[0.6rem] text-cyan-500/50">{{ currentPresetIndex + 1 }} / {{ presets.length }}</div>
            </div>
            <button @click="nextPreset" 
              class="text-cyan-300 hover:text-cyan-100 text-lg font-bold px-2 py-1 hover:bg-blue-800/30 rounded transition-colors">
              ▶
            </button>
          </div>
        </div>
        
        <div class="flex flex-wrap gap-4 justify-center">
          <Knob v-model="roomSize" :min="0" :max="1" :step="0.005" label="Size" unit="" color="#06b6d4" />
          <Knob v-model="preDelay" :min="0" :max="0.5" :step="0.002" label="Pre-Dly" unit="ms" color="#0ea5e9" />
          <Knob v-model="damping" :min="0" :max="1" :step="0.005" label="Damping" unit="" color="#14b8a6" />
          <Knob v-model="wet" :min="0" :max="1" :step="0.005" label="Mix" unit="%" color="#10b981" />
          <Knob v-model="width" :min="0" :max="1" :step="0.005" label="Width" unit="" color="#8b5cf6" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, inject, watch, computed } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  trackNumber: number
  insertId: number
  enabled: boolean
  trackLevelL: number
  trackLevelR: number
  phaseCorrelation: number
  compressorInputDb?: number
}>()

const emit = defineEmits<{
  toggle: []
  remove: []
}>()

const audioEngine = inject('audioEngine') as any
const showModal = ref(false)

// Preset definitions
interface ReverbPreset {
  name: string
  roomSize: number
  damping: number
  wet: number
  width: number
  preDelay: number
}

const presets: ReverbPreset[] = [
  { name: 'CONCERT HALL', roomSize: 0.85, damping: 0.3, wet: 0.35, width: 1.0, preDelay: 0.025 },
  { name: 'LARGE HALL', roomSize: 0.75, damping: 0.4, wet: 0.3, width: 0.95, preDelay: 0.020 },
  { name: 'MEDIUM HALL', roomSize: 0.55, damping: 0.45, wet: 0.25, width: 0.9, preDelay: 0.015 },
  { name: 'SMALL HALL', roomSize: 0.40, damping: 0.5, wet: 0.2, width: 0.85, preDelay: 0.010 },
  { name: 'CHAMBER', roomSize: 0.30, damping: 0.55, wet: 0.25, width: 0.8, preDelay: 0.005 },
  { name: 'LARGE ROOM', roomSize: 0.35, damping: 0.6, wet: 0.2, width: 0.75, preDelay: 0.008 },
  { name: 'MEDIUM ROOM', roomSize: 0.25, damping: 0.65, wet: 0.18, width: 0.7, preDelay: 0.005 },
  { name: 'SMALL ROOM', roomSize: 0.18, damping: 0.7, wet: 0.15, width: 0.65, preDelay: 0.003 },
  { name: 'STUDIO', roomSize: 0.20, damping: 0.75, wet: 0.12, width: 0.6, preDelay: 0.002 },
  { name: 'PLATE', roomSize: 0.50, damping: 0.35, wet: 0.28, width: 1.0, preDelay: 0.001 },
  { name: 'VOCAL PLATE', roomSize: 0.45, damping: 0.4, wet: 0.25, width: 0.9, preDelay: 0.012 },
  { name: 'DRUM ROOM', roomSize: 0.28, damping: 0.8, wet: 0.2, width: 0.7, preDelay: 0.0 },
  { name: 'BRIGHT HALL', roomSize: 0.65, damping: 0.25, wet: 0.3, width: 1.0, preDelay: 0.018 },
  { name: 'DARK HALL', roomSize: 0.70, damping: 0.85, wet: 0.35, width: 0.95, preDelay: 0.022 },
  { name: 'CATHEDRAL', roomSize: 0.95, damping: 0.2, wet: 0.4, width: 1.0, preDelay: 0.050 },
  { name: 'ARENA', roomSize: 0.90, damping: 0.35, wet: 0.38, width: 1.0, preDelay: 0.040 },
]

const currentPresetIndex = ref(0)
const currentPreset = computed(() => presets[currentPresetIndex.value])

// Reverb parameters (0.0 to 1.0, same as ReverbEffect.vue)
const roomSize = ref(0.5)
const preDelay = ref(0.0)
const damping = ref(0.5)
const wet = ref(0.15)
const width = ref(1.0)

function handleToggle() {
  emit('toggle')
}

function nextPreset() {
  currentPresetIndex.value = (currentPresetIndex.value + 1) % presets.length
  applyPreset()
}

function previousPreset() {
  currentPresetIndex.value = (currentPresetIndex.value - 1 + presets.length) % presets.length
  applyPreset()
}

function applyPreset() {
  const preset = currentPreset.value
  roomSize.value = preset.roomSize
  damping.value = preset.damping
  wet.value = preset.wet
  width.value = preset.width
  preDelay.value = preset.preDelay
}

// Watch for parameter changes and send to backend
watch([roomSize, preDelay, damping, wet, width], () => {
  if (audioEngine?.state.value.isRunning && props.enabled) {
    audioEngine.setTrackInsertReverb(
      props.trackNumber - 1,
      props.insertId,
      roomSize.value,
      damping.value,
      wet.value,
      width.value,
      preDelay.value
    )
  }
})
</script>
