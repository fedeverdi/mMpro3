<template>
  <div class="signal-generator bg-gradient-to-b from-gray-900 to-black rounded-lg border border-blue-600/60 p-3 flex flex-col gap-3">
    <div class="text-left">
      <p class="text-sm font-bold text-blue-200 tracking-wide uppercase">Signal Generator</p>
      <p class="text-[10px] text-gray-400">Select signal type</p>
    </div>

    <!-- Signal Type Selector -->
    <div class="grid grid-cols-3 gap-2">
      <button
        v-for="type in signalTypes"
        :key="type.value"
        @click="selectSignal(type.value)"
        :class="[
          'px-3 py-2 text-xs font-bold rounded transition-all',
          selectedSignal === type.value
            ? 'bg-blue-600 text-white scale-105'
            : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
        ]"
      >
        {{ type.label }}
      </button>
    </div>

    <!-- Signal Component -->
    <div class="min-h-[120px]">
      <WhiteNoise v-if="selectedSignal === 'whiteNoise'" :output-node="outputNode" />
      <PinkNoise v-else-if="selectedSignal === 'pinkNoise'" :output-node="outputNode" />
      <SineWave v-else-if="selectedSignal === 'sine'" :output-node="outputNode" />
      <SquareWave v-else-if="selectedSignal === 'square'" :output-node="outputNode" />
      <SawtoothWave v-else-if="selectedSignal === 'sawtooth'" :output-node="outputNode" />
      <TriangleWave v-else-if="selectedSignal === 'triangle'" :output-node="outputNode" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import WhiteNoise from './WhiteNoise.vue'
import PinkNoise from './PinkNoise.vue'
import SineWave from './SineWave.vue'
import SquareWave from './SquareWave.vue'
import SawtoothWave from './SawtoothWave.vue'
import TriangleWave from './TriangleWave.vue'

const props = defineProps<{
  outputNode?: any
}>()

type SignalType = 'whiteNoise' | 'pinkNoise' | 'sine' | 'square' | 'sawtooth' | 'triangle'

const signalTypes = [
  { value: 'whiteNoise' as SignalType, label: 'White Noise' },
  { value: 'pinkNoise' as SignalType, label: 'Pink Noise' },
  { value: 'sine' as SignalType, label: 'Sine' },
  { value: 'square' as SignalType, label: 'Square' },
  { value: 'sawtooth' as SignalType, label: 'Sawtooth' },
  { value: 'triangle' as SignalType, label: 'Triangle' }
]

const selectedSignal = ref<SignalType>('sine')

function selectSignal(type: SignalType) {
  selectedSignal.value = type
}
</script>
