<template>
  <div 
    class="flex flex-col h-full gap-2 relative transition-all duration-300 ease-out rounded-lg overflow-hidden"
    :style="{ width: sectionWidth + 'px' }"
  >
    <!-- Collapse/Expand Button -->
    <button
      @click="toggleCollapse"
      class="absolute top-2 w-5 h-5 -translate-x-1/2 bg-gray-800 hover:bg-blue-600 border border-gray-700 hover:border-blue-500 rounded flex items-center justify-center transition-all shadow-lg z-20"
      :class="{ 'left-1/2' : isCollapsed, 'left-3' : !isCollapsed }"
      :title="isCollapsed ? 'Expand Meters Panel' : 'Collapse Meters Panel'"
    >
      <svg class="w-3 h-3 text-gray-300 transition-transform" :class="{ 'rotate-180': !isCollapsed }" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
        </svg>
    </button>

    <!-- Meters Container -->
    <div class="flex-1 p-0 overflow-y-auto flex flex-col" :class="{ 'space-y-2': !isCollapsed, 'space-y-1': isCollapsed }">
      <!-- Draggable Meter Cards -->
      <div 
        v-for="slot in slotComponents" 
        :key="slot.id"
        class="meter-card bg-gradient-to-b to-gray-900 rounded-lg border-2 p-2 relative group" 
        :class="[
          slot.gradientFrom,
          slot.borderColor,
          { 'flex-1 min-h-0': isCollapsed },
          { 'cursor-pointer hover:opacity-90': isCollapsed }
        ]"
        :style="getDragStyles(slot.id)"
        @click="isCollapsed ? openMeterPopover(slot.id, $event) : null"
        @dragover="handleDragOver($event, slot.id)"
        @drop="handleDrop($event, slot.id)"
      >
        <!-- Drag Handle (only visible when not collapsed) -->
        <div 
          v-if="!isCollapsed"
          class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <div
            draggable="true"
            @dragstart="handleDragStart(slot.id, $event)"
            @dragend="handleDragEnd"
            class="drag-handle bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1"
            :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }"
          >
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
              <path d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z" />
            </svg>
            <span>{{ slot.name }}</span>
          </div>
        </div>

        <!-- Meter Component -->
        <component 
          :is="getMeterComponent(slot.id)"
          v-show="audioEngine?.state.value.isRunning"
          :collapsed="isCollapsed"
          v-bind="getMeterProps(slot.id)"
          @reset="getMeterResetFn(slot.id)"
        />
        
        <!-- Engine not running message -->
        <div v-show="!audioEngine?.state.value.isRunning" class="flex items-center justify-center" :class="{ 'h-full': isCollapsed, 'py-8': !isCollapsed }">
          <div class="text-xs text-gray-500 text-center" :class="{ 'transform -rotate-90': isCollapsed }">
            <div class="text-[0.6rem]">Loading...</div>
            <div v-if="!isCollapsed">Audio engine<br/>not running</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Popover for single meter (when collapsed) -->
    <Teleport to="body">
      <div 
        v-if="popoverMeter !== null"
        class="fixed z-[9999] bg-gray-900 border-2 rounded-lg shadow-2xl p-3 w-64"
        :class="{
          'border-purple-600/70': popoverMeter === 'loudness',
          'border-orange-600/70': popoverMeter === 'dynamicRange',
          'border-cyan-600/70': popoverMeter === 'phaseCorrelation',
          'border-pink-600/70': popoverMeter === 'stereoWidth',
          'border-amber-600/70': popoverMeter === 'headroom'
        }"
        :style="{ left: popoverPosition.x + 'px', top: popoverPosition.y + 'px' }"
      >
        <!-- Close button -->
        <button 
          @click="closePopover"
          class="absolute -top-4 -right-3 w-6 h-6 bg-gray-800 hover:bg-red-600 rounded-full flex items-center justify-center text-gray-400 hover:text-white transition-colors shadow-lg border border-gray-700"
          title="Close"
        >
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
          </svg>
        </button>

        <!-- Meter content -->
        <LoudnessMeter 
          v-if="popoverMeter === 'loudness'"
          :collapsed="false"
          :momentary-lufs="loudnessData?.momentaryLufs"
          :short-term-lufs="loudnessData?.shortTermLufs"
          :integrated-lufs="loudnessData?.integratedLufs"
          :loudness-range-lu="loudnessData?.loudnessRangeLu"
          :true-peak-dbtp="loudnessData?.truePeakDbtp"
          @reset="resetLoudness"
        />

        <DynamicRangeMeter 
          v-if="popoverMeter === 'dynamicRange'"
          :collapsed="false"
          :peak-db-l="dynamicRangeData?.peakDbL"
          :peak-db-r="dynamicRangeData?.peakDbR"
          :rms-db-l="dynamicRangeData?.rmsDbL"
          :rms-db-r="dynamicRangeData?.rmsDbR"
          :dynamic-range-l="dynamicRangeData?.dynamicRangeL"
          :dynamic-range-r="dynamicRangeData?.dynamicRangeR"
          :dynamic-range-stereo="dynamicRangeData?.dynamicRangeStereo"
          @reset="resetDynamicRange"
        />

        <MasterPhaseCorrelationMeter 
          v-if="popoverMeter === 'phaseCorrelation'"
          :collapsed="false"
          :correlation="phaseCorrelationData?.correlation"
          :mono-compatible="phaseCorrelationData?.monoCompatible"
          @reset="resetPhaseCorrelation"
        />

        <StereoWidthMeter 
          v-if="popoverMeter === 'stereoWidth'"
          :collapsed="false"
          :width-percent="stereoWidthData?.widthPercent"
          :mid-rms="stereoWidthData?.midRms"
          :side-rms="stereoWidthData?.sideRms"
          :balance="stereoWidthData?.balance"
          @reset="resetStereoWidth"
        />

        <HeadroomMeter 
          v-if="popoverMeter === 'headroom'"
          :collapsed="false"
          :peak-l="headroomData?.peakL"
          :peak-r="headroomData?.peakR"
          :headroom-l="headroomData?.headroomL"
          :headroom-r="headroomData?.headroomR"
          :headroom-stereo="headroomData?.headroomStereo"
          @reset="resetHeadroom"
        />
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, watch, onMounted, onUnmounted } from 'vue'
import LoudnessMeter from './master/LoudnessMeter.vue'
import DynamicRangeMeter from './master/DynamicRangeMeter.vue'
import MasterPhaseCorrelationMeter from './master/MasterPhaseCorrelationMeter.vue'
import StereoWidthMeter from './master/StereoWidthMeter.vue'
import HeadroomMeter from './master/HeadroomMeter.vue'

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Collapse state
const isCollapsed = ref(true) // Start collapsed
const expandedWidth = 220
const collapsedWidth = 20

