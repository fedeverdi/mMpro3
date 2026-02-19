<template>
  <div class="limiter-effect bg-gray-800/50 justify-center h-full rounded-lg p-3 border border-gray-700 flex flex-col items-center gap-2">
    <span class="text-xs font-bold text-green-300">LIMITER</span>

    <button @click="toggleEffect" :class="[
      'px-4 py-1 text-[0.6rem] font-bold rounded transition-colors w-full',
      isEnabled
        ? 'bg-green-600 text-white'
        : 'bg-gray-700 text-gray-400'
    ]">
      {{ isEnabled ? 'ON' : 'OFF' }}
    </button>

    <button @click="showModal = true"
      class="px-4 py-1 text-[0.6rem] font-bold rounded bg-blue-600 hover:bg-blue-700 text-white w-full">
      <div class="flex items-center gap-1 justify-center text-[0.6rem]">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="white" class="h-3.5 w-3.5">
          <path
            d="M487.4 315.7l-42.6-24.6c4.3-23.2 4.3-47 0-70.2l42.6-24.6c4.9-2.8 7.1-8.6 5.5-14-11.1-35.6-30-67.8-54.7-94.6-3.8-4.1-10-5.1-14.8-2.3L380.8 110c-17.9-15.4-38.5-27.3-60.8-35.1V25.8c0-5.6-3.9-10.5-9.4-11.7-36.7-8.2-74.3-7.8-109.2 0-5.5 1.2-9.4 6.1-9.4 11.7V75c-22.2 7.9-42.8 19.8-60.8 35.1L88.7 85.5c-4.9-2.8-11-1.9-14.8 2.3-24.7 26.7-43.6 58.9-54.7 94.6-1.7 5.4.6 11.2 5.5 14L67.3 221c-4.3 23.2-4.3 47 0 70.2l-42.6 24.6c-4.9 2.8-7.1 8.6-5.5 14 11.1 35.6 30 67.8 54.7 94.6 3.8 4.1 10 5.1 14.8 2.3l42.6-24.6c17.9 15.4 38.5 27.3 60.8 35.1v49.2c0 5.6 3.9 10.5 9.4 11.7 36.7 8.2 74.3 7.8 109.2 0 5.5-1.2 9.4-6.1 9.4-11.7v-49.2c22.2-7.9 42.8-19.8 60.8-35.1l42.6 24.6c4.9 2.8 11 1.9 14.8-2.3 24.7-26.7 43.6-58.9 54.7-94.6 1.5-5.5-.7-11.3-5.6-14.1zM256 336c-44.1 0-80-35.9-80-80s35.9-80 80-80 80 35.9 80 80-35.9 80-80 80z" />
        </svg>
        <div>
          SETTINGS
        </div>
      </div>
    </button>

    <!-- Settings Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
        @click="showModal = false">
        <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-2xl w-full mx-4" @click.stop>
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-xl font-bold text-green-300">Limiter Settings</h3>
            <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
          </div>

          <!-- Limiter Curve Display -->
          <div class="mb-6 bg-black/50 rounded-lg p-4 border border-green-600/30">
            <p class="text-xs text-green-300 font-bold mb-2 text-center">LIMITER CURVE</p>
            <canvas ref="curveCanvas" class="w-full rounded" style="height: 300px;"></canvas>
          </div>

          <div class="flex justify-center">
            <Knob v-model="threshold" :min="-50" :max="3" :step="0.1" label="Threshold" unit="dB" color="#10b981" />
          </div>

        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, inject } from 'vue'
import Knob from './Knob.vue'

const props = defineProps<{
  enabled?: boolean
  initialThreshold?: number
  effectNode?: any
  masterSection?: any // Cambiato: passa l'intero masterSection invece di analysisNode
}>()

const emit = defineEmits<{
  (e: 'toggle', enabled: boolean): void
  (e: 'update', params: { threshold: number }): void
}>()

const isEnabled = ref(props.enabled ?? false)
const showModal = ref(false)
const curveCanvas = ref<HTMLCanvasElement | null>(null)

const threshold = ref(props.initialThreshold ?? -1)

// Realtime level monitoring
const ToneRef = inject<any>('Tone')
let Tone: any = null
let animationFrameId: number | null = null
const currentInputLevel = ref(-60)
const currentGainReduction = ref(0)

