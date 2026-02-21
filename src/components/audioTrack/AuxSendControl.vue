<template>
  <div class="flex flex-col items-center gap-0 bg-gray-800/50 rounded-lg p-2 border border-teal-700/30">
    <!-- Aux name -->
    <div class="text-[0.6rem] font-bold text-teal-300 text-center mb-0.5 -mt-1">
      {{ aux.name }}
    </div>

    <!-- Send level knob -->
    <div class="scale-[0.6] -my-5">
      <Knob 
        :modelValue="auxSendData?.level ?? -60"
        @update:modelValue="(val) => emit('update-level', val)" 
        :min="-60" 
        :max="10" 
        :step="0.1"
        label="Level" 
        unit="dB" 
        color="#14b8a6" 
      />
    </div>

    <!-- Controls row -->
    <div class="w-full flex gap-1 mt-1">
      <!-- Pre/Post fader toggle -->
      <button 
        @click="emit('toggle-pre-post')" 
        :class="[
          'flex-1 py-0.5 px-1 text-[0.5rem] font-bold rounded transition-colors',
          auxSendData?.preFader
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 text-gray-400'
        ]"
      >
        {{ auxSendData?.preFader ? 'PRE' : 'POST' }}
      </button>

      <!-- Mute send -->
      <button 
        @click="emit('toggle-mute')" 
        :class="[
          'flex-1 py-0.5 px-1 text-[0.5rem] font-bold rounded transition-colors',
          auxSendData?.muted
            ? 'bg-yellow-600 text-white'
            : 'bg-gray-700 text-gray-400'
        ]"
      >
        M
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import Knob from '../core/Knob.vue'

interface AuxBus {
  id: string
  name: string
  volume: number
  muted: boolean
  soloed: boolean
  routeToMaster: boolean
  selectedOutputDevice?: string | null
  node?: any
  outputStreamDest?: MediaStreamAudioDestinationNode | null
  outputAudioContext?: AudioContext | null
  outputSource?: MediaStreamAudioSourceNode | null
}

interface AuxSendData {
  level: number
  preFader: boolean
  muted: boolean
}

interface Props {
  aux: AuxBus
  auxSendData?: AuxSendData
}

defineProps<Props>()

const emit = defineEmits<{
  (e: 'update-level', value: number): void
  (e: 'toggle-pre-post'): void
  (e: 'toggle-mute'): void
}>()
</script>
