<template>
  <div class="w-full relative">
    <!-- Main Button (conditional) -->
    <div class="relative">
      <!-- Insert Effect Button (when NO effects) -->
      <button
        v-if="inserts.length === 0"
        ref="addButtonRef"
        @click="showAddEffect = !showAddEffect"
        class="w-full px-2 py-0.5 text-[0.6rem] font-bold rounded bg-green-600/20 hover:bg-green-600/40 text-green-300 border border-green-500/30 transition-all flex items-center justify-center gap-1"
        title="Add Insert Effect"
      >
        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
          <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
        </svg>
        FX
      </button>

      <!-- Show Button (when effects exist) -->
      <button
        v-else
        ref="effectsListButtonRef"
        @click="showEffectsList = !showEffectsList"
        class="w-full px-2 py-0.5 text-[0.6rem] font-bold rounded bg-blue-600/20 hover:bg-blue-600/40 text-blue-300 border border-blue-500/30 transition-all flex items-center justify-center gap-1"
        title="Show Insert Effects"
      >
        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/>
        </svg>
        Show
        <span class="text-[0.55rem] px-1 bg-blue-600 rounded">{{ inserts.length }}</span>
      </button>

      <!-- Popover for Effect Selection -->
      <div
        v-if="showAddEffect"
        class="absolute top-full left-0 mt-1 w-48 bg-gray-800 rounded-lg border border-gray-600 shadow-2xl z-50 p-2"
      >
        <div class="text-[0.65rem] text-gray-300 mb-1.5 font-semibold uppercase tracking-wide">Select Effect:</div>
        <div class="space-y-0.5">
          <button
            v-for="type in effectTypes"
            :key="type.value"
            @click="addEffect(type.value)"
            class="w-full px-2 py-1.5 text-xs rounded bg-gray-700 hover:bg-gray-600 text-gray-200 transition-all text-left font-medium"
          >
            {{ type.label }}
          </button>
        </div>
      </div>

      <!-- Popover for Effects List -->
      <div
        v-if="showEffectsList && inserts.length > 0"
        class="absolute top-full left-0 mt-1 w-full bg-gray-800 rounded-lg border border-gray-600 shadow-2xl z-50 p-2"
      >
        <div class="flex justify-between items-center mb-2">
          <div class="text-[0.65rem] text-gray-300 font-semibold uppercase tracking-wide">FX:</div>
          <button
            @click="showAddEffect = !showAddEffect"
            class="px-1.5 py-0.5 text-[0.6rem] rounded bg-green-600/20 hover:bg-green-600/40 text-green-300 border border-green-500/30 transition-all"
            title="Add Insert Effect"
          >
            + Add
          </button>
        </div>
        
        <draggable
          v-model="localInserts"
          item-key="id"
          handle=".drag-handle"
          @end="handleReorder"
          class="flex flex-col gap-0.5"
        >
          <template #item="{ element }">
            <div class="w-full">
              <!-- Effect Component (renders compact button with integrated drag handle and trash) -->
              <component
                :is="getEffectComponent(element.effect_type)"
                :track-number="trackNumber"
                :insert-id="element.id"
                :enabled="element.enabled"
                :track-level-l="trackLevelL"
                :track-level-r="trackLevelR"
                :phase-correlation="phaseCorrelation"
                @toggle="toggleEffect(element.id)"
                @remove="removeEffect(element.id)"
              />
            </div>
          </template>
        </draggable>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import draggable from 'vuedraggable'
import InsertGate from './InsertGate.vue'
import InsertCompressor from './InsertCompressor.vue'
import InsertReverb from './InsertReverb.vue'
import InsertDelay from './InsertDelay.vue'
import InsertExciter from './InsertExciter.vue'
import InsertDeEsser from './InsertDeEsser.vue'
import InsertChorus from './InsertChorus.vue'

interface InsertEffect {
  id: number
  effect_type: string
  enabled: boolean
}

const props = defineProps<{
  trackNumber: number
  inserts: InsertEffect[]
  trackLevelL: number
  trackLevelR: number
  phaseCorrelation: number
}>()

const emit = defineEmits<{
  add: [effectType: string]
  remove: [insertId: number]
  move: [payload: { insertId: number; position: number }]
  toggle: [payload: { insertId: number; enabled: boolean }]
}>()

const showAddEffect = ref(false)
const showEffectsList = ref(false)
const addButtonRef = ref<HTMLButtonElement | null>(null)
const effectsListButtonRef = ref<HTMLButtonElement | null>(null)

const effectTypes = [
  { value: 'gate', label: 'Gate' },
  { value: 'compressor', label: 'Compressor' },
  { value: 'reverb', label: 'Reverb' },
  { value: 'delay', label: 'Delay' },
  { value: 'exciter', label: 'Exciter' },
  { value: 'deesser', label: 'De-Esser' },
  { value: 'chorus', label: 'Chorus' },
]

// Close popovers when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as Node
  
  // Check if click is outside "Add Effect" popover
  if (addButtonRef.value && !addButtonRef.value.contains(target) && showAddEffect.value) {
    const addPopover = document.querySelector('.absolute.top-full.left-0.mt-1.w-48')
    if (!addPopover || !addPopover.contains(target)) {
      showAddEffect.value = false
    }
  }
  
  // Check if click is outside "Effects List" popover
  if (effectsListButtonRef.value && !effectsListButtonRef.value.contains(target) && showEffectsList.value) {
    const listPopover = document.querySelector('.absolute.top-full.left-0.mt-1.w-full')
    if (!listPopover || !listPopover.contains(target)) {
      showEffectsList.value = false
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// Local copy for drag & drop
const localInserts = computed({
  get: () => props.inserts,
  set: (value) => {
    // This will be called during drag, but we handle it in @end
  }
})

function addEffect(effectType: string) {
  emit('add', effectType)
  showAddEffect.value = false
}

function removeEffect(insertId: number) {
  emit('remove', insertId)
}

function toggleEffect(insertId: number) {
  const insert = props.inserts.find(i => i.id === insertId)
  if (insert) {
    emit('toggle', { insertId, enabled: !insert.enabled })
  }
}

function handleReorder(event: any) {
  const { oldIndex, newIndex } = event
  if (oldIndex !== newIndex) {
    const insertId = props.inserts[oldIndex].id
    emit('move', { insertId, position: newIndex })
  }
}

function getEffectLabel(type: string): string {
  const found = effectTypes.find(t => t.value === type)
  return found ? found.label : type
}

function getEffectComponent(type: string) {
  switch (type) {
    case 'gate': return InsertGate
    case 'compressor': return InsertCompressor
    case 'reverb': return InsertReverb
    case 'delay': return InsertDelay
    case 'exciter': return InsertExciter
    case 'deesser': return InsertDeEsser
    case 'chorus': return InsertChorus
    default: return null
  }
}
</script>

<style scoped>
/* Smooth dragging animation */
.sortable-ghost {
  opacity: 0.4;
  background: #4b5563;
}

.sortable-drag {
  opacity: 0.9;
}
</style>
