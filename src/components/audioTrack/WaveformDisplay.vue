<template>
  <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
    <div class="flex items-center gap-1">
      <div 
        class="flex-1 px-0.5 py-0.5 relative"
        :class="showModeButtons ? 'cursor-pointer hover:opacity-80 transition-opacity' : ''"
        @click="handleCanvasClick"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseLeave"
        :title="showModeButtons ? (internalMode === 'signal' ? 'Click to show full waveform' : 'Click to show real-time signal') : ''"
      >
        <canvas ref="canvasRef" class="w-full h-[30px] rounded border border-gray-700 bg-black"
          style="image-rendering: crisp-edges;"></canvas>
        
        <!-- Time tooltip during drag/hover -->
        <div 
          v-if="showTimeTooltip && tooltipTime !== null"
          class="absolute pointer-events-none bg-gray-800 text-white text-xs px-2 py-1 rounded shadow-lg border border-gray-600"
          :style="{ left: tooltipX + 'px', top: '-30px' }"
        >
          {{ formatTime(tooltipTime) }}
        </div>
      </div>
      
      <!-- Mode selector buttons -->
      <div v-if="showModeButtons" class="flex flex-col gap-0.5">
        <button
          @click="internalMode = 'signal'"
          :class="internalMode === 'signal' ? 'bg-cyan-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
          class="px-1 py-0.5 rounded text-[10px] font-medium transition-colors"
          title="Real-time signal"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <circle cx="12" cy="12" r="8"/>
          </svg>
        </button>
        <button
          @click="internalMode = 'waveform'"
          :class="internalMode === 'waveform' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'"
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
import { ref, watch, onMounted, onUnmounted, inject } from 'vue'

// Import audio engine from context
const audioEngine = inject('audioEngine') as any

interface Props {
  trackNumber: number // Track number (0-based for backend)
  audioBuffer?: AudioBuffer | null
  isPlaying?: boolean
  currentTime?: number // Current playback time in seconds
  mode?: 'signal' | 'waveform' // External mode control
  showModeButtons?: boolean // Show mode toggle buttons
  isActive?: boolean // Whether to draw (play or input active)
  duration?: number // Track duration in seconds
}

const props = withDefaults(defineProps<Props>(), {
  audioBuffer: null,
  isPlaying: false,
  currentTime: 0,
  mode: 'signal',
  showModeButtons: true,
  isActive: false,
  duration: 0
})

const emit = defineEmits<{
  seek: [time: number]
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const internalMode = ref<'signal' | 'waveform'>(props.mode)
let animationId: number | null = null
let isDrawingLoop = false

// Drag and tooltip state
const isDragging = ref(false)
const showTimeTooltip = ref(false)
const tooltipTime = ref<number | null>(null)
const tooltipX = ref(0)

// Sync internal mode with prop
watch(() => props.mode, (newMode) => {
  internalMode.value = newMode
})

// Format time as MM:SS or HH:MM:SS
function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = Math.floor(seconds % 60)
  
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  }
  return `${minutes}:${secs.toString().padStart(2, '0')}`
}

function calculateTimeFromPosition(clientX: number): number | null {
  const canvas = canvasRef.value
  if (!canvas || !props.audioBuffer) return null
  
  const rect = canvas.getBoundingClientRect()
  const clickX = clientX - rect.left
  const width = rect.width
  
  // Calculate time based on full duration
  const duration = props.audioBuffer.duration
  const clickProgress = Math.max(0, Math.min(1, clickX / width))
  const time = clickProgress * duration
  
  return time
}

function handleMouseDown(event: MouseEvent) {
  if (internalMode.value !== 'waveform' || !props.audioBuffer) return
  
  isDragging.value = true
  showTimeTooltip.value = true
  
  const time = calculateTimeFromPosition(event.clientX)
  if (time !== null) {
    tooltipTime.value = time
    tooltipX.value = event.clientX - (event.currentTarget as HTMLElement).getBoundingClientRect().left
    emit('seek', time)
  }
}

function handleMouseMove(event: MouseEvent) {
  if (internalMode.value !== 'waveform' || !props.audioBuffer) return
  
  const time = calculateTimeFromPosition(event.clientX)
  if (time !== null) {
    tooltipX.value = event.clientX - (event.currentTarget as HTMLElement).getBoundingClientRect().left
    
    // Show tooltip on hover in waveform mode
    showTimeTooltip.value = true
    tooltipTime.value = time
    
    // Only seek if dragging
    if (isDragging.value) {
      emit('seek', time)
    }
  }
}

function handleMouseUp(event: MouseEvent) {
  if (isDragging.value) {
    isDragging.value = false
    // Keep tooltip visible briefly after release
    setTimeout(() => {
      if (!isDragging.value) {
        showTimeTooltip.value = false
      }
    }, 500)
  }
}

function handleMouseLeave() {
  isDragging.value = false
  showTimeTooltip.value = false
  tooltipTime.value = null
}

function handleCanvasClick(event: MouseEvent) {
  // Seek is now handled by mousedown/mousemove
  // Only handle mode toggle if NOT in waveform mode
  if (internalMode.value === 'signal' && props.showModeButtons) {
    internalMode.value = 'waveform'
  }
}

