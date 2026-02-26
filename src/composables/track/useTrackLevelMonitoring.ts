import { ref, type Ref } from 'vue'

export interface LevelMonitoringState {
  trackLevelL: Ref<number>
  trackLevelR: Ref<number>
  phaseCorrelation: Ref<number>
}

export interface LevelMonitoringCallbacks {
  getTone?: () => any
  getMeters?: () => {
    meterL: any
    meterR: any
  }
  getAnalysers?: () => {
    analyserL: any
    analyserR: any
  }
  getIsStereo?: () => boolean
}

export function useTrackLevelMonitoring(callbacks: LevelMonitoringCallbacks = {}) {
  // State - exposed to parent
  const trackLevelL = ref(-60) // Left/Mono level
  const trackLevelR = ref(-60) // Right level (only for stereo)
  const phaseCorrelation = ref(0) // Phase correlation: -1 (out of phase) to +1 (mono)

  // Internal state
  let levelMonitorInterval: number | null = null
  const analyserBufferSize = 2048
  const analyserDataL = new Float32Array(analyserBufferSize)
  const analyserDataR = new Float32Array(analyserBufferSize)

  /**
   * Start monitoring audio levels and phase correlation
   */
  function startLevelMonitoring() {
    if (levelMonitorInterval !== null) return // Already running

    levelMonitorInterval = window.setInterval(() => {
      const Tone = callbacks.getTone?.()
      if (!Tone) return

      const meters = callbacks.getMeters?.()
      if (!meters) return

      const { meterL, meterR } = meters

      if (meterL) {
        const levelL = meterL.getValue() as number
        trackLevelL.value = Math.max(-60, levelL)

        const isStereo = callbacks.getIsStereo?.() || false

        // For stereo, also read right channel
        if (isStereo && meterR) {
          const levelR = meterR.getValue() as number
          trackLevelR.value = Math.max(-60, levelR)

          // Calculate phase correlation for stereo tracks
          const analysers = callbacks.getAnalysers?.()
          if (analysers?.analyserL && analysers?.analyserR) {
            const { analyserL, analyserR } = analysers

            // Get time-domain data from both channels
            analyserL.getFloatTimeDomainData(analyserDataL)
            analyserR.getFloatTimeDomainData(analyserDataR)

            // Calculate correlation: correlation = sum(L*R) / sqrt(sum(L²) * sum(R²))
            let sumLR = 0
            let sumL2 = 0
            let sumR2 = 0

            for (let i = 0; i < analyserBufferSize; i++) {
              const l = analyserDataL[i]
              const r = analyserDataR[i]
              sumLR += l * r
              sumL2 += l * l
              sumR2 += r * r
            }

            // Prevent division by zero and check for valid signal
            const denominator = Math.sqrt(sumL2 * sumR2)
            if (denominator > 0.00001) {
              const newCorrelation = sumLR / denominator
              // Smooth the correlation value slightly to reduce jitter
              phaseCorrelation.value = phaseCorrelation.value * 0.7 + newCorrelation * 0.3
            } else {
              // Silence: default to neutral correlation
              phaseCorrelation.value = 0
            }
          }
        } else {
          // Mono: copy left to right for consistency
          trackLevelR.value = trackLevelL.value
          phaseCorrelation.value = 1 // Mono is always perfectly correlated
        }
      }
    }, 50)
  }

  /**
   * Stop monitoring audio levels
   */
  function stopLevelMonitoring() {
    if (levelMonitorInterval !== null) {
      clearInterval(levelMonitorInterval)
      levelMonitorInterval = null
    }
  }

  /**
   * Reset level values to default
   */
  function resetLevels() {
    trackLevelL.value = -60
    trackLevelR.value = -60
    phaseCorrelation.value = 0
  }

  /**
   * Cleanup function - stops monitoring
   */
  function cleanup() {
    stopLevelMonitoring()
    resetLevels()
  }

  return {
    // State
    trackLevelL,
    trackLevelR,
    phaseCorrelation,
    
    // Analyser data (for external use like PhaseCorrelationModal)
    analyserBufferSize,
    analyserDataL,
    analyserDataR,
    
    // Functions
    startLevelMonitoring,
    stopLevelMonitoring,
    resetLevels,
    cleanup
  }
}
