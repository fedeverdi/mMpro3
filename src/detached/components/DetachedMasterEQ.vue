<template>
  <div class="detached-window-container w-full h-screen bg-gradient-to-b from-gray-900 to-gray-950 p-4">
    <MasterEQDisplay 
      :filters-data="masterEqFilters" 
      :master-channel="masterChannel"
      @update:filters-data="handleFiltersUpdate"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from 'vue'
import MasterEQDisplay from '../../components/master/MasterEQDisplay.vue'
import type { DetachedWindowClient } from '../../lib/detachedWindowClient'
import type { DetachedAudioEngineProxy } from '../../lib/detachedAudioEngineProxy'

interface Props {
  wsClient: DetachedWindowClient
  audioEngineProxy: DetachedAudioEngineProxy
}

const props = defineProps<Props>()

const masterEqFilters = ref<any[]>([])
const masterChannel = ref<any>(null)

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
}

// Handle filter updates from user (send commands via audio engine proxy)
const handleFiltersUpdate = async (filters: any[]) => {
  // Convert filters to backend format and send via WebSocket
  if (audioEngine?.setMasterParametricEQFilters) {
    try {
      const backendFilters = filters.map(f => ({
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
})

onUnmounted(() => {
  props.wsClient.off('message', handleMessage)
})
</script>

<style scoped>
.detached-window-container {
  overflow: auto;
}
</style>