// Popover state
type MeterType = 'loudness' | 'dynamicRange' | 'phaseCorrelation' | 'stereoWidth' | 'headroom' | null
const popoverMeter = ref<MeterType>(null)
const popoverPosition = ref({ x: 0, y: 0 })

// Slot Components with drag-and-drop support
interface SlotComponent {
  id: Exclude<MeterType, null>
  name: string
  borderColor: string
  gradientFrom: string
}

const defaultSlotComponents: SlotComponent[] = [
  { id: 'loudness', name: 'LUFS', borderColor: 'border-purple-600/50', gradientFrom: 'from-purple-900/20' },
  { id: 'dynamicRange', name: 'Dynamic Range', borderColor: 'border-orange-600/50', gradientFrom: 'from-orange-900/20' },
  { id: 'phaseCorrelation', name: 'Phase Correlation', borderColor: 'border-cyan-600/50', gradientFrom: 'from-cyan-900/20' },
  { id: 'stereoWidth', name: 'Stereo Width', borderColor: 'border-pink-600/50', gradientFrom: 'from-pink-900/20' },
  { id: 'headroom', name: 'Headroom', borderColor: 'border-amber-600/50', gradientFrom: 'from-amber-900/20' }
]

const slotComponents = ref<SlotComponent[]>([...defaultSlotComponents])

// Drag and drop state
const draggedComponent = ref<Exclude<MeterType, null> | null>(null)
const dragOverComponent = ref<Exclude<MeterType, null> | null>(null)

const sectionWidth = computed(() => {
  return isCollapsed.value ? collapsedWidth : expandedWidth
})

// Loudness data
const loudnessData = computed(() => audioEngine?.state.value.loudnessData)

// Dynamic Range data
const dynamicRangeData = computed(() => audioEngine?.state.value.dynamicRangeData)

// Phase Correlation data
const phaseCorrelationData = computed(() => audioEngine?.state.value.phaseCorrelationData)

// Stereo Width data
const stereoWidthData = computed(() => audioEngine?.state.value.stereoWidthData)

// Headroom data
const headroomData = computed(() => audioEngine?.state.value.headroomData)

// Note: All meter data is now pushed automatically from the audio engine
// every ~50ms via the Response::Levels stream. No polling is needed.
// The computed properties above are reactive and will update automatically.

// Toggle collapse/expand
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
  // Close popover when expanding section
  if (!isCollapsed.value) {
    popoverMeter.value = null
  }
}

