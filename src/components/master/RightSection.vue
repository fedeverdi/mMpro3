<template>
  <div class="flex flex-col h-full gap-2 relative transition-all duration-300 ease-out" :class="{ 'pl-2': !isCollapsed }" :style="{ width: sectionWidth + 'px' }">
    <!-- Resize Handle -->
    <div 
      :class="['absolute left-0 top-0 bottom-0 w-3 z-50 group bg-gray-900/20', isCollapsed ? 'cursor-default' : 'cursor-grab']"
      @mousedown.stop="startResize"
      :title="isCollapsed ? 'Panel collapsed' : 'Drag to resize'"
    >

      <!-- Collapse/Expand Button -->
      <button
        @click.stop="toggleCollapse"
        class="absolute top-2 -left-2 w-5 h-5 -translate-x-1/2 bg-gray-800 hover:bg-blue-600 border border-gray-700 hover:border-blue-500 rounded flex items-center justify-center transition-all shadow-lg"
        :title="isCollapsed ? 'Expand panel' : 'Collapse panel'"
      >
        <svg class="w-3 h-3 text-gray-300 transition-transform" :class="{ 'rotate-180': !isCollapsed }" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
        </svg>
      </button>
      
      <!-- Grip Dots -->
      <div class="absolute top-1/2 -left-[0.17rem] -translate-y-1/2 flex gap-0.5 opacity-50 group-hover:opacity-100 transition-opacity pointer-events-none">
        <!-- Column 1 -->
        <div class="flex flex-col gap-1">
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
        </div>
        <!-- Column 2 -->
        <div class="flex flex-col gap-1">
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
          <div class="w-0.5 h-0.5 rounded-full bg-gray-400"></div>
        </div>
      </div>
    </div>

    <!-- Components (hidden when collapsed) -->
    <template v-for="component in rightSectionComponents" :key="component.id">
      <!-- Master EQ Display -->
      <div v-if="component.id === 'eq'"
        v-show="!isCollapsed"
        :class="[component.size === 'flex' ? 'flex-1 min-h-0' : '', 'w-full mixer-fade-in relative group']"
        :style="getDragStyles(component.id)"
        @dragover="handleDragOver($event, component.id)"
        @drop="handleDrop($event, component.id)">
        <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
          <div
            draggable="true"
            @dragstart="handleDragStart(component.id, $event)"
            @dragend="handleDragEnd"
            class="drag-handle bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1"
            :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
              <path
                d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z" />
            </svg>
            <span>{{ component.name }}</span>
          </div>
        </div>
        <MasterEQDisplay :filters-data="masterEqFiltersData" :master-channel="masterChannel"
          @update:filters-data="handleMasterEQFiltersUpdate"
          @output-node="(node) => emit('master-eq-output-node', node)" />
      </div>

      <!-- Spectrum Meter -->
      <div v-if="component.id === 'spectrum'"
        v-show="!isCollapsed"
        :class="[component.size === 'flex' ? 'flex-1 min-h-0' : '', 'w-full mixer-fade-in relative group']"
        :style="getDragStyles(component.id)"
        @dragover="handleDragOver($event, component.id)"
        @drop="handleDrop($event, component.id)">
        <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
          <div
            draggable="true"
            @dragstart="handleDragStart(component.id, $event)"
            @dragend="handleDragEnd"
            class="drag-handle bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1"
            :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
              <path
                d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z" />
            </svg>
            <span>{{ component.name }}</span>
          </div>
        </div>
        <SpectrumMeter :master-fx-output-node="masterFxOutputNode" />
      </div>

      <!-- Aux Buses -->
      <div v-if="component.id === 'aux'"
        v-show="!isCollapsed"
        class="w-full mixer-fade-in relative group"
        :style="getDragStyles(component.id)"
        @dragover="handleDragOver($event, component.id)"
        @drop="handleDrop($event, component.id)">
        <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
          <div
            draggable="true"
            @dragstart="handleDragStart(component.id, $event)"
            @dragend="handleDragEnd"
            class="drag-handle bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1"
            :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
              <path
                d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z" />
            </svg>
            <span>{{ component.name }}</span>
          </div>
        </div>
        <AuxMaster 
          :aux-buses="auxBuses" 
          :master-channel="masterChannel"
          @add-aux="emit('add-aux')"
          @remove-aux="(index) => emit('remove-aux', index)"
          @update-aux="(index, aux) => emit('update-aux', index, aux)"
        />
      </div>

      <!-- Master FX -->
      <div v-if="component.id === 'fx'"
        v-show="!isCollapsed"
        class="w-full mixer-fade-in relative group"
        :style="getDragStyles(component.id)"
        @dragover="handleDragOver($event, component.id)"
        @drop="handleDrop($event, component.id)">
        <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
          <div
            draggable="true"
            @dragstart="handleDragStart(component.id, $event)"
            @dragend="handleDragEnd"
            class="drag-handle bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1"
            :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
              <path
                d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z" />
            </svg>
            <span>{{ component.name }}</span>
          </div>
        </div>
        <MasterFX :master-eq-output-node="masterEqOutputNode" :master-section="masterSectionRef"
          @output-node="(node) => emit('master-fx-output-node', node)" 
          @component="(component) => emit('master-fx-component', component)" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import MasterEQDisplay from '../master/MasterEQDisplay.vue'
