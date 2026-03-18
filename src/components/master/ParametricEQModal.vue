<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
      <div v-if="modelValue" class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/80 backdrop-blur-sm" @click.self="close">
        <!-- Modal Content -->
        <div class="bg-gradient-to-br from-gray-900 via-gray-900 to-gray-800 rounded-xl shadow-2xl w-[98vw] max-w-[1800px] max-h-[92vh] border border-gray-700/50 overflow-hidden flex flex-col">
          
          <!-- Header -->
          <div class="flex items-center justify-between px-4 py-2 border-b border-gray-700/50 bg-gray-900/50 backdrop-blur">
            <div class="flex items-center gap-2">
              <div class="w-1 h-6 bg-blue-500 rounded-full"></div>
              <h2 class="text-base font-bold text-white tracking-tight">{{ titleText }}</h2>
            </div>
            <div class="flex items-center gap-2">
              <!-- FFT Display Mode Toggle -->
              <div class="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-gray-800 border border-gray-700">
                <span class="text-xs text-gray-400">FFT:</span>
                <button
                  @click="fftDisplayMode = 'curve'"
                  :class="[
                    'px-1.5 py-0.5 rounded text-xs font-medium transition-all',
                    fftDisplayMode === 'curve' 
                      ? 'bg-blue-600 text-white' 
                      : 'bg-transparent text-gray-400 hover:text-white'
                  ]"
                  title="Curve visualization"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12 Q 6 6, 9 12 T 15 12 Q 18 18, 21 12" />
                  </svg>
                </button>
                <button
                  @click="fftDisplayMode = 'bars'"
                  :class="[
                    'px-1.5 py-0.5 rounded text-xs font-medium transition-all',
                    fftDisplayMode === 'bars' 
                      ? 'bg-blue-600 text-white' 
                      : 'bg-transparent text-gray-400 hover:text-white'
                  ]"
                  title="Bars visualization"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                </button>
              </div>
              <!-- Preset Selection -->
              <div class="flex items-center gap-2">
                <label class="text-xs text-gray-400">Preset:</label>
                <select
                  v-model="selectedPreset"
                  @change="applyPreset"
                  class="px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-200 text-xs font-medium border border-gray-700 hover:border-gray-600 cursor-pointer transition-all focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Custom</option>
                  <option value="flat">Flat</option>
                  <option value="rock">Rock</option>
                  <option value="pop">Pop</option>
                  <option value="bass-enhanced">Bass Enhanced</option>
                  <option value="treble-boost">Treble Boost</option>
                  <option value="jazz">Jazz</option>
                  <option value="classical">Classical</option>
                  <option value="electronic">Electronic</option>
                  <option value="vocal">Vocal Boost</option>
                  <option value="dance">Dance</option>
                </select>
              </div>
              <button
                @click="reset"
                class="px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-200 hover:text-white transition-all text-xs font-medium border border-gray-700 hover:border-gray-600"
                title="Reset all filters"
              >
                <span class="flex items-center gap-1.5">
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  Reset
                </span>
              </button>
              <button
                v-if="!props.isDetached"
                @click="close"
                class="text-gray-400 hover:text-white transition-colors p-1.5 hover:bg-gray-800 rounded-lg"
                title="Close"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          
          <!-- Content -->
          <div class="flex-1 overflow-y-auto">
            <div class="p-4 space-y-4">
              <!-- EQ Curve Display -->
              <div class="bg-gradient-to-br from-gray-950 to-gray-900 rounded-xl border border-gray-700/50 shadow-inner relative overflow-hidden">
              <canvas
                ref="eqCanvas"
                class="w-full rounded-xl"
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
                  class="absolute pointer-events-none bg-gray-900/98 border border-gray-600 rounded-xl px-4 py-3 shadow-2xl z-50 backdrop-blur-sm"
                  :style="{
                    left: `${popoverPosition.x}px`,
                    top: `${popoverPosition.y}px`,
                    transform: `translate(70px, ${displayFilters[draggedFilterIndex].gain > 0 ? '-30%' : displayFilters[draggedFilterIndex].gain < 0 ? '-30%' : '-50%'})`
                  }"
                >
                  <div class="text-xs font-mono text-gray-200 space-y-1">
                    <div class="flex items-center gap-3">
                      <span class="text-gray-400 w-12">Freq:</span>
                      <span class="font-semibold text-white">{{ displayFilters[draggedFilterIndex].frequency }} Hz</span>
                    </div>
                    <div class="flex items-center gap-3">
                      <span class="text-gray-400 w-12">Gain:</span>
                      <span class="font-semibold text-white">{{ displayFilters[draggedFilterIndex].gain > 0 ? '+' : '' }}{{ displayFilters[draggedFilterIndex].gain }} dB</span>
                    </div>
                    <div class="flex items-center gap-3">
                      <span class="text-gray-400 w-12">Q:</span>
                      <span class="font-semibold text-white">{{ displayFilters[draggedFilterIndex].Q }}</span>
                    </div>
                  </div>
                </div>
              </Transition>
            </div>
            
            <!-- Filters Grid -->
            <div class="grid grid-cols-1 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 gap-2.5">
              <div
                v-for="(filter, index) in displayFilters"
                :key="filter.id"
                class="rounded-xl p-3 border transition-all hover:shadow-xl relative group"
                :class="draggedFilterIndex === index ? 'scale-[0.98] shadow-lg' : 'shadow-md hover:scale-[1.01]'"
                :style="{ 
                  borderColor: `${filter.color || filterColors[index % filterColors.length]}50`,
                  background: `linear-gradient(135deg, rgba(17, 24, 39, 0.95) 0%, rgba(31, 41, 55, 0.85) 50%, rgba(17, 24, 39, 0.95) 100%)`
                }"
              >
                <!-- Close button -->
                <button
                  v-if="!filter.isSystem"
                  @click="removeFilter(index)"
                  class="absolute -top-2 -right-2 w-6 h-6 rounded-full bg-red-500 hover:bg-red-600 text-white flex items-center justify-center shadow-lg transition-all opacity-0 group-hover:opacity-100 hover:scale-110"
                  title="Remove filter"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>

                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <div 
                      class="w-2.5 h-2.5 rounded-full flex-shrink-0 shadow-lg ring-2 ring-gray-950"
                      :style="{ 
                        backgroundColor: filter.color || filterColors[index % filterColors.length],
                        boxShadow: `0 0 12px ${filter.color || filterColors[index % filterColors.length]}60`
                      }"
                    ></div>
                    <h3 class="text-xs font-bold text-white tracking-wide">
                      {{ filter.isSystem ? 'HPF' : `Filter ${index + 1}` }}
                    </h3>
                    <span v-if="filter.isSystem" class="text-[8px] px-2 py-0.5 bg-blue-500/30 text-blue-300 rounded-full font-semibold border border-blue-500/30">
                      SYSTEM
                    </span>
                  </div>
                  
                  <!-- Filter Type Icons -->
                  <div v-if="!filter.isSystem" class="flex items-center gap-0.5 bg-gray-950/80 rounded-lg p-0.5 border border-gray-700/50">
                    <!-- Low Shelf -->
                    <button
                      @click="filter.type = 'lowshelf'; createFilterChain()"
                      :class="filter.type === 'lowshelf' ? 'bg-blue-500/30 text-blue-400' : 'text-gray-500 hover:text-gray-300'"
                      class="w-6 h-6 flex items-center justify-center rounded transition-all"
                      title="Low Shelf"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 18h8l4-8h6" stroke-linecap="round"/>
                      </svg>
                    </button>
                    <!-- Peaking -->
                    <button
                      @click="filter.type = 'peaking'; createFilterChain()"
                      :class="filter.type === 'peaking' ? 'bg-blue-500/30 text-blue-400' : 'text-gray-500 hover:text-gray-300'"
                      class="w-6 h-6 flex items-center justify-center rounded transition-all"
                      title="Peaking"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 18h4l4-8 4 8h6" stroke-linecap="round"/>
                      </svg>
                    </button>
                    <!-- High Shelf -->
                    <button
                      @click="filter.type = 'highshelf'; createFilterChain()"
                      :class="filter.type === 'highshelf' ? 'bg-blue-500/30 text-blue-400' : 'text-gray-500 hover:text-gray-300'"
                      class="w-6 h-6 flex items-center justify-center rounded transition-all"
                      title="High Shelf"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 10h6l4 8h8" stroke-linecap="round"/>
                      </svg>
                    </button>
                    <!-- Low Pass -->
                    <button
                      @click="filter.type = 'lowpass'; createFilterChain()"
                      :class="filter.type === 'lowpass' ? 'bg-blue-500/30 text-blue-400' : 'text-gray-500 hover:text-gray-300'"
                      class="w-6 h-6 flex items-center justify-center rounded transition-all"
                      title="Low Pass"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 12h8l6-6v12" stroke-linecap="round"/>
                      </svg>
                    </button>
                    <!-- High Pass -->
                    <button
                      @click="filter.type = 'highpass'; createFilterChain()"
                      :class="filter.type === 'highpass' ? 'bg-blue-500/30 text-blue-400' : 'text-gray-500 hover:text-gray-300'"
                      class="w-6 h-6 flex items-center justify-center rounded transition-all"
                      title="High Pass"
                    >
                      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M6 18v-12l6 6h9" stroke-linecap="round"/>
                      </svg>
                    </button>
                  </div>
                </div>
                
                <div class="flex items-center gap-2">
                  <!-- Frequency -->
                  <div class="flex-1">
                    <label class="text-[9px] text-gray-400 block mb-1 font-semibold tracking-wide">FREQ (Hz)</label>
                    <input
                      v-model.number="filter.frequency"
                      type="number"
                      min="20"
                      max="20000"
                      :disabled="filter.isSystem"
                      @input="updateFilterNode(index)"
                      class="w-full px-2 py-1.5 text-[10px] bg-gray-950/80 text-white rounded-lg border border-gray-700/50 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/30 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-all backdrop-blur-sm"
                    />
                  </div>
                  
                  <!-- Gain -->
                  <div v-if="filter.type !== 'lowpass' && filter.type !== 'highpass'" class="w-16">
                    <label class="text-[9px] text-gray-400 block mb-1 font-semibold tracking-wide">GAIN</label>
                    <input
                      v-model.number="filter.gain"
                      type="number"
                      min="-24"
                      max="24"
                      step="0.5"
                      :disabled="filter.isSystem"
                      @input="updateFilterNode(index)"
                      class="w-full px-2 py-1.5 text-[10px] bg-gray-950/80 text-white rounded-lg border border-gray-700/50 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/30 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-all backdrop-blur-sm"
                    />
                  </div>
                  
                  <!-- Q -->
                  <div v-if="filter.type === 'peaking' || filter.type === 'lowpass' || filter.type === 'highpass'" class="w-14">
                    <label class="text-[9px] text-gray-400 block mb-1 font-semibold tracking-wide">Q</label>
                    <input
                      v-model.number="filter.Q"
                      type="number"
                      min="0.1"
                      max="20"
                      step="0.1"
                      :disabled="filter.isSystem"
                      @input="updateFilterNode(index)"
                      @blur="filter.Q = parseFloat(filter.Q.toFixed(2))"
                      class="w-full px-2 py-1.5 text-[10px] bg-gray-950/80 text-white rounded-lg border border-gray-700/50 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/30 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-all backdrop-blur-sm"
                    />
                  </div>
                </div>
              </div>
              
              <!-- Add Filter Card -->
              <button
                @click="addFilter"
                class="bg-gradient-to-br from-gray-900/40 to-gray-800/30 hover:from-blue-900/30 hover:to-blue-800/20 rounded-xl p-3 border-2 border-dashed border-gray-600 hover:border-blue-500 flex flex-col items-center justify-center transition-all group min-h-[88px] hover:shadow-xl hover:shadow-blue-500/20 hover:scale-[1.02]"
              >
                <div class="w-10 h-10 rounded-full bg-gray-800/50 group-hover:bg-blue-500/20 flex items-center justify-center transition-all mb-1 group-hover:scale-110 border border-gray-700 group-hover:border-blue-500/50">
                  <svg class="w-5 h-5 text-gray-500 group-hover:text-blue-400 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
                  </svg>
                </div>
                <span class="text-[10px] font-semibold text-gray-500 group-hover:text-blue-400 transition-colors tracking-wide">ADD FILTER</span>
              </button>
            </div>
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
  isDetached?: boolean  // Hide close button when opened in detached window
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
    { id: 2, type: 'peaking', frequency: 300, gain: 0, Q: 1, color: filterColors[1] },
    { id: 3, type: 'peaking', frequency: 1000, gain: 0, Q: 1, color: filterColors[2] },
    { id: 4, type: 'peaking', frequency: 2000, gain: 0, Q: 1, color: filterColors[3] },
    { id: 5, type: 'highshelf', frequency: 10000, gain: 0, Q: 1, color: filterColors[5] }
  ]
}

