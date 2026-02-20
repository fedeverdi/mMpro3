<template>
  <!-- Limiter Toggle -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between',
    enabled ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <span>LI</span>
    <button :disabled="!enabled" @click.stop="showModal = true"
      class="p-0.5 rounded hover:bg-green-700">
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
  </div>

  <!-- Limiter Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-xl font-bold text-green-300">Track {{ trackNumber }} - Limiter</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
        </div>
        <!-- Limiting Curve Display -->
        <div class="mb-6 bg-black/50 rounded-lg p-4 border border-green-600/30">
          <p class="text-xs text-green-300 font-bold mb-2 text-center">LIMITING CURVE</p>
          <canvas ref="curveCanvas" class="w-full rounded" style="height: 300px;"></canvas>
        </div>

        <div class="flex flex-wrap gap-4 justify-center">
          <Knob v-model="threshold" :min="-30" :max="0" :step="0.5" label="Threshold" unit="dB" color="#10b981" />
          <Knob v-model="attack" :min="0" :max="0.05" :step="0.001" label="Attack" unit="s" color="#f59e0b" />
          <Knob v-model="release" :min="0" :max="1" :step="0.01" label="Release" unit="s" color="#06b6d4" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import Knob from '../core/Knob.vue'

interface Props {
  trackNumber: number
  enabled: boolean
  limiterNode?: any
  meter?: any
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'toggle'): void
}>()

const showModal = ref(false)
const curveCanvas = ref<HTMLCanvasElement | null>(null)

// Internal state for controls (limiter has very high ratio, typically 20:1 or higher)
const threshold = ref(-3)
const attack = ref(0.003)
const release = ref(0.1)

function handleToggle() {
  emit('toggle')
}

function updateLimiterNode() {
  if (!props.limiterNode) return
  
  // Limiter is essentially a compressor with very high ratio
  props.limiterNode.threshold.value = threshold.value
  props.limiterNode.ratio.value = 20 // Fixed high ratio for limiting
  props.limiterNode.attack.value = attack.value
  props.limiterNode.release.value = release.value
  
  // Redraw curve if modal is open
  if (showModal.value) {
    nextTick(() => drawLimitingCurve())
  }
}

// Watch for parameter changes
watch([threshold, attack, release], () => {
  updateLimiterNode()
})

// Expose methods for snapshot save/restore
defineExpose({
  getParams: () => ({
    threshold: threshold.value,
    attack: attack.value,
    release: release.value
  }),
  setParams: (params: { threshold: number, attack: number, release: number }) => {
    threshold.value = params.threshold
    attack.value = params.attack
    release.value = params.release
    updateLimiterNode()
  },
  resetToDefaults: () => {
    threshold.value = -3
    attack.value = 0.003
    release.value = 0.1
    updateLimiterNode()
  },
  getSnapshot: () => ({
    threshold: threshold.value,
    attack: attack.value,
    release: release.value
  }),
  restoreSnapshot: (snapshot: any) => {
    if (snapshot.threshold !== undefined) threshold.value = snapshot.threshold
    if (snapshot.attack !== undefined) attack.value = snapshot.attack
    if (snapshot.release !== undefined) release.value = snapshot.release
    updateLimiterNode()
  }
})

// Track limiter visualization
watch(showModal, (isOpen) => {
  if (isOpen) {
    startLimiterMonitoring()
    nextTick(() => drawLimitingCurve())
  } else {
    stopLimiterMonitoring()
  }
})

let limiterAnimationId: number | null = null

function startLimiterMonitoring() {
  if (limiterAnimationId) return

  function updateTrackLevels() {
    if (!showModal.value || !props.meter) {
      limiterAnimationId = null
      return
    }

    drawLimitingCurve()
    limiterAnimationId = requestAnimationFrame(updateTrackLevels)
  }

  updateTrackLevels()
}

function stopLimiterMonitoring() {
  if (limiterAnimationId) {
    cancelAnimationFrame(limiterAnimationId)
    limiterAnimationId = null
  }
}

function drawLimitingCurve() {
  if (!curveCanvas.value) return

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

  // Draw 1:1 line
  ctx.strokeStyle = '#666666'
  ctx.lineWidth = 2
  ctx.setLineDash([5, 5])
  ctx.beginPath()
  ctx.moveTo(dbToX(-60), dbToY(-60))
  ctx.lineTo(dbToX(0), dbToY(0))
  ctx.stroke()
  ctx.setLineDash([])

  // Draw limiting curve (ratio = 20:1)
  const ratio = 20
  ctx.strokeStyle = '#10b981'
  ctx.lineWidth = 3
  ctx.beginPath()
  
  for (let inputDb = -60; inputDb <= 0; inputDb += 0.5) {
    let outputDb = inputDb
    
    if (inputDb > threshold.value) {
      const excess = inputDb - threshold.value
      const reducedExcess = excess / ratio
      outputDb = threshold.value + reducedExcess
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

  // Draw threshold line (hard limit)
  ctx.strokeStyle = '#ef4444'
  ctx.lineWidth = 2
  ctx.setLineDash([3, 3])
  ctx.beginPath()
  ctx.moveTo(dbToX(threshold.value), padding)
  ctx.lineTo(dbToX(threshold.value), height - padding)
  ctx.stroke()
  ctx.beginPath()
  ctx.moveTo(padding, dbToY(threshold.value))
  ctx.lineTo(width - padding, dbToY(threshold.value))
  ctx.stroke()
  ctx.setLineDash([])

  // Draw current input level indicator
  if (props.meter) {
    let currentInputLevel = -60
    try {
      const meterValue = props.meter.getValue()
      currentInputLevel = Array.isArray(meterValue) 
        ? (meterValue[0] + meterValue[1]) / 2 
        : meterValue
      currentInputLevel = Math.max(-60, Math.min(0, currentInputLevel))
    } catch {}

    // Calculate output level based on limiting curve
    let currentOutputLevel = currentInputLevel
    if (currentInputLevel > threshold.value) {
      const excess = currentInputLevel - threshold.value
      const reducedExcess = excess / ratio
      currentOutputLevel = threshold.value + reducedExcess
    }

    // Draw the point on the limiting curve
    ctx.fillStyle = '#ef4444'
    ctx.beginPath()
    ctx.arc(dbToX(currentInputLevel), dbToY(currentOutputLevel), 5, 0, Math.PI * 2)
    ctx.fill()
    
    // Add glow effect
    ctx.shadowBlur = 10
    ctx.shadowColor = '#ef4444'
    ctx.beginPath()
    ctx.arc(dbToX(currentInputLevel), dbToY(currentOutputLevel), 5, 0, Math.PI * 2)
    ctx.fill()
    ctx.shadowBlur = 0
  }

  // Labels
  ctx.fillStyle = '#ffffff'
  ctx.font = '12px monospace'
  ctx.fillText('Input (dB)', width / 2 - 40, height - 10)
  
  ctx.save()
  ctx.translate(15, height / 2 + 50)
  ctx.rotate(-Math.PI / 2)
  ctx.fillText('Output (dB)', 0, 0)
  ctx.restore()
}
</script>
