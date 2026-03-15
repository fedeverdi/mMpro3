<template>
  <!-- Compressor Button -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between gap-1',
    enabled ? 'bg-orange-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <div class="flex items-center gap-1">
      <div class="drag-handle cursor-move text-gray-400 hover:text-gray-200" @click.stop>
        <svg class="w-2 h-2" fill="currentColor" viewBox="0 0 24 24">
          <path d="M9 3h2v2H9V3zm0 4h2v2H9V7zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm0 4h2v2H9v-2zM13 3h2v2h-2V3zm0 4h2v2h-2V7zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2z"/>
        </svg>
      </div>
      <span>CO</span>
    </div>
    <div class="flex items-center gap-0.5">
      <button :disabled="!enabled" @click.stop="showModal = true"
        class="p-0.5 rounded hover:bg-orange-700 disabled:opacity-50">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>
      <button @click.stop="$emit('remove')" class="p-0.5 rounded hover:bg-red-600 transition-all" title="Remove">
        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Compressor Modal with Canvas -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[1000]"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-orange-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold text-orange-300">Track {{ trackNumber }} - Insert #{{ insertId }} - Compressor</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>
        
        <!-- Canvas Visualization -->
        <div class="mb-6 bg-black rounded p-2">
          <canvas ref="curveCanvas" class="w-full" style="height: 300px;"></canvas>
        </div>

        <!-- Knobs -->
        <div class="flex flex-wrap gap-6 justify-center px-12">
          <div class="scale-[0.8]">
            <Knob v-model="threshold" :min="-60" :max="0" :step="0.5" label="Threshold" unit="dB" color="#f97316" />
          </div>
          <div class="scale-[0.8]">
            <Knob v-model="ratio" :min="1" :max="20" :step="0.1" label="Ratio" unit=":1" color="#f97316" />
          </div>
          <div class="scale-[0.8]">
            <Knob v-model="attack" :min="1" :max="100" :step="1" label="Attack" unit="ms" color="#10b981" />
          </div>
          <div class="scale-[0.8]">
            <Knob v-model="release" :min="10" :max="1000" :step="10" label="Release" unit="ms" color="#06b6d4" />
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, inject, watch, nextTick, onUnmounted } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  trackNumber: number
  insertId: number
  enabled: boolean
  trackLevelL: number
  trackLevelR: number
  phaseCorrelation: number
}>()

const emit = defineEmits<{
  toggle: []
  remove: []
}>()

const audioEngine = inject('audioEngine') as any
const showModal = ref(false)
const curveCanvas = ref<HTMLCanvasElement | null>(null)

// Compressor parameters
const threshold = ref(-20)
const ratio = ref(4)
const attack = ref(5)
const release = ref(50)

// Realtime level monitoring
let animationFrameId: number | null = null
const currentInputLevel = ref(-60)
const currentGainReduction = ref(0)

function handleToggle() {
  emit('toggle')
}

// Watch for parameter changes and send to backend
watch([threshold, ratio, attack, release], () => {
  if (audioEngine?.state.value.isRunning && props.enabled) {
    audioEngine.setTrackInsertCompressor(
      props.trackNumber - 1,
      props.insertId,
      threshold.value,
      ratio.value,
      attack.value / 1000,
      release.value / 1000
    )
  }
  
  if (showModal.value) {
    nextTick(() => {
      drawCompressionCurve()
    })
  }
})

