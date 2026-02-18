<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
      <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="close">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/80 backdrop-blur-sm"></div>
        
        <!-- Modal Content -->
        <div class="relative bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-gray-600 p-6 max-w-[95vw] w-full max-h-[90vh] overflow-y-auto shadow-2xl">
          
          <!-- Action Buttons - Top Right -->
          <div class="absolute top-4 right-4 flex gap-2 z-10">
            <button
              @click="reset"
              class="w-9 h-9 flex items-center justify-center rounded-full bg-gray-700/80 hover:bg-gray-600 text-gray-300 hover:text-white transition-colors text-sm font-medium"
              title="Reset filters"
            >
              R
            </button>
            <button
              @click="close"
              class="w-9 h-9 flex items-center justify-center rounded-full bg-gray-700/80 hover:bg-red-600 text-gray-300 hover:text-white transition-colors text-sm font-bold"
              title="Close"
            >
              X
            </button>
          </div>
          
          <!-- Header -->
          <div class="flex items-center justify-between mb-2">
            <div>
              <h2 class="text-lg font-bold text-blue-400">{{ titleText }}</h2>
            </div>
          </div>
          
          <!-- EQ Curve Display -->
          <div class="bg-gray-900 rounded-lg border border-gray-700 p-4 mb-4">
            <canvas
              ref="eqCanvas"
              class="w-full"
              @mousedown="handleCanvasMouseDown"
              @mousemove="handleCanvasMouseMove"
              @mouseup="handleCanvasMouseUp"
              @mouseleave="handleCanvasMouseUp"
            ></canvas>
          </div>
          
          <!-- Filters List -->
          <div class="flex flex-wrap gap-2 mb-4 max-h-[220px] overflow-y-auto">
            <div
              v-for="(filter, index) in filters"
              :key="filter.id"
              class="rounded-lg p-1.5 border-2 border-gray-700 w-[155px] flex-shrink-0 transition-colors"
              :class="draggedFilterIndex === index ? 'bg-gray-800/90' : 'bg-gray-900/50'"
              :style="{ borderColor: filter.color || filterColors[index % filterColors.length] }"
            >
              <div class="flex items-center justify-between mb-1">
                <div class="flex items-center gap-1.5">
                  <div 
                    class="w-2.5 h-2.5 rounded-full flex-shrink-0"
                    :style="{ backgroundColor: filter.color || filterColors[index % filterColors.length] }"
                  ></div>
                  <h3 class="text-[10px] font-bold text-gray-300">F{{ index + 1 }}</h3>
                </div>
                <button
                  @click="removeFilter(index)"
                  class="text-red-400 hover:text-red-300 text-xs font-bold leading-none"
                  title="Remove filter"
                >
                  ×
                </button>
              </div>
              
              <div class="flex flex-col gap-1">
                <!-- Type -->
                <div>
                  <label class="text-[9px] text-gray-400 block">Type</label>
                  <select
                    v-model="filter.type"
                    @change="createFilterChain()"
                    class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                  >
                    <option value="lowshelf">Low Shelf</option>
                    <option value="peaking">Peaking</option>
                    <option value="highshelf">High Shelf</option>
                    <option value="lowpass">Low Pass</option>
                    <option value="highpass">High Pass</option>
                  </select>
                </div>
                
                <!-- Frequency -->
                <div>
                  <label class="text-[9px] text-gray-400 block">Freq (Hz)</label>
                  <input
                    v-model.number="filter.frequency"
                    type="number"
                    min="20"
                    max="20000"
                    @input="updateFilterNode(index)"
                    class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                  />
                </div>
                
                <!-- Gain and Q in same row -->
                <div class="grid grid-cols-2 gap-1">
                  <!-- Gain -->
                  <div v-if="filter.type !== 'lowpass' && filter.type !== 'highpass'">
                    <label class="text-[9px] text-gray-400 block">Gain</label>
                    <input
                      v-model.number="filter.gain"
                      type="number"
                      min="-24"
                      max="24"
                      step="0.5"
                      @input="updateFilterNode(index)"
                      class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                    />
                  </div>
                  
                  <!-- Q -->
                  <div v-if="filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass'">
                    <label class="text-[9px] text-gray-400 block">Q</label>
                    <input
                      v-model.number="filter.Q"
                      type="number"
                      min="0.1"
                      max="20"
                      step="0.1"
                      @input="updateFilterNode(index)"
                      @blur="filter.Q = parseFloat(filter.Q.toFixed(2))"
                      class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                    />
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Add Filter Card -->
            <button
              @click="addFilter"
              class="bg-gray-900/30 hover:bg-gray-900/50 rounded-lg p-1.5 border-2 border-dashed border-gray-600 hover:border-blue-500 w-[155px] flex-shrink-0 flex items-center justify-center transition-colors group"
            >
              <span class="text-4xl text-gray-600 group-hover:text-blue-500 transition-colors">+</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed } from 'vue'
