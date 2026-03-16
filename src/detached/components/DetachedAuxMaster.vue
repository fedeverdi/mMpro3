<template>
  <div class="detached-window-container w-full h-screen bg-gradient-to-b from-gray-900 to-gray-950 p-4">
    <AuxMaster 
      :aux-buses="auxBuses"
      :master-channel="masterChannel"
      :subgroups="subgroups"
      @add-aux="handleAddAux"
      @remove-aux="handleRemoveAux"
      @update-aux="handleUpdateAux"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from 'vue'
import AuxMaster from '../../components/master/AuxMaster.vue'
import type { DetachedWindowClient } from '../../lib/detachedWindowClient'
import type { DetachedAudioEngineProxy } from '../../lib/detachedAudioEngineProxy'

interface Props {
  wsClient: DetachedWindowClient
  audioEngineProxy: DetachedAudioEngineProxy
}

const props = defineProps<Props>()

// Inject audio engine proxy (provided by parent app)
const audioEngine = inject<DetachedAudioEngineProxy>('audioEngine')

const auxBuses = ref<any[]>([])
const masterChannel = ref<any>(null)
const subgroups = ref<any[]>([])

const handleMessage = (data: any) => {
  // Handle aux buses state updates
  if (data.type === 'aux_buses_state') {
    if (data.auxBuses) {
      auxBuses.value = data.auxBuses
    }
  }
  
  // Handle parameters_changed or levels updates (for backend aux parameters if needed)
  if (data.type === 'parameters' || data.type === 'parameters_changed' || data.type === 'levels') {
    if (data.auxes) {
      // Backend aux parameters (if needed for additional data)
      // Currently not used since aux buses are managed by frontend
    }
  }
}

// Handle aux operations
const handleAddAux = () => {
  // Detached windows are observers - can't add aux from here
  console.warn('[DetachedAuxMaster] Cannot add aux from detached window')
}

const handleRemoveAux = (index: number) => {
  // Detached windows are observers - can't remove aux from here
  console.warn('[DetachedAuxMaster] Cannot remove aux from detached window')
}

const handleUpdateAux = async (index: number, updatedAux: any) => {
  if (!audioEngine || index < 0 || index >= auxBuses.value.length) return
  
  const aux = auxBuses.value[index]
  
  // Extract aux ID from string (e.g., "aux-0" -> 0)
  const auxId = parseInt(aux.id.replace('aux-', ''))

  // Send updates to Rust engine via WebSocket
  // Update volume (gain)
  if (updatedAux.volume !== aux.volume) {
    const linearGain = Math.pow(10, updatedAux.volume / 20)
    await audioEngine.setAuxBusGain(auxId, linearGain)
  }

  // Update mute
  if (updatedAux.muted !== aux.muted) {
    await audioEngine.setAuxBusMute(auxId, updatedAux.muted)
  }

  // Update routing to master
  if (updatedAux.routeToMaster !== aux.routeToMaster) {
    await audioEngine.setAuxBusRouteToMaster(auxId, updatedAux.routeToMaster)
  }

  // Update reverb enabled state
  if (updatedAux.reverbEnabled !== aux.reverbEnabled) {
    const enabled = updatedAux.reverbEnabled ?? false
    const reverbParams = updatedAux.reverbParams
    const roomSize = reverbParams?.roomSize ?? 0.5
    const damping = reverbParams?.damping ?? 0.5
    const wet = reverbParams?.wet ?? 1.0
    const width = reverbParams?.width ?? 1.0
    const preDelay = reverbParams?.preDelay ?? 0.0

    await audioEngine.setAuxBusReverb(auxId, enabled, roomSize, damping, wet, width, preDelay)
  }

  // Update reverb parameters (if only params changed, not enabled state)
  if (updatedAux.reverbEnabled && updatedAux.reverbParams && 
      JSON.stringify(updatedAux.reverbParams) !== JSON.stringify(aux.reverbParams)) {
    const reverbParams = updatedAux.reverbParams
    await audioEngine.setAuxBusReverb(
      auxId,
      true,
      reverbParams.roomSize,
      reverbParams.damping,
      reverbParams.wet,
      reverbParams.width,
      reverbParams.preDelay ?? 0.0
    )
  }

  // Update delay enabled state
  if (updatedAux.delayEnabled !== aux.delayEnabled) {
    const enabled = updatedAux.delayEnabled ?? false
    const delayParams = updatedAux.delayParams
    const time = delayParams?.delayTime ?? 0.5
    const feedback = delayParams?.feedback ?? 0.3
    const wet = delayParams?.wet ?? 0.5

    await audioEngine.setAuxBusDelay(auxId, enabled, time * 1000, feedback, wet)
  }

  // Update delay parameters (if only params changed, not enabled state)
  if (updatedAux.delayEnabled && updatedAux.delayParams &&
      JSON.stringify(updatedAux.delayParams) !== JSON.stringify(aux.delayParams)) {
    const delayParams = updatedAux.delayParams
    await audioEngine.setAuxBusDelay(
      auxId,
      true,
      delayParams.delayTime * 1000,
      delayParams.feedback,
      delayParams.wet
    )
  }

  // Handle output device selection
  if (updatedAux.selectedOutputDevice !== aux.selectedOutputDevice) {
    const deviceId = updatedAux.selectedOutputDevice
    await audioEngine.setAuxBusSelectedOutput(auxId, deviceId)

    const parts = deviceId?.split(':') || []
    const uiChannel = parts[1] ? parseInt(parts[1]) : 1
    const backendChannel = uiChannel - 1
    const leftChannel = backendChannel
    const rightChannel = backendChannel

    if (deviceId && deviceId !== 'no-output') {
      await audioEngine.setAuxBusOutputChannels(auxId, leftChannel, rightChannel)
    }
  }
  
  // Update local state to reflect changes immediately
  auxBuses.value[index] = { ...aux, ...updatedAux }
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
