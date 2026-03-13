<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
      <div v-if="modelValue" class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/70" @click.self="close">
        <!-- Modal Content -->
        <div class="bg-gray-900 rounded-lg shadow-2xl max-w-[95vw] w-full max-h-[90vh] border border-gray-700 overflow-y-auto">
          
          <!-- Header -->
          <div class="flex items-center justify-between p-4 border-b border-gray-700">
            <h2 class="text-lg font-semibold text-white">{{ titleText }}</h2>
            <div class="flex items-center gap-2">
              <button
                @click="reset"
                class="px-3 py-1.5 rounded bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white transition-colors text-sm font-medium"
                title="Reset filters"
              >
                Reset
              </button>
              <button
                @click="close"
                class="text-gray-400 hover:text-white transition-colors"
                title="Close"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          
          <!-- Content -->
          <div class="p-6 space-y-4">
            <!-- EQ Curve Display -->
            <div class="bg-gray-900 rounded-lg border border-gray-700 p-4 relative">
            <canvas
              ref="eqCanvas"
              class="w-full"
              @mousedown="handleCanvasMouseDown"
              @mousemove="handleCanvasMouseMove"
              @mouseup="handleCanvasMouseUp"
              @mouseleave="handleCanvasMouseUp"
              @touchstart.prevent="handleCanvasTouchStart"
              @touchmove.prevent="handleCanvasTouchMove"
              @touchend.prevent="handleCanvasTouchEnd"
              @touchcancel.prevent="handleCanvasTouchEnd"
            ></canvas>
            
            <!-- Drag Popover -->
            <Transition name="fade">
              <div
                v-if="draggedFilterIndex !== null && popoverPosition.x > 0"
                class="absolute pointer-events-none bg-gray-800/95 border border-gray-600 rounded-lg px-3 py-2 shadow-xl z-50"
                :style="{
                  left: `${popoverPosition.x}px`,
                  top: `${popoverPosition.y}px`,
                  transform: `translate(70px, ${displayFilters[draggedFilterIndex].gain > 0 ? '-30%' : displayFilters[draggedFilterIndex].gain < 0 ? '-30%' : '-50%'})`
                }"
              >
                <div class="text-[11px] font-mono text-gray-200 space-y-0.5">
                  <div class="flex items-center gap-2">
                    <span class="text-gray-400">Freq:</span>
                    <span class="font-semibold">{{ displayFilters[draggedFilterIndex].frequency }} Hz</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-gray-400">Gain:</span>
                    <span class="font-semibold">{{ displayFilters[draggedFilterIndex].gain > 0 ? '+' : '' }}{{ displayFilters[draggedFilterIndex].gain }} dB</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-gray-400">Q:</span>
                    <span class="font-semibold">{{ displayFilters[draggedFilterIndex].Q }}</span>
                  </div>
                </div>
              </div>
            </Transition>
          </div>
          
          <!-- Filters List -->
          <div class="flex flex-wrap gap-2 mb-4 max-h-[220px] overflow-y-auto">
            <div
              v-for="(filter, index) in displayFilters"
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
                  <h3 class="text-[10px] font-bold text-gray-300">
                    {{ filter.isSystem ? 'HPF' : `F${index + 1}` }}
                  </h3>
                  <span v-if="filter.isSystem" class="text-[8px] px-1 py-0.5 bg-blue-600/30 text-blue-400 rounded">
                    System
                  </span>
                </div>
                <button
                  v-if="!filter.isSystem"
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
                    :disabled="filter.isSystem"
                    class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
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
                    :disabled="filter.isSystem"
                    @input="updateFilterNode(index)"
                    class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
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
                      :disabled="filter.isSystem"
                      @input="updateFilterNode(index)"
                      class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
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
                      :disabled="filter.isSystem"
                      @input="updateFilterNode(index)"
                      @blur="filter.Q = parseFloat(filter.Q.toFixed(2))"
                      class="w-full px-1.5 py-0.5 text-[10px] bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
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
      </div>
    </Transition>
  </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed, inject } from 'vue'
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
  systemFilters?: any[]  // System filters (like HPF) - only for visualization, not editable
  title?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'update', filters: any): void
}>()

// Inject audio engine for FFT data
const audioEngine = inject<any>('audioEngine')

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

