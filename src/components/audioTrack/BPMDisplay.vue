<template>
  <div v-if="detectedBPM && audioSourceType === 'file' && audioLoaded"
    class="px-1.5 py-0.5 bg-blue-600/20 border border-blue-500/40 text-blue-300 text-[0.45rem] font-bold rounded"
    :title="'Detected BPM: ' + detectedBPM.toFixed(1)">
    {{ Math.round(detectedBPM) }} BPM
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import * as BeatDetector from 'web-audio-beat-detector'

interface Props {
  audioBuffer: AudioBuffer | null
  audioSourceType: 'file' | 'input'
  audioLoaded: boolean
}

const props = defineProps<Props>()

const detectedBPM = ref<number | null>(null)

// Watch for audio buffer changes and detect BPM
watch(() => [props.audioBuffer, props.audioLoaded, props.audioSourceType] as const, ([newBuffer, loaded, sourceType]) => {
  if (newBuffer && loaded && sourceType === 'file') {
    detectBPM(newBuffer)
  } else {
    detectedBPM.value = null
  }
}, { immediate: true })

// Detect BPM from audio buffer using web-audio-beat-detector
async function detectBPM(audioBuffer: AudioBuffer) {
  if (!audioBuffer || !audioBuffer.duration || audioBuffer.duration <= 0) {
    detectedBPM.value = null
    return
  }

  try {
    const tempo = await BeatDetector.analyze(audioBuffer)
    
    if (tempo && typeof tempo === 'number') {
      let correctedTempo = tempo
      
      // Try to find the best tempo in the 70-130 BPM sweet spot
      const candidates = [tempo, tempo / 2, tempo / 1.5, tempo * 2]
      const sweetSpotCandidates = candidates.filter(bpm => bpm >= 70 && bpm <= 130)
      
      if (sweetSpotCandidates.length > 0) {
        // Pick the one closest to 90-100 BPM (most common music range)
        correctedTempo = sweetSpotCandidates.reduce((prev, curr) => 
          Math.abs(curr - 95) < Math.abs(prev - 95) ? curr : prev
        )
      } else if (tempo > 140) {
        // If no sweet spot candidate, halve if too fast
        correctedTempo = tempo / 2
      } else if (tempo < 60) {
        // If too slow, double it
        correctedTempo = tempo * 2
      }
      
      detectedBPM.value = correctedTempo
    } else {
      detectedBPM.value = null
    }
  } catch (error) {
    // Silently handle BPM detection failures (e.g., tracks without detectable beats)
    detectedBPM.value = null
  }
}
</script>
