<template>
  <!-- Aux Sends Button -->
  <button
    @click="togglePanel"
    :class="[
      'w-full py-1 px-2 text-[0.5rem] font-bold rounded transition-all flex items-center justify-center gap-1',
      hasActiveSends 
        ? 'bg-teal-600 hover:bg-teal-700 text-white shadow-lg shadow-teal-500/30' 
        : 'bg-gray-700 hover:bg-gray-600 text-gray-400'
    ]"
    :title="`Aux Sends${hasActiveSends ? ' (Active)' : ''}`"
  >
    AUX
  </button>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import Knob from '../core/Knob.vue'

interface AuxBus {
  id: string | number
  name: string
  volume?: number
  muted?: boolean
  soloed?: boolean
  routeToMaster?: boolean
  selectedOutputDevice?: string | null
  node?: any  // Input node
  outputNode?: any  // Output node
  outputStreamDest?: MediaStreamAudioDestinationNode | null
  // FX Chain
  reverbNode?: any
  reverbEnabled?: boolean
  reverbParams?: { decay: number, preDelay: number, wet: number }
  delayNode?: any
  delayEnabled?: boolean
  delayParams?: { delayTime: number, feedback: number, wet: number }
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
  auxSendsData?: Record<string, { level: number, preFader: boolean, muted: boolean }>
  trackNode?: any
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update-sends', sends: Record<string, AuxSend>): void
  (e: 'toggle-panel'): void
}>()

const auxSends = ref<Record<string, AuxSend>>({})

function togglePanel() {
  emit('toggle-panel')
}

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
  // Use auxSendsData from parent if available, otherwise fall back to local auxSends
  const dataToCheck = props.auxSendsData || auxSends.value
  return Object.values(dataToCheck).some(send => 
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

// Expose functions for parent component
defineExpose({
  auxSends,
  updateAuxSend,
  togglePrePost,
  toggleMute
})
</script>
