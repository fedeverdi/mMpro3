<template>
  <!-- Exciter Button -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between gap-1',
    enabled ? 'bg-amber-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <div class="flex items-center gap-1">
      <div class="drag-handle cursor-move text-gray-400 hover:text-gray-200" @click.stop>
        <svg class="w-2 h-2" fill="currentColor" viewBox="0 0 24 24">
          <path d="M9 3h2v2H9V3zm0 4h2v2H9V7zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm0 4h2v2H9v-2zM13 3h2v2h-2V3zm0 4h2v2h-2V7zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2z"/>
        </svg>
      </div>
      <span>EX</span>
    </div>
    <div class="flex items-center gap-0.5">
      <button :disabled="!enabled" @click.stop="showModal = true"
        class="p-0.5 rounded hover:bg-amber-700 disabled:opacity-50">
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

  <!-- Exciter Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[1000]"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-amber-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold text-amber-300">Track {{ trackNumber }} - Insert #{{ insertId }} - Exciter</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>

        <div class="grid grid-cols-3 gap-6">
          <div class="flex flex-col items-center">
            <label class="text-xs text-amber-300 font-bold mb-2">AMOUNT</label>
            <Knob v-model="amount" :min="0" :max="100" :step="1" color="#f59e0b" unit="%" />
            <p class="text-[10px] text-gray-400 mt-2 text-center">Intensity of harmonic enhancement</p>
          </div>
          <div class="flex flex-col items-center">
            <label class="text-xs text-amber-300 font-bold mb-2">FREQUENCY</label>
            <Knob v-model="frequency" :min="1000" :max="10000" :step="100" color="#f59e0b" unit="Hz" />
            <p class="text-[10px] text-gray-400 mt-2 text-center">Crossover frequency</p>
          </div>
          <div class="flex flex-col items-center">
            <label class="text-xs text-amber-300 font-bold mb-2">MIX</label>
            <Knob v-model="mix" :min="0" :max="100" :step="1" color="#f59e0b" unit="%" />
            <p class="text-[10px] text-gray-400 mt-2 text-center">Wet/dry balance</p>
          </div>
        </div>

        <div class="mt-6 pt-4 border-t border-gray-700">
          <p class="text-xs text-gray-400 text-center">Adds brightness by generating harmonics</p>
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
}>()

const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'remove'): void
}>()

const audioEngine = inject('audioEngine') as any
const showModal = ref(false)
const amount = ref(50)
const frequency = ref(3000)
const mix = ref(50)

function handleToggle() {
  emit('toggle')
}

watch([amount, frequency, mix], () => {
  if (audioEngine) {
    audioEngine.setTrackInsertExciter(props.trackNumber - 1, props.insertId, amount.value / 100, frequency.value, mix.value / 100)
  }
})
</script>