import { PeakingFilter } from '~/lib/filters/peaking.class'
import { LowShelvingFilter } from '~/lib/filters/lowShelving.class'
import { HighShelvingFilter } from '~/lib/filters/highShelving.class'

defineOptions({
  inheritAttrs: false
})

interface Props {
  modelValue: boolean
  trackNumber: number
  eqFilters?: any
  title?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'update', filters: any): void
}>()

const titleText = computed(() => props.title || `Parametric EQ - Track ${props.trackNumber}`)

interface EQFilter {
  id: number
  type: 'lowshelf' | 'peaking' | 'highshelf' | 'lowpass' | 'highpass'
  frequency: number
  gain: number
  Q: number
  color?: string
  node?: any
}

const eqCanvas = ref<HTMLCanvasElement | null>(null)

// Array of colors for filters
const filterColors = [
  '#FF6B6B', // Red
  '#4ECDC4', // Cyan
  '#45B7D1', // Blue
  '#FFA07A', // Light Salmon
  '#98D8C8', // Mint
  '#F7DC6F', // Yellow
  '#BB8FCE', // Purple
  '#85C1E2', // Sky Blue
  '#F8B739', // Orange
  '#52B788', // Green
]

// Initialize filters from prop or use defaults
const getInitialFilters = (): EQFilter[] => {
  if (props.eqFilters && props.eqFilters.length > 0) {
    return props.eqFilters.map((f: any, index: number) => ({
      id: index + 1,
      type: f.type,
      frequency: f.frequency,
      gain: f.gain,
      Q: f.Q,
      color: f.color || filterColors[index % filterColors.length]
    }))
  }
  return [
    { id: 1, type: 'lowshelf', frequency: 100, gain: 0, Q: 1, color: filterColors[0] },
    { id: 2, type: 'peaking', frequency: 1000, gain: 0, Q: 1, color: filterColors[1] },
    { id: 3, type: 'highshelf', frequency: 10000, gain: 0, Q: 1, color: filterColors[2] }
  ]
}

const filters = ref<EQFilter[]>(getInitialFilters())

let Tone: any = null
let filterNodes: any[] = []
let nextFilterId = filters.value.length + 1
let canvasRect: DOMRect | null = null

// Filter calculators
const peakingCalculator = new PeakingFilter()
const lowShelvingCalculator = new LowShelvingFilter()
const highShelvingCalculator = new HighShelvingFilter()

// Dragging state
const draggedFilterIndex = ref<number | null>(null)
const isDragging = ref(false)
const isDraggingQ = ref(false)
let dragStartX = 0
let dragStartQ = 0

onMounted(async () => {
  Tone = await import('tone')
  await nextTick()
  
  setupCanvas()
  // Don't create filters immediately to avoid AudioContext warnings
  // They will be created when user interacts or when modal is opened
  drawEQCurve()
  
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  disposeFilters()
})

function setupCanvas() {
  if (!eqCanvas.value) return
  
  const dpr = window.devicePixelRatio || 1
  const rect = eqCanvas.value.getBoundingClientRect()
  
  eqCanvas.value.width = rect.width * dpr
  eqCanvas.value.height = 450 * dpr
  eqCanvas.value.style.height = '450px'
  
  const ctx = eqCanvas.value.getContext('2d')
  if (ctx) {
    ctx.scale(dpr, dpr)
  }
  
  canvasRect = eqCanvas.value.getBoundingClientRect()
}

function handleResize() {
  setupCanvas()
  drawEQCurve()
}