// FFT data for background visualization
let smoothedFFTLeft: Float32Array | null = null
let smoothedFFTRight: Float32Array | null = null
const SMOOTHING_FACTOR = 0.92
const ATTACK_FACTOR = 0.6

// Get current sample rate from audio engine
const getCurrentSampleRate = () => {
  return audioEngine?.state?.value?.performanceStats?.sampleRate || 
         audioEngine?.state?.value?.fftData?.sampleRate || 
         44100
}

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

// Combine user filters with system filters for visualization only
const displayFilters = computed(() => {
  const system = props.systemFilters || []
  return [...system, ...filters.value]
})

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
const popoverPosition = ref({ x: 0, y: 0 })
let dragStartX = 0
let dragStartQ = 0

// Throttle for emit updates during drag (200ms = max 5 updates/sec)
let lastEmitTime = 0
const EMIT_THROTTLE_MS = 400

onMounted(async () => {
  await nextTick()
  
  setupCanvas()
  // Don't create filters immediately to avoid AudioContext warnings
  // They will be created when user interacts or when modal is opened
  drawEQCurve()
  
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  // No cleanup needed - Rust backend handles all audio
})

// Watch for FFT data updates from audio engine
watch(
  () => audioEngine?.state?.value?.fftData,
  (fftData) => {
    if (fftData) {
      const newLeft = fftData.binsLeft instanceof Float32Array 
        ? fftData.binsLeft 
        : new Float32Array(fftData.binsLeft)
      const newRight = fftData.binsRight instanceof Float32Array 
        ? fftData.binsRight 
        : new Float32Array(fftData.binsRight)
      
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
      
      drawEQCurve() // Redraw with new FFT data
    }
  },
  { immediate: true }
)

// Watch for external changes to eqFilters (e.g., when loading a scene)
// Watch for external changes to eqFilters (user filters only)
watch(() => props.eqFilters, (newFilters) => {
  // Don't update while user is actively dragging a filter on THIS client
  if (isDragging.value || isDraggingQ.value) return
  
  // Only sync if we have actual filter data
  if (!newFilters || newFilters.length === 0) return
  
  // Check if different from current filters
  const isDifferent = newFilters.length !== filters.value.length || 
    newFilters.some((nf: any, i: number) => {
      const cf = filters.value[i]
      return !cf || 
        nf.type !== cf.type || 
        Math.abs(nf.frequency - cf.frequency) > 0.1 || 
        Math.abs(nf.gain - cf.gain) > 0.01 || 
        Math.abs(nf.Q - cf.Q) > 0.01
    })
  
  if (!isDifferent) return
  
  // Sync user filters from external source
  filters.value = newFilters.map((f: any, index: number) => ({
    id: index + 1,
    type: f.type,
    frequency: f.frequency,
    gain: f.gain,
    Q: f.Q,
    color: f.color || filterColors[index % filterColors.length]
  }))
  
  // Redraw curve with new data
  nextTick(() => {
    drawEQCurve()
  })
}, { immediate: true })

// Watch for system filters changes (only affects visualization)
watch(() => props.systemFilters, () => {
  // Just redraw the curve, don't rebuild the filter chain
  drawEQCurve()
}, { deep: true })

// Also sync when modal opens (for immediate visual update)
watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    // Reload filters from props when modal opens (to get latest sync from other clients)
    if (props.eqFilters && props.eqFilters.length > 0) {
      filters.value = props.eqFilters.map((f: any, index: number) => ({
        id: index + 1,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color || filterColors[index % filterColors.length]
      }))
    }
    
    nextTick(() => {
      setupCanvas()
      drawEQCurve()
      // All audio processing handled by Rust backend
    })
  }
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
  // Always emit filter data for backend integration (Rust backend)
  emit('update', {
    input: null,  // No Web Audio nodes needed
    output: null,
    filters: [],  // No Web Audio filters needed
    filtersData: filters.value // Always send filter parameters
  })
  
  // Always draw the curve (visualization doesn't require AudioContext)
  drawEQCurve()
}

