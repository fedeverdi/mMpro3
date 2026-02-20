<template>
  <div class="spectrum-meter bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-purple-600/60 p-3 flex flex-col gap-3 h-full">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-bold text-purple-200 tracking-wide uppercase">Spectrum Analyzer</p>
        <p class="text-[10px] text-gray-400">20 Hz - 20 kHz</p>
      </div>

      <!-- Numero di barre -->
      <div class="flex gap-1">
        <button
          v-for="option in barOptions"
          :key="option"
          @click="bars = option"
          :class="[
            'px-2 py-1 text-[10px] font-bold rounded transition-colors',
            bars === option
              ? 'bg-purple-600 text-white'
              : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
          ]"
        >
          {{ option }}
        </button>
      </div>
    </div>

    <!-- Canvas -->
    <div class="relative flex-1 rounded-lg border border-gray-800 bg-gray-950/80 overflow-hidden shadow-inner">
      <canvas ref="canvas" class="absolute inset-0 w-full h-full"></canvas>
      
      <!-- Modalità visualizzazione -->
      <div class="absolute top-2 right-2 flex gap-1 z-10">
        <button
          @click="displayMode = 'bars'"
          :class="[
            'px-3 py-1 text-[10px] font-bold rounded transition-colors',
            displayMode === 'bars'
              ? 'bg-purple-600 text-white'
              : 'bg-gray-700/90 hover:bg-gray-600 text-gray-300'
          ]"
        >
          Bars
        </button>
        <button
          @click="displayMode = 'curve'"
          :class="[
            'px-3 py-1 text-[10px] font-bold rounded transition-colors',
            displayMode === 'curve'
              ? 'bg-purple-600 text-white'
              : 'bg-gray-700/90 hover:bg-gray-600 text-gray-300'
          ]"
        >
          Curve
        </button>
        <button
          @click="displayMode = 'mirror'"
          :class="[
            'px-3 py-1 text-[10px] font-bold rounded transition-colors',
            displayMode === 'mirror'
              ? 'bg-purple-600 text-white'
              : 'bg-gray-700/90 hover:bg-gray-600 text-gray-300'
          ]"
        >
          Mirror
        </button>
        <button
          @click="displayMode = 'dots'"
          :class="[
            'px-3 py-1 text-[10px] font-bold rounded transition-colors',
            displayMode === 'dots'
              ? 'bg-purple-600 text-white'
              : 'bg-gray-700/90 hover:bg-gray-600 text-gray-300'
          ]"
        >
          Dots
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, inject, toRaw } from 'vue'

interface Props {
  masterFx?: any
}

const props = defineProps<Props>()

const canvas = ref<HTMLCanvasElement | null>(null)
const bars = ref(128)
const barOptions = [16, 24, 48, 96, 128]
const displayMode = ref<'bars' | 'curve' | 'mirror' | 'dots'>('bars')

const ToneRef = inject<any>('Tone')
let Tone: any = null
let analyserLeft: any = null
let analyserRight: any = null
let splitNode: any = null
let animationId: number | null = null
let currentMasterNode: any = null

// Peak hold tracking
let peakValuesLeft: number[] = []
let peakValuesRight: number[] = []
let peakTimestamps: number[] = []
const PEAK_HOLD_TIME = 1500 // ms
const PEAK_DECAY_RATE = 0.002

// Inizializza Tone.js e gli analyser stereo
const initAnalyser = async () => {
  if (!Tone) {
    if (ToneRef?.value) {
      Tone = ToneRef.value
    } else {
      await new Promise<void>((resolve) => {
        const check = setInterval(() => {
          if (ToneRef?.value) {
            Tone = ToneRef.value
            clearInterval(check)
            resolve()
          }
        }, 100)
      })
    }
  }

  if (!splitNode && Tone) {
    // Crea split per separare i canali L/R
    splitNode = new Tone.Split()
    // Crea analyser separati per L e R
    analyserLeft = new Tone.Analyser('fft', 4096)
    analyserRight = new Tone.Analyser('fft', 4096)
    // Connetti: split -> analyserLeft (canale 0) / analyserRight (canale 1)
    splitNode.connect(analyserLeft, 0)
    splitNode.connect(analyserRight, 1)
  }
}

