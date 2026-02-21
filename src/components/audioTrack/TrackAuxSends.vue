<template>
  <div class="relative w-full">
    <!-- Backdrop (click to close) -->
    <Transition name="fade">
      <div 
        v-if="showModal"
        class="fixed inset-0 bg-black/60 z-[9998]"
        @click="showModal = false"
      ></div>
    </Transition>

    <!-- Aux Sends Button -->
    <button
      @click="showModal = !showModal"
      :class="[
        'w-full py-1 px-2 text-[0.5rem] font-bold rounded transition-all flex items-center justify-center gap-1 relative z-[9997]',
        hasActiveSends 
          ? 'bg-teal-600 hover:bg-teal-700 text-white shadow-lg shadow-teal-500/30' 
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

    <!-- Aux Sends Panel (centered on track) -->
    <Transition name="fade-scale">
      <div 
        v-if="showModal" 
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[8.5rem] bg-gray-900 rounded-lg border border-teal-600 shadow-2xl z-[9999] max-h-[70vh] overflow-y-auto custom-scrollbar"
        @click.stop
      >
      <!-- Header -->
      <div class="sticky top-0 bg-gray-900 border-b border-teal-600/50 px-2 py-1.5 flex justify-between items-center">
        <span class="text-[0.65rem] font-bold text-teal-300">AUX SENDS</span>
        <button 
          @click="showModal = false"
          class="text-gray-400 hover:text-white text-lg leading-none"
        >
          ×
        </button>
      </div>

      <!-- Content -->
      <div class="p-2">
        <div v-if="auxBuses.length === 0" class="text-center py-4 text-gray-500 text-[0.6rem]">
          No aux buses
        </div>

        <div v-else class="flex flex-col gap-2">
          <div 
            v-for="(aux, index) in auxBuses" 
            :key="aux.id"
            class="flex flex-col items-center gap-1 bg-gray-800/50 rounded-lg p-2 border border-teal-700/30"
          >
            <!-- Aux name -->
            <div class="text-[0.6rem] font-bold text-teal-300 text-center">
              {{ aux.name }}
            </div>

            <!-- Send level knob -->
            <div class="scale-[0.6] -my-2">
              <Knob
                :modelValue="auxSends[aux.id]?.level || -60"
                @update:modelValue="(val) => updateAuxSend(aux.id, 'level', val)"
                :min="-60"
                :max="10"
                :step="0.1"
                label="Level"
                unit="dB"
                color="#14b8a6"
              />
            </div>

            <!-- Controls row -->
            <div class="w-full flex gap-1">
              <!-- Pre/Post fader toggle -->
              <button
                @click="togglePrePost(aux.id)"
                :class="[
                  'flex-1 py-0.5 px-1 text-[0.5rem] font-bold rounded transition-colors',
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
                  'flex-1 py-0.5 px-1 text-[0.5rem] font-bold rounded transition-colors',
                  auxSends[aux.id]?.muted 
                    ? 'bg-yellow-600 text-white' 
                    : 'bg-gray-700 text-gray-400'
                ]"
              >
                M
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    </Transition>
  </div>
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
  background: rgba(20, 184, 166, 0.5);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(20, 184, 166, 0.7);
}

/* Fade transition for backdrop */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Fade + scale transition for panel */
.fade-scale-enter-active,
.fade-scale-leave-active {
  transition: all 0.25s ease;
}

.fade-scale-enter-from,
.fade-scale-leave-to {
  opacity: 0;
  transform: translate(-50%, -50%) scale(0.9);
}

.fade-scale-enter-to,
.fade-scale-leave-from {
  opacity: 1;
  transform: translate(-50%, -50%) scale(1);
}

</style>
