<template>
  <div v-if="vuMetersHeight > 0"
    class="flex flex-col items-center gap-1 w-full justify-center bg-gray-900 rounded p-1 border border-gray-700">
    <div class="flex gap-2 relative">
      <!-- Left VuMeter -->
      <VuMeter :level="leftLevel" label="L" :height="vuMetersHeight" :width="20" :showValue="false" />
      
      <!-- Central Scale marks with lines -->
      <div class="relative flex-shrink-0 w-8 text-center mt-3" :style="{ height: vuMetersHeight + 'px' }">
        <div v-for="mark in scaleMarks" :key="mark.label" 
          class="absolute left-1/2 -translate-x-1/2 w-full flex items-center justify-center gap-0.5"
          :style="{ bottom: mark.position + '%' }">
          <!-- Left line -->
          <div class="flex-1 h-px"
            :class="{
              'bg-green-500/40': mark.value === 0,
              'bg-yellow-500/30': mark.value > -20 && mark.value < 0,
              'bg-red-500/40': mark.value >= -6,
              'bg-gray-700': mark.value <= -20
            }"></div>
          <!-- Scale label -->
          <div class="font-mono leading-none px-0.5"
            :class="{
              'text-[8px] text-green-500 font-bold': mark.value === 0,
              'text-[8px] text-yellow-400': mark.value > -20 && mark.value < 0,
              'text-[8px] text-red-500 font-bold': mark.value >= -6,
              'text-[8px] text-gray-500': mark.value <= -20
            }">
            {{ mark.label }}
          </div>
          <!-- Right line -->
          <div class="flex-1 h-px"
            :class="{
              'bg-green-500/40': mark.value === 0,
              'bg-yellow-500/30': mark.value > -20 && mark.value < 0,
              'bg-red-500/40': mark.value >= -6,
              'bg-gray-700': mark.value <= -20
            }"></div>
        </div>
      </div>
      
      <!-- Right VuMeter -->
      <VuMeter :level="rightLevel" label="R" :height="vuMetersHeight" :width="20" :showValue="false" />
      
      <!-- RMS label -->
      <div class="text-[8px] text-gray-500 uppercase tracking-wider absolute -bottom-4 left-1/2 transform -translate-x-1/2">
        RMS
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import VuMeter from '../core/VuMeter.vue'

interface Props {
  leftLevel: number   // Level in dB (-60 to 0)
  rightLevel: number  // Level in dB (-60 to 0)
  vuMetersHeight: number
}

const props = defineProps<Props>()

// Scale marks for dB scale (-60 to 0)
const scaleMarks = computed(() => {
  const marks = [0, -6, -12, -18, -24, -30, -40, -60]
  const padding = 2 // VuMeter has p-0.5 (2px padding)
  const effectiveHeight = props.vuMetersHeight - padding * 2
  
  return marks.map(mark => {
    // Normalize -60 to 0 dB to position, accounting for VuMeter padding
    const normalizedPosition = (mark + 60) / 60
    const position = ((normalizedPosition * effectiveHeight + padding) / props.vuMetersHeight) * 100
    
    return {
      value: mark,
      position: position - 1.5, // Adjust for gap-px in VuMeter segments
      label: mark === 0 ? '0' : `${mark}`
    }
  })
})
</script>

<style scoped>
/* Ensure proper alignment */
</style>
