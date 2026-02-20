<template>
  <div class="bg-gray-900/50 rounded-lg p-4 border border-gray-700">
    <div class="text-xs text-gray-400 uppercase tracking-wider mb-2">Waveform</div>
    <canvas ref="waveformCanvas" width="800" height="96" class="w-full bg-black/50 rounded" style="height: 96px;" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  analyser: AnalyserNode | null
  isRecording: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  levelUpdate: [level: number]
}>()

const waveformCanvas = ref<HTMLCanvasElement | null>(null)
let animationFrameId: number | null = null
let dataArray: Uint8Array | null = null
let waveformHistory: number[] = []
let frameCounter = 0
const SAMPLE_RATE = 6 // Add a sample every N frames (higher = more compressed)

function drawWaveform() {
  if (!waveformCanvas.value || !props.analyser) {
    return
  }

  const canvas = waveformCanvas.value
  const canvasCtx = canvas.getContext('2d')
  if (!canvasCtx) return

  // Initialize data array if needed
  if (!dataArray && props.analyser) {
    dataArray = new Uint8Array(props.analyser.frequencyBinCount)
  }

  // Initialize waveform history buffer only if empty (first time)
  if (waveformHistory.length === 0) {
    waveformHistory = []
    frameCounter = 0
  }

  const draw = () => {
    if (!props.analyser || !dataArray || !props.isRecording) return

    animationFrameId = requestAnimationFrame(draw)

    const width = canvas.width
    const height = canvas.height

    // Get waveform data
    props.analyser.getByteTimeDomainData(dataArray as any)

    // Calculate level meter from waveform data
    let sum = 0
    let max = 0
    for (let i = 0; i < dataArray.length; i++) {
      const value = Math.abs(dataArray[i] - 128)
      sum += value
      max = Math.max(max, value)
    }
    
    // Emit level update
    const levelPercent = Math.min(100, (max / 128) * 100)
    emit('levelUpdate', levelPercent)

    // Increment frame counter and only add sample every SAMPLE_RATE frames
    frameCounter++
    if (frameCounter >= SAMPLE_RATE) {
      frameCounter = 0
      
      // Get current waveform sample (center value for stability)
      const centerSample = dataArray[Math.floor(dataArray.length / 2)]
      // Amplify the waveform (multiply by 2 for more visible peaks)
      const normalizedSample = ((centerSample / 128.0 - 1) * 2) * (height / 2) + (height / 2)

      // Add new sample to history and scroll
      waveformHistory.push(normalizedSample)
      // Keep history size equal to canvas width
      if (waveformHistory.length > width) {
        waveformHistory.shift()
      }
    }

    // Clear canvas with solid background
    canvasCtx.fillStyle = '#000000'
    canvasCtx.fillRect(0, 0, width, height)

    // Draw center line (reference)
    canvasCtx.strokeStyle = '#404040' // dark gray
    canvasCtx.lineWidth = 1
    canvasCtx.setLineDash([2, 2]) // dashed line
    canvasCtx.beginPath()
    canvasCtx.moveTo(0, height / 2)
    canvasCtx.lineTo(width, height / 2)
    canvasCtx.stroke()
    canvasCtx.setLineDash([]) // reset to solid

    // Draw waveform history (scrolling timeline)
    canvasCtx.lineWidth = 2
    canvasCtx.strokeStyle = '#3b82f6' // blue-500
    canvasCtx.beginPath()

    for (let i = 0; i < waveformHistory.length; i++) {
      const x = i
      const y = waveformHistory[i]

      if (i === 0) {
        canvasCtx.moveTo(x, y)
      } else {
        canvasCtx.lineTo(x, y)
      }
    }

    canvasCtx.stroke()

    // Draw playhead line (current time indicator)
    if (waveformHistory.length > 0) {
      const playheadX = waveformHistory.length - 1
      canvasCtx.strokeStyle = '#ef4444' // red-500
      canvasCtx.lineWidth = 2
      canvasCtx.beginPath()
      canvasCtx.moveTo(playheadX, 0)
      canvasCtx.lineTo(playheadX, height)
      canvasCtx.stroke()
    }
  }

  draw()
}

// Watch for recording state changes
watch(() => props.isRecording, (isRecording) => {
  if (isRecording && props.analyser) {
    drawWaveform()
  } else {
    // Stop animation when recording stops
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId)
      animationFrameId = null
    }
    // Draw empty canvas with center line
    drawEmptyCanvas()
  }
})

// Watch for analyser changes (when recording starts)
watch(() => props.analyser, (newAnalyser) => {
  if (newAnalyser && props.isRecording) {
    dataArray = new Uint8Array(newAnalyser.frequencyBinCount)
    drawWaveform()
  }
})

// Cleanup on unmount
import { onUnmounted, onMounted } from 'vue'

function drawEmptyCanvas() {
  if (!waveformCanvas.value) return
  
  const canvas = waveformCanvas.value
  const canvasCtx = canvas.getContext('2d')
  if (!canvasCtx) return

  const width = canvas.width
  const height = canvas.height

  // Clear canvas with solid background
  canvasCtx.fillStyle = '#000000'
  canvasCtx.fillRect(0, 0, width, height)

  // Draw center line (reference)
  canvasCtx.strokeStyle = '#404040' // dark gray
  canvasCtx.lineWidth = 1
  canvasCtx.setLineDash([2, 2]) // dashed line
  canvasCtx.beginPath()
  canvasCtx.moveTo(0, height / 2)
  canvasCtx.lineTo(width, height / 2)
  canvasCtx.stroke()
  canvasCtx.setLineDash([]) // reset to solid
}

onMounted(() => {
  drawEmptyCanvas()
})

onUnmounted(() => {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId)
  }
})

// Expose method to restart waveform (for modal reopen)
defineExpose({
  restart: () => {
    if (props.isRecording && props.analyser) {
      drawWaveform()
    }
  }
})
</script>
