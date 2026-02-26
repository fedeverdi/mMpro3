export interface AudioInputCallbacks {
  getTone?: () => any
  getAudioNodes?: () => {
    player: any
    padNode: any
    currentAudioBuffer: any
  }
  setAudioNodes?: (updates: { 
    player?: any
    currentAudioBuffer?: any
    audioInputSource?: MediaStreamAudioSourceNode | null
    channelSplitter?: ChannelSplitterNode | null
    audioInputStream?: MediaStream | null
  }) => void
  getAudioInputNodes?: () => {
    audioInputSource: MediaStreamAudioSourceNode | null
    channelSplitter: ChannelSplitterNode | null
    audioInputStream: MediaStream | null
  }
  getState?: () => {
    selectedAudioInput: string
    audioLoaded: boolean
    isPlaying: boolean
    fileName: string
    fileId: string
    isStereo: boolean
    audioContextStarted: boolean
  }
  setState?: (updates: {
    selectedAudioInput?: string
    audioLoaded?: boolean
    isPlaying?: boolean
    fileName?: string
    fileId?: string
    isStereo?: boolean
    audioContextStarted?: boolean
  }) => void
  getPlaylistState?: () => {
    currentPlaylist: any
    playlistFiles: any[]
    currentPlaylistIndex: number
  }
  setPlaylistState?: (updates: {
    currentPlaylist?: any | null
    playlistFiles?: any[]
    currentPlaylistIndex?: number
  }) => void
  getAudioInputs?: () => any[]
  initAudioNodes?: () => void
  stopAudio?: () => void
  getWaveformDisplayRef?: () => any
  getRouting?: () => { connectToOutput: () => void }
  getMasterChannel?: () => any
  getTrackNumber?: () => number
}

