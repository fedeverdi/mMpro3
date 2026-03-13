<template>
  <div class="detached-window-container w-full h-screen bg-gradient-to-b from-gray-900 to-gray-950 p-4">
    <MasterFX 
      :master-section="masterSection"
      :fx-effects="masterFxEffects"
      @output-node="handleOutputNode"
      @component="handleComponent"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from 'vue'
import MasterFX from '../../components/master/MasterFX.vue'
import type { DetachedWindowClient } from '../../lib/detachedWindowClient'
import type { DetachedAudioEngineProxy } from '../../lib/detachedAudioEngineProxy'

interface Props {
  wsClient: DetachedWindowClient
  audioEngineProxy: DetachedAudioEngineProxy
}

const props = defineProps<Props>()

const masterFxEffects = ref<any[]>([])
const masterSection = ref<any>(null)
const masterFxComponent = ref<any>(null)

// Inject audio engine proxy (provided by parent app)
const audioEngine = inject<DetachedAudioEngineProxy>('audioEngine')

// Handle real-time updates from WebSocket
const handleMessage = (data: any) => {
  console.log('[DetachedMasterFX] Received message:', data.type, data)
  
  // Update master FX effects when parameters change
  if (data.type === 'parameters' || data.type === 'parameters_changed') {
    console.log('[DetachedMasterFX] Checking for fx_effects in master:', data.master)
    if (data.master?.fx_effects && Array.isArray(data.master.fx_effects)) {
      console.log('[DetachedMasterFX] Received fx_effects:', data.master.fx_effects)
      // Direct assignment - backend format is already correct
      masterFxEffects.value = data.master.fx_effects
    } else {
      console.log('[DetachedMasterFX] No fx_effects found in master data')
    }
  }
  
  // Also handle levels updates (in case fx_effects is sent there too)
  if (data.type === 'levels') {
    if (data.master?.fx_effects && Array.isArray(data.master.fx_effects)) {
      console.log('[DetachedMasterFX] Received fx_effects from levels:', data.master.fx_effects)
      masterFxEffects.value = data.master.fx_effects
    }
  }
}

const handleOutputNode = (node: any) => {
  // Output node is not applicable in detached window (no audio routing)
}

const handleComponent = (component: any) => {
  // Store component reference for snapshot/restore
  masterFxComponent.value = component
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
