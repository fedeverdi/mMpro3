<template>
  <!-- De-Esser Button -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between gap-1',
    enabled ? 'bg-pink-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <div class="flex items-center gap-1">
      <div class="drag-handle cursor-move text-gray-400 hover:text-gray-200" @click.stop>
        <svg class="w-2 h-2" fill="currentColor" viewBox="0 0 24 24">
          <path d="M9 3h2v2H9V3zm0 4h2v2H9V7zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm0 4h2v2H9v-2zM13 3h2v2h-2V3zm0 4h2v2h-2V7zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2z"/>
        </svg>
      </div>
      <span>DE</span>
    </div>
    <div class="flex items-center gap-0.5">
      <button :disabled="!enabled" @click.stop="showModal = true"
        class="p-0.5 rounded hover:bg-pink-700 disabled:opacity-50">
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

  <!-- De-Esser Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[1000]"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-pink-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold text-pink-300">Track {{ trackNumber }} - Insert #{{ insertId }} - De-Esser</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>

        <div class="grid grid-cols-3 gap-6">
          <div class="flex flex-col items-center">
            <label class="text-xs text-pink-300 font-bold mb-2">THRESHOLD</label>
            <Knob v-model="threshold" :min="-60" :max="0" :step="0.5" color="#ec4899" unit="dB" />
            <p class="text-[10px] text-gray-400 mt-2 text-center">Level where de-essing starts</p>
          </div>
          <div class="flex flex-col items-center">
            <label class="text-xs text-pink-300 font-bold mb-2">FREQUENCY</label>
            <Knob v-model="frequency" :min="3000" :max="8000" :step="100" color="#ec4899" unit="Hz" />
            <p class="text-[10px] text-gray-400 mt-2 text-center">Sibilance detection freq</p>
          </div>
          <div class="flex flex-col items-center">
            <label class="text-xs text-pink-300 font-bold mb-2">RANGE</label>
            <Knob v-model="range" :min="0" :max="20" :step="0.5" color="#ec4899" unit="dB" />
            <p class="text-[10px] text-gray-400 mt-2 text-center">Maximum reduction</p>
          </div>
        </div>

        <div class="mt-6 pt-4 border-t border-gray-700">
          <p class="text-xs text-gray-400 text-center">Reduces harsh sibilance (S, T, CH) in vocals</p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, inject } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  trackNumber: number
  insertId: number
  enabled: boolean
  compressorInputDb?: number
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'remove'): void
}>()

const audioEngine = inject('audioEngine') as any
const showModal = ref(false)
const threshold = ref(-20)
const frequency = ref(6000)
const range = ref(10)

function handleToggle() {
  emit('toggle')
}

watch([threshold, frequency, range], () => {
  if (audioEngine) {
    audioEngine.setTrackInsertDeEsser(props.trackNumber - 1, props.insertId, threshold.value, frequency.value, range.value)
  }
})
</script>
   