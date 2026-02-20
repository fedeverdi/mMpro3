<template>
  <div class="spectrum-meter bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-purple-600/60 p-3 flex flex-col gap-3 h-full">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-bold text-purple-200 tracking-wide uppercase">Spectrum Meter</p>
        <p class="text-[10px] text-gray-400">Live master analysis</p>
      </div>
      
      <!-- Precisione (per bars, mirror, dots) -->
      <div v-if="displayMode === 'bars' || displayMode === 'mirror' || displayMode === 'dots'" class="flex gap-1">
        <button
          v-for="option in barOptions"
          :key="option.bars"
          @click="barCount = option.bars"
          :class="[
            'px-2 py-1 text-[10px] font-bold rounded transition-colors',
            barCount === option.bars 
              ? 'bg-purple-600 text-white' 
              : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
          ]"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <div class="relative flex-1 rounded-lg border border-gray-800 bg-gray-950/80 overflow-hidden shadow-inner">
      <canvas ref="spectrumCanvas" class="absolute inset-0 w-full h-full" style="image-rendering: pixelated;"></canvas>
    </div>

    <!-- Modalità visualizzazione -->
    <div class="flex justify-center gap-1">
      <button
        @click="displayMode = 'bars'"
        :class="[
          'px-2 py-1 text-[10px] font-bold rounded transition-colors',
          displayMode === 'bars' 
            ? 'bg-purple-600 text-white' 
            : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]"
      >
        Bars
      </button>
      <button
        @click="displayMode = 'curve'"
        :class="[
          'px-2 py-1 text-[10px] font-bold rounded transition-colors',
          displayMode === 'curve' 
            ? 'bg-purple-600 text-white' 
            : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]"
      >
        Curve
      </button>
      <button
        @click="displayMode = 'mirror'"
        :class="[
          'px-2 py-1 text-[10px] font-bold rounded transition-colors',
          displayMode === 'mirror' 
            ? 'bg-purple-600 text-white' 
            : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]"
      >
        Mirror
      </button>
      <button
        @click="displayMode = 'dots'"
        :class="[
          'px-2 py-1 text-[10px] font-bold rounded transition-colors',
          displayMode === 'dots' 
            ? 'bg-purple-600 text-white' 
            : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]"
      >
        Dots
      </button>
      <button
        @click="displayMode = 'line'"
        :class="[
          'px-2 py-1 text-[10px] font-bold rounded transition-colors',
          displayMode === 'line' 
            ? 'bg-purple-600 text-white' 
            : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
        ]"
      >
        Line
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, toRaw, inject } from 'vue'

interface Props {
  masterFx?: any
}

const props = defineProps<Props>()

const spectrumCanvas = ref<HTMLCanvasElement | null>(null)
const barCount = ref(24) // Default: 24 barre
const displayMode = ref<'bars' | 'curve' | 'mirror' | 'dots' | 'line'>('bars') // Default: barre

const barOptions = [
  { label: '16', bars: 16 },
  { label: '24', bars: 24 },
  { label: '48', bars: 48 },
  { label: '96', bars: 96 },
  { label: '128', bars: 128 }
]

const ToneRef = inject<any>('Tone')
let Tone: any = null
let splitNode: any = null // Split stereo into L/R
let analyserLeft: any = null
let analyserRight: any = null
let animationFrame: number | null = null
let currentMasterNode: any = null

// Peak hold per visualizzazione (separati per L/R in modalità mirror)
let peakValues: number[] = []
let peakTimestamps: number[] = []
let peakValuesRight: number[] = [] // Per modalità mirror
let peakTimestampsRight: number[] = [] // Per modalità mirror
const PEAK_HOLD_TIME = 1500 // ms - quanto tempo tenere il peak
const PEAK_DECAY_RATE = 0.002 // velocità di decadimento del peak

const ensureAnalyser = async () => {
  if (!Tone) {
    // Get Tone.js from inject
    if (ToneRef?.value) {
      Tone = ToneRef.value
    } else {
      // Fallback: wait for it
      await new Promise<void>((resolve) => {
        const checkTone = setInterval(() => {
          if (ToneRef?.value) {
            Tone = ToneRef.value
            clearInterval(checkTone)
            resolve()
          }
        }, 100)
      })
    }
  }
  if (!splitNode && Tone) {
    // Create split to separate L/R channels
    splitNode = new Tone.Split()
    // Create separate analysers for L and R
    analyserLeft = new Tone.Analyser('fft', 2048)
    analyserRight = new Tone.Analyser('fft', 2048)
    // Chain: master (stereo) -> splitNode -> analyserLeft (ch0) / analyserRight (ch1)
    splitNode.connect(analyserLeft, 0)
    splitNode.connect(analyserRight, 1)
  }
}