// Open meter in popover
function openMeterPopover(meter: MeterType, event: MouseEvent) {
  if (!isCollapsed.value) return // Only when collapsed
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  
  // Estimate popover height based on meter type (approximate values)
  const popoverHeights: Record<Exclude<MeterType, null>, number> = {
    loudness: 400,
    dynamicRange: 350,
    phaseCorrelation: 280,
    stereoWidth: 380,
    headroom: 330
  }
  
  const estimatedHeight = meter ? popoverHeights[meter] : 300
  let yPosition = rect.top
  
  // Check if popover would overflow below viewport
  if (yPosition + estimatedHeight > window.innerHeight) {
    // Adjust to align bottom with viewport bottom (with 10px margin)
    yPosition = window.innerHeight - estimatedHeight + 20
    // Ensure it doesn't go above viewport top
    if (yPosition < 10) yPosition = 10
  }
  
  popoverPosition.value = {
    x: rect.left - 256 - 10, // 10px to the left (256px = w-64)
    y: yPosition
  }
  popoverMeter.value = meter
}

// Close popover
function closePopover() {
  popoverMeter.value = null
}

// Reset loudness measurements
function resetLoudness() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetLoudness()
  }
}

// Reset dynamic range measurements
function resetDynamicRange() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetDynamicRange()
  }
}

// Reset phase correlation measurements
function resetPhaseCorrelation() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetPhaseCorrelation()
  }
}

// Reset stereo width measurements
function resetStereoWidth() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetStereoWidth()
  }
}

// Reset headroom measurements
function resetHeadroom() {
  if (audioEngine?.state.value.isRunning) {
    audioEngine.resetHeadroom()
  }
}

// Format LUFS values
const formatLufs = (lufs: number | null | undefined): string => {
  if (lufs == null || !isFinite(lufs) || lufs < -90) {
    return '-∞'
  }
  return lufs.toFixed(1)
}

// ============================================
// Drag and Drop Functions
// ============================================

function handleDragStart(slotId: Exclude<MeterType, null>, event: DragEvent) {
  draggedComponent.value = slotId
  
  // Set custom drag image to show the whole component instead of just the handle
  const componentElement = (event.target as HTMLElement).closest('.meter-card')
  if (componentElement) {
    const rect = componentElement.getBoundingClientRect()
    // Calculate the offset from where the user clicked relative to the component
    const offsetX = event.clientX - rect.left
    const offsetY = event.clientY - rect.top
    event.dataTransfer?.setDragImage(componentElement as HTMLElement, offsetX, offsetY)
  }
}

function handleDragOver(event: DragEvent, slotId: Exclude<MeterType, null>) {
  event.preventDefault()
  if (draggedComponent.value !== slotId) {
    dragOverComponent.value = slotId
  }
}

function handleDrop(event: DragEvent, targetId: Exclude<MeterType, null>) {
  event.preventDefault()
  
  if (!draggedComponent.value || draggedComponent.value === targetId) {
    return
  }

  const draggedIndex = slotComponents.value.findIndex(c => c.id === draggedComponent.value)
  const targetIndex = slotComponents.value.findIndex(c => c.id === targetId)

  if (draggedIndex !== -1 && targetIndex !== -1) {
    const newComponents = [...slotComponents.value]
    const [draggedItem] = newComponents.splice(draggedIndex, 1)
    newComponents.splice(targetIndex, 0, draggedItem)
    slotComponents.value = newComponents
  }
  
  draggedComponent.value = null
  dragOverComponent.value = null
}

function handleDragEnd() {
  draggedComponent.value = null
  dragOverComponent.value = null
}

function getDragStyles(slotId: Exclude<MeterType, null>) {
  if (!draggedComponent.value || !dragOverComponent.value) return {}

  const draggedIndex = slotComponents.value.findIndex(c => c.id === draggedComponent.value)
  const currentIndex = slotComponents.value.findIndex(c => c.id === slotId)
  const dragOverIndex = slotComponents.value.findIndex(c => c.id === dragOverComponent.value)

  // Skip the dragged element itself
  if (slotId === draggedComponent.value) {
    return {
      opacity: '0.5',
      transform: 'scale(0.98)',
      transition: 'all 0.2s ease'
    }
  }

  // Calculate if we need to shift this element
  if (draggedIndex < dragOverIndex) {
    // Dragging down: shift elements between dragged and dragOver down
    if (currentIndex > draggedIndex && currentIndex <= dragOverIndex) {
      return {
        transform: 'translateY(-2rem) scale(0.95)',
        transition: 'transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)',
        filter: 'brightness(0.85)'
      }
    }
  } else if (draggedIndex > dragOverIndex) {
    // Dragging up: shift elements between dragOver and dragged up
    if (currentIndex < draggedIndex && currentIndex >= dragOverIndex) {
      return {
        transform: 'translateY(2rem) scale(0.95)',
        transition: 'transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)',
        filter: 'brightness(0.85)'
      }
    }
  }

  return { transition: 'transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)' }
}

