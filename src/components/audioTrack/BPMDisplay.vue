<template>
  <div v-if="detectedBPM && audioSourceType === 'file' && audioLoaded"
    class="px-1.5 py-0.5 bg-blue-600/20 border border-blue-500/40 text-blue-300 text-[0.45rem] font-bold rounded"
    :title="'Detected BPM: ' + detectedBPM.toFixed(1)">
    {{ Math.round(detectedBPM) }} BPM
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  audioBuffer: AudioBuffer | null
  audioSourceType: 'file' | 'input'
  audioLoaded: boolean
}

const props = defineProps<Props>()

const detectedBPM = ref<number | null>(null)

// Watch for audio buffer changes and detect BPM
watch(() => props.audioBuffer, (newBuffer) => {
  if (newBuffer && props.audioSourceType === 'file') {
    detectBPM(newBuffer)
  } else {
    detectedBPM.value = null
  }
}, { immediate: true })

// Detect BPM from audio buffer using peak detection
function detectBPM(audioBuffer: AudioBuffer) {
  if (!audioBuffer) {
    detectedBPM.value = null
    return
  }

  try {
    // Get first channel data
    const channelData = audioBuffer.getChannelData(0)
    const sampleRate = audioBuffer.sampleRate
    
    // Analyze only first 30 seconds for performance
    const maxDuration = 30 // seconds
    const samplesPerSecond = sampleRate
    const totalSamples = Math.min(channelData.length, maxDuration * samplesPerSecond)
    
    // Calculate energy in windows (beat detection)
    const windowSize = Math.floor(sampleRate * 0.05) // 50ms windows
    const energyValues: number[] = []
    
    for (let i = 0; i < totalSamples - windowSize; i += windowSize) {
      let energy = 0
      for (let j = 0; j < windowSize; j++) {
        energy += Math.abs(channelData[i + j])
      }
      energyValues.push(energy / windowSize)
    }
    
    // Find peaks in energy
    const peaks: number[] = []
    const threshold = energyValues.reduce((a, b) => a + b, 0) / energyValues.length * 1.3
    
    for (let i = 1; i < energyValues.length - 1; i++) {
      if (energyValues[i] > threshold && 
          energyValues[i] > energyValues[i - 1] && 
          energyValues[i] > energyValues[i + 1]) {
        peaks.push(i)
      }
    }
    
    if (peaks.length < 2) {
      detectedBPM.value = null
      return
    }
    
    // Calculate intervals between peaks
    const intervals: number[] = []
    for (let i = 1; i < peaks.length; i++) {
      intervals.push(peaks[i] - peaks[i - 1])
    }
    
    // Find most common interval (mode)
    const intervalCounts = new Map<number, number>()
    intervals.forEach(interval => {
      const rounded = Math.round(interval / 2) * 2 // Group similar intervals
      intervalCounts.set(rounded, (intervalCounts.get(rounded) || 0) + 1)
    })
    
    let maxCount = 0
    let mostCommonInterval = 0
    intervalCounts.forEach((count, interval) => {
      if (count > maxCount) {
        maxCount = count
        mostCommonInterval = interval
      }
    })
    
    // Convert interval to BPM
    const secondsPerBeat = (mostCommonInterval * windowSize) / sampleRate
    const bpm = 60 / secondsPerBeat
    
    // Validate BPM range (60-180 typical for music)
    if (bpm >= 60 && bpm <= 200) {
      detectedBPM.value = bpm
    } else if (bpm < 60) {
      // Try doubling if too slow
      detectedBPM.value = bpm * 2
    } else if (bpm > 200) {
      // Try halving if too fast
      detectedBPM.value = bpm / 2
    } else {
      detectedBPM.value = null
    }
  } catch (error) {
    console.error('Error detecting BPM:', error)
    detectedBPM.value = null
  }
}
</script>