function toggleEffect() {
  isEnabled.value = !isEnabled.value
  console.log('[LimiterEffect] Toggle to:', isEnabled.value)
  emit('toggle', isEnabled.value)
}

// Start/stop monitoring based on modal visibility
watch(showModal, async (isOpen) => {
  if (isOpen) {
    if (!Tone) {
      // Get Tone.js from inject
      if (ToneRef?.value) {
        Tone = ToneRef.value
      } else {
        // Fallback: wait for it
        const checkTone = setInterval(() => {
          if (ToneRef?.value) {
            Tone = ToneRef.value
            clearInterval(checkTone)
          }
        }, 100)
      }
    }
    startMonitoring()
    nextTick(() => {
      drawLimiterCurve()
    })
  } else {
    stopMonitoring()
  }
})

function startMonitoring() {
  if (animationFrameId) return

  function updateLevels() {
    if (!showModal.value) {
      animationFrameId = null
      return
    }

    // Get input level BEFORE limiter (pre-limiter meters)
    if (props.masterSection?.getPreLimiterValues) {
      try {
        const meters = props.masterSection.getPreLimiterValues()
        // Use average of left and right
        const avgLevel = (meters.left + meters.right) / 2
        currentInputLevel.value = Math.max(-60, Math.min(0, avgLevel))
      } catch (error) {
        currentInputLevel.value = -60
      }
    }

    // Calculate gain reduction
    const inputDb = currentInputLevel.value
    if (inputDb > threshold.value) {
      currentGainReduction.value = inputDb - threshold.value
    } else {
      currentGainReduction.value = 0
    }

    drawLimiterCurve()
    animationFrameId = requestAnimationFrame(updateLevels)
  }

  updateLevels()
}

function stopMonitoring() {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId)
    animationFrameId = null
  }
}

onUnmounted(() => {
  stopMonitoring()
})