// ============================================
// Component Helper Functions
// ============================================

function getMeterComponent(slotId: Exclude<MeterType, null>) {
  const componentMap = {
    loudness: LoudnessMeter,
    dynamicRange: DynamicRangeMeter,
    phaseCorrelation: MasterPhaseCorrelationMeter,
    stereoWidth: StereoWidthMeter,
    headroom: HeadroomMeter
  }
  return componentMap[slotId]
}

function getMeterProps(slotId: Exclude<MeterType, null>) {
  const propsMap = {
    loudness: {
      momentaryLufs: loudnessData.value?.momentaryLufs,
      shortTermLufs: loudnessData.value?.shortTermLufs,
      integratedLufs: loudnessData.value?.integratedLufs,
      loudnessRangeLu: loudnessData.value?.loudnessRangeLu,
      truePeakDbtp: loudnessData.value?.truePeakDbtp
    },
    dynamicRange: {
      peakDbL: dynamicRangeData.value?.peakDbL,
      peakDbR: dynamicRangeData.value?.peakDbR,
      rmsDbL: dynamicRangeData.value?.rmsDbL,
      rmsDbR: dynamicRangeData.value?.rmsDbR,
      dynamicRangeL: dynamicRangeData.value?.dynamicRangeL,
      dynamicRangeR: dynamicRangeData.value?.dynamicRangeR,
      dynamicRangeStereo: dynamicRangeData.value?.dynamicRangeStereo
    },
    phaseCorrelation: {
      correlation: phaseCorrelationData.value?.correlation,
      monoCompatible: phaseCorrelationData.value?.monoCompatible
    },
    stereoWidth: {
      widthPercent: stereoWidthData.value?.widthPercent,
      midRms: stereoWidthData.value?.midRms,
      sideRms: stereoWidthData.value?.sideRms,
      balance: stereoWidthData.value?.balance
    },
    headroom: {
      peakL: headroomData.value?.peakL,
      peakR: headroomData.value?.peakR,
      headroomL: headroomData.value?.headroomL,
      headroomR: headroomData.value?.headroomR,
      headroomStereo: headroomData.value?.headroomStereo
    }
  }
  return propsMap[slotId]
}

function getMeterResetFn(slotId: Exclude<MeterType, null>) {
  const resetMap = {
    loudness: resetLoudness,
    dynamicRange: resetDynamicRange,
    phaseCorrelation: resetPhaseCorrelation,
    stereoWidth: resetStereoWidth,
    headroom: resetHeadroom
  }
  return resetMap[slotId]
}

// ============================================
// localStorage Functions
// ============================================

// Load components order from localStorage
function loadComponentsOrder() {
  try {
    const saved = localStorage.getItem('slotSectionComponentsOrder')
    if (saved) {
      const order = JSON.parse(saved)
      // Validate that all required components are present
      const requiredIds: Array<Exclude<MeterType, null>> = ['loudness', 'dynamicRange', 'phaseCorrelation', 'stereoWidth', 'headroom']
      const savedIds = order.map((c: SlotComponent) => c.id)

      if (requiredIds.every(id => savedIds.includes(id))) {
        slotComponents.value = order
      }
    }
  } catch (err) {
    console.warn('Failed to load slot components order:', err)
  }
}

// Save components order to localStorage
function saveComponentsOrder() {
  try {
    localStorage.setItem('slotSectionComponentsOrder', JSON.stringify(slotComponents.value))
  } catch (err) {
    console.warn('Failed to save slot components order:', err)
  }
}

// Watch for changes and save automatically
watch(slotComponents, () => {
  saveComponentsOrder()
}, { deep: true })

// Load order on mount
onMounted(() => {
  loadComponentsOrder()
})
</script>

<style scoped>
/* Custom scrollbar styling */
.overflow-y-auto {
  scrollbar-width: thin;
  scrollbar-color: rgba(59, 130, 246, 0.3) rgba(31, 41, 55, 0.15);
}

.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: rgba(31, 41, 55, 0.15);
  border-radius: 2px;
  margin-left: 2px;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: rgba(59, 130, 246, 0.3);
  border-radius: 2px;
  border-left: 2px solid transparent;
  background-clip: padding-box;
  transition: background 0.2s ease;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(59, 130, 246, 0.5);
}

.overflow-y-auto::-webkit-scrollbar-thumb:active {
  background: rgba(59, 130, 246, 0.7);
}
</style>
