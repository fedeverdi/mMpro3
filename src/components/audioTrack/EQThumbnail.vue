<template>
  <div class="px-1 py-2">
    <canvas ref="canvasRef" class="w-full h-[50px] rounded border border-gray-700"
      style="image-rendering: crisp-edges;"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { PeakingFilter } from '~/lib/filters/peaking.class'
import { LowShelvingFilter } from '~/lib/filters/lowShelving.class'
import { HighShelvingFilter } from '~/lib/filters/highShelving.class'

interface Props {
  filters: any[]
  systemFilters?: any[]
}

const props = defineProps<Props>()

const canvasRef = ref<HTMLCanvasElement | null>(null)

// Combine system filters and user filters for display
const displayFilters = computed(() => {
  const system = props.systemFilters || []
  return [...system, ...props.filters]
})

// Filter calculators for curve computation
const peakingCalculator = new PeakingFilter()
const lowShelvingCalculator = new LowShelvingFilter()
const highShelvingCalculator = new HighShelvingFilter()

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  drawCurve()
  
  // Watch for canvas size changes
  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => {
      drawCurve()
    })
    resizeObserver.observe(canvasRef.value)
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
})

// Watch for filter changes
watch(() => props.filters, () => {
  drawCurve()
}, { deep: true })

// Watch for system filter changes
watch(() => props.systemFilters, () => {
  drawCurve()
}, { deep: true })

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
