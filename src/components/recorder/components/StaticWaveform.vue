<template>
  <div class="static-waveform flex items-center gap-2 bg-gray-900/30 rounded border border-gray-700 p-2">
    <div class="text-xs text-gray-400 font-mono w-8 flex-shrink-0">T{{ trackNumber }}</div>
    <div class="relative flex-1 min-w-0">
      <canvas 
        ref="canvasRef" 
        :width="600" 
        :height="32"
        class="w-full rounded bg-black/30"
        style="width: 100%; height: 32px; max-width: 100%;"
      />
      <!-- Playback progress bar -->
      <div 
        v-if="isPlaying && duration > 0"
        class="absolute top-0 bottom-0 w-0.5 bg-red-500 pointer-events-none transition-all duration-100"
        :style="{ left: playbackPosition + '%' }"
      ></div>
    </div>
    <div class="text-xs text-gray-500 truncate w-24 flex-shrink-0" :title="fileName">
      {{ fileName }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, inject, watch } from 'vue'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'

interface Props {
  trackNumber: number
  fileName: string
  fileId: string
  isPlaying?: boolean
  currentTime?: number
  duration?: number
}

const props = withDefaults(defineProps<Props>(), {
  isPlaying: false,
  currentTime: 0,
  duration: 0
})

const canvasRef = ref<HTMLCanvasElement | null>(null)
const { getAudioFile } = useAudioFileStorage()
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Calculate playback position as percentage
const playbackPosition = computed(() => {
  if (!props.duration || props.duration === 0) return 0
  return Math.min(100, (props.currentTime / props.duration) * 100)
})

onMounted(async () => {
  Tone = ToneRef?.value ?? ToneRef
  if (!Tone || !canvasRef.value) return

  try {
    // Load audio file from IndexedDB
    const storedFile = await getAudioFile(props.fileId)
    if (!storedFile) {
      console.error('[StaticWaveform] File not found:', props.fileId)
      return
    }

    // Decode audio data
    const audioBuffer = await Tone.context.decodeAudioData(storedFile.arrayBuffer.slice(0))
    
    // Draw waveform
    drawWaveform(audioBuffer)
  } catch (error) {
    console.error('[StaticWaveform] Error loading waveform:', error)
  }
})

function drawWaveform(audioBuffer: AudioBuffer) {
  if (!canvasRef.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const width = canvas.width
  const height = canvas.height

  // Clear canvas
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)

  // Get audio data (mix down to mono for simplicity)
  const data = audioBuffer.getChannelData(0)
  const step = Math.ceil(data.length / width)
  const amp = height / 2

  // Draw waveform
  ctx.strokeStyle = '#3b82f6' // blue-500
  ctx.lineWidth = 1
  ctx.beginPath()

  for (let i = 0; i < width; i++) {
    let min = 1.0
    let max = -1.0

    // Find min/max in this segment
    for (let j = 0; j < step; j++) {
      const datum = data[i * step + j]
      if (datum < min) min = datum
      if (datum > max) max = datum
    }

    const yMin = (1 + min) * amp
    const yMax = (1 + max) * amp

    if (i === 0) {
      ctx.moveTo(i, yMin)
    }
    ctx.lineTo(i, yMin)
    ctx.lineTo(i, yMax)
  }

  ctx.stroke()

  // Draw center line
  ctx.strokeStyle = '#404040'
  ctx.lineWidth = 1
  ctx.setLineDash([2, 2])
  ctx.beginPath()
  ctx.moveTo(0, height / 2)
  ctx.lineTo(width, height / 2)
  ctx.stroke()
  ctx.setLineDash([])
}
</script>

<style scoped>
.static-waveform {
  user-select: none;
  -webkit-user-select: none;
}
</style>