function createFilterChain() {
  if (!Tone) return
  
  // Try to create filters - they will be created when AudioContext is ready
  try {
    // Dispose old filters
    disposeFilters()
    
    filterNodes = []
    
    filters.value.forEach((filter, index) => {
      const node = new Tone.Filter({
        type: filter.type,
        frequency: filter.frequency,
        Q: filter.Q,
        gain: filter.gain
      })
      
      filter.node = node
      filterNodes.push(node)
      
      // Connect in series
      if (index > 0) {
        filterNodes[index - 1].connect(node)
      }
    })
    
    // Emit the filter chain
    if (filterNodes.length > 0) {
      emit('update', {
        input: filterNodes[0],
        output: filterNodes[filterNodes.length - 1],
        filters: filterNodes,
        filtersData: filters.value // Include original filter data for thumbnail
      })
    }
  } catch (error) {
    // AudioContext not ready yet - filters will be created on next interaction
  }
  
  // Always draw the curve (visualization doesn't require AudioContext)
  drawEQCurve()
}

function disposeFilters() {
  filterNodes.forEach(node => {
    if (node) node.dispose()
  })
  filterNodes = []
}

function updateFilterNode(index: number) {
  const filter = filters.value[index]
  
  // If filter chain hasn't been created yet, create it first
  if (!filter.node || filterNodes.length === 0) {
    createFilterChain()
    return
  }
  
  // Update node parameters
  filter.node.frequency.value = filter.frequency
  if (filter.type !== 'lowpass' && filter.type !== 'highpass') {
    filter.node.gain.value = filter.gain
  }
  if (filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass') {
    filter.node.Q.value = filter.Q
  }
  
  // Emit update event for thumbnail
  emit('update', {
    input: filterNodes[0],
    output: filterNodes[filterNodes.length - 1],
    filters: filterNodes,
    filtersData: filters.value
  })
  
  drawEQCurve()
}

function addFilter() {
  const colorIndex = (filters.value.length) % filterColors.length
  filters.value.push({
    id: nextFilterId++,
    type: 'peaking',
    frequency: 1000,
    gain: 0,
    Q: 1,
    color: filterColors[colorIndex]
  })
  createFilterChain()
}

function removeFilter(index: number) {
  if (filters.value.length <= 1) return
  filters.value.splice(index, 1)
  createFilterChain()
}

function reset() {
  filters.value = [
    { id: nextFilterId++, type: 'lowshelf', frequency: 100, gain: 0, Q: 1, color: filterColors[0] },
    { id: nextFilterId++, type: 'peaking', frequency: 1000, gain: 0, Q: 1, color: filterColors[1] },
    { id: nextFilterId++, type: 'highshelf', frequency: 10000, gain: 0, Q: 1, color: filterColors[2] }
  ]
  createFilterChain()
}

// Canvas interaction
function handleCanvasMouseDown(e: MouseEvent) {
  if (!canvasRect) return
  
  // Ensure filters are created on first interaction
  if (filterNodes.length === 0) {
    createFilterChain()
  }
  
  const x = e.clientX - canvasRect.left
  const y = e.clientY - canvasRect.top
  
  // Check if we clicked on Q arrows
  const width = canvasRect.width
  const height = 450
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)
  
  for (let index = 0; index < filters.value.length; index++) {
    const filter = filters.value[index]
    
    // Only check for peaking, lowpass, highpass filters
    if (filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass') {
      const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
      const actualGain = calculateFilterGain(filter, filter.frequency) * -1
      const filterY = (actualGain * (height / 48)) + (height / 2)
      
      // Position arrows based on gain: above if positive, below if negative
      const arrowY = filterY < height / 2 ? filterY - 28 : filterY + 28
      const arrowSize = 10
      
      // Check if clicked in rectangular area between arrows (from left arrow to right arrow)
      const leftArrowX = filterX - 18 - arrowSize
      const rightArrowX = filterX + 18 + arrowSize
      const topY = arrowY - arrowSize
      const bottomY = arrowY + arrowSize
      
      if (x >= leftArrowX && x <= rightArrowX && y >= topY && y <= bottomY) {
        // Start dragging Q
        draggedFilterIndex.value = index
        isDraggingQ.value = true
        dragStartX = x
        dragStartQ = filter.Q
        return
      }
    }
  }
  
  // Find if we clicked near a filter point
  const filterIndex = findNearestFilter(x, y)
  
  if (filterIndex !== -1) {
    draggedFilterIndex.value = filterIndex
    isDragging.value = true
  }
}

