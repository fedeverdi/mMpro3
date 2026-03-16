<template>
  <div class="reverb-effect bg-gray-800/50 justify-center h-full rounded-lg p-2 border border-gray-700 flex flex-col items-center gap-2">
    <span class="text-xs font-bold text-cyan-300">REVERB</span>

    <button @click="toggleEffect" :class="[
      'px-2 py-1 text-[0.6rem] font-bold rounded transition-colors w-full',
      isEnabled
        ? 'bg-cyan-600 text-white'
        : 'bg-gray-700 text-gray-400'
    ]">
      {{ isEnabled ? 'ON' : 'OFF' }}
    </button>

    <button @click="showModal = true"
      class="px-2 py-1 text-[0.6rem] font-bold rounded bg-blue-600 hover:bg-blue-700 text-white w-full">
      <div class="flex items-center gap-1 justify-center text-[0.6rem]">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="white" class="h-3.5 w-3.5">
          <path
            d="M487.4 315.7l-42.6-24.6c4.3-23.2 4.3-47 0-70.2l42.6-24.6c4.9-2.8 7.1-8.6 5.5-14-11.1-35.6-30-67.8-54.7-94.6-3.8-4.1-10-5.1-14.8-2.3L380.8 110c-17.9-15.4-38.5-27.3-60.8-35.1V25.8c0-5.6-3.9-10.5-9.4-11.7-36.7-8.2-74.3-7.8-109.2 0-5.5 1.2-9.4 6.1-9.4 11.7V75c-22.2 7.9-42.8 19.8-60.8 35.1L88.7 85.5c-4.9-2.8-11-1.9-14.8 2.3-24.7 26.7-43.6 58.9-54.7 94.6-1.7 5.4.6 11.2 5.5 14L67.3 221c-4.3 23.2-4.3 47 0 70.2l-42.6 24.6c-4.9 2.8-7.1 8.6-5.5 14 11.1 35.6 30 67.8 54.7 94.6 3.8 4.1 10 5.1 14.8 2.3l42.6-24.6c17.9 15.4 38.5 27.3 60.8 35.1v49.2c0 5.6 3.9 10.5 9.4 11.7 36.7 8.2 74.3 7.8 109.2 0 5.5-1.2 9.4-6.1 9.4-11.7v-49.2c22.2-7.9 42.8-19.8 60.8-35.1l42.6 24.6c4.9 2.8 11 1.9 14.8-2.3 24.7-26.7 43.6-58.9 54.7-94.6 1.5-5.5-.7-11.3-5.6-14.1zM256 336c-44.1 0-80-35.9-80-80s35.9-80 80-80 80 35.9 80 80-35.9 80-80 80z" />
        </svg>
        <div>
          SETTINGS
        </div>
      </div>
    </button>

    <!-- Settings Modal - Lexicon PCM70 Style -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[9999]"
        @mousedown.self="showModal = false">
        <div class="bg-gradient-to-b from-gray-900 to-gray-950 relative rounded-lg border-2 border-cyan-500 shadow-2xl shadow-cyan-500/50 px-6 pt-6 pb-4 max-w-2xl w-full mx-4" 
          style="font-family: 'Courier New', monospace;" @click.stop>
          
          <!-- Close button -->
          <button @click="showModal = false"
            class="absolute right-4 top-3 text-gray-400 hover:text-white text-2xl">&times;</button>
          
          <!-- PCM70 Header -->
          <div class="flex justify-between items-start mb-4">
            <div>
              <h3 class="text-base font-bold text-cyan-400 tracking-wider mb-1">LEXICON PCM70</h3>
              <div class="text-xs text-cyan-300/70">DIGITAL REVERBERATOR</div>
            </div>
            <button @click="toggleEffect" :class="[
              'px-4 py-1.5 text-xs font-bold rounded transition-colors',
              isEnabled
                ? 'bg-cyan-600 text-white shadow-cyan-500/50 shadow-md'
                : 'bg-gray-700 text-gray-400'
            ]">
              {{ isEnabled ? 'ACTIVE' : 'BYPASS' }}
            </button>
          </div>

          <!-- LCD Display -->
          <div class="bg-gradient-to-b from-blue-900 to-blue-950 border-2 border-blue-800 rounded-md p-4 mb-4 shadow-inner">
            <div class="grid grid-cols-3 gap-4 text-cyan-300 font-mono">
              <!-- Decay Time Display -->
              <div class="bg-blue-950/50 rounded px-3 py-2 border border-blue-800/50">
                <div class="text-[0.65rem] text-cyan-500/70 mb-1">DECAY TIME</div>
                <div class="text-lg font-bold">{{ decayTimeDisplay }}s</div>
              </div>
              
              <!-- Pre-Delay Display -->
              <div class="bg-blue-950/50 rounded px-3 py-2 border border-blue-800/50">
                <div class="text-[0.65rem] text-cyan-500/70 mb-1">PRE-DELAY</div>
                <div class="text-lg font-bold">{{ preDelayDisplay }}ms</div>
              </div>
              
              <!-- Diffusion Display -->
              <div class="bg-blue-950/50 rounded px-3 py-2 border border-blue-800/50">
                <div class="text-[0.65rem] text-cyan-500/70 mb-1">DAMPING</div>
                <div class="text-lg font-bold">{{ dampingDisplay }}%</div>
              </div>
            </div>
          </div>

          <!-- Algorithm Type -->
          <div class="mb-4 bg-gray-800/50 rounded-md p-3 border border-gray-700">
            <div class="text-[0.65rem] text-cyan-400 font-semibold mb-2">ALGORITHM</div>
            <div class="text-sm text-cyan-300">▸ HALL • FDN + Early Reflections</div>
          </div>

          <!-- Controls Grid -->
          <div class="grid grid-cols-5 gap-3 mb-4">
            <Knob class="scale-[0.85]" v-model="roomSize" :min="0" :max="1" :step="0.005" 
              label="Size" unit="" color="#06b6d4" />

            <Knob class="scale-[0.85]" v-model="preDelay" :min="0" :max="0.5" :step="0.002" 
              label="Pre-Dly" unit="ms" color="#0ea5e9" />

            <Knob class="scale-[0.85]" v-model="damping" :min="0" :max="1" :step="0.005" 
              label="Damping" unit="" color="#14b8a6" />

            <Knob class="scale-[0.85]" v-model="wet" :min="0" :max="1" :step="0.005" 
              label="Mix" unit="%" color="#10b981" />

            <Knob class="scale-[0.85]" v-model="width" :min="0" :max="1" :step="0.005" 
              label="Width" unit="" color="#8b5cf6" />
          </div>

          <!-- Parameter Info -->
          <div class="text-[0.65rem] text-gray-500 leading-relaxed bg-gray-800/30 rounded p-3 border border-gray-700/50">
            <div class="grid grid-cols-2 gap-2">
              <div><strong class="text-cyan-400">SIZE:</strong> Reverb decay time (0.1-10 sec)</div>
              <div><strong class="text-cyan-400">PRE-DLY:</strong> Initial delay (0-500 ms)</div>
              <div><strong class="text-cyan-400">DAMPING:</strong> High frequency absorption</div>
              <div><strong class="text-cyan-400">MIX:</strong> Wet/dry balance</div>
              <div><strong class="text-cyan-400">WIDTH:</strong> Stereo imaging</div>
              <div class="text-cyan-300/60">◆ FDN-based algorithm with modulated diffusers</div>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  enabled?: boolean
  initialRoomSize?: number
  initialDamping?: number
  initialWet?: number
  initialWidth?: number
  initialPreDelay?: number
}>()

