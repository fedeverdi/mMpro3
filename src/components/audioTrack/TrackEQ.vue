<template>
  <Transition
    enter-active-class="transition-all duration-300 ease-out"
    leave-active-class="transition-all duration-200 ease-in"
    enter-from-class="opacity-0 -translate-y-2 scale-95"
    enter-to-class="opacity-100 translate-y-0 scale-100"
    leave-from-class="opacity-100 translate-y-0 scale-100"
    leave-to-class="opacity-0 -translate-y-2 scale-95"
  >
    <div v-show="show" class="track-eq bg-gray-900 rounded border border-gray-700 p-2 mb-2">
      <!-- EQ Header -->
      <div class="flex items-center justify-between mb-3">
        <span class="text-[10px] font-bold text-gray-400">EQUALIZER</span>
        <button 
          @click="toggleEnabled"
          class="px-2 py-0.5 text-[9px] font-bold rounded transition-colors"
          :class="enabled ? 'bg-green-600 text-white' : 'bg-gray-700 text-gray-400'"
        >
          {{ enabled ? 'ON' : 'OFF' }}
        </button>
      </div>

      <!-- EQ Bands - Vertical Layout -->
      <div class="flex flex-col gap-3">
        <!-- Low Shelf (80Hz) -->
        <div class="flex items-center gap-2">
          <div class="flex-shrink-0" style="transform: scale(0.8)">
            <Knob 
              v-model="low"
              :min="-24"
              :max="24"
              :step="0.5"
              :centerValue="0"
              label="LOW"
              unit="dB"
              color="#10b981"
              :disabled="!enabled"
            />
          </div>
          <div class="flex-1 text-[9px]">
            <div class="font-mono text-blue-400">80Hz</div>
            <div class="text-gray-500">Low Shelf</div>
          </div>
        </div>

        <!-- Low Mid (400Hz) -->
        <div class="flex items-center gap-2">
          <div class="flex-shrink-0" style="transform: scale(0.8)">
            <Knob 
              v-model="lowMid"
              :min="-24"
              :max="24"
              :step="0.5"
              :centerValue="0"
              label="L-MID"
              unit="dB"
              color="#f59e0b"
              :disabled="!enabled"
            />
          </div>
          <div class="flex-1 text-[9px]">
            <div class="font-mono text-blue-400">400Hz</div>
            <div class="text-gray-500">Peaking</div>
          </div>
        </div>

        <!-- High Mid (2.5kHz) -->
        <div class="flex items-center gap-2">
          <div class="flex-shrink-0" style="transform: scale(0.8)">
            <Knob 
              v-model="highMid"
              :min="-24"
              :max="24"
              :step="0.5"
              :centerValue="0"
              label="H-MID"
              unit="dB"
              color="#f59e0b"
              :disabled="!enabled"
            />
          </div>
          <div class="flex-1 text-[9px]">
            <div class="font-mono text-blue-400">2.5kHz</div>
            <div class="text-gray-500">Peaking</div>
          </div>
        </div>

        <!-- High Shelf (8kHz) -->
        <div class="flex items-center gap-2">
          <div class="flex-shrink-0" style="transform: scale(0.8)">
            <Knob 
              v-model="high"
              :min="-24"
              :max="24"
              :step="0.5"
              :centerValue="0"
              label="HIGH"
              unit="dB"
              color="#ef4444"
              :disabled="!enabled"
            />
          </div>
          <div class="flex-1 text-[9px]">
            <div class="font-mono text-blue-400">8kHz</div>
            <div class="text-gray-500">High Shelf</div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Knob from '../core/Knob.vue'
import { useAudioEngine } from '../../composables/useAudioEngine'

interface Props {
  trackNumber: number
  show?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  show: false
})

const audioEngine = useAudioEngine()

const enabled = ref(true)
const low = ref(0)       // -24 to +24 dB
const lowMid = ref(0)    // -24 to +24 dB
const highMid = ref(0)   // -24 to +24 dB
const high = ref(0)      // -24 to +24 dB

function toggleEnabled() {
  enabled.value = !enabled.value
}

// Watch for changes and send to engine
watch([low, lowMid, highMid, high], () => {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackEQ(
      props.trackNumber - 1,
      low.value,
      lowMid.value,
      highMid.value,
      high.value
    )
  }
})

watch(enabled, (newEnabled) => {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.setTrackEQEnabled(props.trackNumber - 1, newEnabled)
  }
})
</script>