export function useTrackAudioInput(callbacks: AudioInputCallbacks = {}) {
  /**
   * Handle source type change (file <-> input)
   */
  function handleSourceTypeChange() {
    const playlistState = callbacks.getPlaylistState?.()
    
    // Reset playlist state when changing source type
    if (playlistState) {
      callbacks.setPlaylistState?.({
        currentPlaylist: null,
        playlistFiles: [],
        currentPlaylistIndex: 0
      })
    }

    // Stop any current playback
    callbacks.stopAudio?.()

    // Stop waveform visualization
    const waveformRef = callbacks.getWaveformDisplayRef?.()
    waveformRef?.stop()

    // Clean up current source before switching
    const nodes = callbacks.getAudioNodes?.()
    if (nodes?.player) {
      let player = nodes.player
      try {
        if (typeof player.stop === 'function') {
          player.stop()
        }

        // Dispose old buffer to free memory
        if (player.buffer && typeof player.buffer.dispose === 'function') {
          try {
            player.buffer.dispose()
          } catch (e) { }
        }

        player.disconnect()
        player.dispose()
      } catch (e) { }
      
      callbacks.setAudioNodes?.({ player: null })
    }

    // Dispose and clear audio buffer reference
    if (nodes?.currentAudioBuffer) {
      try {
        if (typeof (nodes.currentAudioBuffer as any).dispose === 'function') {
          (nodes.currentAudioBuffer as any).dispose()
        }
      } catch (e) { }
      callbacks.setAudioNodes?.({ currentAudioBuffer: null })
    }

    // Disconnect and clean up audio input source
    const inputNodes = callbacks.getAudioInputNodes?.()
    if (inputNodes?.audioInputSource) {
      try {
        inputNodes.audioInputSource.disconnect()
      } catch (e) { }
      callbacks.setAudioNodes?.({ audioInputSource: null })
    }
    
    if (inputNodes?.channelSplitter) {
      try {
        inputNodes.channelSplitter.disconnect()
      } catch (e) { }
      callbacks.setAudioNodes?.({ channelSplitter: null })
    }

    if (inputNodes?.audioInputStream) {
      inputNodes.audioInputStream.getTracks().forEach(track => track.stop())
      callbacks.setAudioNodes?.({ audioInputStream: null })
    }

    // DON'T destroy audio nodes - they're reusable!
    // Just reset state
    callbacks.setState?.({
      audioLoaded: false,
      isPlaying: false,
      fileName: '',
      fileId: '',
      selectedAudioInput: ''
    })
  }

  /**
   * Handle audio input device change
   */
  async function handleAudioInputChange() {
    const state = callbacks.getState?.()
    const Tone = callbacks.getTone?.()
    
    if (!state?.selectedAudioInput || !Tone) return

    // Initialize audio nodes if needed
    callbacks.initAudioNodes?.()

    try {
      // Start audio context once (required by browsers for user interaction)
      if (!state.audioContextStarted) {
        await Tone.start()
        callbacks.setState?.({ audioContextStarted: true })
      }

      const inputNodes = callbacks.getAudioInputNodes?.()
      
      // Stop previous input stream if any
      if (inputNodes?.audioInputStream) {
        inputNodes.audioInputStream.getTracks().forEach(track => track.stop())
      }

      // Disconnect old audio input source
      if (inputNodes?.audioInputSource) {
        try {
          inputNodes.audioInputSource.disconnect()
        } catch (e) { }
        callbacks.setAudioNodes?.({ audioInputSource: null })
      }
      
      if (inputNodes?.channelSplitter) {
        try {
          inputNodes.channelSplitter.disconnect()
        } catch (e) { }
        callbacks.setAudioNodes?.({ channelSplitter: null })
      }

      // Dispose old player if it exists and is not already a Gain (input wrapper)
      const nodes = callbacks.getAudioNodes?.()
      if (nodes?.player && typeof nodes.player.stop === 'function') {
        // It's a Tone.Player, dispose it
        try {
          nodes.player.stop()
          nodes.player.disconnect()
          nodes.player.dispose()
        } catch (e) { }
        callbacks.setAudioNodes?.({ player: null })
      }

      // Parse composite deviceId (format: "realDeviceId:channelIndex")
      let realDeviceId = state.selectedAudioInput
      let targetChannel: number | null = null
      
      const trackNumber = callbacks.getTrackNumber?.() ?? 0
      
      if (state.selectedAudioInput.includes(':')) {
        const parts = state.selectedAudioInput.split(':')
        realDeviceId = parts[0]
        targetChannel = parseInt(parts[1], 10)
        console.log(`[Track ${trackNumber}] Parsed composite deviceId: device="${realDeviceId}", channel=${targetChannel + 1}`)
      }
      
      // Get audio stream from selected device
      const audioInputStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          deviceId: { exact: realDeviceId },
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
          sampleRate: { ideal: 48000 },
          channelCount: { min: 1, ideal: 8, max: 32 } // Request all available channels
        }
      })

      // Create MediaStreamSource from the stream (native Web Audio API node)
      const audioInputSource = Tone.context.createMediaStreamSource(audioInputStream)

      // Get actual channel count from the source
      if (!audioInputSource) {
        throw new Error('Failed to create audio input source')
      }
      
      const actualChannelCount = audioInputSource.channelCount
      console.log(`[Track ${trackNumber}] MediaStreamSource has ${actualChannelCount} channels`)
      
      // For multi-channel devices with specific channel selected, use channel splitter
      let sourceNode: AudioNode | null = audioInputSource
      let channelSplitter: ChannelSplitterNode | null = null
      
      if (targetChannel !== null && actualChannelCount > 1 && audioInputSource) {
        // Create channel splitter to separate individual channels
        channelSplitter = Tone.context.createChannelSplitter(actualChannelCount) as ChannelSplitterNode
        audioInputSource.connect(channelSplitter)
        
        // Create a merger to convert selected channel to mono output
        const channelMerger = Tone.context.createChannelMerger(1)
        
        // Connect selected channel (0-based) to the merger
        channelSplitter.connect(channelMerger, targetChannel, 0)
        
        sourceNode = channelMerger
        callbacks.setState?.({ isStereo: false }) // Each track uses one channel
        
        console.log(`[Track ${trackNumber}] Using input channel ${targetChannel + 1} of ${actualChannelCount}`)
      } else {
        callbacks.setState?.({ isStereo: actualChannelCount === 2 })
      }

      // Update audio input nodes
      callbacks.setAudioNodes?.({
        audioInputSource,
        channelSplitter,
        audioInputStream
      })

      // Reuse player if it's already a Gain node, otherwise create new
      const currentNodes = callbacks.getAudioNodes?.()
      let player = currentNodes?.player
      
      if (!player || typeof player.stop === 'function') {
        player = new Tone.Gain(1)
        player.connect(currentNodes?.padNode!)
        callbacks.setAudioNodes?.({ player })
      }

      // Connect the source node (either direct or through channel splitter) to player input
      if (sourceNode) {
        sourceNode.connect(player.input)
      }

      // CRITICAL: Ensure volume node is connected to master output
      const routing = callbacks.getRouting?.()
      routing?.connectToOutput()

      // Find device name for display
      const audioInputs = callbacks.getAudioInputs?.() ?? []
      const device = audioInputs.find((d: any) => d.deviceId === state.selectedAudioInput)
      const deviceName = device?.label || 'Audio Input'
      
      callbacks.setState?.({
        fileName: deviceName,
        audioLoaded: true,
        isPlaying: true // Input is always "playing"
      })

      // Ensure master audio elements are playing (critical for output!)
      const masterChannel = callbacks.getMasterChannel?.()
      if (masterChannel?.ensureAudioPlaying) {
        masterChannel.ensureAudioPlaying()
      }

      // Start waveform visualization for audio input
      const waveformRef = callbacks.getWaveformDisplayRef?.()
      waveformRef?.start()

      // Note: No playback time tracking for audio input (it's live)

    } catch (error) {
      const trackNumber = callbacks.getTrackNumber?.() ?? 0
      console.error(`[Track ${trackNumber}] Error connecting audio input:`, error)
      alert('Error accessing audio input. Please check permissions and try again.')
      callbacks.setState?.({
        audioLoaded: false,
        isPlaying: false
      })
    }
  }

  /**
   * Handle input device selection from InputSelector
   */
  function handleInputSelect(deviceId: string | null) {
    if (deviceId === null || deviceId === '') {
      // Clear input selection
      callbacks.setState?.({ selectedAudioInput: '' })
      callbacks.stopAudio?.()
    } else {
      callbacks.setState?.({ selectedAudioInput: deviceId })
      handleAudioInputChange()
    }
  }

  return {
    handleSourceTypeChange,
    handleAudioInputChange,
    handleInputSelect
  }
}
