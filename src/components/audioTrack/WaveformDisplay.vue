<template>
  <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
    <div class="flex items-center gap-1">
      <div 
        class="flex-1 px-0.5 py-0.5 cursor-pointer hover:opacity-80 transition-opacity"
        @click="handleClick"
        :title="mode === 'signal' ? 'Click to show full waveform' : 'Click to show real-time signal'"
      >
        <canvas ref="canvasRef" class="w-full h-[30px] rounded border border-gray-700 bg-black"
          style="image-rendering: crisp-edges;"></canvas>
      </div>
      
      <!-- Mode selector buttons -->
      <div class="flex flex-col gap-0.5">
        <button
          @click="mode = 'signal'"
          :class="mode === 'signal' ? 'bg-cyan-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
          class="px-1 py-0.5 rounded text-[10px] font-medium transition-colors"
          title="Real-time signal"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <circle cx="12" cy="12" r="8"/>
          </svg>
        </button>
        <button
          @click="mode = 'waveform'"
          :class="mode === 'waveform' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
          class="px-1 py-0.5 rounded text-[10px] font-medium transition-colors"
          title="Full waveform"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path d="M3 9h2v6H3V9zm4-6h2v18H7V3zm4 3h2v12h-2V6zm4-2h2v14h-2V4zm4 4h2v8h-2V8z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

interface Props {
  waveformNode?: any
  audioBuffer?: AudioBuffer | null
  isPlaying?: boolean
  currentTime?: number // Current playback time in seconds
}

const props = withDefaults(defineProps<Props>(), {
  audioBuffer: null,
  isPlaying: false,
  currentTime: 0
})

const canvasRef = ref<HTMLCanvasElement | null>(null)
const mode = ref<'signal' | 'waveform'>('signal')
let animationId: number | null = null

function handleClick() {
  mode.value = mode.value === 'signal' ? 'waveform' : 'signal'
}

onMounted(() => {
  if (mode.value === 'signal') {
    startSignalDrawing()
  } else {
    drawFullWaveform()
  }
})

onUnmounted(() => {
  stopDrawing()
})

// Watch for mode or waveformNode changes
watch(() => mode.value, (newMode) => {
  stopDrawing()
  if (newMode === 'signal') {
    startSignalDrawing()
  } else {
    drawFullWaveform()
  }
})

watch(() => props.audioBuffer, () => {
  if (mode.value === 'waveform') {
    drawFullWaveform()
  }
})

watch(() => props.currentTime, () => {
  if (mode.value === 'waveform') {
    drawFullWaveform()
  }
})

watch(() => props.waveformNode, () => {
  if (mode.value === 'signal') {
    stopDrawing()
    startSignalDrawing()
  }
})

// Real-time signal oscilloscope
function drawSignal() {
  if (!canvasRef.value || !props.waveformNode) {
    return
  }

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()

  // Set canvas size accounting for device pixel ratio
  canvas.width = rect.width * dpr
  canvas.height = 50 * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = 50

  // Clear canvas
  ctx.fillStyle = '#000000' // black background
  ctx.fillRect(0, 0, width, height)

  // Get waveform data
  const values = props.waveformNode.getValue()
  
  if (!values || values.length === 0) {
    drawCenterLine(ctx, width, height)
    return
  }

  // Draw waveform
  ctx.strokeStyle = '#22d3ee' // cyan-400
  ctx.lineWidth = 1.5
  ctx.beginPath()

  const sliceWidth = width / values.length
  let x = 0

  for (let i = 0; i < values.length; i++) {
    // Normalize value from -1,1 to 0,height
    const v = (values[i] + 1) / 2
    const y = v * height

    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }

    x += sliceWidth
  }

  ctx.stroke()

  // Draw center line
  drawCenterLine(ctx, width, height)
}

