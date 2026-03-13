<template>
  <div class="detached-window-container w-full h-screen bg-gradient-to-b from-gray-900 to-gray-950 p-4">
    <SpectrumMeter 
      :master-fx-output-node="masterFxOutputNode"
      :fft-data="fftData"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import SpectrumMeter from '../../components/master/SpectrumMeter.vue'
import type { DetachedWindowClient } from '../../lib/detachedWindowClient'

interface Props {
  wsClient: DetachedWindowClient
}

const props = defineProps<Props>()

const masterFxOutputNode = ref<any>(null)
const fftData = ref<{
  binsLeft: number[]
  binsRight: number[]
  sampleRate: number
} | null>(null)

const handleMessage = (data: any) => {
  // Handle FFT data updates
  if (data.type === 'fft') {
    // FFT data from backend
    fftData.value = {
      binsLeft: data.bins_left || data.binsLeft || [],
      binsRight: data.bins_right || data.binsRight || [],
      sampleRate: data.sample_rate || data.sampleRate || 48000
    }
  }
}

onMounted(() => {
  props.wsClient.on('message', handleMessage)
  props.wsClient.on('fft', handleMessage)
})

onUnmounted(() => {
  props.wsClient.off('message', handleMessage)
  props.wsClient.off('fft', handleMessage)
})
</script>

<style scoped>
.detached-window-container {
  overflow: auto;
}
</style>
