import { nextTick, type Ref } from 'vue'

export interface LibraryLoaderCallbacks {
  getTone?: () => any
  getAudioNodes?: () => {
    player: any
    padNode: any
    currentAudioBuffer: any
    gainNode: any
    eq3: any
    volumeMerge: any
  }
  setAudioNodes?: (updates: { player?: any, currentAudioBuffer?: any }) => void
  getState?: () => {
    fileName: string
    fileId: string
    isLoading: boolean
    audioLoaded: boolean
    isStereo: boolean
    isPlaying: boolean
  }
  setState?: (updates: {
    fileName?: string
    fileId?: string
    isLoading?: boolean
    audioLoaded?: boolean
    isStereo?: boolean
    isPlaying?: boolean
  }) => void
  getPlaylistState?: () => {
    currentPlaylist: any
    playlistFiles: any[]
    currentPlaylistIndex: number
    nextTrack: any
    manualStop: boolean
  }
  setPlaylistState?: (updates: {
    currentPlaylist?: any | null
    playlistFiles?: any[]
    currentPlaylistIndex?: number
    nextTrack?: any | null
    manualStop?: boolean
  }) => void
  getAudioFile?: (fileId: string) => Promise<any>
  initAudioNodes?: () => void
  getWaveformDisplayRef?: () => any
  getPlaybackTime?: () => {
    startPlaybackTimeTracking: () => void
  }
}

