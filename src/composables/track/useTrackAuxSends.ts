import { ref, toRaw, type Ref } from 'vue'

export interface AuxSendsState {
  auxSendsData: Ref<Record<string, { level: number, preFader: boolean, muted: boolean }>>
}

export interface AuxSendsCallbacks {
  getTone?: () => any
  getAudioNodes?: () => {
    volumeMerge: any
    balanceMerge: any
  }
  getAuxBuses?: () => Array<{ id: string, node?: any }> | undefined
  getIsMuted?: () => boolean
}

export function useTrackAuxSends(callbacks: AuxSendsCallbacks = {}) {
  // State
  const auxSendsData = ref<Record<string, { level: number, preFader: boolean, muted: boolean }>>({})
  
  // Internal aux send nodes map
  const auxSendNodes = new Map<string, { node: any, preFader: boolean }>()

  /**
   * Update aux send level
   */
  function updateLocalAuxSend(auxId: string, property: 'level', value: number) {
    if (!auxSendsData.value[auxId]) {
      auxSendsData.value[auxId] = {
        level: -60,
        preFader: false,
        muted: true
      }
    }

    auxSendsData.value[auxId].level = value

    // If level is increased from minimum, unmute automatically
    if (value > -60 && auxSendsData.value[auxId].muted) {
      auxSendsData.value[auxId].muted = false
    }

    // Trigger audio routing update
    handleAuxSendsUpdate(auxSendsData.value)
  }

  /**
   * Toggle pre/post fader for an aux send
   */
  function toggleLocalPrePost(auxId: string) {
    if (!auxSendsData.value[auxId]) return

    auxSendsData.value[auxId].preFader = !auxSendsData.value[auxId].preFader
    handleAuxSendsUpdate(auxSendsData.value)
  }

  /**
   * Toggle mute for an aux send
   */
  function toggleLocalMute(auxId: string) {
    if (!auxSendsData.value[auxId]) return

    auxSendsData.value[auxId].muted = !auxSendsData.value[auxId].muted
    handleAuxSendsUpdate(auxSendsData.value)
  }

  /**
   * Handle aux sends update - create/update/remove aux send nodes
   */
  function handleAuxSendsUpdate(sends: Record<string, any>) {
    const Tone = callbacks.getTone?.()
    const nodes = callbacks.getAudioNodes?.()
    
    if (!Tone || !nodes || !nodes.volumeMerge || !nodes.balanceMerge) return

    const { volumeMerge, balanceMerge } = nodes
    const isMuted = callbacks.getIsMuted?.() || false

    // Get all aux IDs from the sends object
    const sendIds = Object.keys(sends)

    // Remove sends that are no longer in the update or are inactive
    auxSendNodes.forEach((sendInfo, auxId) => {
      const send = sends[auxId]
      const isActive = send && send.level > -60 && !send.muted

      if (!send || !isActive) {
        // Disconnect and dispose the send node
        try {
          sendInfo.node.disconnect()
          sendInfo.node.dispose()
        } catch (e) {
          console.warn('Error disposing aux send node:', e)
        }
        auxSendNodes.delete(auxId)
      }
    })

    // Update or create active sends
    sendIds.forEach(auxId => {
      const send = sends[auxId]
      const isActive = send && send.level > -60 && !send.muted

      if (!isActive) return

      // Find the corresponding aux bus
      const auxBuses = callbacks.getAuxBuses?.()
      const auxBus = auxBuses?.find(bus => bus.id === auxId)
      if (!auxBus || !auxBus.node) {
        console.warn(`Aux bus ${auxId} not found or has no node`)
        return
      }

      // Get existing send info
      let sendInfo = auxSendNodes.get(auxId)
      let sendNode: any
      let needsNewNode = false

      // Check if preFader changed - need to recreate node to avoid double connections
      if (sendInfo && sendInfo.preFader !== send.preFader) {
        // Dispose old node completely
        try {
          sendInfo.node.disconnect()
          sendInfo.node.dispose()
        } catch (e) {
          console.warn('Error disposing old send node:', e)
        }
        auxSendNodes.delete(auxId)
        sendInfo = undefined
        needsNewNode = true
      }

      // Create new node if needed
      if (!sendInfo) {
        sendNode = new Tone.Gain(0)
        sendInfo = { node: sendNode, preFader: send.preFader }
        auxSendNodes.set(auxId, sendInfo)
        needsNewNode = true
      } else {
        sendNode = sendInfo.node
      }

      // Update send level (convert dB to gain)
      // If track is muted, force send to 0 regardless of send level
      const gainValue = isMuted ? 0 : Tone.dbToGain(send.level)
      sendNode.gain.value = gainValue

      // Connect new nodes
      if (needsNewNode) {
        try {
          // Choose source based on pre/post fader
          const source = send.preFader ? balanceMerge : volumeMerge
          source.connect(sendNode)
          sendNode.connect(toRaw(auxBus.node))

        } catch (e) {
          console.error('Error connecting aux send:', e)
        }
      }
    })
  }

  /**
   * Get aux send nodes map
   */
  function getAuxSendNodes() {
    return auxSendNodes
  }

  /**
   * Cleanup - dispose all aux send nodes
   */
  function cleanup() {
    auxSendNodes.forEach((sendInfo) => {
      try {
        sendInfo.node.disconnect()
        sendInfo.node.dispose()
      } catch (e) { }
    })
    auxSendNodes.clear()
  }

  return {
    // State
    auxSendsData,

    // Functions
    updateLocalAuxSend,
    toggleLocalPrePost,
    toggleLocalMute,
    handleAuxSendsUpdate,
    getAuxSendNodes,
    cleanup
  }
}