// Start/stop monitoring based on modal visibility
watch(showModal, (isOpen) => {
  if (isOpen) {
    startMonitoring()
    nextTick(() => {
      drawCompressionCurve()
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

    // Use meter values from props
    const avgLevel = (props.trackLevelL + props.trackLevelR) / 2
    currentInputLevel.value = Math.max(-60, Math.min(0, avgLevel))

    // Calculate gain reduction
    const inputDb = currentInputLevel.value
    if (inputDb > threshold.value) {
      const excess = inputDb - threshold.value
      const reducedExcess = excess / ratio.value
      currentGainReduction.value = excess - reducedExcess
    } else {
      currentGainReduction.value = 0
    }

    drawCompressionCurve()
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

// Draw compression curve
function drawCompressionCurve() {
  if (!curveCanvas.value) return
  if (!showModal.value) return

  const canvas = curveCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const rect = canvas.getBoundingClientRect()
  const width = rect.width
  const height = rect.height
  const dpr = window.devicePixelRatio || 1

  canvas.width = width * dpr
  canvas.height = height * dpr
  ctx.scale(dpr, dpr)

  ctx.fillStyle = '#000000'
  ctx.fillRect(0, 0, width, height)

  const padding = 40
  const graphWidth = width - padding * 2
  const graphHeight = height - padding * 2

  const dbToX = (db: number) => padding + ((db + 60) / 60) * graphWidth
  const dbToY = (db: number) => height - padding - ((db + 60) / 60) * graphHeight

  // Draw grid
  ctx.strokeStyle = '#333333'
  ctx.lineWidth = 1
  for (let db = -60; db <= 0; db += 10) {
    ctx.beginPath()
    ctx.moveTo(dbToX(db), padding)
    ctx.lineTo(dbToX(db), height - padding)
    ctx.stroke()

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

  for (let db = -60; db <= 0; db += 20) {
    ctx.fillText(`${db}`, dbToX(db), height - padding + 15)
  }
  ctx.fillText('Input (dB)', width / 2, height - 5)

  ctx.textAlign = 'right'
  for (let db = -60; db <= 0; db += 20) {
    ctx.fillText(`${db}`, padding - 5, dbToY(db) + 3)
  }
  ctx.save()
  ctx.translate(12, height / 2)
  ctx.rotate(-Math.PI / 2)
  ctx.textAlign = 'center'
  ctx.fillText('Output (dB)', 0, 0)
  ctx.restore()

  // Draw 1:1 reference line
  ctx.strokeStyle = '#555555'
  ctx.lineWidth = 1
  ctx.setLineDash([5, 5])
  ctx.beginPath()
  ctx.moveTo(dbToX(-60), dbToY(-60))
  ctx.lineTo(dbToX(0), dbToY(0))
  ctx.stroke()
  ctx.setLineDash([])

  // Draw compression curve
  ctx.strokeStyle = '#f97316'
  ctx.lineWidth = 3
  ctx.beginPath()

  const knee = 6
  const thresholdValue = threshold.value
  const ratioValue = ratio.value

  for (let inputDb = -60; inputDb <= 0; inputDb += 0.5) {
    let outputDb: number

    if (inputDb < thresholdValue - knee / 2) {
      outputDb = inputDb
    } else if (inputDb > thresholdValue + knee / 2) {
      const excess = inputDb - thresholdValue
      outputDb = thresholdValue + excess / ratioValue
    } else {
      const kneeInput = inputDb - thresholdValue + knee / 2
      const kneeRatio = kneeInput / knee
      const excess = inputDb - thresholdValue
      outputDb = inputDb + ((excess / ratioValue) - excess) * kneeRatio
    }

    const x = dbToX(inputDb)
    const y = dbToY(outputDb)

    if (inputDb === -60) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }

  ctx.stroke()

  // Draw threshold line
  ctx.strokeStyle = '#ef4444'
  ctx.lineWidth = 2
  ctx.setLineDash([3, 3])
  ctx.beginPath()
  ctx.moveTo(dbToX(thresholdValue), padding)
  ctx.lineTo(dbToX(thresholdValue), height - padding)
  ctx.stroke()
  ctx.setLineDash([])

  // Threshold label
  ctx.fillStyle = '#ef4444'
  ctx.font = 'bold 11px sans-serif'
  ctx.textAlign = 'center'
  ctx.fillText(`Threshold: ${thresholdValue}dB`, dbToX(thresholdValue), padding - 10)

  // Ratio label
  ctx.fillStyle = '#f97316'
  ctx.textAlign = 'left'
  ctx.fillText(`Ratio: ${ratioValue.toFixed(1)}:1`, width - padding - 80, padding + 15)

  // Draw realtime signal indicator
  if (currentInputLevel.value > -60) {
    const inputDb = currentInputLevel.value

    // Calculate output level with compression
    let outputDb = inputDb
    if (inputDb < thresholdValue - knee / 2) {
      outputDb = inputDb
    } else if (inputDb > thresholdValue + knee / 2) {
      const excess = inputDb - thresholdValue
      outputDb = thresholdValue + excess / ratioValue
    } else {
      const kneeInput = inputDb - thresholdValue + knee / 2
      const kneeRatio = kneeInput / knee
      const excess = inputDb - thresholdValue
      outputDb = inputDb + ((excess / ratioValue) - excess) * kneeRatio
    }

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

    // Gain reduction display
    const gainReduction = inputDb - outputDb
    if (gainReduction > 0.1) {
      ctx.fillStyle = '#fbbf24'
      ctx.font = 'bold 12px sans-serif'
      ctx.textAlign = 'right'
      ctx.fillText(`GR: -${gainReduction.toFixed(1)}dB`, width - padding - 10, padding + 35)
    }
  }
}
</script>
