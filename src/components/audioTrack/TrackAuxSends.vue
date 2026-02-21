<template>
  <!-- Aux Sends Button -->
  <button
    @click="showModal = true"
    :class="[
      'w-full py-1.5 px-2 text-xs font-bold rounded transition-all flex items-center justify-center gap-1',
      hasActiveSends 
        ? 'bg-purple-600 hover:bg-purple-700 text-white shadow-lg shadow-purple-500/30' 
        : 'bg-gray-700 hover:bg-gray-600 text-gray-400'
    ]"
    :title="`Aux Sends${hasActiveSends ? ' (Active)' : ''}`"
  >
    <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
      <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
      <path fill-rule="evenodd" d="M3.293 14.707a1 1 0 010-1.414L6.586 10 3.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
    </svg>
    AUX
  </button>

  <!-- Aux Sends Modal -->
  <Teleport to="body">
    <div 
      v-if="showModal" 
      class="fixed inset-0 bg-black/80 flex items-center justify-center z-[10000]"
      @mousedown.self="showModal = false"
    >
      <div 
        class="bg-gray-900 rounded-lg border-2 border-purple-600 p-4 max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto custom-scrollbar"
        @click.stop
      >
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold text-purple-300">Aux Sends - Track {{ trackNumber }}</h3>
          <button 
            @click="showModal = false"
            class="text-gray-400 hover:text-white text-2xl leading-none"
          >
            ×
          </button>
        </div>

        <div v-if="auxBuses.length === 0" class="text-center py-8 text-gray-500">
          No aux buses available. Add one in the Aux Master section.
        </div>

        <div v-else class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
          <div 
            v-for="(aux, index) in auxBuses" 
            :key="aux.id"
            class="flex flex-col items-center gap-2 bg-gray-800/50 rounded-lg p-3 border border-purple-700/30"
          >
            <!-- Aux name -->
            <div class="text-xs font-bold text-purple-300 text-center">
              {{ aux.name }}
            </div>

            <!-- Send level knob -->
            <Knob
              :modelValue="auxSends[aux.id]?.level || 0"
              @update:modelValue="(val) => updateAuxSend(aux.id, 'level', val)"
              :min="-60"
              :max="10"
              :step="0.1"
              label="Level"
              unit="dB"
              color="#a855f7"
              :size="60"
            />

            <!-- Pre/Post fader toggle -->
            <button
              @click="togglePrePost(aux.id)"
              :class="[
                'w-full py-1 px-2 text-[0.6rem] font-bold rounded transition-colors',
                auxSends[aux.id]?.preFader 
                  ? 'bg-blue-600 text-white' 
                  : 'bg-gray-700 text-gray-400'
              ]"
            >
              {{ auxSends[aux.id]?.preFader ? 'PRE' : 'POST' }}
            </button>

            <!-- Mute send -->
            <button
              @click="toggleMute(aux.id)"
              :class="[
                'w-full py-1 px-2 text-[0.6rem] font-bold rounded transition-colors',
                auxSends[aux.id]?.muted 
                  ? 'bg-yellow-600 text-white' 
                  : 'bg-gray-700 text-gray-400'
              ]"
            >
              {{ auxSends[aux.id]?.muted ? 'MUTED' : 'MUTE' }}
            </button>
          </div>
        </div>

        <div class="mt-4 flex justify-end">
          <button
            @click="showModal = false"
            class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded font-bold"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import Knob from '../core/Knob.vue'

interface AuxBus {
  id: string
  name: string
  volume: number
  muted: boolean
  soloed: boolean
  routeToMaster: boolean
  node?: any
}

interface AuxSend {
  level: number
  preFader: boolean
  muted: boolean
  sendNode?: any
}

interface Props {
  trackNumber: number
  auxBuses: AuxBus[]
  trackNode?: any
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update-sends', sends: Record<string, AuxSend>): void
}>()

const showModal = ref(false)
const auxSends = ref<Record<string, AuxSend>>({})

// Initialize sends for each aux bus
watch(() => props.auxBuses, (newBuses) => {
  newBuses.forEach(aux => {
    if (!auxSends.value[aux.id]) {
      auxSends.value[aux.id] = {
        level: -60, // Start at minimum
        preFader: false,
        muted: true // Start muted
      }
    }
  })
}, { immediate: true, deep: true })

// Check if any sends are active (level > -60 and not muted)
const hasActiveSends = computed(() => {
  return Object.values(auxSends.value).some(send => 
    send.level > -60 && !send.muted
  )
})

// Update aux send level
function updateAuxSend(auxId: string, property: 'level', value: number) {
  if (!auxSends.value[auxId]) {
    auxSends.value[auxId] = {
      level: -60,
      preFader: false,
      muted: true
    }
  }
  
  auxSends.value[auxId].level = value
  
  // If level is increased from minimum, unmute automatically
  if (value > -60 && auxSends.value[auxId].muted) {
    auxSends.value[auxId].muted = false
  }
  
  emit('update-sends', auxSends.value)
}

// Toggle pre/post fader
function togglePrePost(auxId: string) {
  if (!auxSends.value[auxId]) return
  
  auxSends.value[auxId].preFader = !auxSends.value[auxId].preFader
  emit('update-sends', auxSends.value)
}

// Toggle mute
function toggleMute(auxId: string) {
  if (!auxSends.value[auxId]) return
  
  auxSends.value[auxId].muted = !auxSends.value[auxId].muted
  emit('update-sends', auxSends.value)
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(168, 85, 247, 0.5);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(168, 85, 247, 0.7);
}
</style>
