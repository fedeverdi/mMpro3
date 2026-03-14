<template>
  <div class="px-1 py-2">
    <canvas ref="canvasRef" class="w-full h-[50px] rounded border border-gray-700"
      style="image-rendering: crisp-edges;"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed, inject, type Ref } from 'vue'
import { PeakingFilter } from '~/lib/filters/peaking.class'
import { LowShelvingFilter } from '~/lib/filters/lowShelving.class'
import { HighShelvingFilter } from '~/lib/filters/highShelving.class'

interface Props {
  filters: any[]
  systemFilters?: any[]
  trackNumber: number
}

const props = defineProps<Props>()

const canvasRef = ref<HTMLCanvasElement | null>(null)

// Inject audio engine for FFT data
const audioEngine = inject<any>('audioEngine')

// FFT smoothing state
let smoothedFFTLeft: Float32Array | null = null
let smoothedFFTRight: Float32Array | null = null
let currentSampleRate = 48000 // Sample rate corrente (variabile)
const SMOOTHING_FACTOR = 0.7
const ATTACK_FACTOR = 0.3

// FFT rendering throttle (ridotto per vezzo grafico)
let fftRenderPending = false
let lastFFTRenderTime = 0
const FFT_RENDER_INTERVAL = 100 // ms - aggiorna solo ogni 150ms

// Combine system filters and user filters for display
const displayFilters = computed(() => {
  const system = props.systemFilters || []
  return [...system, ...props.filters]
})

// Filter calculators for curve computation
const peakingCalculator = new PeakingFilter()
const lowShelvingCalculator = new LowShelvingFilter()
const highShelvingCalculator = new HighShelvingFilter()

let drawCurveTimeout: ReturnType<typeof setTimeout> | null = null

// Use centralized resize trigger from parent
const resizeTrigger = inject<Ref<number>>('resizeTrigger', ref(0))

onMounted(() => {
  drawCurve()
  
  // Watch for centralized resize trigger
  watch(resizeTrigger, () => {
    // Throttle redraw to prevent blocking during window animations
    if (drawCurveTimeout) return
    drawCurveTimeout = setTimeout(() => {
      drawCurve()
      drawCurveTimeout = null
    }, 16) // ~60fps
  })
  
  // Cleanup on unmount
  onUnmounted(() => {
    if (drawCurveTimeout) {
      clearTimeout(drawCurveTimeout)
    }
  })
})

// Watch for filter changes
watch(() => props.filters, () => {
  drawCurve()
}, { deep: true })

// Watch for system filter changes
watch(() => props.systemFilters, () => {
  drawCurve()
}, { deep: true })

// Watch for FFT data updates - use array to ensure reactivity
watch(
  [
    () => audioEngine?.state?.value?.trackFFTData,
    () => props.trackNumber
  ],
  ([trackFFTData, trackNum]) => {
    if (!trackFFTData || !trackNum) return
    // trackNumber is 1-based in UI, but engine uses 0-based IDs
    const fftData = trackFFTData[trackNum - 1]
    if (!fftData) return
    
    const newLeft = fftData.binsLeft instanceof Float32Array 
      ? fftData.binsLeft 
      : new Float32Array(fftData.binsLeft)
    const newRight = fftData.binsRight instanceof Float32Array 
      ? fftData.binsRight 
      : new Float32Array(fftData.binsRight)
    
    // Salva il sample rate corrente
    if (fftData.sampleRate) {
      currentSampleRate = fftData.sampleRate
    }
    
    if (!smoothedFFTLeft || smoothedFFTLeft.length !== newLeft.length) {
      smoothedFFTLeft = new Float32Array(newLeft)
      smoothedFFTRight = new Float32Array(newRight)
    } else {
      for (let i = 0; i < newLeft.length; i++) {
        if (newLeft[i] > smoothedFFTLeft![i]) {
          smoothedFFTLeft![i] = smoothedFFTLeft![i] * ATTACK_FACTOR + newLeft[i] * (1 - ATTACK_FACTOR)
        } else {
          smoothedFFTLeft![i] = Math.max(newLeft[i], smoothedFFTLeft![i] * SMOOTHING_FACTOR)
        }
        
        if (newRight[i] > smoothedFFTRight![i]) {
          smoothedFFTRight![i] = smoothedFFTRight![i] * ATTACK_FACTOR + newRight[i] * (1 - ATTACK_FACTOR)
        } else {
          smoothedFFTRight![i] = Math.max(newRight[i], smoothedFFTRight![i] * SMOOTHING_FACTOR)
        }
      }
    }
    
    // Throttle FFT rendering to reduce CPU load (solo un vezzo grafico)
    const now = performance.now()
    if (!fftRenderPending && (now - lastFFTRenderTime) >= FFT_RENDER_INTERVAL) {
      fftRenderPending = true
      lastFFTRenderTime = now
      requestAnimationFrame(() => {
        fftRenderPending = false
        drawCurve()
      })
    }
  },
  { deep: true, immediate: true }
)

