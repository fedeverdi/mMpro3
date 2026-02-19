<template>
  <div class="reverb-effect bg-gray-800/50 rounded-lg p-3 border border-gray-700 flex flex-col items-center gap-2">
    <span class="text-xs font-bold text-green-300">REVERB</span>

    <button @click="toggleEffect" :class="[
      'px-4 py-1 text-[0.6rem] font-bold rounded transition-colors w-full',
      isEnabled
        ? 'bg-green-600 text-white'
        : 'bg-gray-700 text-gray-400'
    ]">
      {{ isEnabled ? 'ON' : 'OFF' }}
    </button>

    <button @click="showModal = true"
      class="px-4 py-1 text-[0.6rem] font-bold rounded bg-blue-600 hover:bg-blue-700 text-white w-full">
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

    <!-- Settings Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
        @click="showModal = false">
        <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-md w-full mx-4" @click.stop>
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-xl font-bold text-green-300">Reverb Settings</h3>
            <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
          </div>

          <p class="text-xs text-yellow-500 mb-2 text-center">* Decay requires toggle OFF/ON to apply</p>

          <div class="grid grid-cols-3 gap-4">
            <Knob v-model="decay" :min="0.1" :max="10" :step="0.1" label="Decay" unit="s" color="#10b981" />

            <Knob v-model="preDelay" :min="0" :max="0.1" :step="0.001" label="Pre-Delay" unit="s" color="#10b981" />

            <Knob v-model="wet" :min="0" :max="1" :step="0.01" label="Wet" unit="%" color="#10b981" />
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import Knob from './Knob.vue'

const props = defineProps<{
  enabled?: boolean
  initialDecay?: number
  initialPreDelay?: number
  initialWet?: number
}>()

const emit = defineEmits<{
  (e: 'toggle', enabled: boolean): void
  (e: 'update', params: { decay: number, preDelay: number, wet: number }): void
}>()

const isEnabled = ref(props.enabled ?? false)
const showModal = ref(false)

const decay = ref(props.initialDecay ?? 1.5)
const preDelay = ref(props.initialPreDelay ?? 0.01)
const wet = ref(props.initialWet ?? 0.3)

function toggleEffect() {
  isEnabled.value = !isEnabled.value
  emit('toggle', isEnabled.value)
}

watch([decay, preDelay, wet], () => {
  emit('update', {
    decay: decay.value,
    preDelay: preDelay.value,
    wet: wet.value
  })
})
</script>

<style scoped>
/* No additional styles needed - Knob component handles its own styling */
</style>