function handleCanvasMouseMove(e: MouseEvent) {
  if (!canvasRect) return
  
  const x = e.clientX - canvasRect.left
  const y = e.clientY - canvasRect.top
  
  if (!isDragging.value && !isDraggingQ.value) {
    // Update cursor based on hover state
    const nearestFilter = findNearestFilter(x, y)
    const overQArea = isOverQArea(x, y)
    
    if (eqCanvas.value) {
      if (nearestFilter !== -1 || overQArea !== -1) {
        eqCanvas.value.style.cursor = 'pointer'
      } else {
        eqCanvas.value.style.cursor = 'crosshair'
      }
    }
    return
  }
  
  if (draggedFilterIndex.value === null) return
  
  const filter = filters.value[draggedFilterIndex.value]
  
  if (isDraggingQ.value) {
    // Dragging Q arrows - only change Q based on horizontal movement
    const deltaX = x - dragStartX
    const sensitivity = 0.05 // Q change per pixel
    const newQ = dragStartQ + (deltaX * sensitivity)
    filter.Q = parseFloat(Math.max(0.1, Math.min(20, newQ)).toFixed(2))
    
    // Update the filter node
    if (filter.node) {
      filter.node.Q.value = filter.Q
    }
  } else {
    // Dragging main point - change frequency and gain
    // Convert x position to frequency (log scale)
    const width = canvasRect.width
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
    filter.frequency = Math.max(20, Math.min(20000, Math.round(freq)))
    
    // Convert y position to gain
    const height = 450
    const gain = 24 - (y / height) * 48 // -24 to +24 dB
    filter.gain = Math.max(-24, Math.min(24, Math.round(gain * 2) / 2))
    
    // Update the filter node
    if (filter.node) {
      filter.node.frequency.value = filter.frequency
      if (filter.type !== 'lowpass' && filter.type !== 'highpass') {
        filter.node.gain.value = filter.gain
      }
    }
  }
  
  // Emit update event for real-time thumbnail update
  if (filterNodes.length > 0) {
    emit('update', {
      input: filterNodes[0],
      output: filterNodes[filterNodes.length - 1],
      filters: filterNodes,
      filtersData: filters.value
    })
  }
  
  drawEQCurve()
}

function handleCanvasMouseUp() {
  isDragging.value = false
  isDraggingQ.value = false
  draggedFilterIndex.value = null
}

// Check if mouse is over a Q control area
function isOverQArea(x: number, y: number): number {
  if (!canvasRect) return -1
  
  const width = canvasRect.width
  const height = 450
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)
  
  // Check from last to first (highest index first)
  for (let index = filters.value.length - 1; index >= 0; index--) {
    const filter = filters.value[index]
    
    if (filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass') {
      const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
      const actualGain = calculateFilterGain(filter, filter.frequency) * -1
      const filterY = (actualGain * (height / 48)) + (height / 2)
      const arrowY = filterY < height / 2 ? filterY - 28 : filterY + 28
      const arrowSize = 10
      
      const leftArrowX = filterX - 18 - arrowSize
      const rightArrowX = filterX + 18 + arrowSize
      const topY = arrowY - arrowSize
      const bottomY = arrowY + arrowSize
      
      if (x >= leftArrowX && x <= rightArrowX && y >= topY && y <= bottomY) {
        return index
      }
    }
  }
  
  return -1
}

function findNearestFilter(x: number, y: number): number {
  if (!canvasRect) return -1
  
  const width = canvasRect.width
  const height = 450
  const threshold = 20 // pixels
  
  let nearestIndex = -1
  let minDistance = threshold
  
  // Check from last to first to prioritize higher index filters
  for (let index = filters.value.length - 1; index >= 0; index--) {
    const filter = filters.value[index]
    
    // Convert filter frequency to x position
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const freqNormalized = (Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)
    const filterX = freqNormalized * width
    
    // Calculate actual gain at this frequency using the filter class
    const actualGain = calculateFilterGain(filter, filter.frequency) * -1
    const filterY = (actualGain * (height / 48)) + (height / 2)
    
    const distance = Math.sqrt(Math.pow(x - filterX, 2) + Math.pow(y - filterY, 2))
    
    // Use <= to prioritize higher index when distances are equal
    if (distance <= minDistance) {
      minDistance = distance
      nearestIndex = index
    }
  }
  
  return nearestIndex
}

