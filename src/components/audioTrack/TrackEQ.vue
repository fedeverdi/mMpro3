<template>
  <Transition
    enter-active-class="transition-all duration-300 ease-out"
    leave-active-class="transition-all duration-200 ease-in"
    enter-from-class="opacity-0 -translate-y-2 scale-95"
    enter-to-class="opacity-100 translate-y-0 scale-100"
    leave-from-class="opacity-100 translate-y-0 scale-100"
    leave-to-class="opacity-0 -translate-y-2 scale-95"
  >
    <div v-show="show" class="grid grid-cols-2 gap-2 items-center max-h-[10.5rem] -mt-4">
      <!-- Left column: Mid -->
      <div class="flex justify-center">
        <div class="scale-[0.75]">
          <Knob v-model="eqMid" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Mid" unit="dB" color="#f59e0b" />
        </div>
      </div>
      
      <!-- Right column: High and Low stacked -->
      <div class="flex flex-col -space-y-6">
        <div class="scale-[0.75]">
          <Knob v-model="eqHigh" :min="-12" :max="12" :step="0.5" :centerValue="0" label="High" unit="dB" color="#ef4444" />
        </div>
        <div class="scale-[0.75]">
          <Knob v-model="eqLow" :min="-12" :max="12" :step="0.5" :centerValue="0" label="Low" unit="dB" color="#10b981" />
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watchEffect, onUnmounted } from 'vue'
import Knob from '../core/Knob.vue'

interface Props {
  eq3Node?: any
  show?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  show: false
})

const emit = defineEmits<{
  (e: 'params-changed', params: { low: number, mid: number, high: number }): void
}>()

// Internal state
const eqLow = ref(0)
const eqMid = ref(0)
const eqHigh = ref(0)

let rafId: number | null = null

// Update EQ3 node when values change - throttled with requestAnimationFrame
function updateEQ() {
  if (!props.eq3Node) return
  
  // Ensure values are not undefined before setting
  const lowVal = eqLow.value ?? 0
  const midVal = eqMid.value ?? 0
  const highVal = eqHigh.value ?? 0
  
  props.eq3Node.low.value = lowVal
  props.eq3Node.mid.value = midVal
  props.eq3Node.high.value = highVal
}

// Watch for changes with throttling
watchEffect(() => {
  // Track dependencies
  const low = eqLow.value
  const mid = eqMid.value
  const high = eqHigh.value
  
  // Throttle updates with requestAnimationFrame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  
  rafId = requestAnimationFrame(() => {
    updateEQ()
    
    // Emit params changed for Rust engine
    emit('params-changed', {
      low,
      mid,
      high
    })
    
    rafId = null
  })
})

onUnmounted(() => {
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})

// Expose methods for snapshot system
function getParams() {
  return {
    low: eqLow.value,
    mid: eqMid.value,
    high: eqHigh.value
  }
}

function setParams(params: { low: number, mid: number, high: number }) {
  eqLow.value = params.low ?? 0
  eqMid.value = params.mid ?? 0
  eqHigh.value = params.high ?? 0
}

defineExpose({
  getParams,
  setParams
})
</script>