export function useTrackLibraryLoader(callbacks: LibraryLoaderCallbacks = {}) {
  /**
   * Load audio file from library (IndexedDB stored file)
   */
  async function loadFileFromLibrary(storedFile: any, preservePlaylist = false) {
    const Tone = callbacks.getTone?.()
    if (!Tone || !storedFile) {
      console.error('Cannot load file from library')
      return
    }

    const state = callbacks.getState?.()
    const playlistState = callbacks.getPlaylistState?.()
    
    if (!state) return

    // Reset playlist state when loading a single file (unless called from playlist)
    if (!preservePlaylist && playlistState) {
      callbacks.setPlaylistState?.({
        currentPlaylist: null,
        playlistFiles: [],
        currentPlaylistIndex: 0
      })
    }

    // The file is already in IndexedDB, just use its ID
    callbacks.setState?.({ fileId: storedFile.id })
    const displayName = storedFile.title || storedFile.fileName
    callbacks.setState?.({ fileName: storedFile.artist ? `${storedFile.artist} - ${displayName}` : displayName })
    callbacks.setState?.({ isLoading: true })

    try {
      // Decode the audio buffer
      const audioBuffer = await Tone.context.decodeAudioData(storedFile.arrayBuffer.slice(0))
      
      // Initialize audio nodes on first use
      callbacks.initAudioNodes?.()

      const nodes = callbacks.getAudioNodes?.()
      if (!nodes) return

      let { player } = nodes
      const { padNode, currentAudioBuffer } = nodes

      // Check if we need to create a new player or just swap buffer
      if (player && typeof player.stop === 'function' && 'buffer' in player) {
        // It's already a Tone.Player - just swap the buffer
        try {
          player.stop()
        } catch (e) { }

        if (player.buffer && typeof player.buffer.dispose === 'function') {
          try {
            player.buffer.dispose()
          } catch (e) { }
        }

        if (currentAudioBuffer && currentAudioBuffer !== player.buffer) {
          try {
            if (typeof (currentAudioBuffer as any).dispose === 'function') {
              (currentAudioBuffer as any).dispose()
            }
          } catch (e) { }
        }

        player.buffer = audioBuffer
      } else {
        // First time or was audio input - create new Tone.Player
        if (player) {
          try {
            player.disconnect()
            player.dispose()
          } catch (e) { }
        }

        if (currentAudioBuffer) {
          try {
            if (typeof (currentAudioBuffer as any).dispose === 'function') {
              (currentAudioBuffer as any).dispose()
            }
          } catch (e) { }
        }

        const playlistStateForLoop = callbacks.getPlaylistState?.()
        player = new Tone.Player({
          url: audioBuffer,
          loop: playlistStateForLoop?.nextTrack ? false : true, // Don't loop if there's a next track
          playbackRate: 1.0,
          onstop: () => {
            const currentPlaylistState = callbacks.getPlaylistState?.()
            if (!currentPlaylistState?.manualStop && currentPlaylistState?.nextTrack) {
              loadFileFromLibrary(currentPlaylistState.nextTrack, true).then(() => {
                const stateForStart = callbacks.getState?.()
                const nodesForStart = callbacks.getAudioNodes?.()
                if (nodesForStart?.player && stateForStart) {
                  callbacks.setPlaylistState?.({ manualStop: false })
                  nodesForStart.player.start()
                  callbacks.setState?.({ isPlaying: true })
                  const waveformRef = callbacks.getWaveformDisplayRef?.()
                  waveformRef?.start()
                  callbacks.getPlaybackTime?.()?.startPlaybackTimeTracking()
                }
              })
            } else {
              callbacks.setPlaylistState?.({ manualStop: false })
            }
          },
        })
        player.connect(padNode)
        callbacks.setAudioNodes?.({ player })
      }

      callbacks.setAudioNodes?.({ currentAudioBuffer: audioBuffer })
      callbacks.setState?.({ isStereo: audioBuffer.numberOfChannels === 2 })

      const nodesForValidation = callbacks.getAudioNodes?.()
      if (!nodesForValidation?.gainNode || !nodesForValidation?.eq3 || !nodesForValidation?.volumeMerge) {
        alert('Audio system not ready. Please refresh the page.')
        callbacks.setState?.({ isLoading: false })
        return
      }

      callbacks.setState?.({ audioLoaded: true, isLoading: false })
      
      // Set next track if in playlist mode
      const currentPlaylistState = callbacks.getPlaylistState?.()
      if (currentPlaylistState?.currentPlaylist && currentPlaylistState.playlistFiles.length > 0) {
        const nextIndex = (currentPlaylistState.currentPlaylistIndex + 1) % currentPlaylistState.playlistFiles.length
        callbacks.setPlaylistState?.({ nextTrack: currentPlaylistState.playlistFiles[nextIndex] })
      } else {
        callbacks.setPlaylistState?.({ nextTrack: null })
      }
      
      await nextTick()
    } catch (error) {
      console.error('❌ Error loading audio from library:', error)
      alert('Error loading audio from library: ' + error)
      callbacks.setState?.({ isLoading: false })
    }
  }

  /**
   * Load playlist from library
   */
  async function loadPlaylistFromLibrary(playlist: any) {
    if (!playlist || !playlist.fileIds || playlist.fileIds.length === 0) {
      alert('Playlist is empty')
      return
    }

    const state = callbacks.getState?.()
    if (!state) return

    try {
      // Reset state to avoid BPM detection on old data
      callbacks.setState?.({ audioLoaded: false })
      callbacks.setAudioNodes?.({ currentAudioBuffer: null })
      
      // Import usePlaylist to get files
      const { usePlaylist } = await import('~/composables/usePlaylist')
      const { getPlaylistFiles } = usePlaylist()
      const files = await getPlaylistFiles(playlist.id)
      
      if (files.length === 0) {
        alert('No files found in playlist')
        return
      }

      // Store playlist state
      callbacks.setPlaylistState?.({
        currentPlaylist: playlist,
        playlistFiles: files,
        currentPlaylistIndex: 0
      })
      
      // Load first file (preserve playlist state)
      await loadFileFromLibrary(files[0], true)
      
      const trackName = files[0].title || files[0].fileName
      const trackDisplay = files[0].artist ? `${files[0].artist} - ${trackName}` : trackName
      callbacks.setState?.({ fileName: `${playlist.name} (1/${files.length}) - ${trackDisplay}` })
      
    } catch (error) {
      console.error('❌ Error loading playlist:', error)
      alert('Error loading playlist: ' + error)
    }
  }

  /**
   * Load audio file from IndexedDB (for restoring from scene)
   */
  async function loadFileFromIndexedDB(savedFileId: string, silent: boolean = false) {
    const Tone = callbacks.getTone?.()
    if (!Tone) {
      console.error('Tone.js not loaded')
      return
    }

    const state = callbacks.getState?.()
    if (!state) return

    // Initialize audio nodes on first use
    callbacks.initAudioNodes?.()

    // Don't show spinner when restoring from scene (silent mode)
    if (!silent) {
      callbacks.setState?.({ isLoading: true })
    }

    try {
      // Retrieve file from IndexedDB
      const storedFile = await callbacks.getAudioFile?.(savedFileId)

      if (!storedFile) {
        console.error('File not found in IndexedDB')
        alert('Could not restore audio file from scene. File may have been deleted.')
        if (!silent) {
          callbacks.setState?.({ isLoading: false })
        }
        return
      }

      // Decode audio buffer
      const audioBuffer = await Tone.context.decodeAudioData(storedFile.arrayBuffer)

      const nodes = callbacks.getAudioNodes?.()
      if (!nodes) return

      let { player } = nodes
      const { padNode, currentAudioBuffer } = nodes

      // Check if we need to create a new player or just swap buffer
      if (player && typeof player.stop === 'function' && 'buffer' in player) {
        // It's already a Tone.Player - just swap the buffer

        // CRITICAL: Stop player first to reset internal state
        try {
          player.stop()
        } catch (e) { }

        if (player.buffer && typeof player.buffer.dispose === 'function') {
          try {
            player.buffer.dispose()
          } catch (e) { }
        }

        if (currentAudioBuffer && currentAudioBuffer !== player.buffer) {
          try {
            if (typeof (currentAudioBuffer as any).dispose === 'function') {
              (currentAudioBuffer as any).dispose()
            }
          } catch (e) { }
        }

        // Assign new buffer
        player.buffer = audioBuffer

        // CRITICAL: Reset playback rate to 1.0
        player.playbackRate = 1.0

      } else {
        // First time or was audio input - create new Tone.Player

        if (player) {
          try {
            player.disconnect()
            player.dispose()
          } catch (e) { }
        }

        if (currentAudioBuffer) {
          try {
            if (typeof (currentAudioBuffer as any).dispose === 'function') {
              (currentAudioBuffer as any).dispose()
            }
          } catch (e) { }
        }

        const playlistState = callbacks.getPlaylistState?.()
        // Create new Tone.Player
        player = new Tone.Player({
          url: audioBuffer,
          loop: playlistState?.nextTrack ? false : true,
          playbackRate: 1.0,
          fadeIn: 0.01,   // Prevent resampling artifacts
          fadeOut: 0.01,   // Prevent clicks on stop
          onstop: () => {
            const currentPlaylistState = callbacks.getPlaylistState?.()
            if (!currentPlaylistState?.manualStop && currentPlaylistState?.nextTrack) {
              loadFileFromLibrary(currentPlaylistState.nextTrack, true).then(() => {
                const stateForStart = callbacks.getState?.()
                const nodesForStart = callbacks.getAudioNodes?.()
                if (nodesForStart?.player && stateForStart) {
                  callbacks.setPlaylistState?.({ manualStop: false })
                  nodesForStart.player.start()
                  callbacks.setState?.({ isPlaying: true })
                  const waveformRef = callbacks.getWaveformDisplayRef?.()
                  waveformRef?.start()
                  callbacks.getPlaybackTime?.()?.startPlaybackTimeTracking()
                }
              })
            } else {
              callbacks.setPlaylistState?.({ manualStop: false })
            }
          },
        })
        player.connect(padNode)
        callbacks.setAudioNodes?.({ player })
      }

      // Update current buffer reference for waveform
      callbacks.setAudioNodes?.({ currentAudioBuffer: audioBuffer })

      // Detect if stereo or mono
      callbacks.setState?.({ isStereo: audioBuffer.numberOfChannels === 2 })

      // Verify audio chain is connected
      const nodesForValidation = callbacks.getAudioNodes?.()
      if (!nodesForValidation?.gainNode || !nodesForValidation?.eq3 || !nodesForValidation?.volumeMerge) {
        alert('Audio system not ready. Please refresh the page.')
        if (!silent) {
          callbacks.setState?.({ isLoading: false })
        }
        return
      }

      callbacks.setState?.({ audioLoaded: true })
      if (!silent) {
        callbacks.setState?.({ isLoading: false })
      }

      // Force DOM update
      await nextTick()
    } catch (error) {
      console.error('❌ Error loading audio file from IndexedDB:', error)
      alert('Error loading audio file from scene: ' + error)
      if (!silent) {
        callbacks.setState?.({ isLoading: false })
      }
    }
  }

  return {
    loadFileFromLibrary,
    loadPlaylistFromLibrary,
    loadFileFromIndexedDB
  }
}
