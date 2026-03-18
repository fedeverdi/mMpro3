<template>
  <div class="relative" v-click-outside="closeDropdown">
    <button 
      ref="buttonRef"
      @click="toggleDropdown"
      class="w-full text-[0.65rem] font-semibold bg-gray-700 text-gray-200 border border-gray-600 rounded px-2 py-0.5 hover:bg-gray-600 hover:border-blue-500 focus:border-blue-500 focus:outline-none transition-all flex items-center justify-between">
      <span class="flex items-center gap-1.5 min-w-0">
        <!-- EQ Icon -->
        <svg class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm2 4v-2H3c0 1.1.89 2 2 2zM3 9h2V7H3v2zm12 12h2v-2h-2v2zM5 21v-2H3c0 1.1.89 2 2 2zm-2-12h2V7H3v2zM9 3H7v2h2V3zm2 18h2v-2h-2v2zm8-8h2v-2h-2v2zm0 8c1.1 0 2-.9 2-2h-2v2zm0-12h2V7h-2v2zm0 8h2v-2h-2v2zm-4 4h2v-2h-2v2zm0-16h2V3h-2v2zM7 21h2v-2H7v2zm12-16h2V3c-1.1 0-2 .9-2 2zm-4 16h2v-2h-2v2z"/>
        </svg>
        <span class="whitespace-nowrap truncate">{{ displayName }}</span>
      </span>
      <svg class="w-2.5 h-2.5 flex-shrink-0 transition-transform" :class="{ 'rotate-180': isOpen }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>
    
    <!-- Dropdown with Teleport to escape parent overflow -->
    <Teleport to="body">
      <Transition name="dropdown">
        <div v-if="isOpen" 
          class="fixed z-[1100] bg-gray-700 border border-gray-600 rounded-lg shadow-2xl overflow-hidden max-h-[300px] overflow-y-auto scrollbar-thin"
          :style="dropdownStyle"
        >
          <button
            v-for="preset in presets"
            :key="preset.value"
            @click="selectPreset(preset.value)"
            class="w-full px-3 py-2 text-left text-xs hover:bg-gray-600 transition-colors flex items-center gap-2"
            :class="modelValue === preset.value ? 'bg-blue-600 text-white' : 'text-gray-200'"
          >
            <span class="flex-1 font-medium whitespace-nowrap">{{ preset.label }}</span>
            <!-- Checkmark -->
            <svg v-if="modelValue === preset.value" class="w-3.5 h-3.5 flex-shrink-0" fill="currentColor" viewBox="0 0 24 24">
              <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
            </svg>
          </button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'

interface Preset {
  value: string
  label: string
}

interface Props {
  modelValue: string
  presets?: Preset[]
}

const props = withDefaults(defineProps<Props>(), {
  presets: () => [
    { value: '', label: 'Custom' },
    { value: 'flat', label: 'Flat' },
    { value: 'rock', label: 'Rock' },
    { value: 'pop', label: 'Pop' },
    { value: 'bass-enhanced', label: 'Bass Enhanced' },
    { value: 'treble-boost', label: 'Treble Boost' },
    { value: 'jazz', label: 'Jazz' },
    { value: 'classical', label: 'Classical' },
    { value: 'electronic', label: 'Electronic' },
    { value: 'vocal', label: 'Vocal Boost' },
    { value: 'dance', label: 'Dance' }
  ]
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'change': []
}>()

const isOpen = ref(false)
const buttonRef = ref<HTMLElement | null>(null)
const dropdownPosition = ref({ top: 0, left: 0, width: 0 })

const displayName = computed(() => {
  const preset = props.presets.find(p => p.value === props.modelValue)
  return preset ? preset.label : 'Custom'
})

const dropdownStyle = computed(() => ({
  top: `${dropdownPosition.value.top}px`,
  left: `${dropdownPosition.value.left}px`,
  minWidth: `${dropdownPosition.value.width}px`,
  width: `${dropdownPosition.value.width}px`
}))

function updateDropdownPosition() {
  if (buttonRef.value) {
    const rect = buttonRef.value.getBoundingClientRect()
    dropdownPosition.value = {
      top: rect.bottom + 4,
      left: rect.left,
      width: rect.width
    }
  }
}

async function toggleDropdown() {
  if (!isOpen.value) {
    // Update position BEFORE showing dropdown
    updateDropdownPosition()
    await nextTick()
    isOpen.value = true
  } else {
    isOpen.value = false
  }
}

function closeDropdown() {
  isOpen.value = false
}

function selectPreset(value: string) {
  emit('update:modelValue', value)
  emit('change')
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
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* Thin scrollbar for preset list */
.scrollbar-thin::-webkit-scrollbar {
  width: 4px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: #374151;
  border-radius: 2px;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background: #6b7280;
  border-radius: 2px;
  transition: background 0.2s;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}

/* Firefox scrollbar */
.scrollbar-thin {
  scrollbar-width: thin;
  scrollbar-color: #6b7280 #374151;
}
</style>
