<template>
  <div class="master-eq-track bg-gradient-to-b from-gray-900 to-gray-950 rounded-lg border-2 border-blue-500/70 p-3 flex flex-col gap-3 h-full">
    <div>
      <p class="text-sm font-bold text-blue-300 tracking-wide uppercase">Master Parametric EQ</p>
      <p class="text-[10px] text-gray-400">Global tone shaping</p>
    </div>

    <div class="relative w-full flex-1 min-h-0 rounded-lg border border-gray-800 bg-black/50 overflow-hidden shadow-inner">
      <canvas
        ref="masterEqCanvas"
        class="absolute inset-0 w-full h-full"
        style="image-rendering: crisp-edges;"
      ></canvas>
    </div>

    <button
      @click="showMasterEQ = true"
      class="flex items-center justify-center gap-2 text-xs font-semibold px-3 py-2 rounded bg-blue-600 hover:bg-blue-500 text-white transition-colors"
    >
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="white" class="h-2.5 w-2.5"><path d="M487.4 315.7l-42.6-24.6c4.3-23.2 4.3-47 0-70.2l42.6-24.6c4.9-2.8 7.1-8.6 5.5-14-11.1-35.6-30-67.8-54.7-94.6-3.8-4.1-10-5.1-14.8-2.3L380.8 110c-17.9-15.4-38.5-27.3-60.8-35.1V25.8c0-5.6-3.9-10.5-9.4-11.7-36.7-8.2-74.3-7.8-109.2 0-5.5 1.2-9.4 6.1-9.4 11.7V75c-22.2 7.9-42.8 19.8-60.8 35.1L88.7 85.5c-4.9-2.8-11-1.9-14.8 2.3-24.7 26.7-43.6 58.9-54.7 94.6-1.7 5.4.6 11.2 5.5 14L67.3 221c-4.3 23.2-4.3 47 0 70.2l-42.6 24.6c-4.9 2.8-7.1 8.6-5.5 14 11.1 35.6 30 67.8 54.7 94.6 3.8 4.1 10 5.1 14.8 2.3l42.6-24.6c17.9 15.4 38.5 27.3 60.8 35.1v49.2c0 5.6 3.9 10.5 9.4 11.7 36.7 8.2 74.3 7.8 109.2 0 5.5-1.2 9.4-6.1 9.4-11.7v-49.2c22.2-7.9 42.8-19.8 60.8-35.1l42.6 24.6c4.9 2.8 11 1.9 14.8-2.3 24.7-26.7 43.6-58.9 54.7-94.6 1.5-5.5-.7-11.3-5.6-14.1zM256 336c-44.1 0-80-35.9-80-80s35.9-80 80-80 80 35.9 80 80-35.9 80-80 80z"/></svg>

      Open Master EQ
    </button>
  </div>

  <ParametricEQModal
    v-model="showMasterEQ"
    :trackNumber="0"
    :eq-filters="props.filtersData"
    title="Parametric EQ - Master Output"
    @update="handleMasterParametricEQUpdate"
  />
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, toRaw, inject } from 'vue'
import { PeakingFilter } from '~/lib/filters/peaking.class'
import { LowShelvingFilter } from '~/lib/filters/lowShelving.class'
import { HighShelvingFilter } from '~/lib/filters/highShelving.class'
import ParametricEQModal from './ParametricEQModal.vue'

interface Props {
  filtersData?: any[]
  masterChannel?: any
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:filtersData', value: any[]): void
}>()

// Inject Tone.js
const ToneRef = inject<any>('Tone')
let Tone: any = null

const showMasterEQ = ref(false)
const masterEqCanvas = ref<HTMLCanvasElement | null>(null)
const internalFiltersData = ref<any[]>([])

const peakingCalculator = new PeakingFilter()
const lowShelvingCalculator = new LowShelvingFilter()
const highShelvingCalculator = new HighShelvingFilter()

let rafId: number | null = null
let needsRedraw = false
let masterParametricEQFilters: any = null
let outputNode: any = null  // Output node for chaining to MasterSection

