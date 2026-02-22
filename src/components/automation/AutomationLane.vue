<template>
  <div class="automation-lane relative h-24 bg-gray-900/50 border-t border-gray-700/50">
    <!-- Label and Mode Selector -->
    <div class="absolute top-1 left-2 flex items-center gap-2 z-20">
      <span class="text-[0.6rem] font-bold text-teal-300">{{ label }}</span>
      <select 
        :value="lane.mode" 
        @change="handleModeChange"
        class="text-[0.55rem] px-1 py-0.5 bg-gray-800 border border-gray-600 rounded text-gray-300 focus:outline-none focus:border-teal-500"
      >
        <option value="off">OFF</option>
        <option value="read">READ</option>
        <option value="write">WRITE</option>
        <option value="touch">TOUCH</option>
        <option value="latch">LATCH</option>
      </select>
    </div>

    <!-- Value Range Labels -->
    <div class="absolute top-0 right-1 bottom-0 flex flex-col justify-between py-1 text-[0.55rem] text-gray-600 pointer-events-none">
      <span>{{ maxLabel }}</span>
      <span>{{ minLabel }}</span>
    </div>

    <!-- Automation Graph -->
    <svg 
      ref="svgRef"
      class="absolute inset-0 w-full h-full cursor-crosshair"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseLeave"
      @dblclick="handleDoubleClick"
    >
      <!-- Grid Lines -->
      <g class="grid-lines" opacity="0.1">
        <line v-for="i in 5" :key="`h-${i}`" 
          :x1="0" :y1="i * 20" 
          :x2="100" :y2="i * 20" 
          stroke="white" stroke-width="0.5" 
          vector-effect="non-scaling-stroke" 
        />
      </g>

      <!-- Automation Curve -->
      <polyline 
        v-if="points.length > 0"
        :points="curvePoints"
        fill="none"
        stroke="rgb(20, 184, 166)"
        stroke-width="2"
        vector-effect="non-scaling-stroke"
      />

      <!-- Automation Points -->
      <g v-for="(point, index) in points" :key="index">
        <!-- Point Circle -->
        <circle 
          :cx="point.x"
          :cy="point.y"
          :r="hoveredPoint === index || selectedPoint === index ? 5 : 3.5"
          :fill="selectedPoint === index ? 'rgb(251, 146, 60)' : 'rgb(20, 184, 166)'"
          :stroke="selectedPoint === index ? 'white' : 'rgb(13, 148, 136)'"
          stroke-width="1.5"
          class="cursor-pointer"
          @mouseenter="hoveredPoint = index"
          @mouseleave="hoveredPoint = null"
          @mousedown.stop="startDrag(index)"
        />
        
        <!-- Value Label on Hover -->
        <text 
          v-if="hoveredPoint === index"
          :x="point.x"
          :y="point.y - 10"
          text-anchor="middle"
          class="text-[0.6rem] fill-white pointer-events-none"
          style="text-shadow: 0 0 3px black"
        >
          {{ formatValue(point.value) }}
        </text>
      </g>

      <!-- Preview Point (when hovering to add) -->
      <circle 
        v-if="previewPoint && !isDragging"
        :cx="previewPoint.x"
        :cy="previewPoint.y"
        r="3"
        fill="rgb(20, 184, 166)"
        opacity="0.5"
        class="pointer-events-none"
      />
    </svg>

    <!-- Playhead Indicator -->
    <div 
      :style="{ left: `${playheadPosition * 100}%` }"
      class="absolute top-0 bottom-0 w-px bg-teal-400/50 pointer-events-none z-10"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { AutomationLane, AutomationPoint } from '~/composables/useAutomation'

interface Props {
  lane: AutomationLane
  duration: number
  playheadPosition: number
  minValue: number
  maxValue: number
  label: string
  valueFormatter?: (value: number) => string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'add-point', time: number, value: number): void
  (e: 'update-point', index: number, time: number, value: number): void
  (e: 'remove-point', index: number): void
  (e: 'change-mode', mode: string): void
}>()

const svgRef = ref<SVGSVGElement | null>(null)
const hoveredPoint = ref<number | null>(null)
const selectedPoint = ref<number | null>(null)
const isDragging = ref(false)
const previewPoint = ref<{ x: number, y: number, value: number } | null>(null)

const minLabel = computed(() => formatValue(props.minValue))
const maxLabel = computed(() => formatValue(props.maxValue))

// Convert automation points to SVG coordinates
const points = computed(() => {
  if (!svgRef.value) return []
  
  const rect = svgRef.value.getBoundingClientRect()
  
  return props.lane.points.map(point => {
    const x = (point.time / props.duration) * rect.width
    const normalizedValue = (point.value - props.minValue) / (props.maxValue - props.minValue)
    const y = rect.height - (normalizedValue * rect.height)
    
    return {
      x,
      y,
      time: point.time,
      value: point.value
    }
  })
})

