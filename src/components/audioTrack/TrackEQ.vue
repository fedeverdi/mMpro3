<template>
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
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Knob from '../core/Knob.vue'

interface Props {
  eq3Node?: any
  show?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  show: false
})

// Internal state
const eqLow = ref(0)
const eqMid = ref(0)
const eqHigh = ref(0)

// Update EQ3 node when values change
function updateEQ() {
  if (!props.eq3Node) return
  props.eq3Node.low.value = eqLow.value
  props.eq3Node.mid.value = eqMid.value
  props.eq3Node.high.value = eqHigh.value
}

// Watch for changes
watch([eqLow, eqMid, eqHigh], updateEQ)

// Expose methods for snapshot system
function getParams() {
  return {
    low: eqLow.value,
    mid: eqMid.value,
    high: eqHigh.value
  }
}

function setParams(params: { low: number, mid: number, high: number }) {
  eqLow.value = params.low
  eqMid.value = params.mid
  eqHigh.value = params.high
}

defineExpose({
  getParams,
  setParams
})
</script>
