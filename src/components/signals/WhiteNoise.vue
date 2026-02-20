<template>
  <div class="white-noise-generator bg-gray-800/50 rounded-lg p-3 border border-gray-700">
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs font-bold text-blue-300">WHITE NOISE</span>
      <button @click="toggleNoise" :class="[
        'px-3 py-1 text-[0.6rem] font-bold rounded transition-colors',
        isPlaying ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400'
      ]">
        {{ isPlaying ? 'ON' : 'OFF' }}
      </button>
    </div>

    <div class="flex flex-col gap-3">
      <Knob v-model="volume" :min="-60" :max="0" :step="1" label="Volume" unit="dB" color="#3b82f6" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, inject } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  outputNode?: any
}>()

const ToneRef = inject<any>('Tone')
let Tone: any = null
let noiseNode: any = null
let volumeNode: any = null

const isPlaying = ref(false)
const volume = ref(-12)

async function initNoise() {
  if (!Tone || !props.outputNode) return

  if (!noiseNode) {
    // Create white noise generator
    noiseNode = new Tone.Noise('white')
    volumeNode = new Tone.Volume(volume.value)
    
    // Connect: noise -> volume -> output
    noiseNode.connect(volumeNode)
    volumeNode.connect(props.outputNode)
  }
}

function toggleNoise() {
  if (!noiseNode) return

  if (isPlaying.value) {
    noiseNode.stop()
    isPlaying.value = false
  } else {
    noiseNode.start()
    isPlaying.value = true
  }
}

watch(volume, (newVolume) => {
  if (volumeNode) {
    volumeNode.volume.value = newVolume
  }
})

watch(() => props.outputNode, async (newOutput) => {
  if (newOutput) {
    await initNoise()
  }
}, { immediate: true })

onMounted(async () => {
  if (ToneRef?.value) {
    Tone = ToneRef.value
    await initNoise()
  }
})

onUnmounted(() => {
  if (noiseNode) {
    if (isPlaying.value) {
      noiseNode.stop()
    }
    noiseNode.dispose()
  }
  if (volumeNode) {
    volumeNode.dispose()
  }
})
</script>