// Ottieni il nodo master da MasterFX
const getMasterNode = () => {
  if (!props.masterFx?.getOutputNode) return null
  const node = props.masterFx.getOutputNode()
  return node ? toRaw(node) : null
}

// Connetti gli analyser al master
const connectAnalyser = async () => {
  await initAnalyser()
  if (!splitNode) return

  const masterNode = getMasterNode()
  if (!masterNode) return

  // Disconnetti il nodo precedente
  if (currentMasterNode && currentMasterNode !== masterNode) {
    try {
      currentMasterNode.disconnect(splitNode)
    } catch (e) {
      console.warn('Unable to disconnect previous node:', e)
    }
  }

  if (currentMasterNode === masterNode) return
  
  // Connetti il nuovo nodo allo split
  try {
    masterNode.connect(splitNode)
    currentMasterNode = masterNode
  } catch (e) {
    console.warn('Unable to connect analyser:', e)
  }
}

// Watch masterFx changes
watch(
  () => props.masterFx,
  () => {
    setTimeout(() => connectAnalyser(), 100)
  },
  { immediate: true }
)

// Converti frequenza in posizione X logaritmica (0-1)
const freqToPosition = (freq: number): number => {
  const minFreq = 20
  const maxFreq = 20000
  const logMin = Math.log10(minFreq)
  const logMax = Math.log10(maxFreq)
  const clamped = Math.max(minFreq, Math.min(maxFreq, freq))
  const logFreq = Math.log10(clamped)
  return (logFreq - logMin) / (logMax - logMin)
}

// Calcola le bande di frequenza logaritmiche usando il numero REALE di bin FFT
const getLogBands = (numBands: number, sampleRate: number, fftBins: number) => {
  const bands: Array<{
    freqStart: number
    freqEnd: number
    binStart: number
    binEnd: number
  }> = []

  const minFreq = 20
  const maxFreq = 20000
  const logMin = Math.log10(minFreq)
  const logMax = Math.log10(maxFreq)
  const logStep = (logMax - logMin) / numBands
  const nyquist = sampleRate / 2

  for (let i = 0; i < numBands; i++) {
    const freqStart = Math.pow(10, logMin + i * logStep)
    const freqEnd = Math.pow(10, logMin + (i + 1) * logStep)

    // Mapping frequenza -> bin FFT
    // bin = (freq / nyquist) * fftBins, poi clampato a fftBins - 1
    const rawStart = Math.floor((freqStart / nyquist) * fftBins)
    const rawEnd = Math.ceil((freqEnd / nyquist) * fftBins)

    const binStart = Math.max(0, Math.min(rawStart, fftBins - 1))
    const binEnd = Math.max(binStart, Math.min(rawEnd, fftBins - 1))

    bands.push({
      freqStart,
      freqEnd,
      binStart,
      binEnd
    })
  }

  return bands
}

