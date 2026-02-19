<template>
  <!-- Reverb Toggle -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between',
    enabled ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <span>RE</span>
    <button :disabled="!enabled" @click.stop="showModal = true"
      class="p-0.5 rounded hover:bg-green-700">
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
  </div>

  <!-- Reverb Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-xl font-bold text-green-300">Track {{ trackNumber }} - Reverb</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>
        <div class="flex flex-wrap gap-4 justify-center">
          <Knob v-model="decay" :min="0.1" :max="10" :step="0.1" label="Decay" unit="s" color="#10b981"
            @update:modelValue="emitUpdate" />
          <Knob v-model="preDelay" :min="0" :max="0.1" :step="0.001" label="Pre-Delay" unit="s" color="#f59e0b"
            @update:modelValue="emitUpdate" />
          <Knob v-model="wet" :min="0" :max="1" :step="0.01" label="Wet" unit="%" color="#06b6d4"
            @update:modelValue="emitUpdate" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Knob from './Knob.vue'

interface Props {
  trackNumber: number
  enabled: boolean
  decay: number
  preDelay: number
  wet: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'update', params: { decay: number, preDelay: number, wet: number }): void
}>()

const showModal = ref(false)

// Local state for controls
const decay = ref(props.decay)
const preDelay = ref(props.preDelay)
const wet = ref(props.wet)

// Sync props to local state
watch(() => props.decay, (val) => decay.value = val)
watch(() => props.preDelay, (val) => preDelay.value = val)
watch(() => props.wet, (val) => wet.value = val)

function handleToggle() {
  emit('toggle')
}

function emitUpdate() {
  emit('update', {
    decay: decay.value,
    preDelay: preDelay.value,
    wet: wet.value
  })
}
</script>