onMounted(() => {
  if (internalMode.value === 'waveform') {
    drawFullWaveform()
  } else if (internalMode.value === 'signal') {
    if (props.isActive) {
      startSignalLoop()
    } else {
      drawStaticCenterLine()
    }
  }
})

onUnmounted(() => {
  stopSignalLoop()
})

// Watch for mode changes
watch(() => internalMode.value, (newMode) => {
  stopSignalLoop()
  
  // Hide tooltip when leaving waveform mode
  if (newMode !== 'waveform') {
    showTimeTooltip.value = false
    isDragging.value = false
    tooltipTime.value = null
  }
  
  if (newMode === 'waveform') {
    drawFullWaveform()
  } else if (newMode === 'signal') {
    if (props.isActive) {
      startSignalLoop()
    } else {
      drawStaticCenterLine()
    }
  }
})

// Watch for isActive changes in signal mode
watch(() => props.isActive, (isActive) => {
  if (internalMode.value === 'signal') {
    if (isActive) {
      startSignalLoop()
    } else {
      stopSignalLoop()
      drawStaticCenterLine()
    }
  }
})

watch(() => props.audioBuffer, () => {
  if (internalMode.value === 'waveform') {
    drawFullWaveform()
  }
})

watch(() => props.currentTime, () => {
  if (internalMode.value === 'waveform' && props.audioBuffer) {
    drawFullWaveform()
  }
})

watch(() => props.isPlaying, () => {
  if (internalMode.value === 'waveform' && props.audioBuffer) {
    drawFullWaveform()
  }
})

// Real-time signal oscilloscope (using streamed data from Rust backend)
function drawSignal() {
  if (!canvasRef.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()

  // Set canvas size accounting for device pixel ratio
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = rect.height

  // Clear canvas
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)

  // Get waveform data from stream (updated automatically by audio engine)
  const values = audioEngine?.state.value.trackWaveforms.get(props.trackNumber)
  
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
  canvas.height = rect.height * dpr  // Use actual rect height
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = rect.height  // Use actual rect height

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
  
  // Show entire track (we have decimated data, not full resolution)
  const currentTime = props.currentTime || 0
  
  const step = Math.max(1, Math.ceil(channelData.length / width))
  const amp = height / 2

  ctx.strokeStyle = '#3b82f6' // blue-500
  ctx.lineWidth = 1

  for (let i = 0; i < width; i++) {
    let min = 1.0
    let max = -1.0

    const startSample = Math.floor((i / width) * channelData.length)
    const endSample = Math.floor(((i + 1) / width) * channelData.length)

    for (let sampleIndex = startSample; sampleIndex < endSample && sampleIndex < channelData.length; sampleIndex++) {
      const datum = channelData[sampleIndex]
      if (datum < min) min = datum
      if (datum > max) max = datum
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

  // Draw playback position (red line) - visible only when playing or after seek
  if (props.isPlaying || currentTime > 0) {
    const progress = currentTime / duration
    const x = progress * width
    
    // Draw playback cursor (red line) - shorter with margins
    if (x >= 0 && x <= width) {
      const marginTop = height * 0.15 // 15% margin from top
      const marginBottom = height * 0.15 // 15% margin from bottom
      
      ctx.strokeStyle = '#ef4444' // red-500
      ctx.lineWidth = 2 // More visible
      ctx.beginPath()
      ctx.moveTo(x, marginTop)
      ctx.lineTo(x, height - marginBottom)
      ctx.stroke()
      
      // Add glow effect
      ctx.shadowColor = '#ef4444'
      ctx.shadowBlur = 4
      ctx.lineWidth = 1
      ctx.beginPath()
      ctx.moveTo(x, marginTop)
      ctx.lineTo(x, height - marginBottom)
      ctx.stroke()
      ctx.shadowBlur = 0
    }
  }
}

function drawCenterLine(ctx: CanvasRenderingContext2D, width: number, height: number) {
  ctx.strokeStyle = 'rgba(100, 116, 139, 0.8)' // gray-500 with higher opacity for visibility
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(0, height / 2)
  ctx.lineTo(width, height / 2)
  ctx.stroke()
}

// Start continuous rendering loop for signal mode (reads from stream)
function startSignalLoop() {
  if (isDrawingLoop) return
  
  isDrawingLoop = true
  
  const loop = () => {
    if (!isDrawingLoop) return
    
    drawSignal()
    animationId = requestAnimationFrame(loop)
  }
  
  loop()
}

// Stop rendering loop
function stopSignalLoop() {
  isDrawingLoop = false
  if (animationId !== null) {
    cancelAnimationFrame(animationId)
    animationId = null
  }
}

// Draw static center line when inactive
function drawStaticCenterLine() {
  if (!canvasRef.value) return
  
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  
  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  
  canvas.width = rect.width * dpr
  canvas.height = rect.height * dpr
  ctx.scale(dpr, dpr)
  
  const width = rect.width
  const height = rect.height
  
  // Clear canvas with black
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)
  
  // Draw center line
  drawCenterLine(ctx, width, height)
}

// Expose methods for external control
defineExpose({
  redraw: () => {
    if (internalMode.value === 'waveform') {
      drawFullWaveform()
    } else if (internalMode.value === 'signal') {
      drawSignal()
    }
  }
})
</script>