function drawEQCurve() {
  if (!eqCanvas.value) return
  
  const ctx = eqCanvas.value.getContext('2d')
  if (!ctx) return
  
  const width = eqCanvas.value.getBoundingClientRect().width
  const height = 450
  
  // Clear canvas
  ctx.clearRect(0, 0, width, height)
  
  // Draw grid
  ctx.strokeStyle = '#374151'
  ctx.lineWidth = 1
  
  // Horizontal lines (dB)
  for (let db = -24; db <= 24; db += 6) {
    const y = height / 2 - (db / 48) * height
    ctx.beginPath()
    ctx.moveTo(0, y)
    ctx.lineTo(width, y)
    ctx.stroke()
    
    // Label
    ctx.fillStyle = '#6b7280'
    ctx.font = '10px monospace'
    ctx.fillText(`${db > 0 ? '+' : ''}${db}dB`, 5, y - 2)
  }
  
  // Vertical lines (frequency)
  const freqs = [100, 1000, 10000]
  freqs.forEach(freq => {
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const x = ((Math.log10(freq) - minFreq) / (maxFreq - minFreq)) * width
    ctx.beginPath()
    ctx.moveTo(x, 0)
    ctx.lineTo(x, height)
    ctx.stroke()
    
    ctx.fillStyle = '#6b7280'
    ctx.fillText(`${freq}Hz`, x + 2, height - 5)
  })
  
  // Draw 0dB line
  ctx.strokeStyle = '#4b5563'
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(0, height / 2)
  ctx.lineTo(width, height / 2)
  ctx.stroke()
  
  const points = 2000
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)
  
  // Draw individual filter areas (filled)
  filters.value.forEach((filter, filterIndex) => {
    const filterColor = filter.color || filterColors[filterIndex % filterColors.length]
    
    // Create filled area
    ctx.globalAlpha = 0.15
    ctx.fillStyle = filterColor
    ctx.beginPath()
    
    // Start from baseline
    ctx.moveTo(0, height / 2)
    
    // Draw the curve
    for (let i = 0; i <= points; i++) {
      const x = (i / points) * width
      const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
      
      // Calculate gain for this single filter
      const filterGain = calculateFilterGain(filter, freq) * -1
      const y = (filterGain * (height / 48)) + (height / 2)
      
      ctx.lineTo(x, y)
    }
    
    // Close to baseline
    ctx.lineTo(width, height / 2)
    ctx.closePath()
    ctx.fill()
  })
  
  // Draw individual filter curves
  filters.value.forEach((filter, filterIndex) => {
    ctx.strokeStyle = filter.color || filterColors[filterIndex % filterColors.length]
    ctx.lineWidth = 1.5
    ctx.lineCap = 'round'
    ctx.lineJoin = 'round'
    ctx.globalAlpha = 0.6
    ctx.beginPath()
    
    for (let i = 0; i < points; i++) {
      const x = (i / points) * width
      const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
      
      // Calculate gain for this single filter
      const filterGain = calculateFilterGain(filter, freq) * -1
      const y = (filterGain * (height / 48)) + (height / 2)
      
      if (i === 0) {
        ctx.moveTo(x, y)
      } else {
        ctx.lineTo(x, y)
      }
    }
    ctx.stroke()
  })
  
  // Reset alpha for convolution curve
  ctx.globalAlpha = 1.0
  
  // Draw convolution curve (combined frequency response)
  ctx.strokeStyle = '#FFFFFF'
  ctx.lineWidth = 2.5
  ctx.lineCap = 'round'
  ctx.lineJoin = 'round'
  ctx.beginPath()
  
  for (let i = 0; i < points; i++) {
    const x = (i / points) * width
    const freq = Math.pow(10, minFreq + (x / width) * (maxFreq - minFreq))
    
    // Calculate combined gain from all filters (convolution)
    let totalGain = 0
    filters.value.forEach(filter => {
      // Multiply by -1 as in the original implementation
      totalGain += (calculateFilterGain(filter, freq) * -1)
    })
    
    // Scale dB: value * (canvasHeight / 48) for -24 to +24 range
    const y = (totalGain * (height / 48)) + (height / 2)
    
    if (i === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  }
  ctx.stroke()
  
  // Draw filter points - first non-dragged, then dragged on top
  const drawFilterPoint = (filter: EQFilter, index: number) => {
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const x = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
    
    // Calculate actual gain at this frequency using the filter class
    const actualGain = calculateFilterGain(filter, filter.frequency) * -1
    const y = (actualGain * (height / 48)) + (height / 2)
    
    const filterColor = filter.color || filterColors[index % filterColors.length]
    const isDragged = draggedFilterIndex.value === index
    
    // Draw shadow/glow for depth
    ctx.save()
    ctx.shadowColor = isDragged ? filterColor : 'rgba(0, 0, 0, 0.5)'
    ctx.shadowBlur = isDragged ? 15 : 8
    ctx.shadowOffsetX = 0
    ctx.shadowOffsetY = 2
    
    // Draw outer circle with transparency
    ctx.globalAlpha = isDragged ? 0.95 : 0.85
    ctx.fillStyle = isDragged ? '#FFFFFF' : filterColor
    ctx.beginPath()
    ctx.arc(x, y, 14, 0, Math.PI * 2)
    ctx.fill()
    
    // Draw border
    ctx.shadowBlur = 0
    ctx.shadowOffsetX = 0
    ctx.shadowOffsetY = 0
    ctx.strokeStyle = filterColor
    ctx.lineWidth = isDragged ? 3 : 2
    ctx.globalAlpha = 1
    ctx.stroke()
    
    ctx.restore()
    
    // Draw filter number - perfectly centered
    ctx.save()
    ctx.globalAlpha = 1
    ctx.fillStyle = isDragged ? filterColor : '#000000'
    ctx.font = 'bold 13px Arial'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText((index + 1).toString(), x, y + 1)
    ctx.restore()
    
    // Draw Q control arrows (only for peaking, lowpass, highpass)
    if (filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass') {
      // Position arrows based on gain: above if positive (y < height/2), below if negative
      const arrowY = y < height / 2 ? y - 28 : y + 28
      const arrowSize = 10
      
      // Left arrow (decrease Q) - pointing left
      ctx.fillStyle = filterColor
      ctx.beginPath()
      ctx.moveTo(x - 18 - arrowSize, arrowY)
      ctx.lineTo(x - 18, arrowY - arrowSize/2)
      ctx.lineTo(x - 18, arrowY + arrowSize/2)
      ctx.closePath()
      ctx.fill()
      
      // Right arrow (increase Q) - pointing right
      ctx.beginPath()
      ctx.moveTo(x + 18 + arrowSize, arrowY)
      ctx.lineTo(x + 18, arrowY - arrowSize/2)
      ctx.lineTo(x + 18, arrowY + arrowSize/2)
      ctx.closePath()
      ctx.fill()
    }
  }
  
  // Draw all non-dragged filters first
  filters.value.forEach((filter, index) => {
    if (draggedFilterIndex.value !== index) {
      drawFilterPoint(filter, index)
    }
  })
  
  // Draw dragged filter last (on top)
  if (draggedFilterIndex.value !== null && filters.value[draggedFilterIndex.value]) {
    drawFilterPoint(filters.value[draggedFilterIndex.value], draggedFilterIndex.value)
  }
}

