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
import { ref, inject, watch } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  trackNumber: number
  insertId: number
  enabled: boolean
  trackLevelL: number
  trackLevelR: number
  phaseCorrelation: number
}>()

const emit = defineEmits<{
  toggle: []
  remove: []
}>()

const audioEngine = inject('audioEngine') as any
const showModal = ref(false)

// Reverb parameters (0.0 to 1.0, same as ReverbEffect.vue)
const roomSize = ref(0.5)
const preDelay = ref(0.0)
const damping = ref(0.5)
const wet = ref(0.15)
const width = ref(1.0)

function handleToggle() {
  emit('toggle')
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
