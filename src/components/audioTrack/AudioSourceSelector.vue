<template>
  <div class="relative" v-click-outside="closeDropdown">
    <button @click="toggleDropdown"
      class="w-full text-xs bg-gray-700 text-gray-200 border border-gray-600 rounded px-2 py-1 hover:bg-gray-600 hover:border-blue-500 focus:border-blue-500 focus:outline-none transition-all flex items-center justify-between">
      <span class="flex items-center gap-1.5 min-w-0">
        <!-- Microphone Icon -->
        <svg v-if="modelValue === 'input'" class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
          <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
        </svg>
        <!-- File Icon -->
        <svg v-else class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
        </svg>
        <span class="whitespace-nowrap">{{ modelValue === 'input' ? 'Audio Input' : 'Audio File' }}</span>
      </span>
      <svg class="w-2.5 h-2.5 flex-shrink-0 transition-transform" :class="{ 'rotate-180': isOpen }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>
    
    <!-- Dropdown -->
    <Transition name="dropdown">
      <div v-if="isOpen" class="absolute z-[1100] mt-1 w-full min-w-[140px] bg-gray-700 border border-gray-600 rounded-lg shadow-2xl overflow-hidden">
        <button
          @click="selectType('input')"
          class="w-full px-3 py-2 text-left text-xs hover:bg-gray-600 transition-colors flex items-center gap-2"
          :class="modelValue === 'input' ? 'bg-blue-600 text-white' : 'text-gray-200'"
        >
          <!-- Microphone Icon -->
          <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
            <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
          </svg>
          <span class="flex-1 font-medium whitespace-nowrap">Audio Input</span>
          <!-- Checkmark -->
          <svg v-if="modelValue === 'input'" class="w-3.5 h-3.5 flex-shrink-0" fill="currentColor" viewBox="0 0 24 24">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
          </svg>
        </button>
        <button
          @click="selectType('file')"
          class="w-full px-3 py-2 text-left text-xs hover:bg-gray-600 transition-colors flex items-center gap-2"
          :class="modelValue === 'file' ? 'bg-blue-600 text-white' : 'text-gray-200'"
        >
          <!-- File Icon -->
          <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
            <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
          </svg>
          <span class="flex-1 font-medium whitespace-nowrap">Audio File</span>
          <!-- Checkmark -->
          <svg v-if="modelValue === 'file'" class="w-3.5 h-3.5 flex-shrink-0" fill="currentColor" viewBox="0 0 24 24">
            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
          </svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Props {
  modelValue: 'input' | 'file'
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: 'input' | 'file']
}>()

const isOpen = ref(false)

function toggleDropdown() {
  isOpen.value = !isOpen.value
}

function closeDropdown() {
  isOpen.value = false
}

function selectType(type: 'input' | 'file') {
  emit('update:modelValue', type)
  isOpen.value = false
}

// Click outside directive
const vClickOutside = {
  mounted(el: HTMLElement, binding: any) {
    (el as any).clickOutsideEvent = (event: Event) => {
      if (!(el === event.target || el.contains(event.target as Node))) {
        binding.value()
      }
    }
    document.addEventListener('click', (el as any).clickOutsideEvent)
  },
  unmounted(el: HTMLElement) {
    document.removeEventListener('click', (el as any).clickOutsideEvent)
  }
}
</script>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.15s ease-out;
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
