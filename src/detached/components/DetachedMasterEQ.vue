<template>
  <div class="detached-window-container w-full h-screen bg-gradient-to-b from-gray-900 to-gray-950">
    <ParametricEQModal
      v-model="isModalOpen"
      :trackNumber="0"
      :eq-filters="masterEqFilters"
      :is-detached="true"
      title="Parametric EQ - Master Output"
      @update="handleFiltersUpdate"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from 'vue'
import ParametricEQModal from '../../components/master/ParametricEQModal.vue'
import type { DetachedWindowClient } from '../../lib/detachedWindowClient'
import type { DetachedAudioEngineProxy } from '../../lib/detachedAudioEngineProxy'

interface Props {
  wsClient: DetachedWindowClient
  audioEngineProxy: DetachedAudioEngineProxy
}

const props = defineProps<Props>()

const masterEqFilters = ref<any[]>([])
const isModalOpen = ref(true) // Always open in detached window

// Inject audio engine proxy (provided by parent app)
const audioEngine = inject<DetachedAudioEngineProxy>('audioEngine')

// Handle real-time updates from WebSocket
const handleMessage = (data: any) => {
  // Update master EQ filters when parameters change
  if (data.type === 'parameters' || data.type === 'parameters_changed') {
    if (data.master?.eq_filters) {
      // Convert backend format (q lowercase) to frontend format (Q uppercase)
      masterEqFilters.value = data.master.eq_filters.map((f: any) => ({
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.q // Backend uses lowercase 'q', frontend expects uppercase 'Q'
      }))
    }
  }
  
  // Handle levels updates (if needed for visualizations)
  if (data.type === 'levels') {
    if (data.master?.eq_filters) {
      // Convert backend format to frontend format
      masterEqFilters.value = data.master.eq_filters.map((f: any) => ({
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.q
      }))
    }
  }
  
  // Handle FFT data updates for visualization in EQ modal
  if (data.type === 'fft' && audioEngine?.state?.value) {
    audioEngine.state.value.fftData = {
      binsLeft: new Float32Array(data.bins_left || data.binsLeft || []),
      binsRight: new Float32Array(data.bins_right || data.binsRight || []),
      sampleRate: data.sample_rate || data.sampleRate || 48000
    }
  }
}

// Handle filter updates from user (send commands via audio engine proxy)
const handleFiltersUpdate = async (filters: any) => {
  // ParametricEQModal passes an object with filtersData property
  const filtersArray = filters?.filtersData || filters
  
  if (!filtersArray || !Array.isArray(filtersArray)) return
  
  // Update local state for real-time preview
  masterEqFilters.value = filtersArray.map((f: any) => ({
    type: f.type,
    frequency: f.frequency,
    gain: f.gain,
    Q: f.Q
  }))
  
  // Convert filters to backend format and send via WebSocket
  if (audioEngine?.setMasterParametricEQFilters) {
    try {
      const backendFilters = filtersArray.map((f: any) => ({
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        q: f.Q
      }))
      
      await audioEngine.setMasterParametricEQFilters(backendFilters)
    } catch (error) {
      console.error('[DetachedMasterEQ] Failed to update filters:', error)
    }
  }
}

onMounted(() => {
  props.wsClient.on('message', handleMessage)
  props.wsClient.on('fft', handleMessage)
})

onUnmounted(() => {
  props.wsClient.off('message', handleMessage)
  props.wsClient.off('fft', handleMessage)
})
</script>

<style scoped>
.detached-window-container {
  overflow: auto;
}
</style>
