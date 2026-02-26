import { type Ref } from 'vue'

export interface TrackCompressorCallbacks {
  getCompressorEnabled?: () => boolean
  setCompressorEnabled?: (enabled: boolean) => void
  getAudioNodes?: () => {
    eq3: any
    compressor: any
    balanceSplit: any
  }
  getTrackCompressorRef?: () => any
}

export function useTrackCompressor(callbacks: TrackCompressorCallbacks = {}) {
  /**
   * Toggle compressor on/off and reconnect audio chain
   * Physically reconnects chain to preserve stereo
   */
  function toggleCompressor() {
    const currentEnabled = callbacks.getCompressorEnabled?.() ?? false
    callbacks.setCompressorEnabled?.(!currentEnabled)
    const newEnabled = !currentEnabled

    const nodes = callbacks.getAudioNodes?.()
    if (!nodes?.eq3 || !nodes?.compressor) return

    const { eq3, compressor, balanceSplit } = nodes

    // Disconnect eq3 from everything
    try {
      eq3.disconnect(compressor)
    } catch (e) { }
    try {
      eq3.disconnect(balanceSplit)
    } catch (e) { }

    // Disconnect compressor from balanceSplit
    try {
      compressor.disconnect(balanceSplit)
    } catch (e) { }

    if (newEnabled) {
      // Apply real parameters from component
      const trackCompressorRef = callbacks.getTrackCompressorRef?.()
      const params = trackCompressorRef?.getParams() || {
        threshold: -20,
        ratio: 4,
        attack: 0.1,
        release: 0.25
      }
      
      // Chain: eq3 → compressor → balanceSplit
      eq3.connect(compressor)
      compressor.connect(balanceSplit)
      
      // Smooth fade in by ramping ratio from 1:1 (bypass) to target
      compressor.threshold.value = params.threshold
      compressor.ratio.value = 1
      compressor.attack.value = params.attack
      compressor.release.value = params.release
      compressor.ratio.rampTo(params.ratio, 0.15)
    } else {
      // Bypass compressor completely: eq3 → balanceSplit (skip compressor)
      eq3.connect(balanceSplit)
    }
  }

  return {
    toggleCompressor
  }
}