const filters = ref<EQFilter[]>(getInitialFilters())

// EQ Presets loaded from backend
interface EQPresetFromBackend {
  name: string
  filters: Array<{
    filter_type: string
    frequency: number
    gain: number
    q: number
  }>
}

const eqPresetsFromBackend = ref<EQPresetFromBackend[]>([])

// Load presets from backend
async function loadEQPresetsFromBackend() {
  try {
    const response = await audioEngine.getEQPresets()
    if (response && response.presets) {
      eqPresetsFromBackend.value = response.presets
    }
  } catch (error) {
    console.error('[ParametricEQModal] Failed to load EQ presets from backend:', error)
  }
}

const selectedPreset = ref<string>('')

async function applyPreset() {
  if (!selectedPreset.value || selectedPreset.value === '') {
    return
  }
  
  try {
    // Apply preset on backend - updated filters will arrive via ParametersChanged
    await audioEngine.applyEQPreset(selectedPreset.value)
  } catch (error) {
    console.error('[ParametricEQModal] Failed to apply preset:', error)
  }
}

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
let dragStartY = 0
let dragStartQ = 0
let dragStartFrequency = 0
let dragStartGain = 0

// FFT visualization mode
const fftDisplayMode = ref<'curve' | 'bars'>('curve') // 'curve' or 'bars'

