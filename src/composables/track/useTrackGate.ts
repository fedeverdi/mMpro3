export interface TrackGateCallbacks {
  getGateEnabled?: () => boolean
  setGateEnabled?: (enabled: boolean) => void
  getAudioNodes?: () => {
    gainNode: any
    gate: any
    eq3: any
    gateMeter: any
  }
  getTrackGateRef?: () => any
  getGateState?: () => {
    gateThreshold: number
    gateAttack: number
    gateRelease: number
    gateRange: number
    gateIsOpen: boolean
    gateMonitoringId: number | null
  }
  setGateState?: (updates: {
    gateThreshold?: number
    gateAttack?: number
    gateRelease?: number
    gateRange?: number
    gateIsOpen?: boolean
    gateMonitoringId?: number | null
  }) => void
}

export function useTrackGate(callbacks: TrackGateCallbacks = {}) {
  /**
   * Toggle gate on/off and reconnect audio chain
   * Physically reconnects chain to preserve stereo
   */
  function toggleGate() {
    const currentEnabled = callbacks.getGateEnabled?.() ?? false
    callbacks.setGateEnabled?.(!currentEnabled)
    const newEnabled = !currentEnabled

    const nodes = callbacks.getAudioNodes?.()
    if (!nodes?.gainNode || !nodes?.gate || !nodes?.eq3) return

    const { gainNode, gate, eq3, gateMeter } = nodes

    // Disconnect gainNode from eq3 and gate
    try {
      gainNode.disconnect(eq3)
    } catch (e) { }
    try {
      gainNode.disconnect(gate)
    } catch (e) { }

    // Disconnect gate from eq3
    try {
      gate.disconnect(eq3)
    } catch (e) { }

    if (newEnabled) {
      // Apply real parameters from component
      const trackGateRef = callbacks.getTrackGateRef?.()
      const params = trackGateRef?.getParams() || {
        threshold: -45,
        attack: 0.005,
        release: 0.3,
        range: -30
      }
      
      callbacks.setGateState?.({
        gateThreshold: params.threshold,
        gateAttack: params.attack,
        gateRelease: params.release,
        gateRange: params.range
      })

      // Reset gate to open state
      gate.gain.value = 1
      callbacks.setGateState?.({ gateIsOpen: true })

      // Chain: gainNode → gate → eq3
      gainNode.connect(gate)
      gate.connect(eq3)

      // Ensure gateMeter stays connected for monitoring
      if (gateMeter) {
        try {
          gainNode.connect(gateMeter)
        } catch (e) { }
      }

      // Start gate monitoring
      startGateMonitoring()
    } else {
      // Stop gate monitoring
      stopGateMonitoring()

      // Bypass gate: gainNode → eq3 (skip gate)
      gainNode.connect(eq3)

      // Ensure gateMeter stays connected for monitoring
      if (gateMeter) {
        try {
          gainNode.connect(gateMeter)
        } catch (e) { }
      }
    }
  }

  /**
   * Update gate parameters
   */
  function handleGateParamsUpdate(params: { threshold: number, attack: number, release: number, range: number }) {
    const nodes = callbacks.getAudioNodes?.()
    if (!nodes?.gate) return

    // Update custom gate parameters
    callbacks.setGateState?.({
      gateThreshold: params.threshold,
      gateAttack: params.attack,
      gateRelease: params.release,
      gateRange: params.range
    })
  }

  /**
   * Start gate monitoring
   * Custom gate monitoring using requestAnimationFrame
   */
  function startGateMonitoring() {
    const state = callbacks.getGateState?.()
    if (state?.gateMonitoringId || !callbacks.getGateEnabled?.()) return

    const nodes = callbacks.getAudioNodes?.()
    if (!nodes?.gateMeter || !nodes?.gate) return

    function updateGate() {
      const currentEnabled = callbacks.getGateEnabled?.() ?? false
      const currentNodes = callbacks.getAudioNodes?.()
      
      if (!currentEnabled || !currentNodes?.gateMeter || !currentNodes?.gate) {
        callbacks.setGateState?.({ gateMonitoringId: null })
        return
      }

      try {
        const currentState = callbacks.getGateState?.()
        if (!currentState) return

        const { gateMeter, gate } = currentNodes
        const { gateThreshold, gateAttack, gateRelease, gateRange, gateIsOpen } = currentState

        // Get current level from gateMeter (already in dB)
        const currentLevel = gateMeter.getValue()

        // Determine if gate should be open
        const shouldBeOpen = currentLevel > gateThreshold

        if (shouldBeOpen !== gateIsOpen) {
          callbacks.setGateState?.({ gateIsOpen: shouldBeOpen })

          if (shouldBeOpen) {
            // Open gate: ramp to unity gain (0dB = 1)
            gate.gain.rampTo(1, gateAttack)
          } else {
            // Close gate: ramp to range attenuation
            // Convert dB to linear gain: gain = 10^(dB/20)
            const targetGain = Math.pow(10, gateRange / 20)
            gate.gain.rampTo(targetGain, gateRelease)
          }
        }
      } catch (error) {
        console.error('[Gate] Monitoring error:', error)
      }

      const monitoringId = requestAnimationFrame(updateGate)
      callbacks.setGateState?.({ gateMonitoringId: monitoringId })
    }

    updateGate()
  }

  /**
   * Stop gate monitoring
   */
  function stopGateMonitoring() {
    const state = callbacks.getGateState?.()
    if (state?.gateMonitoringId) {
      cancelAnimationFrame(state.gateMonitoringId)
      callbacks.setGateState?.({ gateMonitoringId: null })
    }

    // Reset gate to unity gain when stopping
    const nodes = callbacks.getAudioNodes?.()
    if (nodes?.gate && nodes.gate.gain) {
      nodes.gate.gain.rampTo(1, 0.05)
    }
    callbacks.setGateState?.({ gateIsOpen: false })
  }

  return {
    toggleGate,
    handleGateParamsUpdate,
    startGateMonitoring,
    stopGateMonitoring
  }
}
