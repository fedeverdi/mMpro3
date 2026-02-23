<template>
  <div class="w-full flex flex-col gap-1 mb-4">
    <!-- Headphones Output Selector -->
    <div class="w-full bg-gray-900 rounded p-1.5 border border-gray-700">
      <OutputSelector
        icon="🎧"
        title="Select Headphones Output"
        :devices="devices"
        :selected-device-id="selectedDeviceId"
        default-label="Off"
        default-description="No headphones output"
        default-icon="🔇"
        @select="$emit('select', $event)"
      />
    </div>

    <!-- Headphones Volume Control & Meter -->
    <div class="w-full flex items-center justify-center gap-2 bg-gray-900 rounded border border-gray-700 px-2">
      <div class="scale-[0.7] -mt-2 -mb-2">
        <Knob 
          :model-value="volume" 
          @update:model-value="$emit('update:volume', $event)"
          :min="-60" 
          :max="6" 
          :step="0.5" 
          label="HP Vol" 
          unit="dB" 
          color="#14b8a6" 
        />
      </div>
      
      <!-- HP Level Indicator -->
      <div class="flex flex-col gap-1">
        <div 
          class="w-2.5 h-2.5 rounded-full transition-all" 
          :class="level > -6 ? 'bg-red-500 shadow-lg shadow-red-500/50' : 'bg-gray-700'"
        ></div>
        <div 
          class="w-2.5 h-2.5 rounded-full transition-all" 
          :class="level > -20 ? 'bg-yellow-500 shadow-lg shadow-yellow-500/50' : 'bg-gray-700'"
        ></div>
        <div 
          class="w-2.5 h-2.5 rounded-full transition-all" 
          :class="level > -35 ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-700'"
        ></div>
        <div 
          class="w-2.5 h-2.5 rounded-full transition-all" 
          :class="level > -50 ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-700'"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import Knob from '../core/Knob.vue'
import OutputSelector from './OutputSelector.vue'

interface Props {
  devices: MediaDeviceInfo[]
  selectedDeviceId: string | null
  volume: number
  level: number
}

defineProps<Props>()

defineEmits<{
  select: [deviceId: string | null]
  'update:volume': [volume: number]
}>()
</script>
