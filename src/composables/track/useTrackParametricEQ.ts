import { ref, toRaw, type Ref } from 'vue'

export interface ParametricEQState {
  eqFiltersData: Ref<any[]>
}

export interface ParametricEQCallbacks {
  getAudioNodes?: () => {
    eq3: any
    channelSplit: any
    waveform: any
    compressor: any
    balanceSplit: any
  }
  getCompressorEnabled?: () => boolean
}

export function useTrackParametricEQ(callbacks: ParametricEQCallbacks = {}) {
  // State
  const eqFiltersData = ref<any[]>([])
  
  // Internal filter chain reference
  let parametricEQFilters: any = null

  /**
   * Handle parametric EQ filter updates from modal
   */
  function handleParametricEQUpdate(filters: any) {
    if (!filters) return

    // Store the latest filter chain
    const previousFilters = parametricEQFilters
    parametricEQFilters = filters

    // Store filter data for thumbnail
    if (filters.filtersData) {
      // Convert Vue reactive proxy to raw array
      const rawFiltersData = toRaw(filters.filtersData)

      // Map to plain objects for storage
      eqFiltersData.value = rawFiltersData.map((f: any) => ({
        id: f.id,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      }))
    }

    // Only reconnect the audio chain if the filter chain structure changed
    // (e.g., filters added/removed, or first time initialization)
    // If it's just parameter updates (dragging), the nodes are already connected
    const shouldReconnect = !previousFilters ||
      !previousFilters.input ||
      !previousFilters.output ||
      previousFilters.input !== filters.input ||
      previousFilters.output !== filters.output

    if (shouldReconnect) {
      applyParametricEQ()
    }
  }

  /**
   * Insert or remove parametric EQ from the chain with minimal disconnections
   */
  function applyParametricEQ() {
    const nodes = callbacks.getAudioNodes?.()
    if (!nodes || !nodes.eq3) return

    const { eq3, channelSplit, waveform, compressor, balanceSplit } = nodes

    // Disconnect only eq3 (meters and waveform stay connected)
    try {
      eq3.disconnect()
    } catch (e) {
      // Ignore disconnection errors
    }

    // Reconnect meters and waveform to eq3
    if (channelSplit) eq3.connect(channelSplit)
    if (waveform) eq3.connect(waveform)

    // Determine next node in chain (compressor if enabled, balanceSplit if not)
    const compressorEnabled = callbacks.getCompressorEnabled?.() || false
    const nextNode = (compressorEnabled && compressor) ? compressor : balanceSplit

    // Insert parametric EQ between eq3 and next node if present
    if (parametricEQFilters && parametricEQFilters.input && parametricEQFilters.output) {
      eq3.connect(parametricEQFilters.input)

      // Disconnect old parametric output if needed
      try {
        parametricEQFilters.output.disconnect()
      } catch (e) { }

      parametricEQFilters.output.connect(nextNode)
    } else {
      // No parametric EQ: connect eq3 directly to next node
      eq3.connect(nextNode)
    }
  }

  /**
   * Get current parametric EQ filters
   */
  function getParametricEQFilters() {
    return parametricEQFilters
  }

  /**
   * Set parametric EQ filters (for scene restoration)
   */
  function setParametricEQFilters(filters: any) {
    parametricEQFilters = filters
  }

  /**
   * Cleanup
   */
  function cleanup() {
    if (parametricEQFilters) {
      try {
        if (parametricEQFilters.dispose) {
          parametricEQFilters.dispose()
        }
      } catch (e) { }
      parametricEQFilters = null
    }
  }

  return {
    // State
    eqFiltersData,

    // Functions
    handleParametricEQUpdate,
    applyParametricEQ,
    getParametricEQFilters,
    setParametricEQFilters,
    cleanup
  }
}
