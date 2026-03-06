<template>
  <div class="w-full relative mb-4">
    <!-- Headphones Button -->
    <button
      @click="isOpen = !isOpen"
      class="w-full bg-gray-900 rounded border border-gray-700 px-2 py-1.5 hover:bg-gray-800 hover:border-blue-500 transition-all flex items-center justify-center gap-2"
      :class="{ 'border-blue-500 bg-gray-800': isOpen }"
    >
      <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 1c-4.97 0-9 4.03-9 9v7c0 1.66 1.34 3 3 3h1v-8H5v-2c0-3.87 3.13-7 7-7s7 3.13 7 7v2h-2v8h1c1.66 0 3-1.34 3-3v-7c0-4.97-4.03-9-9-9z"/>
      </svg>
      <span class="text-xs text-gray-400">Headphones</span>
    </button>

    <!-- Headphones Panel (Position Absolute) -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="isOpen"
        class="absolute top-0 left-0 z-50 w-full bg-gray-900 rounded-lg border-2 border-blue-500 shadow-2xl shadow-blue-500/20 p-3 flex flex-col gap-3"
      >
        <!-- Close Button -->
        <button
          @click="isOpen = false"
          class="absolute top-1 right-1 w-5 h-5 flex items-center justify-center text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-all"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        <!-- Headphones Output Selector -->
        <div class="w-full">
          <div class="text-[0.65rem] text-gray-400 mb-1 font-medium">Output</div>
          <div class="bg-gray-800 rounded p-1.5 border border-gray-700">
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
        </div>

        <!-- Volume Control & Level Indicator -->
        <div class="w-full flex items-center justify-center gap-3">
          <div class="scale-[0.8]">
            <Knob 
              :model-value="volume" 
              @update:model-value="$emit('update:volume', $event)"
              :min="-60" 
              :max="6" 
              :step="0.5" 
              label="Volume" 
              unit="dB" 
              color="#14b8a6" 
            />
          </div>
          
          <!-- HP Level Indicator -->
          <div class="flex flex-col gap-1">
            <div 
              class="w-3 h-3 rounded-full transition-all" 
              :class="level > -6 ? 'bg-red-500 shadow-lg shadow-red-500/50' : 'bg-gray-700'"
            ></div>
            <div 
              class="w-3 h-3 rounded-full transition-all" 
              :class="level > -20 ? 'bg-yellow-500 shadow-lg shadow-yellow-500/50' : 'bg-gray-700'"
            ></div>
            <div 
              class="w-3 h-3 rounded-full transition-all" 
              :class="level > -35 ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-700'"
            ></div>
            <div 
              class="w-3 h-3 rounded-full transition-all" 
              :class="level > -50 ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-700'"
            ></div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Knob from '../core/Knob.vue'
import OutputSelector from './OutputSelector.vue'

interface Props {
  devices: any[]
  selectedDeviceId: string | null
  volume: number
  level: number
}

defineProps<Props>()

defineEmits<{
  select: [deviceId: string | null]
  'update:volume': [volume: number]
}>()

const isOpen = ref(false)
</script>