// Load FFT display preference from localStorage
onMounted(() => {
  const savedMode = localStorage.getItem('fftDisplayMode')
  if (savedMode === 'bars' || savedMode === 'curve') {
    fftDisplayMode.value = savedMode
  }
})

// Save FFT display preference when changed
watch(fftDisplayMode, (newMode) => {
  localStorage.setItem('fftDisplayMode', newMode)
})

// Watch for manual filter changes to detect if preset is still active
watch(filters, (newFilters) => {
  // If no preset is selected, nothing to check
  if (!selectedPreset.value || selectedPreset.value === '') return
  
  const preset = eqPresetsFromBackend.value.find(p => p.name === selectedPreset.value)
  if (!preset || preset.filters.length !== 5) return
  
  // Check if current filter values match the selected preset
  const matches = 
    newFilters[0]?.gain === preset.filters[0].gain &&
    newFilters[1]?.gain === preset.filters[1].gain &&
    newFilters[2]?.gain === preset.filters[2].gain &&
    newFilters[3]?.gain === preset.filters[3].gain &&
    newFilters[4]?.gain === preset.filters[4].gain
  
  // If they don't match, user modified manually - reset to Custom
  if (!matches) {
    selectedPreset.value = ''
  }
}, { deep: true })

// Throttle for emit updates during drag (200ms = max 5 updates/sec)
let lastEmitTime = 0
const EMIT_THROTTLE_MS = 400