const getMasterNode = () => {
  // Get output node from MasterFX
  if (!props.masterFx || !props.masterFx.getOutputNode) return null
  const node = props.masterFx.getOutputNode()
  return node ? toRaw(node) : null
}

const connectAnalyser = async () => {
  await ensureAnalyser()
  if (!splitNode) return

  const masterNode = getMasterNode()
  if (!masterNode) return

  if (currentMasterNode && currentMasterNode !== masterNode) {
    try {
      currentMasterNode.disconnect(splitNode)
    } catch (error) {
      console.warn('Spectrum meter: unable to disconnect previous analyser tap', error)
    }
    currentMasterNode = null
  }

  if (currentMasterNode === masterNode) return
  currentMasterNode = masterNode

  try {
    // Connect to splitNode which separates L/R channels to separate analysers
    masterNode.connect(splitNode)
  } catch (error) {
    console.warn('Spectrum meter: unable to connect to master channel', error)
  }
}

watch(
  () => {
    // Get output node from MasterEQDisplay
    if (!props.masterEqDisplay || !props.masterEqDisplay.getOutputNode) return null
    return props.masterEqDisplay.getOutputNode()
  },
  (newNode) => {
    if (newNode) {
      connectAnalyser()
    } else {
      // Wait a bit for outputNode to be created
      setTimeout(() => {
        connectAnalyser()
      }, 100)
    }
  },
  { immediate: true }
)