function syncFiltersData(newData?: any[]) {
  internalFiltersData.value = (newData || []).map(filter => ({ ...filter }))
  requestRedraw()
}

function requestRedraw() {
  if (needsRedraw) return
  needsRedraw = true
  
  if (rafId) cancelAnimationFrame(rafId)
  rafId = requestAnimationFrame(() => {
    drawMasterEQPreview()
    needsRedraw = false
    rafId = null
  })
}

watch(() => props.filtersData, (newVal) => {
  syncFiltersData(newVal)
}, { immediate: true })

// Watch for masterChannel to become available
watch(() => props.masterChannel, (newVal) => {
  if (newVal && Tone && !outputNode) {
    outputNode = new Tone.Gain(1)
    // Connect master to output
    const masterChan = toRaw(newVal)
    masterChan.connect(outputNode)
  }
}, { immediate: true })

onMounted(async () => {
  // Get Tone.js
  if (ToneRef?.value) {
    Tone = ToneRef.value
  }
  
  await nextTick()
  requestRedraw()
  window.addEventListener('resize', requestRedraw)
  
  // Create output node for chaining
  if (Tone && props.masterChannel && !outputNode) {
    outputNode = new Tone.Gain(1)
    // Initially connect master directly to output
    const masterChan = toRaw(props.masterChannel)
    masterChan.connect(outputNode)
  }
})

onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId)
  window.removeEventListener('resize', requestRedraw)
  
  // Cleanup audio nodes
  if (outputNode) {
    outputNode.dispose()
  }
})

// Handle parametric EQ update
function handleMasterParametricEQUpdate(filters: any) {
  if (!filters) return
  
  // Store the latest filter chain
  masterParametricEQFilters = filters
  
  // Store filter data for thumbnail AND update the preview
  if (filters.filtersData) {
    const rawFiltersData = toRaw(filters.filtersData)
    internalFiltersData.value = rawFiltersData.map((f: any) => ({
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      Q: f.Q,
      color: f.color
    }))
    
    // Emit to parent so masterEqFiltersData stays synchronized
    emit('update:filtersData', internalFiltersData.value)
    
    // Redraw the preview when filters change
    requestRedraw()
  }
  
  // Apply the parametric EQ to the audio chain
  applyMasterEQ()
}

// Insert or remove parametric EQ from the master chain
function applyMasterEQ() {
  if (!props.masterChannel || !outputNode) return
  
  const masterChan = toRaw(props.masterChannel)
  
  // Disconnect master channel
  try {
    masterChan.disconnect()
  } catch (e) {
    // Ignore disconnection errors
  }
  
  // Insert parametric EQ between master and output if present
  if (masterParametricEQFilters && masterParametricEQFilters.input && masterParametricEQFilters.output) {
    masterChan.connect(masterParametricEQFilters.input)
    
    // Disconnect old parametric output if needed
    try {
      masterParametricEQFilters.output.disconnect()
    } catch (e) { }
    
    masterParametricEQFilters.output.connect(outputNode)
  } else {
    // No parametric EQ: connect master directly to output
    masterChan.connect(outputNode)
  }
}