// Throttle for FFT visual updates using requestAnimationFrame
let rafPending = false
let lastFFTDrawTime = 0
const MIN_FFT_DRAW_INTERVAL = 33 // ~30fps maximum for FFT visualization

// Throttle for drag visual updates using requestAnimationFrame
let dragRafPending = false
let pendingDragUpdate: (() => void) | null = null

onMounted(async () => {
  await nextTick()
  
  // Load EQ presets from backend
  await loadEQPresetsFromBackend()
  
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

// Throttled draw function for FFT updates - uses RAF to avoid excessive redraws
function scheduleFFTDraw() {
  // During drag, slightly reduce FFT update rate to prioritize drag smoothness
  const minInterval = MIN_FFT_DRAW_INTERVAL;
  
  // Only schedule if not already pending
  if (!rafPending) {
    rafPending = true
    requestAnimationFrame(() => {
      const now = performance.now()
      // Time-based throttling
      if (now - lastFFTDrawTime >= minInterval) {
        drawEQCurve()
        lastFFTDrawTime = now
      }
      rafPending = false
    })
  }
}

// Throttled draw function for drag updates - uses RAF for smooth 60fps dragging
function scheduleDragDraw(updateFn: () => void) {
  pendingDragUpdate = updateFn
  
  if (!dragRafPending) {
    dragRafPending = true
    requestAnimationFrame(() => {
      if (pendingDragUpdate) {
        pendingDragUpdate()
        pendingDragUpdate = null
      }
      dragRafPending = false
    })
  }
}

// Watch for FFT data updates from audio engine (only when modal is open)
watch(
  [
    () => props.modelValue, // Watch modal open state
    () => props.title && props.title.includes('Master') ? audioEngine?.state?.value?.fftData : audioEngine?.state?.value?.trackFFTData, // Watch entire object
    () => props.trackNumber // Watch track number changes
  ],
  ([isOpen, fftDataOrTrackFFT, trackNum]) => {
    // Only process FFT data when modal is open
    if (!isOpen) return
    
    const isMasterEQ = props.title && props.title.includes('Master')
    const fftData = isMasterEQ 
      ? fftDataOrTrackFFT 
      : (fftDataOrTrackFFT as any)?.[trackNum]
    
    if (!fftData) return
    
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
    
    // Use throttled RAF-based draw instead of immediate draw
    scheduleFFTDraw()
  }
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
    { id: nextFilterId++, type: 'peaking', frequency: 300, gain: 0, Q: 1, color: filterColors[1] },
    { id: nextFilterId++, type: 'peaking', frequency: 1000, gain: 0, Q: 1, color: filterColors[2] },
    { id: nextFilterId++, type: 'peaking', frequency: 2000, gain: 0, Q: 1, color: filterColors[3] },
    { id: nextFilterId++, type: 'highshelf', frequency: 10000, gain: 0, Q: 1, color: filterColors[5] }
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
    
    // Save initial drag state
    const filter = displayFilters.value[filterIndex]
    dragStartX = x
    dragStartY = y
    dragStartFrequency = filter.frequency
    dragStartGain = filter.gain
    
    // Set initial popover position
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
    // Dragging main point - use delta from start position
    const width = canvasRect.width
    const height = 450
    const minFreq = Math.log10(20)
    const maxFreq = Math.log10(20000)
    
    // Calculate delta in pixels
    const deltaX = x - dragStartX
    const deltaY = y - dragStartY
    
    // Convert deltaX to frequency change (log scale)
    // Each pixel represents a portion of the log frequency range
    const freqRatio = Math.pow(10, (deltaX / width) * (maxFreq - minFreq))
    const newFreq = dragStartFrequency * freqRatio
    filter.frequency = Math.max(20, Math.min(20000, Math.round(newFreq)))
    
    // Convert deltaY to gain change
    const gainChange = -(deltaY / height) * 48 // negative because Y increases downward
    const newGain = dragStartGain + gainChange
    filter.gain = Math.max(-24, Math.min(24, Math.round(newGain * 2) / 2))
  }
  
  // Schedule redraw using RAF for smooth 60fps dragging
  scheduleDragDraw(() => {
    drawEQCurve()
  })
  
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
    
    // Save initial drag state
    const filter = displayFilters.value[filterIndex]
    dragStartX = x
    dragStartY = y
    dragStartFrequency = filter.frequency
    dragStartGain = filter.gain
    
    // Set initial popover position
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
    
    // Schedule redraw using RAF
    scheduleDragDraw(() => {
      drawEQCurve()
    })
    
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
    
    // Calculate delta in pixels
    const deltaX = x - dragStartX
    const deltaY = y - dragStartY
    
    // Convert deltaX to frequency change (log scale)
    const freqRatio = Math.pow(10, (deltaX / width) * (maxFreq - minFreq))
    const newFreq = dragStartFrequency * freqRatio
    
    // Clamp frequency to valid range
    const clampedFreq = Math.max(20, Math.min(20000, newFreq))
    
    // Clamp gain based on filter type
    let newGain = 0
    if (filter.type === 'peaking' || filter.type === 'lowshelf' || filter.type === 'highshelf') {
      const gainChange = -(deltaY / height) * 48 // negative because Y increases downward
      newGain = dragStartGain + gainChange
      newGain = Math.max(-24, Math.min(24, newGain))
    }
    
    // Update filter parameters
    filters.value[draggedFilterIndex.value].frequency = clampedFreq
    if (filter.type === 'peaking' || filter.type === 'lowshelf' || filter.type === 'highshelf') {
      filters.value[draggedFilterIndex.value].gain = newGain
    }
    
    // Schedule redraw using RAF
    scheduleDragDraw(() => {
      drawEQCurve()
    })
    
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
  
  // Draw grid
  ctx.strokeStyle = '#374151'
  ctx.lineWidth = 1
  
  // Horizontal lines (dB) - skip top and bottom lines
  for (let db = -18; db <= 18; db += 6) {
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
  
  // Draw FFT curve over grid (if data available)
  if (smoothedFFTLeft && smoothedFFTRight) {
    const CALIBRATION_OFFSET_DB = -25.0
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

    // Only draw FFT curves in curve mode (bars will be drawn later on top)
    if (fftDisplayMode.value === 'curve') {
      // ==== CURVE MODE: Draw smooth filled area and curves ====
      
      // Adaptive number of points based on canvas width (reduce for performance)
      // At high sample rates, use fewer points since we're already bandwidth-limited
      const sampleRate = getCurrentSampleRate()
      const basePoints = Math.min(300, Math.floor(width / 2)) // Scale with canvas width
      const numPoints = sampleRate > 96000 ? Math.floor(basePoints * 0.6) : basePoints
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
      
      // Draw the curve with smooth bezier curves (average of both channels)
      const points: Array<{x: number, y: number}> = []
      for (let i = 0; i <= numPoints; i++) {
        const logFreq = logMin + (i / numPoints) * (logMax - logMin)
        const freq = Math.pow(10, logFreq)
        const dbLeft = getDbAtFreq(freq, fftDbLeft)
        const dbRight = getDbAtFreq(freq, fftDbRight)
        // Use average of both channels
        const dbAvg = (dbLeft + dbRight) / 2
        
        // Map dB to Y position (from -100dB to +6dB range, same as spectrum analyzer)
        const normalized = Math.max(0, Math.min(1, (dbAvg + 100) / 106))
        const x = freqToX(freq)
        const y = height - (normalized * height)
        
        points.push({x, y})
      }
      
      // Draw smooth curve using quadratic bezier
      if (points.length > 0) {
        ctx.lineTo(points[0].x, points[0].y)
        for (let i = 0; i < points.length - 1; i++) {
          const xc = (points[i].x + points[i + 1].x) / 2
          const yc = (points[i].y + points[i + 1].y) / 2
          ctx.quadraticCurveTo(points[i].x, points[i].y, xc, yc)
        }
        // Draw last segment
        if (points.length > 1) {
          const last = points[points.length - 1]
          ctx.quadraticCurveTo(points[points.length - 2].x, points[points.length - 2].y, last.x, last.y)
        }
      }
      
      // Close to bottom right
      ctx.lineTo(width, height)
      ctx.closePath()
      ctx.fill()
      
      ctx.globalAlpha = 1.0
    } // End of curve mode
  } // End of FFT data available
  
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
  
  // ==== BARS MODE: Draw FFT bars on top of EQ curves ====
  // This is done after filter curves so bars are visible in foreground
  if (fftDisplayMode.value === 'bars' && smoothedFFTLeft && smoothedFFTRight) {
    const CALIBRATION_OFFSET_DB = -25.0
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

    const minFreq = 20
    const maxFreq = 20000
    const logMin = Math.log10(minFreq)
    const logMax = Math.log10(maxFreq)
    
    const numBars = Math.min(150, Math.floor(width / 4)) // Adaptive number of bars
    const barWidth = width / numBars
    
    for (let i = 0; i < numBars; i++) {
      const logFreq = logMin + (i / numBars) * (logMax - logMin)
      const freq = Math.pow(10, logFreq)
      const dbLeft = getDbAtFreq(freq, fftDbLeft)
      const dbRight = getDbAtFreq(freq, fftDbRight)
      const dbAvg = (dbLeft + dbRight) / 2
      
      // Map dB to bar height - using full FFT range for visibility
      const normalized = Math.max(0, Math.min(1, (dbAvg + 100) / 106))
      const barHeight = normalized * height
      
      const x = freqToX(freq)
      
      // White bars with subtle gradient (same as curve fill)
      const barColor = 'rgba(255, 255, 255, 0.12)'
      const barColorLight = 'rgba(255, 255, 255, 0.15)'
      const barColorDark = 'rgba(255, 255, 255, 0.08)'
      
      // Draw bar with gradient and rounded top corners
      const gradient = ctx.createLinearGradient(0, height, 0, height - barHeight)
      gradient.addColorStop(0, barColorDark)
      gradient.addColorStop(0.5, barColor)
      gradient.addColorStop(1, barColorLight)
      
      const barX = x - barWidth / 2
      const barY = height - barHeight
      const barW = barWidth * 0.5
      const radius = Math.min(1.5, barW / 2) // Border radius for top corners
      
      // Draw rounded rectangle (rounded only at top)
      ctx.fillStyle = gradient
      ctx.beginPath()
      ctx.moveTo(barX, height) // Bottom left
      ctx.lineTo(barX, barY + radius) // Left side
      ctx.arcTo(barX, barY, barX + radius, barY, radius) // Top left corner
      ctx.lineTo(barX + barW - radius, barY) // Top side
      ctx.arcTo(barX + barW, barY, barX + barW, barY + radius, radius) // Top right corner
      ctx.lineTo(barX + barW, height) // Right side
      ctx.closePath()
      ctx.fill()
      
      // Subtle white outline with rounded corners
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)'
      ctx.lineWidth = 0.5
      ctx.stroke()
    }
  }
  
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
