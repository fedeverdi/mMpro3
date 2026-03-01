<template>
  <div class="triangle-wave-generator bg-gray-800/50 rounded-lg p-3 border border-gray-700">
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs font-bold text-purple-300">TRIANGLE WAVE</span>
      <button @click="toggleOscillator" :class="[
        'px-3 py-1 text-[0.6rem] font-bold rounded transition-colors',
        isPlaying ? 'bg-purple-600 text-white' : 'bg-gray-700 text-gray-400'
      ]">
        {{ isPlaying ? 'ON' : 'OFF' }}
      </button>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <Knob v-model="frequency" :min="20" :max="20000" :step="1" label="Frequency" unit="Hz" color="#a855f7" />
      <Knob v-model="volume" :min="-60" :max="0" :step="1" label="Volume" unit="dB" color="#a855f7" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, inject } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  outputNode?: any
}>()

const ToneRef = inject<any>('Tone', null)
let Tone: any = null
let oscillatorNode: any = null
let volumeNode: any = null

const isPlaying = ref(false)
const frequency = ref(1000)
const volume = ref(-12)

async function initOscillator() {
  if (!Tone || !props.outputNode) return

  if (!oscillatorNode) {
    oscillatorNode = new Tone.Oscillator({
      type: 'triangle',
      frequency: frequency.value
    })
    volumeNode = new Tone.Volume(volume.value)
    
    oscillatorNode.connect(volumeNode)
    volumeNode.connect(props.outputNode)
  }
}

function toggleOscillator() {
  if (!oscillatorNode) return

  if (isPlaying.value) {
    oscillatorNode.stop()
    isPlaying.value = false
  } else {
    oscillatorNode.start()
    isPlaying.value = true
  }
}

watch(frequency, (newFreq) => {
  if (oscillatorNode) {
    oscillatorNode.frequency.value = newFreq
  }
})

watch(volume, (newVolume) => {
  if (volumeNode) {
    volumeNode.volume.value = newVolume
  }
})

watch(() => props.outputNode, async (newOutput) => {
  if (newOutput) {
    await initOscillator()
  }
}, { immediate: true })

onMounted(async () => {
  if (ToneRef?.value) {
    Tone = ToneRef.value
    await initOscillator()
  }
})

onUnmounted(() => {
  if (oscillatorNode) {
    if (isPlaying.value) {
      oscillatorNode.stop()
    }
    oscillatorNode.dispose()
  }
  if (volumeNode) {
    volumeNode.dispose()
  }
})
</script>