function drawMasterEQPreview() {
  if (!masterEqCanvas.value) return
  const canvas = masterEqCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const rect = canvas.getBoundingClientRect()
  const width = rect.width || 400
  const height = rect.height || 225

  canvas.width = width * dpr
  canvas.height = height * dpr
  ctx.setTransform(1, 0, 0, 1, 0, 0)
  ctx.scale(dpr, dpr)

  ctx.fillStyle = '#0f172a'
  ctx.fillRect(0, 0, width, height)

  ctx.strokeStyle = 'rgba(148, 163, 184, 0.25)'
  ctx.lineWidth = 1
  ctx.font = '7px monospace'
  ctx.fillStyle = 'rgba(148, 163, 184, 0.6)'
  
  for (let db = -24; db <= 24; db += 6) {
    const y = height / 2 - (db / 48) * height
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(width, y)
    ctx.stroke()
    
    // Etichetta gain (dB)
    if (db !== 0) {
      const label = db > 0 ? `+${db}dB` : `${db}dB`
      ctx.fillText(label, 4, y - 2)
    }
  }

  const freqs = [50, 200, 1000, 5000, 10000]
  freqs.forEach(freq => {
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const x = ((Math.log10(freq) - minFreq) / (maxFreq - minFreq)) * width
    ctx.beginPath()
    ctx.moveTo(x, 0)
    ctx.lineTo(x, height)
    ctx.stroke()
    
    // Etichetta frequenza (Hz/kHz)
    const label = freq >= 1000 ? `${freq / 1000}k` : `${freq}`
    ctx.fillText(label, x + 2, height - 4)
  })

  ctx.strokeStyle = '#1f2937'
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(0, height / 2)
  ctx.lineTo(width, height / 2)
  ctx.stroke()
  
  // Etichetta 0dB
  ctx.fillStyle = 'rgba(148, 163, 184, 0.8)'
  ctx.fillText('0dB', 4, height / 2 - 2)

  if (internalFiltersData.value.length === 0) {
    ctx.strokeStyle = '#93c5fd'
    ctx.lineWidth = 2
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    ctx.lineTo(width, height / 2)
    ctx.stroke()
    return
  }

  const points = 500
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)

  // Disegna i singoli filtri con i loro colori
  internalFiltersData.value.forEach((filter, index) => {
    const colors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#FFA07A', '#98D8C8', '#F7DC6F', '#BB8FCE', '#85C1E2', '#F8B739', '#52B788']
    const color = filter.color || colors[index % colors.length]
    
    // Disegna l'area riempita del filtro
    ctx.fillStyle = color
    ctx.globalAlpha = 0.15
    ctx.beginPath()
    ctx.moveTo(0, height / 2)
    
    for (let i = 0; i <= points; i++) {
      const x = (i / points) * width
      const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
      const gain = calculateFilterGain(filter, freq)
      const y = height / 2 - (gain * (height / 48))
      ctx.lineTo(x, y)
    }
    
    ctx.lineTo(width, height / 2)
    ctx.closePath()
    ctx.fill()
    
    // Disegna la curva del filtro
    ctx.strokeStyle = color
    ctx.globalAlpha = 0.7
    ctx.lineWidth = 2
    ctx.beginPath()

    for (let i = 0; i <= points; i++) {
      const x = (i / points) * width
      const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
      const gain = calculateFilterGain(filter, freq)
      const y = height / 2 - (gain * (height / 48))

      if (i === 0) {
        ctx.moveTo(x, y)
      } else {
        ctx.lineTo(x, y)
      }
    }
    ctx.stroke()
  })

  ctx.globalAlpha = 1
  ctx.strokeStyle = '#f8fafc'
  ctx.lineWidth = 1.5
  ctx.beginPath()

  for (let i = 0; i <= points; i++) {
    const x = (i / points) * width
    const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
    let totalGain = 0
    internalFiltersData.value.forEach(filter => {
      totalGain += calculateFilterGain(filter, freq)
    })
    const y = height / 2 - (totalGain * (height / 48))
    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }
  ctx.stroke()
}

function calculateFilterGain(filter: any, freq: number): number {
  if (filter.type === 'peaking') {
    return peakingCalculator.computeResponseAtFrequency(freq, filter.frequency, filter.gain, filter.Q)
  }
  if (filter.type === 'lowshelf') {
    return lowShelvingCalculator.computeResponseAtFrequency(freq, filter.frequency, filter.gain, filter.Q)
  }
  if (filter.type === 'highshelf') {
    return highShelvingCalculator.computeResponseAtFrequency(freq, filter.frequency, filter.gain, filter.Q)
  }
  return 0
}

// Expose output node for MasterSection to connect to
defineExpose({
  getOutputNode: () => outputNode
})
</script>
