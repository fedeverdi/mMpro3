/**
 * Composable for track resource cleanup on unmount
 * 
 * Handles:
 * - Audio nodes disposal
 * - Player cleanup
 * - Buffer memory release
 * - Event listener removal
 * - Interval clearing
 * 
 * @param callbacks - Functions to access parent component resources
 * @returns Object with cleanup method
 */
export function useTrackCleanup(callbacks: {
  // Automation
  getAutomationRecordingInterval: () => any
  setAutomationRecordingInterval: (value: any) => void
  
  // Audio input
  getAudioInputSource: () => any
  setAudioInputSource: (value: any) => void
  getChannelSplitter: () => any
  setChannelSplitter: (value: any) => void
  getAudioInputStream: () => MediaStream | null
  setAudioInputStream: (value: MediaStream | null) => void
  
  // Players and buffers
  getPlayer: () => any
  getNextPlayer: () => any
  getCurrentAudioBuffer: () => AudioBuffer | null
  setCurrentAudioBuffer: (value: AudioBuffer | null) => void
  getNextBuffer: () => AudioBuffer | null
  setNextBuffer: (value: AudioBuffer | null) => void
  
  // Audio nodes
  getCrossFade: () => any
  getGainNode: () => any
  getEq3: () => any
  getBalanceSplit: () => any
  getBalanceLeft: () => any
  getBalanceRight: () => any
  getBalanceMerge: () => any
  getVolumeSplit: () => any
  getVolumeNodeL: () => any
  getVolumeNodeR: () => any
  getVolumeMerge: () => any
  getMeterL: () => any
  getMeterR: () => any
  getGateMeter: () => any
  getChannelSplit: () => any
  getWaveform: () => any
  getCompressor: () => any
  
  // Composables
  stopGateMonitoring: () => void
  auxSendsCleanup: () => void
  parametricEQCleanup: () => void
  levelMonitoringCleanup: () => void
  playbackTimeCleanup: () => void
  
  // UI refs
  getResizeObserver: () => ResizeObserver | null
  getWaveformDisplayRef: () => any
  
  // Audio devices
  refreshAudioInputs: () => void
}) {
  /**
   * Cleanup all track resources
   */
  function cleanup() {
    // Clear automation recording interval
    const automationInterval = callbacks.getAutomationRecordingInterval()
    if (automationInterval) {
      clearInterval(automationInterval)
      callbacks.setAutomationRecordingInterval(null)
    }

    // Disconnect audio input source if active
    const audioInputSource = callbacks.getAudioInputSource()
    if (audioInputSource) {
      try {
        audioInputSource.disconnect()
      } catch (e) { }
      callbacks.setAudioInputSource(null)
    }
    
    const channelSplitter = callbacks.getChannelSplitter()
    if (channelSplitter) {
      try {
        channelSplitter.disconnect()
      } catch (e) { }
      callbacks.setChannelSplitter(null)
    }

    // Stop audio input stream if active
    const audioInputStream = callbacks.getAudioInputStream()
    if (audioInputStream) {
      audioInputStream.getTracks().forEach(track => track.stop())
      callbacks.setAudioInputStream(null)
    }

    // Cleanup player and free buffer memory
    const player = callbacks.getPlayer()
    if (player) {
      // Dispose buffer first to free memory
      if (player.buffer && typeof player.buffer.dispose === 'function') {
        try {
          player.buffer.dispose()
        } catch (e) { }
      }
      player.dispose()
    }

    // Cleanup next player if pre-loaded
    const nextPlayer = callbacks.getNextPlayer()
    if (nextPlayer) {
      if (nextPlayer.buffer && typeof nextPlayer.buffer.dispose === 'function') {
        try {
          nextPlayer.buffer.dispose()
        } catch (e) { }
      }
      nextPlayer.dispose()
    }

    // Dispose and clear audio buffer reference
    const currentAudioBuffer = callbacks.getCurrentAudioBuffer()
    if (currentAudioBuffer) {
      try {
        if (typeof (currentAudioBuffer as any).dispose === 'function') {
          (currentAudioBuffer as any).dispose()
        }
      } catch (e) { }
    }
    callbacks.setCurrentAudioBuffer(null)

    // Dispose next buffer if pre-loaded
    const nextBuffer = callbacks.getNextBuffer()
    if (nextBuffer) {
      try {
        if (typeof (nextBuffer as any).dispose === 'function') {
          (nextBuffer as any).dispose()
        }
      } catch (e) { }
    }
    callbacks.setNextBuffer(null)

    // Dispose crossfade node
    const crossFade = callbacks.getCrossFade()
    if (crossFade) crossFade.dispose()

    // Dispose audio chain nodes
    const gainNode = callbacks.getGainNode()
    if (gainNode) gainNode.dispose()
    
    const eq3 = callbacks.getEq3()
    if (eq3) eq3.dispose()
    
    const balanceSplit = callbacks.getBalanceSplit()
    if (balanceSplit) balanceSplit.dispose()
    
    const balanceLeft = callbacks.getBalanceLeft()
    if (balanceLeft) balanceLeft.dispose()
    
    const balanceRight = callbacks.getBalanceRight()
    if (balanceRight) balanceRight.dispose()
    
    const balanceMerge = callbacks.getBalanceMerge()
    if (balanceMerge) balanceMerge.dispose()
    
    const volumeSplit = callbacks.getVolumeSplit()
    if (volumeSplit) volumeSplit.dispose()
    
    const volumeNodeL = callbacks.getVolumeNodeL()
    if (volumeNodeL) volumeNodeL.dispose()
    
    const volumeNodeR = callbacks.getVolumeNodeR()
    if (volumeNodeR) volumeNodeR.dispose()
    
    const volumeMerge = callbacks.getVolumeMerge()
    if (volumeMerge) volumeMerge.dispose()
    
    const meterL = callbacks.getMeterL()
    if (meterL) meterL.dispose()
    
    const meterR = callbacks.getMeterR()
    if (meterR) meterR.dispose()
    
    const gateMeter = callbacks.getGateMeter()
    if (gateMeter) gateMeter.dispose()
    
    const channelSplit = callbacks.getChannelSplit()
    if (channelSplit) channelSplit.dispose()

    // Stop gate monitoring
    callbacks.stopGateMonitoring()

    // Cleanup aux send nodes
    callbacks.auxSendsCleanup()
    
    const waveform = callbacks.getWaveform()
    if (waveform) waveform.dispose()
    
    const compressor = callbacks.getCompressor()
    if (compressor) compressor.dispose()

    // Cleanup parametric EQ
    callbacks.parametricEQCleanup()

    const resizeObserver = callbacks.getResizeObserver()
    if (resizeObserver) {
      resizeObserver.disconnect()
    }

    // Cleanup level monitoring
    callbacks.levelMonitoringCleanup()

    // Cleanup playback time tracking
    callbacks.playbackTimeCleanup()

    // Stop waveform drawing
    const waveformDisplayRef = callbacks.getWaveformDisplayRef()
    if (waveformDisplayRef) {
      waveformDisplayRef.stop()
    }
  }

  return {
    cleanup
  }
}