// Draw limiter curve
function drawLimiterCurve() {
  if (!curveCanvas.value) return
  if (!showModal.value) return // Don't draw if modal is closed

  const canvas = curveCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // Get actual display size
  const rect = canvas.getBoundingClientRect()
  const width = rect.width
  const height = rect.height
  const dpr = window.devicePixelRatio || 1

  // Scale for high DPI
  canvas.width = width * dpr
  canvas.height = height * dpr
  ctx.scale(dpr, dpr)

  // Clear
  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)

  const padding = 40
  const graphWidth = width - padding * 2
  const graphHeight = height - padding * 2

  // Helper to map dB to canvas coordinates (range: -60 to +3 dB)
  const minDb = -60
  const maxDb = 3
  const dbRange = maxDb - minDb
  const dbToX = (db: number) => padding + ((db - minDb) / dbRange) * graphWidth
  const dbToY = (db: number) => height - padding - ((db - minDb) / dbRange) * graphHeight

  // Draw grid
  ctx.strokeStyle = '#333333'
  ctx.lineWidth = 1
  for (let db = -60; db <= 3; db += 10) {
    // Vertical lines
    ctx.beginPath()
    ctx.moveTo(dbToX(db), padding)
    ctx.lineTo(dbToX(db), height - padding)
    ctx.stroke()

    // Horizontal lines
    ctx.beginPath()
    ctx.moveTo(padding, dbToY(db))
    ctx.lineTo(width - padding, dbToY(db))
    ctx.stroke()
  }

  // Draw axes
  ctx.strokeStyle = '#666666'
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(padding, padding)
  ctx.lineTo(padding, height - padding)
  ctx.lineTo(width - padding, height - padding)
  ctx.stroke()

  // Axis labels
  ctx.fillStyle = '#999999'
  ctx.font = '10px monospace'
  ctx.textAlign = 'center'

  // X-axis labels (Input)
  for (let db = -60; db <= 0; db += 20) {
    ctx.fillText(`${db}`, dbToX(db), height - padding + 15)
  }
  ctx.fillText('Input (dB)', width / 2, height - 5)

  // Y-axis labels (Output)
  ctx.textAlign = 'right'
  for (let db = -60; db <= 0; db += 20) {
    ctx.fillText(`${db}`, padding - 5, dbToY(db) + 3)
  }
  // Add +3dB label
  ctx.fillText('+3', padding - 5, dbToY(3) + 3)
  ctx.save()
  ctx.translate(12, height / 2)
  ctx.rotate(-Math.PI / 2)
  ctx.textAlign = 'center'
  ctx.fillText('Output (dB)', 0, 0)
  ctx.restore()

  // Draw 1:1 reference line (no limiting)
  ctx.strokeStyle = '#555555'
  ctx.lineWidth = 1
  ctx.setLineDash([5, 5])
  ctx.beginPath()
  ctx.moveTo(dbToX(-60), dbToY(-60))
  ctx.lineTo(dbToX(3), dbToY(3))
  ctx.stroke()
  ctx.setLineDash([])

  // Draw limiter curve
  ctx.strokeStyle = '#10b981'
  ctx.lineWidth = 3
  ctx.beginPath()

  const thresholdValue = threshold.value

  // Limiter curve: 1:1 until threshold, then flat ceiling
  ctx.moveTo(dbToX(-60), dbToY(-60))
  ctx.lineTo(dbToX(thresholdValue), dbToY(thresholdValue))
  ctx.lineTo(dbToX(3), dbToY(thresholdValue)) // Flat ceiling extended to +3dB

  ctx.stroke()

  // Draw threshold line (vertical)
  ctx.strokeStyle = '#ef4444'
  ctx.lineWidth = 2
  ctx.setLineDash([3, 3])
  ctx.beginPath()
  ctx.moveTo(dbToX(thresholdValue), padding)
  ctx.lineTo(dbToX(thresholdValue), height - padding)
  ctx.stroke()

  // Draw threshold line (horizontal)
  ctx.beginPath()
  ctx.moveTo(padding, dbToY(thresholdValue))
  ctx.lineTo(width - padding, dbToY(thresholdValue))
  ctx.stroke()
  ctx.setLineDash([])

  // Threshold label
  ctx.fillStyle = '#ef4444'
  ctx.font = 'bold 11px sans-serif'
  ctx.textAlign = 'center'
  ctx.fillText(`Threshold: ${thresholdValue}dB`, width / 2, padding - 10)

  // Info label
  ctx.fillStyle = '#10b981'
  ctx.textAlign = 'left'
  ctx.fillText('Hard Ceiling', width - padding - 90, padding + 15)

  // Draw realtime signal indicator
  if (currentInputLevel.value > -60) {
    const inputDb = currentInputLevel.value

    // Calculate output level (hard ceiling at threshold only if enabled)
    const outputDb = isEnabled.value ? Math.min(inputDb, thresholdValue) : inputDb

    // Draw input level marker (vertical line)
    ctx.strokeStyle = '#fbbf24'
    ctx.lineWidth = 2
    ctx.setLineDash([2, 2])
    ctx.beginPath()
    ctx.moveTo(dbToX(inputDb), padding)
    ctx.lineTo(dbToX(inputDb), height - padding)
    ctx.stroke()
    ctx.setLineDash([])

    // Draw signal point on curve
    ctx.fillStyle = '#fbbf24'
    ctx.beginPath()
    ctx.arc(dbToX(inputDb), dbToY(outputDb), 5, 0, 2 * Math.PI)
    ctx.fill()

    // Add glow
    ctx.shadowBlur = 15
    ctx.shadowColor = '#fbbf24'
    ctx.beginPath()
    ctx.arc(dbToX(inputDb), dbToY(outputDb), 5, 0, 2 * Math.PI)
    ctx.fill()
    ctx.shadowBlur = 0

    // Gain reduction display (only show if effect is enabled and working)
    if (isEnabled.value && currentGainReduction.value > 0.1) {
      ctx.fillStyle = '#fbbf24'
      ctx.font = 'bold 12px sans-serif'
      ctx.textAlign = 'right'
      ctx.fillText(`GR: -${currentGainReduction.value.toFixed(1)}dB`, width - padding - 10, padding + 35)
    }
  }
}


watch(threshold, () => {
  // Clamp threshold to valid range
  const clampedThreshold = Math.max(-50, Math.min(3, threshold.value))
  if (clampedThreshold !== threshold.value) {
    threshold.value = clampedThreshold
    return
  }

  emit('update', {
    threshold: threshold.value
  })

  // Redraw curve when threshold changes (only if modal is open)
  if (showModal.value) {
    nextTick(() => {
      drawLimiterCurve()
    })
  }
})

watch(showModal, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      drawLimiterCurve()
    })
  }
})
</script>

<style scoped>
/* No additional styles needed - Knob component handles its own styling */
</style>