import SpectrumMeter from '../master/SpectrumMeter.vue'
import MasterFX from '../master/MasterFX.vue'
import AuxMaster from '../master/AuxMaster.vue'

interface AuxBus {
  id: string
  name: string
  volume: number
  muted: boolean
  soloed: boolean
  routeToMaster: boolean
  node?: any
}

interface Props {
  masterChannel?: any
  masterSectionRef?: any
  masterEqOutputNode?: any
  masterFxOutputNode?: any
  auxBuses?: AuxBus[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'master-eq-output-node': [node: any]
  'master-fx-output-node': [node: any]
  'master-fx-component': [component: any]
  'update:master-eq-filters': [filters: any[]]
  'add-aux': []
  'remove-aux': [index: number]
  'update-aux': [index: number, aux: AuxBus]
}>()

// Right section components order (draggable)
interface RightSectionComponent {
  id: string
  name: string
  size: 'flex' | 'fixed'
}

const rightSectionComponents = ref<RightSectionComponent[]>([
  { id: 'eq', name: 'Master EQ', size: 'flex' },
  { id: 'spectrum', name: 'Spectrum', size: 'flex' },
  { id: 'aux', name: 'Aux Buses', size: 'fixed' },
  { id: 'fx', name: 'Master FX', size: 'fixed' }
])

const draggedComponent = ref<string | null>(null)
const dragOverComponent = ref<string | null>(null)

// Resize functionality
const sectionWidth = ref(576) // Default: 36rem = 576px
const isResizing = ref(false)
const isCollapsed = ref(false)
const savedWidth = ref(576) // Store width before collapse
let startX = 0
let startWidth = 0
let resizeRafId: number | null = null
let pendingWidth: number | null = null

// Master EQ filters data
const masterEqFiltersData = ref<any[]>([])

// Handle master EQ filters update from MasterEQDisplay
function handleMasterEQFiltersUpdate(filters: any[]) {
  masterEqFiltersData.value = filters
  emit('update:master-eq-filters', filters)
}

function handleDragStart(componentId: string, event: DragEvent) {
  draggedComponent.value = componentId
  
  // Set custom drag image to show the whole component instead of just the handle
  const componentElement = (event.target as HTMLElement).closest('.mixer-fade-in')
  if (componentElement) {
    const rect = componentElement.getBoundingClientRect()
    // Calculate the offset from where the user clicked relative to the component
    const offsetX = event.clientX - rect.left
    const offsetY = event.clientY - rect.top
    event.dataTransfer?.setDragImage(componentElement as HTMLElement, offsetX, offsetY)
  }
}

function handleDragOver(event: DragEvent, componentId: string) {
  event.preventDefault()
  if (draggedComponent.value !== componentId) {
    dragOverComponent.value = componentId
  }
}

function handleDrop(event: DragEvent, targetComponentId: string) {
  event.preventDefault()

  if (!draggedComponent.value || draggedComponent.value === targetComponentId) {
    return
  }

  const draggedIndex = rightSectionComponents.value.findIndex(c => c.id === draggedComponent.value)
  const targetIndex = rightSectionComponents.value.findIndex(c => c.id === targetComponentId)

  if (draggedIndex !== -1 && targetIndex !== -1) {
    const components = [...rightSectionComponents.value]
    const [removed] = components.splice(draggedIndex, 1)
    components.splice(targetIndex, 0, removed)
    rightSectionComponents.value = components
  }

  draggedComponent.value = null
}

function handleDragEnd() {
  draggedComponent.value = null
  dragOverComponent.value = null
}

// Helper to compute drag effect styles
function getDragStyles(componentId: string) {
  if (!draggedComponent.value || !dragOverComponent.value) return {}

  const draggedIndex = rightSectionComponents.value.findIndex(c => c.id === draggedComponent.value)
  const currentIndex = rightSectionComponents.value.findIndex(c => c.id === componentId)
  const dragOverIndex = rightSectionComponents.value.findIndex(c => c.id === dragOverComponent.value)

  // Skip the dragged element itself
  if (componentId === draggedComponent.value) {
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

// Toggle collapse/expand
function toggleCollapse() {
  if (isCollapsed.value) {
    // Expand
    sectionWidth.value = savedWidth.value || 576
    isCollapsed.value = false
  } else {
    // Collapse
    savedWidth.value = sectionWidth.value
    sectionWidth.value = 6 // Collapsed width (just the handle)
    isCollapsed.value = true
  }
  saveWidth()
}

// Resize functionality
function startResize(event: MouseEvent) {
  if (isCollapsed.value) return // Don't allow resize when collapsed
  
  isResizing.value = true
  startX = event.clientX
  startWidth = sectionWidth.value
  
  document.addEventListener('mousemove', onResize)
  document.addEventListener('mouseup', stopResize)
  
  // Prevent text selection during resize
  event.preventDefault()
  document.body.style.cursor = 'grabbing'
  document.body.style.userSelect = 'none'
}

function onResize(event: MouseEvent) {
  if (!isResizing.value) return
  
  // Calculate new width (drag left = reduce width, drag right = increase width)
  const deltaX = startX - event.clientX // Inverted because we're dragging from left edge
  const maxWidth = window.innerWidth / 2 // Max: half of window width
  const newWidth = Math.max(300, Math.min(maxWidth, startWidth + deltaX)) // Min 300px
  
  // Store pending width
  pendingWidth = newWidth
  
  // Use RAF to throttle updates to once per frame
  if (resizeRafId === null) {
    resizeRafId = requestAnimationFrame(() => {
      if (pendingWidth !== null) {
        sectionWidth.value = pendingWidth
        pendingWidth = null
      }
      resizeRafId = null
    })
  }
}

function stopResize() {
  if (isResizing.value) {
    isResizing.value = false
    document.removeEventListener('mousemove', onResize)
    document.removeEventListener('mouseup', stopResize)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    
    // Cancel any pending RAF and apply final width
    if (resizeRafId !== null) {
      cancelAnimationFrame(resizeRafId)
      resizeRafId = null
    }
    if (pendingWidth !== null) {
      sectionWidth.value = pendingWidth
      pendingWidth = null
    }
    
    // Save to localStorage
    saveWidth()
  }
}

function saveWidth() {
  try {
    localStorage.setItem('rightSectionWidth', sectionWidth.value.toString())
    localStorage.setItem('rightSectionCollapsed', isCollapsed.value.toString())
    if (!isCollapsed.value) {
      localStorage.setItem('rightSectionSavedWidth', savedWidth.value.toString())
    }
  } catch (err) {
    console.warn('Failed to save right section width:', err)
  }
}

function loadWidth() {
  try {
    const saved = localStorage.getItem('rightSectionWidth')
    const collapsed = localStorage.getItem('rightSectionCollapsed')
    const savedWidthStr = localStorage.getItem('rightSectionSavedWidth')
    
    const maxWidth = window.innerWidth / 2
    
    if (saved) {
      const width = parseInt(saved, 10)
      if (!isNaN(width) && width >= 6 && width <= maxWidth) {
        sectionWidth.value = width
      }
    }
    
    if (savedWidthStr) {
      const width = parseInt(savedWidthStr, 10)
      if (!isNaN(width) && width >= 300 && width <= maxWidth) {
        savedWidth.value = width
      }
    }
    
    if (collapsed === 'true') {
      isCollapsed.value = true
    }
  } catch (err) {
    console.warn('Failed to load right section width:', err)
  }
}

// Load components order from localStorage
function loadComponentsOrder() {
  try {
    const saved = localStorage.getItem('rightSectionComponentsOrder')
    if (saved) {
      const order = JSON.parse(saved)
      // Validate that all required components are present
      const requiredIds = ['eq', 'spectrum', 'aux', 'fx']
      const savedIds = order.map((c: RightSectionComponent) => c.id)

      if (requiredIds.every(id => savedIds.includes(id))) {
        rightSectionComponents.value = order
      }
    }
  } catch (err) {
    console.warn('Failed to load components order:', err)
  }
}

// Save components order to localStorage
function saveComponentsOrder() {
  try {
    localStorage.setItem('rightSectionComponentsOrder', JSON.stringify(rightSectionComponents.value))
  } catch (err) {
    console.warn('Failed to save components order:', err)
  }
}

// Watch for changes and save automatically
watch(rightSectionComponents, () => {
  saveComponentsOrder()
}, { deep: true })

// Expose masterEqFiltersData for access from parent
defineExpose({
  masterEqFiltersData
})

onMounted(() => {
  loadComponentsOrder()
  loadWidth()
})

onUnmounted(() => {
  // Clean up resize event listeners if still active
  if (isResizing.value) {
    document.removeEventListener('mousemove', onResize)
    document.removeEventListener('mouseup', stopResize)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }
  
  // Cancel any pending RAF
  if (resizeRafId !== null) {
    cancelAnimationFrame(resizeRafId)
    resizeRafId = null
  }
})
</script>

<style scoped>
.mixer-fade-in {
  animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