// Full waveform with timeline
function drawFullWaveform() {
  if (!canvasRef.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()

  canvas.width = rect.width * dpr
  canvas.height = 50 * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = 50

  // Clear canvas
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)

  // If no audio buffer, just show center line
  if (!props.audioBuffer) {
    drawCenterLine(ctx, width, height)
    return
  }

  const duration = props.audioBuffer.duration
  const sampleRate = props.audioBuffer.sampleRate
  const channelData = props.audioBuffer.getChannelData(0)
  
  // Show 10 second window
  const windowDuration = 10 // seconds
  const currentTime = props.currentTime || 0
  
  // Calculate window start/end to keep playhead centered or scroll
  let windowStart = Math.max(0, currentTime - windowDuration / 2)
  let windowEnd = windowStart + windowDuration
  
  // If we're near the end, adjust window
  if (windowEnd > duration) {
    windowEnd = duration
    windowStart = Math.max(0, windowEnd - windowDuration)
  }
  
  const windowStartSample = Math.floor(windowStart * sampleRate)
  const windowEndSample = Math.floor(windowEnd * sampleRate)
  const windowSamples = windowEndSample - windowStartSample
  
  const step = Math.ceil(windowSamples / width)
  const amp = height / 2

  ctx.strokeStyle = '#3b82f6' // blue-500
  ctx.lineWidth = 1

  for (let i = 0; i < width; i++) {
    let min = 1.0
    let max = -1.0

    for (let j = 0; j < step; j++) {
      const sampleIndex = windowStartSample + (i * step) + j
      if (sampleIndex < channelData.length) {
        const datum = channelData[sampleIndex]
        if (datum < min) min = datum
        if (datum > max) max = datum
      }
    }

    const x = i
    const yMin = (1 + min) * amp
    const yMax = (1 + max) * amp

    ctx.beginPath()
    ctx.moveTo(x, yMin)
    ctx.lineTo(x, yMax)
    ctx.stroke()
  }

  // Draw center line
  drawCenterLine(ctx, width, height)

  // Draw playback position (red line)
  if (props.isPlaying || currentTime > 0) {
    const relativeTime = currentTime - windowStart
    const progress = relativeTime / (windowEnd - windowStart)
    const x = progress * width
    
    // Only draw if within visible window
    if (x >= 0 && x <= width) {
      // Calculate vertical margins for centered, shorter line
      const verticalMargin = height * 0.1 // 10% margin top and bottom
      const lineStart = verticalMargin
      const lineEnd = height - verticalMargin
      
      ctx.strokeStyle = '#ef4444' // red-500
      ctx.lineWidth = 1 // Thinner line
      ctx.beginPath()
      ctx.moveTo(x, lineStart)
      ctx.lineTo(x, lineEnd)
      ctx.stroke()
      
      // Add subtle glow effect
      ctx.shadowColor = '#ef4444'
      ctx.shadowBlur = 6
      ctx.stroke()
      ctx.shadowBlur = 0
    }
  }
}

function drawCenterLine(ctx: CanvasRenderingContext2D, width: number, height: number) {
  ctx.strokeStyle = 'rgba(100, 116, 139, 0.3)' // gray-500 with opacity
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, height / 2)
  ctx.lineTo(width, height / 2)
  ctx.stroke()
}

// Start real-time signal animation loop
function startSignalDrawing() {
  if (animationId !== null || mode.value !== 'signal') return

  function animate() {
    drawSignal()
    animationId = requestAnimationFrame(animate)
  }

  animationId = requestAnimationFrame(animate)
}

// Stop animation loop
function stopDrawing() {
  if (animationId !== null) {
    cancelAnimationFrame(animationId)
    animationId = null
  }
  
  // Clear the canvas
  if (canvasRef.value) {
    const canvas = canvasRef.value
    const ctx = canvas.getContext('2d')
    if (ctx) {
      const rect = canvas.getBoundingClientRect()
      ctx.fillStyle = '#000000'
      ctx.fillRect(0, 0, rect.width, 50)
      
      // Draw center line on empty canvas
      drawCenterLine(ctx, rect.width, 50)
    }
  }
}

// Expose methods for external control
defineExpose({
  start: startSignalDrawing,
  stop: stopDrawing,
  redraw: () => {
    if (mode.value === 'waveform') {
      drawFullWaveform()
    }
  }
})
</script>