function updateFilterNode(displayIndex: number) {
  const filter = displayFilters.value[displayIndex]
  
  // Skip system filters - they cannot be edited
  if (filter?.isSystem) return
  
  // Calculate the index in the user filters array
  const systemFiltersCount = (props.systemFilters || []).length
  const userFilterIndex = displayIndex - systemFiltersCount
  
  if (userFilterIndex < 0 || userFilterIndex >= filters.value.length) return
  
  emit('update', {
    input: null,
    output: null,
    filters: [],
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

function removeFilter(displayIndex: number) {
  const filter = displayFilters.value[displayIndex]
  
  // Cannot remove system filters
  if (filter?.isSystem) return
  
  // If this is the last user filter, keep at least one
  if (filters.value.length <= 1) return
  
  // Calculate the index in the user filters array
  const systemFiltersCount = (props.systemFilters || []).length
  const userFilterIndex = displayIndex - systemFiltersCount
  
  if (userFilterIndex >= 0 && userFilterIndex < filters.value.length) {
    filters.value.splice(userFilterIndex, 1)
    createFilterChain()
  }
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
  
  // All audio processing handled by Rust backend - just handle UI
  
  const x = e.clientX - canvasRect.left
  const y = e.clientY - canvasRect.top
  
  // Check if we clicked on Q arrows
  const width = canvasRect.width
  const height = 450
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)
  
  for (let index = 0; index < displayFilters.value.length; index++) {
    const filter = displayFilters.value[index]
    
    // Skip system filters - no Q control for them
    if (filter.isSystem) continue
    
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
        
        // Set initial popover position
        popoverPosition.value = { x: filterX, y: filterY }
        return
      }
    }
  }
  
  // Find if we clicked near a filter point
  const filterIndex = findNearestFilter(x, y)
  
  if (filterIndex !== -1) {
    draggedFilterIndex.value = filterIndex
    isDragging.value = true
    
    // Set initial popover position
    const filter = displayFilters.value[filterIndex]
    const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
    const actualGain = calculateFilterGain(filter, filter.frequency) * -1
    const filterY = (actualGain * (height / 48)) + (height / 2)
    popoverPosition.value = { x: filterX, y: filterY }
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
  
  // Update popover position during drag - calculate from filter position, not mouse position
  if (draggedFilterIndex.value !== null) {
    const filter = displayFilters.value[draggedFilterIndex.value]
    const width = canvasRect.width
    const height = 450
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    
    // Calculate x from filter frequency
    const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
    
    // Calculate y from filter gain
    const actualGain = calculateFilterGain(filter, filter.frequency) * -1
    const filterY = (actualGain * (height / 48)) + (height / 2)
    
    popoverPosition.value = { x: filterX, y: filterY }
  }
  
  if (draggedFilterIndex.value === null) return
  
  const filter = displayFilters.value[draggedFilterIndex.value]
  
  // Don't allow dragging system filters
  if (filter.isSystem) {
    handleCanvasMouseUp()
    return
  }
  
  if (isDraggingQ.value) {
    // Dragging Q arrows - only change Q based on horizontal movement
    const deltaX = x - dragStartX
    const sensitivity = 0.05 // Q change per pixel
    const newQ = dragStartQ + (deltaX * sensitivity)
    filter.Q = parseFloat(Math.max(0.1, Math.min(20, newQ)).toFixed(2))
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
  }
  
  // Redraw immediately for smooth visual
  drawEQCurve()
  
  // Throttle emit to reduce backend load
  const now = Date.now()
  if (now - lastEmitTime >= EMIT_THROTTLE_MS) {
    lastEmitTime = now
    emit('update', {
      input: null,
      output: null,
      filters: [],
      filtersData: filters.value
    })
  }
}

function handleCanvasMouseUp() {
  // Emit final update when dragging ends
  if (draggedFilterIndex.value !== null) {
    emit('update', {
      input: null,
      output: null,
      filters: [],
      filtersData: filters.value
    })
  }
  
  isDragging.value = false
  isDraggingQ.value = false
  draggedFilterIndex.value = null
  popoverPosition.value = { x: 0, y: 0 }
  lastEmitTime = 0
}

// Touch handlers
function handleCanvasTouchStart(e: TouchEvent) {
  if (!canvasRect || e.touches.length === 0) return
  
  const touch = e.touches[0]
  const x = touch.clientX - canvasRect.left
  const y = touch.clientY - canvasRect.top
  
  // Check if we touched on Q arrows
  const width = canvasRect.width
  const height = 450
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)
  
  for (let index = 0; index < displayFilters.value.length; index++) {
    const filter = displayFilters.value[index]
    
    // Skip system filters - no Q control for them
    if (filter.isSystem) continue
    
    // Only check for peaking, lowpass, highpass filters
    if (filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass') {
      const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
      const actualGain = calculateFilterGain(filter, filter.frequency) * -1
      const filterY = (actualGain * (height / 48)) + (height / 2)
      
      // Position arrows based on gain: above if positive, below if negative
      const arrowY = filterY < height / 2 ? filterY - 28 : filterY + 28
      const arrowSize = 10
      
      // Check if touched in rectangular area between arrows (from left arrow to right arrow)
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
        
        // Set initial popover position
        popoverPosition.value = { x: filterX, y: filterY }
        return
      }
    }
  }
  
  // Find if we touched near a filter point
  const filterIndex = findNearestFilter(x, y)
  
  if (filterIndex !== -1) {
    draggedFilterIndex.value = filterIndex
    isDragging.value = true
    
    // Set initial popover position
    const filter = displayFilters.value[filterIndex]
    const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
    const actualGain = calculateFilterGain(filter, filter.frequency) * -1
    const filterY = (actualGain * (height / 48)) + (height / 2)
    popoverPosition.value = { x: filterX, y: filterY }
  }
}

