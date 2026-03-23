<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
       @click="handleBackdropClick">
    <div class="bg-gray-900 rounded-xl border-2 border-gray-800 shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-y-auto m-4"
         @click.stop>
      
      <!-- Header -->
      <div class="sticky top-0 bg-gray-900 border-b border-gray-800 p-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-3 h-3 rounded-full" :class="`bg-${themeColor}-400`"></div>
          <h2 class="text-lg font-bold text-white">
            Keyboard Mapping - Deck {{ deckName }}
          </h2>
        </div>
        <button @click="close" 
                class="w-8 h-8 rounded-full bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white transition-colors flex items-center justify-center">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-4">
        <p class="text-sm text-gray-400">
          Click on a button below and press a key to assign it. Press ESC to clear a mapping.
        </p>

        <!-- Mapping List -->
        <div class="space-y-2">
          <!-- Play/Pause -->
          <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
            <span class="text-sm font-medium text-white">Play/Pause</span>
            <button @click="startMapping('play-pause')"
                    :class="getMappingButtonClass('play-pause')"
                    class="px-4 py-2 rounded font-mono text-sm transition-all">
              {{ getMappingLabel('play-pause') }}
            </button>
          </div>

          <!-- CUE -->
          <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
            <span class="text-sm font-medium text-white">CUE</span>
            <button @click="startMapping('cue')"
                    :class="getMappingButtonClass('cue')"
                    class="px-4 py-2 rounded font-mono text-sm transition-all">
              {{ getMappingLabel('cue') }}
            </button>
          </div>

          <!-- TAP Tempo -->
          <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
            <span class="text-sm font-medium text-white">TAP Tempo</span>
            <button @click="startMapping('tap')"
                    :class="getMappingButtonClass('tap')"
                    class="px-4 py-2 rounded font-mono text-sm transition-all">
              {{ getMappingLabel('tap') }}
            </button>
          </div>

          <!-- Loop IN -->
          <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
            <span class="text-sm font-medium text-white">Loop IN</span>
            <button @click="startMapping('loop-in')"
                    :class="getMappingButtonClass('loop-in')"
                    class="px-4 py-2 rounded font-mono text-sm transition-all">
              {{ getMappingLabel('loop-in') }}
            </button>
          </div>

          <!-- Loop OUT -->
          <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
            <span class="text-sm font-medium text-white">Loop OUT</span>
            <button @click="startMapping('loop-out')"
                    :class="getMappingButtonClass('loop-out')"
                    class="px-4 py-2 rounded font-mono text-sm transition-all">
              {{ getMappingLabel('loop-out') }}
            </button>
          </div>

          <!-- Loop Speed Buttons -->
          <div class="border-t border-gray-700 pt-3 mt-3">
            <p class="text-xs text-gray-500 mb-2 uppercase tracking-wider">Loop Speed</p>
            
            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
              <span class="text-sm font-medium text-white">Loop 1</span>
              <button @click="startMapping('loop-1')"
                      :class="getMappingButtonClass('loop-1')"
                      class="px-4 py-2 rounded font-mono text-sm transition-all">
                {{ getMappingLabel('loop-1') }}
              </button>
            </div>

            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
              <span class="text-sm font-medium text-white">Loop 1/2</span>
              <button @click="startMapping('loop-1/2')"
                      :class="getMappingButtonClass('loop-1/2')"
                      class="px-4 py-2 rounded font-mono text-sm transition-all">
                {{ getMappingLabel('loop-1/2') }}
              </button>
            </div>

            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
              <span class="text-sm font-medium text-white">Loop 1/4</span>
              <button @click="startMapping('loop-1/4')"
                      :class="getMappingButtonClass('loop-1/4')"
                      class="px-4 py-2 rounded font-mono text-sm transition-all">
                {{ getMappingLabel('loop-1/4') }}
              </button>
            </div>

            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
              <span class="text-sm font-medium text-white">Loop 1/8</span>
              <button @click="startMapping('loop-1/8')"
                      :class="getMappingButtonClass('loop-1/8')"
                      class="px-4 py-2 rounded font-mono text-sm transition-all">
                {{ getMappingLabel('loop-1/8') }}
              </button>
            </div>

            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
              <span class="text-sm font-medium text-white">Loop 1/16</span>
              <button @click="startMapping('loop-1/16')"
                      :class="getMappingButtonClass('loop-1/16')"
                      class="px-4 py-2 rounded font-mono text-sm transition-all">
                {{ getMappingLabel('loop-1/16') }}
              </button>
            </div>

            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg hover:bg-gray-800 transition-colors">
              <span class="text-sm font-medium text-white">Loop 1/32</span>
              <button @click="startMapping('loop-1/32')"
                      :class="getMappingButtonClass('loop-1/32')"
                      class="px-4 py-2 rounded font-mono text-sm transition-all">
                {{ getMappingLabel('loop-1/32') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="sticky bottom-0 bg-gray-900 border-t border-gray-800 p-4 flex justify-between items-center">
        <button @click="resetAll"
                class="px-4 py-2 bg-red-900/50 hover:bg-red-900 text-red-400 hover:text-red-300 rounded font-medium text-sm transition-colors">
          Reset All
        </button>
        <button @click="close"
                class="px-6 py-2 rounded font-medium text-sm transition-colors"
                :class="`bg-${themeColor}-600 hover:bg-${themeColor}-700 text-white`">
          Done
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

interface Props {
  isOpen: boolean
  deckName: string
  themeColor: 'cyan' | 'orange'
  initialMappings?: Record<string, string>
}

const props = withDefaults(defineProps<Props>(), {
  isOpen: false,
  initialMappings: () => ({})
})

const emit = defineEmits<{
  close: []
  'update:mappings': [mappings: Record<string, string>]
}>()

const mappings = ref<Record<string, string>>({ ...props.initialMappings })
const waitingForKey = ref<string | null>(null)

function startMapping(action: string) {
  waitingForKey.value = action
}

function getMappingLabel(action: string): string {
  if (waitingForKey.value === action) {
    return 'Press a key...'
  }
  return mappings.value[action] || 'Not mapped'
}

function getMappingButtonClass(action: string): string {
  if (waitingForKey.value === action) {
    return 'bg-yellow-600 text-white animate-pulse'
  }
  return mappings.value[action] 
    ? 'bg-green-900/50 text-green-400 border border-green-700' 
    : 'bg-gray-700 text-gray-400 border border-gray-600'
}

function handleKeyDown(event: KeyboardEvent) {
  if (!waitingForKey.value) return
  
  event.preventDefault()
  event.stopPropagation()
  
  // ESC to clear mapping
  if (event.key === 'Escape') {
    delete mappings.value[waitingForKey.value]
    waitingForKey.value = null
    emit('update:mappings', { ...mappings.value })
    return
  }
  
  // Check if key is already mapped to another action
  const existingAction = Object.entries(mappings.value).find(([_, key]) => key === event.key)
  if (existingAction && existingAction[0] !== waitingForKey.value) {
    // Remove old mapping
    delete mappings.value[existingAction[0]]
  }
  
  // Set new mapping
  mappings.value[waitingForKey.value] = event.key
  waitingForKey.value = null
  
  emit('update:mappings', { ...mappings.value })
}

function handleBackdropClick() {
  if (!waitingForKey.value) {
    close()
  }
}

function close() {
  waitingForKey.value = null
  emit('close')
}

function resetAll() {
  if (confirm('Are you sure you want to reset all keyboard mappings?')) {
    mappings.value = {}
    emit('update:mappings', {})
  }
}

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    document.addEventListener('keydown', handleKeyDown)
  } else {
    document.removeEventListener('keydown', handleKeyDown)
    waitingForKey.value = null
  }
})

watch(() => props.initialMappings, (newMappings) => {
  mappings.value = { ...newMappings }
}, { deep: true })

onMounted(() => {
  if (props.isOpen) {
    document.addEventListener('keydown', handleKeyDown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
/* Add any custom styles if needed */
</style>