const emit = defineEmits<{
  (e: 'toggle', enabled: boolean): void
  (e: 'update', params: { roomSize: number, damping: number, wet: number, width: number, preDelay: number }): void
}>()

const isEnabled = ref(props.enabled ?? false)
const showModal = ref(false)

const roomSize = ref(props.initialRoomSize ?? 0.3)
const damping = ref(props.initialDamping ?? 0.5)
const wet = ref(props.initialWet ?? 0.2)
const width = ref(props.initialWidth ?? 1.0)
const preDelay = ref(props.initialPreDelay ?? 0.0)

// Computed displays for LCD screen
const decayTimeDisplay = computed(() => {
  // Map room_size (0-1) to decay_time (0.1-10 seconds)
  const decayTime = 0.1 + (roomSize.value * 9.9)
  return decayTime.toFixed(2)
})

const preDelayDisplay = computed(() => {
  // Pre-delay in milliseconds
  return (preDelay.value * 1000).toFixed(0)
})

const dampingDisplay = computed(() => {
  return (damping.value * 100).toFixed(0)
})

function toggleEffect() {
  isEnabled.value = !isEnabled.value
  emit('toggle', isEnabled.value)
}

watch([roomSize, damping, wet, width, preDelay], () => {
  emit('update', {
    roomSize: roomSize.value,
    damping: damping.value,
    wet: wet.value,
    width: width.value,
    preDelay: preDelay.value
  })
})
</script>

<style scoped>
/* PCM70-style enhancements */
</style>