function handleCanvasTouchMove(e: TouchEvent) {
  if (!canvasRect || e.touches.length === 0) return
  
  const touch = e.touches[0]
  const x = touch.clientX - canvasRect.left
  const y = touch.clientY - canvasRect.top
  
  if (!isDragging.value && !isDraggingQ.value) {
    return
  }
  
  // Update popover position during drag
  if (draggedFilterIndex.value !== null) {
    const filter = displayFilters.value[draggedFilterIndex.value]
    const width = canvasRect.width
    const height = 450
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    
    // Calculate x from filter frequency
    const filterX = ((Math.log10(filter.frequency) - minFreq) / (maxFreq - minFreq)) * width
    
    // Calculate y from filter gain
    const actualGain = calculateFilterGain(filter, filter.frequency) * -1
    const filterY = (actualGain * (height / 48)) + (height / 2)
    
    popoverPosition.value = { x: filterX, y: filterY }
  }
  
  if (isDraggingQ.value && draggedFilterIndex.value !== null) {
    // Dragging Q value horizontally
    const deltaX = x - dragStartX
    const sensitivity = 0.01
    const newQ = Math.max(0.1, Math.min(10, dragStartQ + deltaX * sensitivity))
    
    filters.value[draggedFilterIndex.value].Q = newQ
    
    drawEQCurve()
    
    drawEQCurve()
    
    // Throttle emit
    const now = Date.now()
    if (now - lastEmitTime >= EMIT_THROTTLE_MS) {
      lastEmitTime = now
      emit('update', {
        input: null,
        output: null,
        filters: [],
        filtersData: filters.value
      })
    }
  } else if (isDragging.value && draggedFilterIndex.value !== null) {
    const filter = displayFilters.value[draggedFilterIndex.value]
    
    // Don't allow dragging system filters
    if (filter.isSystem) return
    
    const width = canvasRect.width
    const height = 450
    
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const freqRatio = (x / width) * (maxFreq - minFreq) + minFreq
    const newFreq = Math.pow(10, freqRatio)
    
    // Clamp frequency to valid range
    const clampedFreq = Math.max(20, Math.min(20000, newFreq))
    
    // Clamp gain based on filter type
    let newGain = 0
    if (filter.type === 'peaking' || filter.type === 'lowshelf' || filter.type === 'highshelf') {
      const gainRange = 24 // ±24 dB
      newGain = 24 - (y / height) * 48 // -24 to +24 dB (same as mouse handler)
      newGain = Math.max(-gainRange, Math.min(gainRange, newGain))
    }
    
    // Update filter parameters
    filters.value[draggedFilterIndex.value].frequency = clampedFreq
    if (filter.type === 'peaking' || filter.type === 'lowshelf' || filter.type === 'highshelf') {
      filters.value[draggedFilterIndex.value].gain = newGain
    }
    
    drawEQCurve()
    
    // Throttle emit
    const now = Date.now()
    if (now - lastEmitTime >= EMIT_THROTTLE_MS) {
      lastEmitTime = now
      emit('update', {
        input: null,
        output: null,
        filters: [],
        filtersData: filters.value
      })
    }
  }
}