// Render loop
const render = () => {
  animationId = requestAnimationFrame(render)

  if (!canvas.value || !analyserLeft || !analyserRight || !Tone) return

  const ctx = canvas.value.getContext('2d')
  if (!ctx) return

  // Setup canvas
  const dpr = window.devicePixelRatio || 1
  const width = canvas.value.clientWidth
  const height = canvas.value.clientHeight

  if (width <= 0 || height <= 0) return

  const padding = { top: 10, right: 0, bottom: 20, left: 35 }
  const graphWidth = width - padding.left - padding.right
  const graphHeight = height - padding.top - padding.bottom

  // Reset transform per evitare accumulo dello scale ad ogni frame
  canvas.value.width = Math.floor(width * dpr)
  canvas.value.height = Math.floor(height * dpr)
  ctx.setTransform(1, 0, 0, 1, 0, 0)
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)

  // Clear
  ctx.fillStyle = '#030712'
  ctx.fillRect(0, 0, width, height)

  // Griglia dB orizzontale
  ctx.strokeStyle = 'rgba(148, 163, 184, 0.15)'
  ctx.lineWidth = 1
  ctx.font = '8px monospace'
  ctx.fillStyle = 'rgba(148, 163, 184, 0.7)'
  ctx.textAlign = 'right'

  const dbLevels = [0, -35, -70, -105, -140]
  dbLevels.forEach((db, i) => {
    const y = padding.top + (i / 4) * graphHeight
    
    ctx.beginPath()
    ctx.moveTo(padding.left, y)
    ctx.lineTo(width - padding.right, y)
    ctx.stroke()

    ctx.fillText(`${db}dB`, padding.left - 3, y + 3)
  })

  // Griglia frequenze verticale
  const freqLabels = [
    { label: '20Hz', freq: 20 },
    { label: '100Hz', freq: 100 },
    { label: '500Hz', freq: 500 },
    { label: '1kHz', freq: 1000 },
    { label: '5kHz', freq: 5000 },
    { label: '10kHz', freq: 10000 },
    { label: '20kHz', freq: 20000 }
  ]

  ctx.textAlign = 'center'
  freqLabels.forEach(({ label, freq }) => {
    const x = padding.left + graphWidth * freqToPosition(freq)

    ctx.beginPath()
    ctx.moveTo(x, padding.top)
    ctx.lineTo(x, height - padding.bottom)
    ctx.stroke()

    ctx.fillText(label, x, height - 5)
  })

  // Ottieni dati FFT da entrambi i canali
  const fftDataLeft = analyserLeft.getValue() as Float32Array
  const fftDataRight = analyserRight.getValue() as Float32Array
  if (!fftDataLeft || fftDataLeft.length === 0 || !fftDataRight || fftDataRight.length === 0) return

  // Calcola bande con lunghezza reale dei bin
  const sampleRate = Tone.context.sampleRate
  const bands = getLogBands(bars.value, sampleRate, fftDataLeft.length)

  // Inizializza array peak se necessario
  const numBands = bands.length
  if (peakValuesLeft.length !== numBands) {
    peakValuesLeft = new Array(numBands).fill(0)
    peakValuesRight = new Array(numBands).fill(0)
    peakTimestamps = new Array(numBands).fill(0)
  }

  const currentTime = Date.now()

  if (displayMode.value === 'bars') {
    // Disegna barre stereo (metà L, metà R)
    for (let i = 0; i < bands.length; i++) {
      const band = bands[i]
      
      // Canale LEFT
      let maxDbLeft = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataLeft.length; bin++) {
        maxDbLeft = Math.max(maxDbLeft, fftDataLeft[bin])
      }
      const normalizedLeft = Math.max(0, Math.min(1, (maxDbLeft + 140) / 140))

      // Canale RIGHT
      let maxDbRight = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataRight.length; bin++) {
        maxDbRight = Math.max(maxDbRight, fftDataRight[bin])
      }
      const normalizedRight = Math.max(0, Math.min(1, (maxDbRight + 140) / 140))

      // Aggiorna peak hold
      const maxNormalized = Math.max(normalizedLeft, normalizedRight)
      if (maxNormalized > peakValuesLeft[i]) {
        peakValuesLeft[i] = normalizedLeft
        peakValuesRight[i] = normalizedRight
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesLeft[i] = Math.max(0, peakValuesLeft[i] - PEAK_DECAY_RATE)
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      const posStart = freqToPosition(band.freqStart)
      const posEnd = freqToPosition(band.freqEnd)

      const xStart = padding.left + graphWidth * posStart
      const xEnd = padding.left + graphWidth * posEnd
      const barWidth = Math.max(1, xEnd - xStart)
      const halfBarWidth = barWidth / 2

      // Barra LEFT (metà sinistra - viola)
      const barHeightLeft = normalizedLeft * graphHeight
      const yTopLeft = height - padding.bottom - barHeightLeft

      const gradientLeft = ctx.createLinearGradient(xStart, yTopLeft, xStart, height - padding.bottom)
      gradientLeft.addColorStop(0, '#a855f7')
      gradientLeft.addColorStop(0.5, '#8b5cf6')
      gradientLeft.addColorStop(1, '#7c3aed')

      ctx.fillStyle = gradientLeft
      ctx.fillRect(
        xStart,
        yTopLeft,
        Math.max(1, halfBarWidth - 0.5),
        barHeightLeft
      )

      // Barra RIGHT (metà destra - blu)
      const barHeightRight = normalizedRight * graphHeight
      const yTopRight = height - padding.bottom - barHeightRight

      const gradientRight = ctx.createLinearGradient(xStart + halfBarWidth, yTopRight, xStart + halfBarWidth, height - padding.bottom)
      gradientRight.addColorStop(0, '#3b82f6')
      gradientRight.addColorStop(0.5, '#2563eb')
      gradientRight.addColorStop(1, '#1d4ed8')

      ctx.fillStyle = gradientRight
      ctx.fillRect(
        xStart + halfBarWidth,
        yTopRight,
        Math.max(1, halfBarWidth - 0.5),
        barHeightRight
      )

      // Peak markers
      if (peakValuesLeft[i] > 0.01) {
        const peakYLeft = height - padding.bottom - (peakValuesLeft[i] * graphHeight)
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 1.5
        ctx.beginPath()
        ctx.moveTo(xStart, peakYLeft)
        ctx.lineTo(xStart + halfBarWidth - 0.5, peakYLeft)
        ctx.stroke()
      }

      if (peakValuesRight[i] > 0.01) {
        const peakYRight = height - padding.bottom - (peakValuesRight[i] * graphHeight)
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 1.5
        ctx.beginPath()
        ctx.moveTo(xStart + halfBarWidth, peakYRight)
        ctx.lineTo(xStart + barWidth - 0.5, peakYRight)
        ctx.stroke()
      }
    }
  } else if (displayMode.value === 'curve') {
    // Disegna due curve separate (L e R)
    const pointsLeft: Array<{ x: number; y: number }> = []
    const pointsRight: Array<{ x: number; y: number }> = []
    const peakPointsLeft: Array<{ x: number; y: number }> = []
    const peakPointsRight: Array<{ x: number; y: number }> = []

    for (let i = 0; i < bands.length; i++) {
      const band = bands[i]
      
      // Canale LEFT
      let maxDbLeft = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataLeft.length; bin++) {
        maxDbLeft = Math.max(maxDbLeft, fftDataLeft[bin])
      }
      const normalizedLeft = Math.max(0, Math.min(1, (maxDbLeft + 140) / 140))

      // Canale RIGHT
      let maxDbRight = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataRight.length; bin++) {
        maxDbRight = Math.max(maxDbRight, fftDataRight[bin])
      }
      const normalizedRight = Math.max(0, Math.min(1, (maxDbRight + 140) / 140))

      // Aggiorna peak hold
      const maxNormalized = Math.max(normalizedLeft, normalizedRight)
      if (maxNormalized > peakValuesLeft[i]) {
        peakValuesLeft[i] = normalizedLeft
        peakValuesRight[i] = normalizedRight
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesLeft[i] = Math.max(0, peakValuesLeft[i] - PEAK_DECAY_RATE)
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      // Usa la frequenza centrale della banda
      const freqCenter = Math.sqrt(band.freqStart * band.freqEnd)
      const x = padding.left + graphWidth * freqToPosition(freqCenter)
      const yLeft = height - padding.bottom - (normalizedLeft * graphHeight)
      const yRight = height - padding.bottom - (normalizedRight * graphHeight)

      pointsLeft.push({ x, y: yLeft })
      pointsRight.push({ x, y: yRight })

      // Aggiungi punti peak
      if (peakValuesLeft[i] > 0.01) {
        const peakYLeft = height - padding.bottom - (peakValuesLeft[i] * graphHeight)
        peakPointsLeft.push({ x, y: peakYLeft })
      }
      if (peakValuesRight[i] > 0.01) {
        const peakYRight = height - padding.bottom - (peakValuesRight[i] * graphHeight)
        peakPointsRight.push({ x, y: peakYRight })
      }
    }

    // Disegna curva LEFT (viola)
    if (pointsLeft.length > 0) {
      // Area sotto la curva
      ctx.fillStyle = 'rgba(168, 85, 247, 0.15)'
      ctx.beginPath()
      ctx.moveTo(padding.left, height - padding.bottom)
      ctx.lineTo(pointsLeft[0].x, pointsLeft[0].y)

      for (let i = 0; i < pointsLeft.length - 1; i++) {
        const current = pointsLeft[i]
        const next = pointsLeft[i + 1]
        const xMid = (current.x + next.x) / 2
        const yMid = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xMid, yMid)
      }

      const lastLeft = pointsLeft[pointsLeft.length - 1]
      ctx.lineTo(lastLeft.x, lastLeft.y)
      ctx.lineTo(padding.left + graphWidth, height - padding.bottom)
      ctx.closePath()
      ctx.fill()

      // Linea della curva
      ctx.strokeStyle = '#a855f7'
      ctx.lineWidth = 2.5
      ctx.beginPath()
      ctx.moveTo(pointsLeft[0].x, pointsLeft[0].y)

      for (let i = 0; i < pointsLeft.length - 1; i++) {
        const current = pointsLeft[i]
        const next = pointsLeft[i + 1]
        const xMid = (current.x + next.x) / 2
        const yMid = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xMid, yMid)
      }

      ctx.lineTo(lastLeft.x, lastLeft.y)
      ctx.stroke()
    }

    // Disegna curva RIGHT (blu)
    if (pointsRight.length > 0) {
      // Area sotto la curva
      ctx.fillStyle = 'rgba(59, 130, 246, 0.15)'
      ctx.beginPath()
      ctx.moveTo(padding.left, height - padding.bottom)
      ctx.lineTo(pointsRight[0].x, pointsRight[0].y)

      for (let i = 0; i < pointsRight.length - 1; i++) {
        const current = pointsRight[i]
        const next = pointsRight[i + 1]
        const xMid = (current.x + next.x) / 2
        const yMid = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xMid, yMid)
      }

      const lastRight = pointsRight[pointsRight.length - 1]
      ctx.lineTo(lastRight.x, lastRight.y)
      ctx.lineTo(padding.left + graphWidth, height - padding.bottom)
      ctx.closePath()
      ctx.fill()

      // Linea della curva
      ctx.strokeStyle = '#3b82f6'
      ctx.lineWidth = 2.5
      ctx.beginPath()
      ctx.moveTo(pointsRight[0].x, pointsRight[0].y)

      for (let i = 0; i < pointsRight.length - 1; i++) {
        const current = pointsRight[i]
        const next = pointsRight[i + 1]
        const xMid = (current.x + next.x) / 2
        const yMid = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xMid, yMid)
      }

      ctx.lineTo(lastRight.x, lastRight.y)
      ctx.stroke()
    }

    // Disegna curve peak LEFT
    if (peakPointsLeft.length > 1) {
      ctx.strokeStyle = '#fbbf24'
      ctx.lineWidth = 1.5
      ctx.globalAlpha = 0.8
      ctx.beginPath()
      ctx.moveTo(peakPointsLeft[0].x, peakPointsLeft[0].y)

      for (let i = 0; i < peakPointsLeft.length - 1; i++) {
        const current = peakPointsLeft[i]
        const next = peakPointsLeft[i + 1]
        const xMid = (current.x + next.x) / 2
        const yMid = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xMid, yMid)
      }

      const lastPeakLeft = peakPointsLeft[peakPointsLeft.length - 1]
      ctx.lineTo(lastPeakLeft.x, lastPeakLeft.y)
      ctx.stroke()
      ctx.globalAlpha = 1
    }

    // Disegna curve peak RIGHT
    if (peakPointsRight.length > 1) {
      ctx.strokeStyle = '#fbbf24'
      ctx.lineWidth = 1.5
      ctx.globalAlpha = 0.8
      ctx.beginPath()
      ctx.moveTo(peakPointsRight[0].x, peakPointsRight[0].y)

      for (let i = 0; i < peakPointsRight.length - 1; i++) {
        const current = peakPointsRight[i]
        const next = peakPointsRight[i + 1]
        const xMid = (current.x + next.x) / 2
        const yMid = (current.y + next.y) / 2
        ctx.quadraticCurveTo(current.x, current.y, xMid, yMid)
      }

      const lastPeakRight = peakPointsRight[peakPointsRight.length - 1]
      ctx.lineTo(lastPeakRight.x, lastPeakRight.y)
      ctx.stroke()
      ctx.globalAlpha = 1
    }
  } else if (displayMode.value === 'mirror') {
    // Disegna mirror (L in alto, R specchiato in basso)
    const centerY = height / 2
    const halfGraphHeight = graphHeight / 2

    for (let i = 0; i < bands.length; i++) {
      const band = bands[i]
      
      // Canale LEFT
      let maxDbLeft = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataLeft.length; bin++) {
        maxDbLeft = Math.max(maxDbLeft, fftDataLeft[bin])
      }
      const normalizedLeft = Math.max(0, Math.min(1, (maxDbLeft + 140) / 140))

      // Canale RIGHT
      let maxDbRight = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataRight.length; bin++) {
        maxDbRight = Math.max(maxDbRight, fftDataRight[bin])
      }
      const normalizedRight = Math.max(0, Math.min(1, (maxDbRight + 140) / 140))

      // Aggiorna peak hold
      const maxNormalized = Math.max(normalizedLeft, normalizedRight)
      if (maxNormalized > peakValuesLeft[i]) {
        peakValuesLeft[i] = normalizedLeft
        peakValuesRight[i] = normalizedRight
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesLeft[i] = Math.max(0, peakValuesLeft[i] - PEAK_DECAY_RATE)
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      const posStart = freqToPosition(band.freqStart)
      const posEnd = freqToPosition(band.freqEnd)

      const xStart = padding.left + graphWidth * posStart
      const xEnd = padding.left + graphWidth * posEnd
      const barWidth = Math.max(1, xEnd - xStart)

      // Barra LEFT superiore (viola) - cresce verso l'alto da centerY
      const barHeightLeft = normalizedLeft * halfGraphHeight
      const yTopLeft = centerY - barHeightLeft

      const gradientLeft = ctx.createLinearGradient(xStart, yTopLeft, xStart, centerY)
      gradientLeft.addColorStop(0, '#a855f7')
      gradientLeft.addColorStop(0.5, '#8b5cf6')
      gradientLeft.addColorStop(1, '#7c3aed')

      ctx.fillStyle = gradientLeft
      ctx.fillRect(
        xStart,
        yTopLeft,
        Math.max(1, barWidth - 1),
        barHeightLeft
      )

      // Barra RIGHT inferiore (blu) - cresce verso il basso da centerY
      const barHeightRight = normalizedRight * halfGraphHeight

      const gradientRight = ctx.createLinearGradient(xStart, centerY, xStart, centerY + barHeightRight)
      gradientRight.addColorStop(0, '#3b82f6')
      gradientRight.addColorStop(0.5, '#2563eb')
      gradientRight.addColorStop(1, '#1d4ed8')

      ctx.fillStyle = gradientRight
      ctx.fillRect(
        xStart,
        centerY,
        Math.max(1, barWidth - 1),
        barHeightRight
      )

      // Peak markers
      if (peakValuesLeft[i] > 0.01) {
        const peakYLeft = centerY - (peakValuesLeft[i] * halfGraphHeight)
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 1.5
        ctx.beginPath()
        ctx.moveTo(xStart, peakYLeft)
        ctx.lineTo(xStart + barWidth - 1, peakYLeft)
        ctx.stroke()
      }

      if (peakValuesRight[i] > 0.01) {
        const peakYRight = centerY + (peakValuesRight[i] * halfGraphHeight)
        ctx.strokeStyle = '#fbbf24'
        ctx.lineWidth = 1.5
        ctx.beginPath()
        ctx.moveTo(xStart, peakYRight)
        ctx.lineTo(xStart + barWidth - 1, peakYRight)
        ctx.stroke()
      }
    }

    // Linea centrale
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.3)'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(padding.left, centerY)
    ctx.lineTo(width - padding.right, centerY)
    ctx.stroke()
  } else if (displayMode.value === 'dots') {
    // Disegna dots stereo (L e R come punti)
    for (let i = 0; i < bands.length; i++) {
      const band = bands[i]
      
      // Canale LEFT
      let maxDbLeft = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataLeft.length; bin++) {
        maxDbLeft = Math.max(maxDbLeft, fftDataLeft[bin])
      }
      const normalizedLeft = Math.max(0, Math.min(1, (maxDbLeft + 140) / 140))

      // Canale RIGHT
      let maxDbRight = -140
      for (let bin = band.binStart; bin <= band.binEnd && bin < fftDataRight.length; bin++) {
        maxDbRight = Math.max(maxDbRight, fftDataRight[bin])
      }
      const normalizedRight = Math.max(0, Math.min(1, (maxDbRight + 140) / 140))

      // Aggiorna peak hold
      const maxNormalized = Math.max(normalizedLeft, normalizedRight)
      if (maxNormalized > peakValuesLeft[i]) {
        peakValuesLeft[i] = normalizedLeft
        peakValuesRight[i] = normalizedRight
        peakTimestamps[i] = currentTime
      } else {
        const timeSincePeak = currentTime - peakTimestamps[i]
        if (timeSincePeak > PEAK_HOLD_TIME) {
          peakValuesLeft[i] = Math.max(0, peakValuesLeft[i] - PEAK_DECAY_RATE)
          peakValuesRight[i] = Math.max(0, peakValuesRight[i] - PEAK_DECAY_RATE)
        }
      }

      const freqCenter = Math.sqrt(band.freqStart * band.freqEnd)
      const x = padding.left + graphWidth * freqToPosition(freqCenter)
      
      // Calcola dimensione punto basata sulla larghezza della banda
      const posStart = freqToPosition(band.freqStart)
      const posEnd = freqToPosition(band.freqEnd)
      const bandPixelWidth = graphWidth * (posEnd - posStart)
      const dotRadius = Math.min(Math.max(bandPixelWidth * 0.4, 2), 6)

      // Punto LEFT (viola)
      const yLeft = height - padding.bottom - (normalizedLeft * graphHeight)
      
      ctx.beginPath()
      ctx.arc(x - dotRadius * 0.6, yLeft, dotRadius, 0, Math.PI * 2)
      ctx.fillStyle = '#a855f7'
      ctx.fill()
      ctx.strokeStyle = '#7c3aed'
      ctx.lineWidth = 1
      ctx.stroke()

      // Punto RIGHT (blu)
      const yRight = height - padding.bottom - (normalizedRight * graphHeight)
      
      ctx.beginPath()
      ctx.arc(x + dotRadius * 0.6, yRight, dotRadius, 0, Math.PI * 2)
      ctx.fillStyle = '#3b82f6'
      ctx.fill()
      ctx.strokeStyle = '#1d4ed8'
      ctx.lineWidth = 1
      ctx.stroke()

      // Peak dots
      if (peakValuesLeft[i] > 0.01) {
        const peakYLeft = height - padding.bottom - (peakValuesLeft[i] * graphHeight)
        ctx.beginPath()
        ctx.arc(x - dotRadius * 0.6, peakYLeft, dotRadius * 0.6, 0, Math.PI * 2)
        ctx.fillStyle = '#fbbf24'
        ctx.fill()
      }

      if (peakValuesRight[i] > 0.01) {
        const peakYRight = height - padding.bottom - (peakValuesRight[i] * graphHeight)
        ctx.beginPath()
        ctx.arc(x + dotRadius * 0.6, peakYRight, dotRadius * 0.6, 0, Math.PI * 2)
        ctx.fillStyle = '#fbbf24'
        ctx.fill()
      }
    }
  }
}

// Lifecycle
onMounted(() => {
  connectAnalyser().then(() => render())
})

onUnmounted(() => {
  if (animationId !== null) {
    cancelAnimationFrame(animationId)
    animationId = null
  }

  if (currentMasterNode && splitNode) {
    try {
      currentMasterNode.disconnect(splitNode)
    } catch {
      // noop
    }
  }

  if (analyserLeft) {
    analyserLeft.dispose()
    analyserLeft = null
  }
  
  if (analyserRight) {
    analyserRight.dispose()
    analyserRight = null
  }
  
  if (splitNode) {
    splitNode.dispose()
    splitNode = null
  }
})
</script>