const renderSpectrum = () => {
  animationFrame = requestAnimationFrame(renderSpectrum)
  if (!analyserLeft || !analyserRight || !spectrumCanvas.value) return

  const canvas = spectrumCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const totalWidth = canvas.clientWidth || 200
  const totalHeight = canvas.clientHeight || 200
  const labelHeightBottom = 12 // Spazio riservato per le label Hz in basso
  const labelWidthLeft = 30 // Spazio riservato per le label dB a sinistra
  const width = totalWidth - labelWidthLeft // Larghezza del grafico
  const height = totalHeight - labelHeightBottom // Altezza del grafico
  
  canvas.width = totalWidth * dpr
  canvas.height = totalHeight * dpr
  ctx.setTransform(1, 0, 0, 1, 0, 0)
  ctx.scale(dpr, dpr)

  ctx.fillStyle = '#030712'
  ctx.fillRect(0, 0, totalWidth, totalHeight)

  // Griglia orizzontale con etichette dB
  ctx.strokeStyle = 'rgba(148, 163, 184, 0.15)'
  ctx.lineWidth = 1
  ctx.font = '7px monospace'
  ctx.fillStyle = 'rgba(148, 163, 184, 0.7)'
  
  for (let i = 1; i < 4; i++) {
    const y = (height / 4) * i
    
    // Linea orizzontale (solo nell'area del grafico)
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.15)'
    ctx.beginPath()
    ctx.moveTo(labelWidthLeft, y)
    ctx.lineTo(totalWidth, y)
    ctx.stroke()
    
    // Etichette dB (nell'area riservata a sinistra)
    const dbValue = Math.round(-140 * (1 - i / 4))
    ctx.fillStyle = 'rgba(148, 163, 184, 0.7)'
    ctx.textAlign = 'right'
    ctx.fillText(`${dbValue}dB`, labelWidthLeft - 3, y + 3)
  }
  
  // Griglia verticale e etichette frequenze
  const freqLabels = [
    { label: '20', pos: 0.02 },
    { label: '100', pos: 0.15 },
    { label: '500', pos: 0.35 },
    { label: '1k', pos: 0.5 },
    { label: '5k', pos: 0.7 },
    { label: '10k', pos: 0.85 },
    { label: '20k', pos: 0.98 }
  ]
  
  ctx.textAlign = 'left'
  freqLabels.forEach(({ label, pos }) => {
    const x = labelWidthLeft + width * pos
    
    // Linea verticale (solo nell'area del grafico)
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.15)'
    ctx.beginPath()
    ctx.moveTo(x, 0)
    ctx.lineTo(x, height)
    ctx.stroke()
    
    // Etichetta frequenza (nell'area riservata sotto il grafico)
    ctx.fillStyle = 'rgba(148, 163, 184, 0.7)'
    ctx.fillText(label, x + 2, height + 9)
  })

  // Get values from both channels
  const valuesLeft = analyserLeft.getValue() as Float32Array
  const valuesRight = analyserRight.getValue() as Float32Array
  if (!valuesLeft || !valuesLeft.length || !valuesRight || !valuesRight.length) return
  
  // For non-mirror modes, average L+R channels
  const values = new Float32Array(valuesLeft.length)
  for (let i = 0; i < valuesLeft.length; i++) {
    values[i] = (valuesLeft[i] + valuesRight[i]) / 2
  }

  const currentTime = Date.now()
  
  // Helper function per mapping logaritmico delle frequenze
  const getLogFrequencyBands = (numBands: number, fftSize: number, sampleRate = 44100) => {
    const minFreq = 20
    const maxFreq = sampleRate / 2 // Nyquist frequency
    const bands = []
    const usedBins = new Set<number>() // Track per evitare sovrapposizioni
    
    for (let i = 0; i < numBands; i++) {
      const logMin = Math.log10(minFreq)
      const logMax = Math.log10(maxFreq)
      const logStep = (logMax - logMin) / numBands
      
      const freqStart = Math.pow(10, logMin + i * logStep)
      const freqEnd = Math.pow(10, logMin + (i + 1) * logStep)
      
      // Converte frequenze in bin index
      let binStart = Math.floor((freqStart / sampleRate) * fftSize)
      let binEnd = Math.floor((freqEnd / sampleRate) * fftSize)
      
      // Assicura che ogni banda abbia almeno 1 bin e non si sovrapponga
      binStart = Math.min(binStart, values.length - 1)
      binEnd = Math.min(binEnd, values.length - 1)
      
      // Per le barre con alta risoluzione, assicura separazione minima
      if (binStart === binEnd) {
        binEnd = Math.min(binStart + 1, values.length - 1)
      }
      
      // Evita sovrapposizioni: se il binStart è già usato, sposta avanti
      while (usedBins.has(binStart) && binStart < values.length - 1) {
        binStart++
      }
      
      // Aggiorna binEnd se necessario
      if (binEnd <= binStart) {
        binEnd = Math.min(binStart + 1, values.length - 1)
      }
      
      // Marca i bin come usati
      for (let bin = binStart; bin <= binEnd; bin++) {
        usedBins.add(bin)
      }
      
      // Solo aggiungi bande valide
      if (binStart < values.length && binEnd < values.length && binStart <= binEnd) {
        bands.push({
          freqStart,
          freqEnd,
          binStart,
          binEnd
        })
      }
    }
    
    return bands
  }

  if (displayMode.value === 'bars') {
    // Modalità barre stereo - L e R affiancate
    const bars = barCount.value
    const bands = getLogFrequencyBands(bars, 2048)
    
    // Inizializza peak arrays se necessario
    if (peakValues.length !== bars) {
      peakValues = new Array(bars).fill(0)
      peakTimestamps = new Array(bars).fill(0)
      peakValuesRight = new Array(bars).fill(0)
      peakTimestampsRight = new Array(bars).fill(0)
    }

    for (let i = 0; i < bars; i++) {
      const band = bands[i]
      if (!band || band.binStart >= band.binEnd) continue
      
      // LEFT CHANNEL
      let sumLeft = 0
      let countLeft = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesLeft.length) {
          sumLeft += valuesLeft[binIdx] || -100
          countLeft++
        }
      }
      
      const avgLeft = countLeft > 0 ? sumLeft / countLeft : -100
      const normalizedLeft = Math.max(0, Math.min(1, (avgLeft + 100) / 100))
      const barHeightLeft = Math.max(0.02, normalizedLeft) * height

      // Aggiorna peak LEFT
      if (normalizedLeft > peakValues[i]) {
        peakValues[i] = normalizedLeft
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValues[i] = Math.max(0, peakValues[i] - PEAK_DECAY_RATE)
        }
      }

      // RIGHT CHANNEL
      let sumRight = 0
      let countRight = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesRight.length) {
          sumRight += valuesRight[binIdx] || -100
          countRight++
        }
      }
      
      const avgRight = countRight > 0 ? sumRight / countRight : -100
      const normalizedRight = Math.max(0, Math.min(1, (avgRight + 100) / 100))
      const barHeightRight = Math.max(0.02, normalizedRight) * height

      // Aggiorna peak RIGHT
      if (normalizedRight > peakValuesRight[i]) {
        peakValuesRight[i] = normalizedRight
        peakTimestampsRight[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestampsRight[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      const barWidth = width / bars
      const x = labelWidthLeft + i * barWidth
      const halfBarWidth = barWidth / 2

      // Disegna barra LEFT (metà sinistra - viola)
      const yLeft = height - barHeightLeft
      const gradientLeft = ctx.createLinearGradient(x, yLeft, x, height)
      gradientLeft.addColorStop(0, '#a855f7')
      gradientLeft.addColorStop(0.5, '#8b5cf6')
      gradientLeft.addColorStop(1, '#7c3aed')
      
      ctx.fillStyle = gradientLeft
      ctx.fillRect(x, yLeft, Math.max(1, halfBarWidth - 0.5), barHeightLeft)
      
      // Disegna barra RIGHT (metà destra - blu)
      const yRight = height - barHeightRight
      const gradientRight = ctx.createLinearGradient(x + halfBarWidth, yRight, x + halfBarWidth, height)
      gradientRight.addColorStop(0, '#3b82f6')
      gradientRight.addColorStop(0.5, '#2563eb')
      gradientRight.addColorStop(1, '#0ea5e9')
      
      ctx.fillStyle = gradientRight
      ctx.fillRect(x + halfBarWidth, yRight, Math.max(1, halfBarWidth - 0.5), barHeightRight)

      // Peak marker LEFT
      if (peakValues[i] > 0.02) {
        const peakYLeft = height - (peakValues[i] * height)
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 1.5
        ctx.beginPath()
        ctx.moveTo(x, peakYLeft)
        ctx.lineTo(x + halfBarWidth - 0.5, peakYLeft)
        ctx.stroke()
      }
      
      // Peak marker RIGHT
      if (peakValuesRight[i] > 0.02) {
        const peakYRight = height - (peakValuesRight[i] * height)
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 1.5
        ctx.beginPath()
        ctx.moveTo(x + halfBarWidth, peakYRight)
        ctx.lineTo(x + barWidth - 0.5, peakYRight)
        ctx.stroke()
      }
    }
  } else if (displayMode.value === 'curve') {
    // Modalità curve stereo - L e R sovrapposte
    const points = 64
    const bands = getLogFrequencyBands(points, 2048)
    
    // Inizializza peak arrays se necessario
    if (peakValues.length !== points) {
      peakValues = new Array(points).fill(0)
      peakTimestamps = new Array(points).fill(0)
      peakValuesRight = new Array(points).fill(0)
      peakTimestampsRight = new Array(points).fill(0)
    }
    
    // Calcola valori per LEFT e RIGHT
    const curveDataLeft: Array<{x: number, y: number, normalized: number}> = []
    const curveDataRight: Array<{x: number, y: number, normalized: number}> = []
    
    for (let i = 0; i < points; i++) {
      const band = bands[i]
      
      // LEFT CHANNEL
      let avgLeft = -100
      let normalizedLeft = 0
      if (band && band.binStart < band.binEnd) {
        let sumLeft = 0
        let countLeft = 0
        for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
          if (binIdx < valuesLeft.length) {
            sumLeft += valuesLeft[binIdx] || -100
            countLeft++
          }
        }
        avgLeft = countLeft > 0 ? sumLeft / countLeft : -100
        normalizedLeft = Math.max(0, Math.min(1, (avgLeft + 100) / 100))
      }
      
      const yLeft = height - Math.max(0.02, normalizedLeft) * height
      const x = labelWidthLeft + (i / (points - 1)) * width
      curveDataLeft.push({ x, y: yLeft, normalized: normalizedLeft })
      
      // Aggiorna peak LEFT
      if (normalizedLeft > peakValues[i]) {
        peakValues[i] = normalizedLeft
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValues[i] = Math.max(0, peakValues[i] - PEAK_DECAY_RATE)
        }
      }
      
      // RIGHT CHANNEL
      let avgRight = -100
      let normalizedRight = 0
      if (band && band.binStart < band.binEnd) {
        let sumRight = 0
        let countRight = 0
        for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
          if (binIdx < valuesRight.length) {
            sumRight += valuesRight[binIdx] || -100
            countRight++
          }
        }
        avgRight = countRight > 0 ? sumRight / countRight : -100
        normalizedRight = Math.max(0, Math.min(1, (avgRight + 100) / 100))
      }
      
      const yRight = height - Math.max(0.02, normalizedRight) * height
      curveDataRight.push({ x, y: yRight, normalized: normalizedRight })
      
      // Aggiorna peak RIGHT
      if (normalizedRight > peakValuesRight[i]) {
        peakValuesRight[i] = normalizedRight
        peakTimestampsRight[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestampsRight[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }
    }
    
    // Disegna area sotto la curva LEFT (viola)
    if (curveDataLeft.length > 0) {
      ctx.fillStyle = 'rgba(168, 85, 247, 0.15)'
      ctx.beginPath()
      ctx.moveTo(labelWidthLeft, height)
      ctx.lineTo(curveDataLeft[0].x, curveDataLeft[0].y)
      
      for (let i = 0; i < curveDataLeft.length - 1; i++) {
        const current = curveDataLeft[i]
        const next = curveDataLeft[i + 1]
        const xc = (current.x + next.x) / 2
        const yc = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xc, yc)
      }
      
      const last = curveDataLeft[curveDataLeft.length - 1]
      ctx.lineTo(last.x, last.y)
      ctx.lineTo(labelWidthLeft + width, height)
      ctx.closePath()
      ctx.fill()
    }
    
    // Disegna area sotto la curva RIGHT (blu)
    if (curveDataRight.length > 0) {
      ctx.fillStyle = 'rgba(59, 130, 246, 0.15)'
      ctx.beginPath()
      ctx.moveTo(labelWidthLeft, height)
      ctx.lineTo(curveDataRight[0].x, curveDataRight[0].y)
      
      for (let i = 0; i < curveDataRight.length - 1; i++) {
        const current = curveDataRight[i]
        const next = curveDataRight[i + 1]
        const xc = (current.x + next.x) / 2
        const yc = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xc, yc)
      }
      
      const last = curveDataRight[curveDataRight.length - 1]
      ctx.lineTo(last.x, last.y)
      ctx.lineTo(labelWidthLeft + width, height)
      ctx.closePath()
      ctx.fill()
    }
    
    // Disegna curva LEFT (viola)
    if (curveDataLeft.length > 0) {
      ctx.strokeStyle = '#a855f7'
      ctx.lineWidth = 2.5
      ctx.beginPath()
      ctx.moveTo(curveDataLeft[0].x, curveDataLeft[0].y)
      
      for (let i = 0; i < curveDataLeft.length - 1; i++) {
        const current = curveDataLeft[i]
        const next = curveDataLeft[i + 1]
        const xc = (current.x + next.x) / 2
        const yc = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xc, yc)
      }
      
      const last = curveDataLeft[curveDataLeft.length - 1]
      ctx.lineTo(last.x, last.y)
      ctx.stroke()
    }
    
    // Disegna curva RIGHT (blu)
    if (curveDataRight.length > 0) {
      ctx.strokeStyle = '#3b82f6'
      ctx.lineWidth = 2.5
      ctx.beginPath()
      ctx.moveTo(curveDataRight[0].x, curveDataRight[0].y)
      
      for (let i = 0; i < curveDataRight.length - 1; i++) {
        const current = curveDataRight[i]
        const next = curveDataRight[i + 1]
        const xc = (current.x + next.x) / 2
        const yc = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xc, yc)
      }
      
      const last = curveDataRight[curveDataRight.length - 1]
      ctx.lineTo(last.x, last.y)
      ctx.stroke()
    }
    
    // Disegna peak curve LEFT
    const peakPointsLeft: Array<{x: number, y: number}> = []
    for (let i = 0; i < points; i++) {
      if (peakValues[i] > 0.02) {
        const peakY = height - (peakValues[i] * height)
        const x = labelWidthLeft + (i / (points - 1)) * width
        peakPointsLeft.push({ x, y: peakY })
      }
    }
    
    if (peakPointsLeft.length > 1) {
      ctx.strokeStyle = '#fbbf24'
      ctx.lineWidth = 1.5
      ctx.globalAlpha = 0.7
      ctx.beginPath()
      ctx.moveTo(peakPointsLeft[0].x, peakPointsLeft[0].y)
      
      for (let i = 0; i < peakPointsLeft.length - 1; i++) {
        const current = peakPointsLeft[i]
        const next = peakPointsLeft[i + 1]
        const xc = (current.x + next.x) / 2
        const yc = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xc, yc)
      }
      
      const lastPeak = peakPointsLeft[peakPointsLeft.length - 1]
      ctx.lineTo(lastPeak.x, lastPeak.y)
      ctx.stroke()
      ctx.globalAlpha = 1
    }
    
    // Disegna peak curve RIGHT
    const peakPointsRight: Array<{x: number, y: number}> = []
    for (let i = 0; i < points; i++) {
      if (peakValuesRight[i] > 0.02) {
        const peakY = height - (peakValuesRight[i] * height)
        const x = labelWidthLeft + (i / (points - 1)) * width
        peakPointsRight.push({ x, y: peakY })
      }
    }
    
    if (peakPointsRight.length > 1) {
      ctx.strokeStyle = '#fbbf24'
      ctx.lineWidth = 1.5
      ctx.globalAlpha = 0.7
      ctx.beginPath()
      ctx.moveTo(peakPointsRight[0].x, peakPointsRight[0].y)
      
      for (let i = 0; i < peakPointsRight.length - 1; i++) {
        const current = peakPointsRight[i]
        const next = peakPointsRight[i + 1]
        const xc = (current.x + next.x) / 2
        const yc = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xc, yc)
      }
      
      const lastPeak = peakPointsRight[peakPointsRight.length - 1]
      ctx.lineTo(lastPeak.x, lastPeak.y)
      ctx.stroke()
      ctx.globalAlpha = 1
    }
  } else if (displayMode.value === 'mirror') {
    // Modalità mirror (speculare) - LEFT superiore, RIGHT inferiore
    const bars = barCount.value
    const bands = getLogFrequencyBands(bars, 2048)
    const halfHeight = height / 2
    
    if (peakValues.length !== bars) {
      peakValues = new Array(bars).fill(0)
      peakTimestamps = new Array(bars).fill(0)
      peakValuesRight = new Array(bars).fill(0)
      peakTimestampsRight = new Array(bars).fill(0)
    }

    for (let i = 0; i < bars; i++) {
      const band = bands[i]
      if (!band || band.binStart >= band.binEnd) continue
      
      // LEFT CHANNEL (parte superiore - barre verso l'alto)
      let sumLeft = 0
      let countLeft = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesLeft.length) {
          sumLeft += valuesLeft[binIdx] || -100
          countLeft++
        }
      }
      
      const avgLeft = countLeft > 0 ? sumLeft / countLeft : -100
      const normalizedLeft = Math.max(0, Math.min(1, (avgLeft + 100) / 100))
      const barHeightLeft = Math.max(0.02, normalizedLeft) * halfHeight

      // Aggiorna peak LEFT
      if (normalizedLeft > peakValues[i]) {
        peakValues[i] = normalizedLeft
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValues[i] = Math.max(0, peakValues[i] - PEAK_DECAY_RATE)
        }
      }

      // RIGHT CHANNEL (parte inferiore - barre verso il basso)
      let sumRight = 0
      let countRight = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesRight.length) {
          sumRight += valuesRight[binIdx] || -100
          countRight++
        }
      }
      
      const avgRight = countRight > 0 ? sumRight / countRight : -100
      const normalizedRight = Math.max(0, Math.min(1, (avgRight + 100) / 100))
      const barHeightRight = Math.max(0.02, normalizedRight) * halfHeight

      // Aggiorna peak RIGHT
      if (normalizedRight > peakValuesRight[i]) {
        peakValuesRight[i] = normalizedRight
        peakTimestampsRight[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestampsRight[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      const barWidth = width / bars
      const x = labelWidthLeft + i * barWidth
      
      // Gradient per LEFT (superiore): viola -> blu
      const gradientLeft = ctx.createLinearGradient(x, halfHeight - barHeightLeft, x, halfHeight)
      gradientLeft.addColorStop(0, '#a855f7')
      gradientLeft.addColorStop(1, '#3b82f6')
      
      // Gradient per RIGHT (inferiore): blu -> cyan
      const gradientRight = ctx.createLinearGradient(x, halfHeight, x, halfHeight + barHeightRight)
      gradientRight.addColorStop(0, '#3b82f6')
      gradientRight.addColorStop(1, '#0ea5e9')

      // Barra superiore (LEFT)
      ctx.fillStyle = gradientLeft
      ctx.fillRect(x, halfHeight - barHeightLeft, Math.max(1, barWidth - 1), barHeightLeft)
      
      // Barra inferiore (RIGHT)
      ctx.fillStyle = gradientRight
      ctx.fillRect(x, halfHeight, Math.max(1, barWidth - 1), barHeightRight)
      
      // Peak marker LEFT (superiore)
      if (peakValues[i] > 0.02) {
        const peakHeightLeft = peakValues[i] * halfHeight
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 2
        ctx.beginPath()
        ctx.moveTo(x, halfHeight - peakHeightLeft)
        ctx.lineTo(x + barWidth - 1, halfHeight - peakHeightLeft)
        ctx.stroke()
      }
      
      // Peak marker RIGHT (inferiore)
      if (peakValuesRight[i] > 0.02) {
        const peakHeightRight = peakValuesRight[i] * halfHeight
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 2
        ctx.beginPath()
        ctx.moveTo(x, halfHeight + peakHeightRight)
        ctx.lineTo(x + barWidth - 1, halfHeight + peakHeightRight)
        ctx.stroke()
      }
    }
  } else if (displayMode.value === 'dots') {
    // Modalità dots stereo - L e R affiancati
    const bars = barCount.value
    const bands = getLogFrequencyBands(bars, 2048)
    
    if (peakValues.length !== bars) {
      peakValues = new Array(bars).fill(0)
      peakTimestamps = new Array(bars).fill(0)
      peakValuesRight = new Array(bars).fill(0)
      peakTimestampsRight = new Array(bars).fill(0)
    }

    for (let i = 0; i < bars; i++) {
      const band = bands[i]
      if (!band || band.binStart >= band.binEnd) continue
      
      // LEFT CHANNEL
      let sumLeft = 0
      let countLeft = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesLeft.length) {
          sumLeft += valuesLeft[binIdx] || -100
          countLeft++
        }
      }
      
      const avgLeft = countLeft > 0 ? sumLeft / countLeft : -100
      const normalizedLeft = Math.max(0, Math.min(1, (avgLeft + 100) / 100))

      // Aggiorna peak LEFT
      if (normalizedLeft > peakValues[i]) {
        peakValues[i] = normalizedLeft
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValues[i] = Math.max(0, peakValues[i] - PEAK_DECAY_RATE)
        }
      }

      // RIGHT CHANNEL
      let sumRight = 0
      let countRight = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesRight.length) {
          sumRight += valuesRight[binIdx] || -100
          countRight++
        }
      }
      
      const avgRight = countRight > 0 ? sumRight / countRight : -100
      const normalizedRight = Math.max(0, Math.min(1, (avgRight + 100) / 100))

      // Aggiorna peak RIGHT
      if (normalizedRight > peakValuesRight[i]) {
        peakValuesRight[i] = normalizedRight
        peakTimestampsRight[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestampsRight[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      const barWidth = width / bars
      const x = labelWidthLeft + i * barWidth + barWidth / 2
      
      // Dot LEFT (viola - sinistra)
      const yLeft = height - Math.max(0.02, normalizedLeft) * height
      const xLeft = x - 2
      const dotGradientLeft = ctx.createRadialGradient(xLeft, yLeft, 0, xLeft, yLeft, 3)
      dotGradientLeft.addColorStop(0, '#a855f7')
      dotGradientLeft.addColorStop(1, '#7c3aed')
      
      ctx.fillStyle = dotGradientLeft
      ctx.beginPath()
      ctx.arc(xLeft, yLeft, 3, 0, Math.PI * 2)
      ctx.fill()
      
      // Dot RIGHT (blu - destra)
      const yRight = height - Math.max(0.02, normalizedRight) * height
      const xRight = x + 2
      const dotGradientRight = ctx.createRadialGradient(xRight, yRight, 0, xRight, yRight, 3)
      dotGradientRight.addColorStop(0, '#3b82f6')
      dotGradientRight.addColorStop(1, '#0ea5e9')
      
      ctx.fillStyle = dotGradientRight
      ctx.beginPath()
      ctx.arc(xRight, yRight, 3, 0, Math.PI * 2)
      ctx.fill()
      
      // Peak dot LEFT
      if (peakValues[i] > 0.02) {
        const peakYLeft = height - (peakValues[i] * height)
        ctx.fillStyle = '#fbbf24'
        ctx.beginPath()
        ctx.arc(xLeft, peakYLeft, 2, 0, Math.PI * 2)
        ctx.fill()
      }
      
      // Peak dot RIGHT
      if (peakValuesRight[i] > 0.02) {
        const peakYRight = height - (peakValuesRight[i] * height)
        ctx.fillStyle = '#fbbf24'
        ctx.beginPath()
        ctx.arc(xRight, peakYRight, 2, 0, Math.PI * 2)
        ctx.fill()
      }
    }
  } else if (displayMode.value === 'line') {
    // Modalità line stereo - L e R sovrapposte
    const points = 256
    const bands = getLogFrequencyBands(points, 2048)
    
    if (peakValues.length !== points) {
      peakValues = new Array(points).fill(0)
      peakTimestamps = new Array(points).fill(0)
      peakValuesRight = new Array(points).fill(0)
      peakTimestampsRight = new Array(points).fill(0)
    }
    
    const curveDataLeft: Array<{x: number, y: number, normalized: number}> = []
    const curveDataRight: Array<{x: number, y: number, normalized: number}> = []
    
    for (let i = 0; i < points; i++) {
      const band = bands[i]
      if (!band || band.binStart >= band.binEnd) continue
      
      // LEFT CHANNEL
      let sumLeft = 0
      let countLeft = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesLeft.length) {
          sumLeft += valuesLeft[binIdx] || -100
          countLeft++
        }
      }
      
      const avgLeft = countLeft > 0 ? sumLeft / countLeft : -100
      const normalizedLeft = Math.max(0, Math.min(1, (avgLeft + 100) / 100))
      const yLeft = height - Math.max(0.02, normalizedLeft) * height
      const x = labelWidthLeft + (i / (points - 1)) * width
      
      curveDataLeft.push({ x, y: yLeft, normalized: normalizedLeft })
      
      // Aggiorna peak LEFT
      if (normalizedLeft > peakValues[i]) {
        peakValues[i] = normalizedLeft
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValues[i] = Math.max(0, peakValues[i] - PEAK_DECAY_RATE)
        }
      }
      
      // RIGHT CHANNEL
      let sumRight = 0
      let countRight = 0
      for (let binIdx = band.binStart; binIdx <= band.binEnd; binIdx++) {
        if (binIdx < valuesRight.length) {
          sumRight += valuesRight[binIdx] || -100
          countRight++
        }
      }
      
      const avgRight = countRight > 0 ? sumRight / countRight : -100
      const normalizedRight = Math.max(0, Math.min(1, (avgRight + 100) / 100))
      const yRight = height - Math.max(0.02, normalizedRight) * height
      
      curveDataRight.push({ x, y: yRight, normalized: normalizedRight })
      
      // Aggiorna peak RIGHT
      if (normalizedRight > peakValuesRight[i]) {
        peakValuesRight[i] = normalizedRight
        peakTimestampsRight[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestampsRight[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }
    }
    
    // Disegna linea LEFT (viola con glow)
    if (curveDataLeft.length > 0) {
      ctx.shadowBlur = 15
      ctx.shadowColor = '#a855f7'
      ctx.strokeStyle = '#a855f7'
      ctx.lineWidth = 3
      ctx.beginPath()
      
      curveDataLeft.forEach((point, i) => {
        if (i === 0) {
          ctx.moveTo(point.x, point.y)
        } else {
          ctx.lineTo(point.x, point.y)
        }
      })
      
      ctx.stroke()
      ctx.shadowBlur = 0
    }
    
    // Disegna linea RIGHT (blu con glow)
    if (curveDataRight.length > 0) {
      ctx.shadowBlur = 15
      ctx.shadowColor = '#3b82f6'
      ctx.strokeStyle = '#3b82f6'
      ctx.lineWidth = 3
      ctx.beginPath()
      
      curveDataRight.forEach((point, i) => {
        if (i === 0) {
          ctx.moveTo(point.x, point.y)
        } else {
          ctx.lineTo(point.x, point.y)
        }
      })
      
      ctx.stroke()
      ctx.shadowBlur = 0
    }
    
    // Peak line LEFT
    ctx.strokeStyle = '#fbbf24'
    ctx.lineWidth = 1.5
    ctx.globalAlpha = 0.7
    ctx.beginPath()
    
    for (let i = 0; i < points; i++) {
      if (peakValues[i] > 0.02) {
        const peakY = height - (peakValues[i] * height)
        const x = labelWidthLeft + (i / (points - 1)) * width
        
        if (i === 0 || peakValues[i - 1] <= 0.02) {
          ctx.moveTo(x, peakY)
        } else {
          ctx.lineTo(x, peakY)
        }
      }
    }
    
    ctx.stroke()
    
    // Peak line RIGHT
    ctx.beginPath()
    
    for (let i = 0; i < points; i++) {
      if (peakValuesRight[i] > 0.02) {
        const peakY = height - (peakValuesRight[i] * height)
        const x = labelWidthLeft + (i / (points - 1)) * width
        
        if (i === 0 || peakValuesRight[i - 1] <= 0.02) {
          ctx.moveTo(x, peakY)
        } else {
          ctx.lineTo(x, peakY)
        }
      }
    }
    
    ctx.stroke()
    ctx.globalAlpha = 1
  }
}

onMounted(async () => {
  await connectAnalyser()
  renderSpectrum()
})

onUnmounted(() => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame)
  }
  if (currentMasterNode && splitNode) {
    try {
      currentMasterNode.disconnect(splitNode)
    } catch (error) {
      console.warn('Spectrum meter: unable to disconnect analyser', error)
    }
  }
  if (analyserLeft) {
    analyserLeft.dispose()
  }
  if (analyserRight) {
    analyserRight.dispose()
  }
  if (splitNode) {
    splitNode.dispose()
  }
})
</script>