function handleCanvasTouchEnd() {
  // Emit final update when dragging ends
  if (draggedFilterIndex.value !== null) {
    // Always emit final update for Rust backend
    emit('update', {
      input: null,
      output: null,
      filters: [],
      filtersData: filters.value
    })
  }
  
  isDragging.value = false
  isDraggingQ.value = false
  draggedFilterIndex.value = null
  popoverPosition.value = { x: 0, y: 0 }
  lastEmitTime = 0
}

// Check if mouse is over a Q control area
function isOverQArea(x: number, y: number): number {
  if (!canvasRect) return -1
  
  const width = canvasRect.width
  const height = 450
  const minFreq = Math.log10(20)
  const maxFreq = Math.log10(20000)
  
  // Check from last to first (highest index first)
  // Only check user filters (not system filters)
  for (let index = displayFilters.value.length - 1; index >= 0; index--) {
    const filter = displayFilters.value[index]
    
    // Skip system filters
    if (filter.isSystem) continue
    
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
  // Only consider user filters (not system filters)
  for (let index = displayFilters.value.length - 1; index >= 0; index--) {
    const filter = displayFilters.value[index]
    
    // Skip system filters - they cannot be dragged
    if (filter.isSystem) continue
    
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
  
  // Draw FFT curve in background (if data available)
  if (smoothedFFTLeft && smoothedFFTRight) {
    const CALIBRATION_OFFSET_DB = -18.0
    const convertToDb = (magnitude: number): number => {
      if (magnitude <= 0) return -140
      return 20 * Math.log10(magnitude) + CALIBRATION_OFFSET_DB
    }

    const fftDbLeft = Array.from(smoothedFFTLeft).map(convertToDb)
    const fftDbRight = Array.from(smoothedFFTRight).map(convertToDb)

    // Helper to convert frequency to X position (logarithmic)
    const freqToX = (freq: number): number => {
      const minFreq = Math.log10(20)
      const maxFreq = Math.log10(20000)
      const clamped = Math.max(20, Math.min(20000, freq))
      const logFreq = Math.log10(clamped)
      return ((logFreq - minFreq) / (maxFreq - minFreq)) * width
    }

    // Helper to get dB at specific frequency from FFT data
    const getDbAtFreq = (freq: number, fftDb: number[]): number => {
      const nyquist = getCurrentSampleRate() / 2
      const bin = Math.floor((freq / nyquist) * fftDb.length)
      const clampedBin = Math.max(0, Math.min(bin, fftDb.length - 1))
      return fftDb[clampedBin]
    }

    // Draw curve with many points for smoothness
    const numPoints = 500
    const minFreq = 20
    const maxFreq = 20000
    const logMin = Math.log10(minFreq)
    const logMax = Math.log10(maxFreq)

    // Draw average of both channels as filled area (white opaque)
    ctx.globalAlpha = 0.12
    ctx.fillStyle = 'rgba(255, 255, 255, 1)'
    ctx.beginPath()
    
    // Start from bottom left
    ctx.moveTo(0, height)
    
    // Draw the curve (average of left and right)
    for (let i = 0; i <= numPoints; i++) {
      const logFreq = logMin + (i / numPoints) * (logMax - logMin)
      const freq = Math.pow(10, logFreq)
      const dbLeft = getDbAtFreq(freq, fftDbLeft)
      const dbRight = getDbAtFreq(freq, fftDbRight)
      const dbAvg = (dbLeft + dbRight) / 2
      
      // Map dB to Y position (from -100dB to +6dB range)
      const normalized = Math.max(0, Math.min(1, (dbAvg + 100) / 106))
      const x = freqToX(freq)
      const y = height - (normalized * height)
      
      ctx.lineTo(x, y)
    }
    
    // Close to bottom right
    ctx.lineTo(width, height)
    ctx.closePath()
    ctx.fill()

    // Draw left channel curve outline (purple)
    ctx.globalAlpha = 0.5
    ctx.strokeStyle = '#a855f7'
    ctx.lineWidth = 1.5
    ctx.beginPath()
    
    for (let i = 0; i <= numPoints; i++) {
      const logFreq = logMin + (i / numPoints) * (logMax - logMin)
      const freq = Math.pow(10, logFreq)
      const db = getDbAtFreq(freq, fftDbLeft)
      
      // Map dB to Y position (from -100dB to +6dB range)
      const normalized = Math.max(0, Math.min(1, (db + 100) / 106))
      const x = freqToX(freq)
      const y = height - (normalized * height)
      
      if (i === 0) {
        ctx.moveTo(x, y)
      } else {
        ctx.lineTo(x, y)
      }
    }
    ctx.stroke()

    // Draw right channel curve outline (blue)
    ctx.strokeStyle = '#3b82f6'
    ctx.lineWidth = 1.5
    ctx.beginPath()
    
    for (let i = 0; i <= numPoints; i++) {
      const logFreq = logMin + (i / numPoints) * (logMax - logMin)
      const freq = Math.pow(10, logFreq)
      const db = getDbAtFreq(freq, fftDbRight)
      
      const normalized = Math.max(0, Math.min(1, (db + 100) / 106))
      const x = freqToX(freq)
      const y = height - (normalized * height)
      
      if (i === 0) {
        ctx.moveTo(x, y)
      } else {
        ctx.lineTo(x, y)
      }
    }
    ctx.stroke()
    
    ctx.globalAlpha = 1.0
  }
  
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
  const freqs = [30, 40, 50, 60, 80, 100, 200, 300, 400, 500, 600, 800, 1000, 2000, 4000, 6000, 8000, 10000, 20000]
  freqs.forEach(freq => {
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    const x = ((Math.log10(freq) - minFreq) / (maxFreq - minFreq)) * width
    ctx.beginPath()
    ctx.moveTo(x, 0)
    ctx.lineTo(x, height)
    ctx.stroke()
    
    ctx.fillStyle = '#6b7280'
    const label = freq >= 1000 ? `${freq/1000}k` : `${freq}`
    ctx.fillText(`${label}Hz`, x + 2, height - 5)
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
  displayFilters.value.forEach((filter, filterIndex) => {
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
  displayFilters.value.forEach((filter, filterIndex) => {
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
    displayFilters.value.forEach(filter => {
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
  displayFilters.value.forEach((filter, index) => {
    if (draggedFilterIndex.value !== index) {
      drawFilterPoint(filter, index)
    }
  })
  
  // Draw dragged filter last (on top)
  if (draggedFilterIndex.value !== null && displayFilters.value[draggedFilterIndex.value]) {
    drawFilterPoint(displayFilters.value[draggedFilterIndex.value], draggedFilterIndex.value)
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
    } else if (filter.type === 'highpass') {
      // Highpass filter: second order biquad
      // |H(ω)|² = (ω/ω0)⁴ / [((ω/ω0)² - 1)² + (ω/ω0)²/Q²]
      const w = freq / filterFrequency // normalized frequency
      const w2 = w * w
      const w4 = w2 * w2
      const numerator = w4
      const denominator = Math.pow(w2 - 1, 2) + (w2 / (filterQ * filterQ))
      const magnitudeSquared = numerator / denominator
      // Return gain in dB: 10 * log10(|H(ω)|²)
      return 10 * Math.log10(Math.max(magnitudeSquared, 1e-10))
    } else if (filter.type === 'lowpass') {
      // Lowpass filter: second order biquad
      // |H(ω)|² = 1 / [((ω/ω0)² - 1)² + (ω/ω0)²/Q²]
      const w = freq / filterFrequency // normalized frequency
      const w2 = w * w
      const numerator = 1
      const denominator = Math.pow(w2 - 1, 2) + (w2 / (filterQ * filterQ))
      const magnitudeSquared = numerator / denominator
      // Return gain in dB: 10 * log10(|H(ω)|²)
      return 10 * Math.log10(Math.max(magnitudeSquared, 1e-10))
    }
  } catch (error) {
    console.warn('Error calculating filter gain:', error)
  }
  
  return 0
}

// All audio processing handled by Rust backend - these watches are no longer needed

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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