// Filter gain calculation using correct filter classes
function calculateFilterGain(filter: EQFilter, freq: number): number {
  const filterFrequency = filter.frequency
  const filterBoost = filter.gain
  const filterQ = filter.Q
  
  try {
    if (filter.type === 'peaking') {
      return peakingCalculator.computeResponseAtFrequency(freq, filterFrequency, filterBoost, filterQ)
    } else if (filter.type === 'lowshelf') {
      return lowShelvingCalculator.computeResponseAtFrequency(freq, filterFrequency, filterBoost, filterQ)
    } else if (filter.type === 'highshelf') {
      return highShelvingCalculator.computeResponseAtFrequency(freq, filterFrequency, filterBoost, filterQ)
    } else if (filter.type === 'lowpass' || filter.type === 'highpass') {
      // For now, return 0 for lowpass/highpass (not implemented in the classes)
      return 0
    }
  } catch (error) {
    console.warn('Error calculating filter gain:', error)
  }
  
  return 0
}

// Watch for filter changes (array length changes, type changes, etc.)
// Note: for parameter updates via input, we use updateFilterNode instead of recreating the chain
watch(() => filters.value.length, () => {
  // Recreate chain when filters are added/removed
  if (filterNodes.length > 0) {
    createFilterChain()
  }
})

// Watch for modal opening to create filters if needed
watch(() => props.modelValue, (isOpen) => {
  if (isOpen && Tone) {
    // Load filters from prop when modal opens (e.g., after loading a scene)
    if (props.eqFilters && props.eqFilters.length > 0) {
      filters.value = props.eqFilters.map((f: any, index: number) => ({
        id: index + 1,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color || filterColors[index % filterColors.length]
      }))
      nextFilterId = filters.value.length + 1
    }
    
    // Re-setup canvas when modal opens (in case it was hidden before)
    nextTick(() => {
      setupCanvas()
      drawEQCurve()
      
      // Create or recreate filter chain when modal opens
      createFilterChain()
    })
  }
})

function close() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