// Generate curve points string for polyline
const curvePoints = computed(() => {
  if (points.value.length === 0) return ''
  
  // Add start and end points at edges if needed
  const allPoints = [...points.value]
  
  // If no point at start, add one
  if (allPoints.length > 0 && allPoints[0].x > 0) {
    allPoints.unshift({
      x: 0,
      y: allPoints[0].y,
      time: 0,
      value: allPoints[0].value
    })
  }
  
  // If no point at end, add one
  if (allPoints.length > 0 && svgRef.value) {
    const rect = svgRef.value.getBoundingClientRect()
    const lastPoint = allPoints[allPoints.length - 1]
    if (lastPoint.x < rect.width) {
      allPoints.push({
        x: rect.width,
        y: lastPoint.y,
        time: props.duration,
        value: lastPoint.value
      })
    }
  }
  
  return allPoints.map(p => `${p.x},${p.y}`).join(' ')
})

function formatValue(value: number): string {
  if (props.valueFormatter) {
    return props.valueFormatter(value)
  }
  return value.toFixed(1)
}

function handleModeChange(event: Event) {
  const target = event.target as HTMLSelectElement
  emit('change-mode', target.value)
}

function getPointFromEvent(event: MouseEvent): { time: number, value: number } | null {
  if (!svgRef.value) return null
  
  const rect = svgRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  
  const time = (x / rect.width) * props.duration
  const normalizedValue = 1 - (y / rect.height)
  const value = props.minValue + normalizedValue * (props.maxValue - props.minValue)
  
  return {
    time: Math.max(0, Math.min(props.duration, time)),
    value: Math.max(props.minValue, Math.min(props.maxValue, value))
  }
}

function handleMouseMove(event: MouseEvent) {
  if (!isDragging.value && selectedPoint.value === null) {
    // Show preview point
    const point = getPointFromEvent(event)
    if (point && svgRef.value) {
      const rect = svgRef.value.getBoundingClientRect()
      const x = (point.time / props.duration) * rect.width
      const normalizedValue = (point.value - props.minValue) / (props.maxValue - props.minValue)
      const y = rect.height - (normalizedValue * rect.height)
      
      previewPoint.value = { x, y, value: point.value }
    }
  } else if (isDragging.value && selectedPoint.value !== null) {
    // Drag existing point
    const point = getPointFromEvent(event)
    if (point) {
      emit('update-point', selectedPoint.value, point.time, point.value)
    }
  }
}

function handleMouseLeave() {
  previewPoint.value = null
}

function handleMouseDown(event: MouseEvent) {
  // Check if clicking near a point
  const clickPoint = getPointFromEvent(event)
  if (!clickPoint || !svgRef.value) return
  
  const rect = svgRef.value.getBoundingClientRect()
  const clickX = (clickPoint.time / props.duration) * rect.width
  const normalizedClickValue = (clickPoint.value - props.minValue) / (props.maxValue - props.minValue)
  const clickY = rect.height - (normalizedClickValue * rect.height)
  
  // Find if we clicked near a point (within 10px)
  const nearPoint = points.value.findIndex(p => {
    const dist = Math.sqrt(Math.pow(p.x - clickX, 2) + Math.pow(p.y - clickY, 2))
    return dist < 10
  })
  
  if (nearPoint >= 0) {
    selectedPoint.value = nearPoint
  }
}

function handleMouseUp(event: MouseEvent) {
  if (isDragging.value) {
    isDragging.value = false
    selectedPoint.value = null
  }
}

function handleDoubleClick(event: MouseEvent) {
  const point = getPointFromEvent(event)
  if (!point || !svgRef.value) return
  
  const rect = svgRef.value.getBoundingClientRect()
  const clickX = (point.time / props.duration) * rect.width
  const normalizedClickValue = (point.value - props.minValue) / (props.maxValue - props.minValue)
  const clickY = rect.height - (normalizedClickValue * rect.height)
  
  // Check if double-clicking on existing point to delete
  const nearPoint = points.value.findIndex(p => {
    const dist = Math.sqrt(Math.pow(p.x - clickX, 2) + Math.pow(p.y - clickY, 2))
    return dist < 10
  })
  
  if (nearPoint >= 0) {
    // Delete point
    emit('remove-point', nearPoint)
  } else {
    // Add new point
    emit('add-point', point.time, point.value)
  }
}

function startDrag(index: number) {
  selectedPoint.value = index
  isDragging.value = true
}

// Keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  if (selectedPoint.value !== null) {
    if (event.key === 'Delete' || event.key === 'Backspace') {
      emit('remove-point', selectedPoint.value)
      selectedPoint.value = null
      event.preventDefault()
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
.automation-lane {
  position: relative;
}

svg {
  shape-rendering: geometricPrecision;
}
</style>
