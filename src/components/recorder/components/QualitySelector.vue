<template>
  <div class="quality-selector relative" ref="selectorRef">
    <!-- <div >
      <label class="text-[10px] text-gray-400 uppercase tracking-wider">Quality</label>
    </div> -->
    <button
      @click="toggleDropdown"
      :disabled="disabled"
      class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-white text-left flex items-center justify-between hover:border-gray-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:border-gray-600"
    >
      <span>{{ selectedQuality?.label }}</span>
      <svg 
        class="w-4 h-4 transition-transform"
        :class="{ 'rotate-180': isOpen }"
        fill="none" 
        stroke="currentColor" 
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <!-- Dropdown -->
    <Transition
      enter-from-class="opacity-0 scale-95"
      enter-active-class="transition-all duration-150"
      enter-to-class="opacity-100 scale-100"
      leave-from-class="opacity-100 scale-100"
      leave-active-class="transition-all duration-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div 
        v-if="isOpen"
        class="absolute top-full left-0 right-0 mt-1 bg-gray-800 border border-gray-600 rounded shadow-xl z-50 overflow-hidden"
      >
        <button
          v-for="option in qualityOptions"
          :key="option.value"
          @click="selectQuality(option)"
          class="w-full px-3 py-2 text-sm text-left hover:bg-gray-700 transition-colors flex items-center justify-between"
          :class="{ 'bg-gray-700 text-red-400': modelValue === option.value }"
        >
          <span>{{ option.label }}</span>
          <svg 
            v-if="modelValue === option.value"
            class="w-4 h-4 text-red-500" 
            fill="currentColor" 
            viewBox="0 0 20 20"
          >
            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

interface QualityOption {
  value: string
  label: string
  bitrate: number
}

interface Props {
  modelValue: string
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const isOpen = ref(false)
const selectorRef = ref<HTMLElement | null>(null)

const qualityOptions: QualityOption[] = [
  { value: '64', label: 'Low (64 kbps)', bitrate: 64 },
  { value: '128', label: 'Medium (128 kbps)', bitrate: 128 },
  { value: '192', label: 'High (192 kbps)', bitrate: 192 },
  { value: '256', label: 'Highest (256 kbps)', bitrate: 256 }
]

const selectedQuality = computed(() => {
  return qualityOptions.find(q => q.value === props.modelValue)
})

function toggleDropdown() {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
  }
}

function selectQuality(option: QualityOption) {
  emit('update:modelValue', option.value)
  isOpen.value = false
}

function handleClickOutside(event: MouseEvent) {
  if (selectorRef.value && !selectorRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.quality-selector {
  user-select: none;
  -webkit-user-select: none;
  min-width: 160px;
}
</style>
