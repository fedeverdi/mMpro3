import { nextTick } from 'vue'

/**
 * Composable for track state snapshot and restoration (for scenes)
 * 
 * Handles:
 * - Creating snapshots of all track state
 * - Restoring track state from snapshots
 * - Managing audio file loading during restoration
 * 
 * @param callbacks - Functions to access parent component state and methods
 * @returns Object with getSnapshot and restoreFromSnapshot methods
 */
export function useTrackSnapshot(callbacks: {
  // Props
  getTrackNumber: () => number
  getOrder: () => number
  getSubgroups: () => any[]
  
  // State getters
  getVolume: () => number
  getGain: () => number
  getPadEnabled: () => boolean
  getHpfEnabled: () => boolean
  getPhaseInverted: () => boolean
  getPan: () => number
  getIsMuted: () => boolean
  getIsSolo: () => boolean
  getRouteToMaster: () => boolean
  getRoutedSubgroups: () => Set<number>
  getAudioSourceType: () => 'file' | 'input'
  getSelectedAudioInput: () => string
  getFileName: () => string
  getFileId: () => string
  getIsPlaying: () => boolean
  getCurrentPlaybackTime: () => number
  getCurrentAudioBuffer: () => AudioBuffer | null
  getEqFiltersData: () => any[]
  getCompressorEnabled: () => boolean
  getGateEnabled: () => boolean
  getAuxSendsData: () => any
  
  // State setters
  setVolume: (value: number) => void
  setGain: (value: number) => void
  setPadEnabled: (value: boolean) => void
  setHpfEnabled: (value: boolean) => void
  setPhaseInverted: (value: boolean) => void
  setPan: (value: number) => void
  setIsMuted: (value: boolean) => void
  setIsSolo: (value: boolean) => void
  setRouteToMaster: (value: boolean) => void
  setRoutedSubgroups: (value: Set<number>) => void
  setAudioSourceType: (value: 'file' | 'input') => void
  setSelectedAudioInput: (value: string) => void
  setFileName: (value: string) => void
  setFileId: (value: string) => void
  setEqFiltersData: (value: any[]) => void
  setCompressorEnabled: (value: boolean) => void
  setGateEnabled: (value: boolean) => void
  setAuxSendsData: (value: any) => void
  setAudioLoaded: (value: boolean) => void
  
  // Component refs
  getTrackEQRef: () => any
  getTrackCompressorRef: () => any
  getTrackGateRef: () => any
  
  // Audio nodes
  getPhaseInvertNode: () => any
  getPlayer: () => any
  
  // Functions
  initAudioNodes: () => void
  updatePad: () => void
  updateHPF: () => void
  handleAudioInputChange: () => void
  loadFileFromIndexedDB: (fileId: string, silent: boolean) => Promise<void>
  toggleCompressor: () => void
  toggleGate: () => void
  
  // Composables
  routingConnectToOutput: () => void
  auxSendsHandleUpdate: (auxSendsData: any) => void
  parametricEQHandleUpdate: (data: { filtersData: any[] }) => void
  parametricEQGetFilters: () => any
  parametricEQSetFilters: (filters: any) => void
}) {
  /**
   * Create a snapshot of current track state
   */
  function getSnapshot() {
    const trackEQRef = callbacks.getTrackEQRef()
    const trackCompressorRef = callbacks.getTrackCompressorRef()
    const trackGateRef = callbacks.getTrackGateRef()
    const currentAudioBuffer = callbacks.getCurrentAudioBuffer()
    const audioSourceType = callbacks.getAudioSourceType()
    
    return {
      trackNumber: callbacks.getTrackNumber(),
      order: callbacks.getOrder(),
      volume: callbacks.getVolume(),
      gain: callbacks.getGain(),
      padEnabled: callbacks.getPadEnabled(),
      hpfEnabled: callbacks.getHpfEnabled(),
      phaseInverted: callbacks.getPhaseInverted(),
      pan: callbacks.getPan(),
      muted: callbacks.getIsMuted(),
      soloed: callbacks.getIsSolo(),
      routeToMaster: callbacks.getRouteToMaster(),
      routedSubgroups: Array.from(callbacks.getRoutedSubgroups()), // Convert Set to Array for serialization
      sourceType: audioSourceType,
      selectedInputDevice: audioSourceType === 'input' ? callbacks.getSelectedAudioInput() : undefined,
      fileName: audioSourceType === 'file' ? callbacks.getFileName() : undefined,
      fileId: audioSourceType === 'file' ? callbacks.getFileId() : undefined,
      isPlaying: callbacks.getIsPlaying(),
      currentTime: callbacks.getCurrentPlaybackTime(),
      duration: currentAudioBuffer?.duration || 0,
      eq3: trackEQRef?.getParams(),
      parametricEQFilters: callbacks.getEqFiltersData().map(f => ({
        id: f.id,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      })),
      compressorEnabled: callbacks.getCompressorEnabled(),
      compressor: trackCompressorRef?.getParams(),
      gateEnabled: callbacks.getGateEnabled(),
      gate: trackGateRef?.getParams(),
      auxSends: { ...callbacks.getAuxSendsData() }
    }
  }

  /**
   * Restore track state from a snapshot
   */
  async function restoreFromSnapshot(snapshot: any) {
    // Initialize audio nodes if not already done
    callbacks.initAudioNodes()

    // Restore volume, gain, pad, hpf, and pan
    callbacks.setVolume(snapshot.volume)
    if (snapshot.gain !== undefined) {
      callbacks.setGain(snapshot.gain)
    }
    if (snapshot.padEnabled !== undefined) {
      callbacks.setPadEnabled(snapshot.padEnabled)
    }
    if (snapshot.hpfEnabled !== undefined) {
      callbacks.setHpfEnabled(snapshot.hpfEnabled)
    }
    if (snapshot.phaseInverted !== undefined) {
      callbacks.setPhaseInverted(snapshot.phaseInverted)
      // Apply phase inversion with ramp to avoid clicks
      const phaseInvertNode = callbacks.getPhaseInvertNode()
      if (phaseInvertNode) {
        phaseInvertNode.gain.rampTo(snapshot.phaseInverted ? -1 : 1, 0.01)
      }
    }
    callbacks.setPan(snapshot.pan)
    callbacks.setIsMuted(snapshot.muted)
    callbacks.setIsSolo(snapshot.soloed)

    // Ensure PAD and HPF audio nodes are properly connected after restore
    nextTick(() => {
      callbacks.updatePad()
      callbacks.updateHPF()
    })

    // Restore output routing
    if (snapshot.routeToMaster !== undefined) {
      callbacks.setRouteToMaster(snapshot.routeToMaster)
    }

    // Restore routed subgroups (support both old and new format)
    if (snapshot.routedSubgroups && Array.isArray(snapshot.routedSubgroups)) {
      callbacks.setRoutedSubgroups(new Set(snapshot.routedSubgroups))
    } else if (snapshot.routeToSubgroup) {
      // Legacy format - assume routing to first subgroup if exists
      const subgroups = callbacks.getSubgroups()
      if (subgroups && subgroups.length > 0) {
        callbacks.setRoutedSubgroups(new Set([subgroups[0].id]))
      }
    }

    // Reconnect to correct destination(s)
    nextTick(() => {
      callbacks.routingConnectToOutput()
    })

    // Restore aux sends
    if (snapshot.auxSends) {
      callbacks.setAuxSendsData({ ...snapshot.auxSends })
      // Trigger aux sends update to reconnect nodes
      nextTick(() => {
        callbacks.auxSendsHandleUpdate(snapshot.auxSends)
      })
    }

    // Restore source type and related data
    callbacks.setAudioSourceType(snapshot.sourceType || 'file')
    if (snapshot.selectedInputDevice) {
      callbacks.setSelectedAudioInput(snapshot.selectedInputDevice)
      nextTick(() => {
        callbacks.handleAudioInputChange()
      })
    }
    if (snapshot.fileName && snapshot.fileId) {
      // Restore audio file from IndexedDB
      callbacks.setFileName(snapshot.fileName)
      callbacks.setFileId(snapshot.fileId)
      nextTick(async () => {
        // silent=true to avoid showing spinner during scene animation
        await callbacks.loadFileFromIndexedDB(snapshot.fileId!, true)
      })
    }

    // Restore 3-band EQ
    if (snapshot.eq3) {
      const trackEQRef = callbacks.getTrackEQRef()
      trackEQRef?.setParams(snapshot.eq3)
    }

    // Restore parametric EQ
    if (snapshot.parametricEQFilters && snapshot.parametricEQFilters.length > 0) {
      const filters = snapshot.parametricEQFilters.map((f: any) => ({
        id: f.id,
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.Q,
        color: f.color
      }))
      callbacks.setEqFiltersData(filters)
      // Apply EQ filters via the update handler
      callbacks.parametricEQHandleUpdate({ filtersData: filters })
    }

    // Restore compressor
    const shouldEnableCompressor = snapshot.compressorEnabled || false
    if (snapshot.compressor) {
      const trackCompressorRef = callbacks.getTrackCompressorRef()
      trackCompressorRef?.setParams(snapshot.compressor)
    }
    if (shouldEnableCompressor !== callbacks.getCompressorEnabled()) {
      callbacks.toggleCompressor()
    }

    // Restore gate
    const shouldEnableGate = snapshot.gateEnabled || false
    if (snapshot.gate) {
      const trackGateRef = callbacks.getTrackGateRef()
      trackGateRef?.setParams(snapshot.gate)
    }
    if (shouldEnableGate !== callbacks.getGateEnabled()) {
      callbacks.toggleGate()
    }
  }

  /**
   * Reset track to default values
   */
  function resetToDefaults() {
    // Reset volume and pan
    callbacks.setVolume(0)
    callbacks.setPan(0)
    callbacks.setIsMuted(false)
    callbacks.setIsSolo(false)

    // Reset routing
    callbacks.setRouteToMaster(true)
    callbacks.setRoutedSubgroups(new Set())

    // Reset audio source
    callbacks.setAudioSourceType('file')
    callbacks.setSelectedAudioInput('')
    callbacks.setFileName('')
    callbacks.setFileId('')
    callbacks.setAudioLoaded(false)

    // Stop player if active
    const player = callbacks.getPlayer()
    if (player && typeof player.stop === 'function') {
      try {
        player.stop()
      } catch (e) { }
    }

    // Reset 3-band EQ to defaults
    const trackEQRef = callbacks.getTrackEQRef()
    if (trackEQRef?.setParams) {
      trackEQRef.setParams({
        low: 0,
        mid: 0,
        high: 0
      })
    }

    // Clear parametric EQ
    callbacks.setEqFiltersData([])
    const filters = callbacks.parametricEQGetFilters()
    if (filters) {
      try {
        filters.input?.disconnect()
        filters.output?.disconnect()
      } catch (e) { }
      callbacks.parametricEQSetFilters(null)
    }

    // Disable and reset compressor
    if (callbacks.getCompressorEnabled()) {
      callbacks.toggleCompressor()
    }
    const trackCompressorRef = callbacks.getTrackCompressorRef()
    if (trackCompressorRef?.setParams) {
      trackCompressorRef.setParams({
        threshold: -24,
        ratio: 4,
        attack: 0.003,
        release: 0.25
      })
    }
  }

  return {
    getSnapshot,
    restoreFromSnapshot,
    resetToDefaults
  }
}