function drawCurve() {
  if (!canvasRef.value) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()

  canvas.width = rect.width * dpr
  canvas.height = 40 * dpr
  ctx.scale(dpr, dpr)

  const width = rect.width
  const height = 40

  // Clear canvas
  ctx.fillStyle = '#111827' // bg-gray-900
  ctx.fillRect(0, 0, width, height)

  // Draw grid
  ctx.strokeStyle = 'rgba(75, 85, 99, 0.3)' // gray-600 with opacity
  ctx.lineWidth = 0.5

  // Horizontal lines (0dB center line more visible)
  for (let i = 0; i <= 2; i++) {
    const y = (height / 2) * i
    ctx.globalAlpha = i === 1 ? 0.5 : 0.3
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(width, y)
    ctx.stroke()
  }
  ctx.globalAlpha = 1

  // Draw FFT bars first (as background, before EQ curve)
  if (smoothedFFTLeft && smoothedFFTRight && smoothedFFTLeft.length > 0) {
    const numBars = 40 // More bars for better resolution
    const minFreqLog = Math.log10(20)
    const maxFreqLog = Math.log10(20000)
    const barWidth = width / numBars
    
    // Convert magnitude to dB with -25dB offset to reduce amplitude
    const convertToDb = (mag: number) => {
      const db = 20 * Math.log10(Math.max(mag, 1e-10))// -25dB offset
      return Math.max(-80, Math.min(0, db))
    }
    
    // Usa il sample rate corrente invece di assumere 48kHz
    const nyquist = currentSampleRate / 2
    
    for (let i = 0; i < numBars; i++) {
      const x = i * barWidth
      
      // Calculate frequency for this bar (logarithmic)
      const logFreq = minFreqLog + (i / numBars) * (maxFreqLog - minFreqLog)
      const freq = Math.pow(10, logFreq)
      
      // Map frequency to FFT bin usando il sample rate corrente
      const binIndex = Math.floor((freq / nyquist) * smoothedFFTLeft.length)
      const clampedIndex = Math.max(0, Math.min(smoothedFFTLeft.length - 1, binIndex))
      
      // Get average of left and right channels
      const magLeft = smoothedFFTLeft[clampedIndex]
      const magRight = smoothedFFTRight[clampedIndex]
      const dbLeft = convertToDb(magLeft)
      const dbRight = convertToDb(magRight)
      const dbAvg = (dbLeft + dbRight) / 2
      
      // Map dB (-80 to 0) to bar height con riduzione ampiezza verticale
      // Apply a power curve to make quiet signals more visible
      const normalizedDb = Math.pow((dbAvg + 80) / 80, 0.5) // Square root for better visibility
      const barHeight = normalizedDb * height * 0.5 // Ridotta ampiezza verticale al 50%
      
      // Draw bar from bottom with semi-transparent blue (ridotta opacità per vezzo grafico)
      if (barHeight > 0.5) {
        ctx.fillStyle = 'rgba(59, 130, 246, 0.25)' // Blue-500 with lower transparency
        ctx.fillRect(x, height - barHeight, barWidth - 1, barHeight)
      }
    }
  }

  // If no filters, just show flat line
  if (!displayFilters.value || displayFilters.value.length === 0) {
    ctx.strokeStyle = '#6B7280' // gray-500
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()
    return
  }

  // Calculate convolution curve
  const points = 200
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)

  ctx.strokeStyle = '#FFFFFF'
  ctx.lineWidth = 1.5
  ctx.beginPath()

  for (let i = 0; i < points; i++) {
    const x = (i / points) * width
    const logFreq = minFreq + (i / points) * (maxFreq - minFreq)
    const freq = Math.pow(10, logFreq)

    // Calculate total gain at this frequency
    let totalGain = 0

    displayFilters.value.forEach(filter => {
      let gain = 0

      if (filter.type === 'peaking') {
        gain = peakingCalculator.computeResponseAtFrequency(
          freq,
          filter.frequency,
          filter.gain,
          filter.Q
        )
      } else if (filter.type === 'lowshelf') {
        gain = lowShelvingCalculator.computeResponseAtFrequency(
          freq,
          filter.frequency,
          filter.gain,
          filter.Q
        )
      } else if (filter.type === 'highshelf') {
        gain = highShelvingCalculator.computeResponseAtFrequency(
          freq,
          filter.frequency,
          filter.gain,
          filter.Q
        )
      } else if (filter.type === 'highpass') {
        // Highpass filter: second order biquad
        const w = freq / filter.frequency // normalized frequency
        const w2 = w * w
        const w4 = w2 * w2
        const numerator = w4
        const denominator = Math.pow(w2 - 1, 2) + (w2 / (filter.Q * filter.Q))
        const magnitudeSquared = numerator / denominator
        gain = 10 * Math.log10(Math.max(magnitudeSquared, 1e-10))
      } else if (filter.type === 'lowpass') {
        // Lowpass filter: second order biquad
        const w = freq / filter.frequency // normalized frequency
        const w2 = w * w
        const numerator = 1
        const denominator = Math.pow(w2 - 1, 2) + (w2 / (filter.Q * filter.Q))
        const magnitudeSquared = numerator / denominator
        gain = 10 * Math.log10(Math.max(magnitudeSquared, 1e-10))
      }

      totalGain += gain
    })

    // Map gain to y position (scale: -24dB to +24dB)
    const y = height / 2 - (totalGain * (height / 48))

    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }

  ctx.stroke()
}

// Expose method for external redraw trigger
defineExpose({
  redraw: drawCurve
})
</script>
