import { nextTick, type Ref } from 'vue'

export interface LibraryLoaderCallbacks {
  getTone?: () => any
  getAudioNodes?: () => {
    player: any
    nextPlayer: any
    nextBuffer: any
    crossFade: any
    padNode: any
    currentAudioBuffer: any
    gainNode: any
    eq3: any
    volumeMerge: any
  }
  setAudioNodes?: (updates: { 
    player?: any
    nextPlayer?: any
    nextBuffer?: any
    crossFade?: any
    currentAudioBuffer?: any 
  }) => void
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
    isPreloading: boolean
  }
  setPlaylistState?: (updates: {
    currentPlaylist?: any | null
    playlistFiles?: any[]
    currentPlaylistIndex?: number
    nextTrack?: any | null
    manualStop?: boolean
    isPreloading?: boolean
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
   * Pre-load next track in background for seamless playback
   */
  async function preloadNextTrack() {
    const playlistState = callbacks.getPlaylistState?.()
    if (!playlistState?.currentPlaylist || !playlistState.playlistFiles || playlistState.isPreloading) {
      return // No playlist active or already preloading
    }

    // Don't pre-load if playlist has only 1 track
    if (playlistState.playlistFiles.length === 1) {
      return
    }

    const nextIndex = (playlistState.currentPlaylistIndex + 1) % playlistState.playlistFiles.length
    const nextFile = playlistState.playlistFiles[nextIndex]
    
    if (!nextFile || !nextFile.arrayBuffer) return

    // Safety check: if next track is the same as current, we're at the end and should loop
    // For playlists > 1 track, this shouldn't happen, but we handle it gracefully
    const currentFile = playlistState.playlistFiles[playlistState.currentPlaylistIndex]
    if (nextFile.id === currentFile?.id && playlistState.playlistFiles.length > 1) {
      console.warn('⚠️ Next track is same as current, skipping pre-load')
      return
    }

    try {
      callbacks.setPlaylistState?.({ isPreloading: true })
      
      const Tone = callbacks.getTone?.()
      if (!Tone) return

      // Decode next file's audio buffer (file is already loaded with arrayBuffer from playlist)
      const nextBuffer = await Tone.context.decodeAudioData(nextFile.arrayBuffer.slice(0))
      
      const nodes = callbacks.getAudioNodes?.()
      if (!nodes) {
        callbacks.setPlaylistState?.({ isPreloading: false })
        return
      }

      // Create next player (connected but not started)
      const { nextPlayer: existingNextPlayer, crossFade, padNode } = nodes
      
      // Clean up existing next player if any
      if (existingNextPlayer) {
        try {
          existingNextPlayer.disconnect()
          existingNextPlayer.dispose()
        } catch (e) { }
      }

      // Create new next player
      const nextPlayer = new Tone.Player({
        url: nextBuffer,
        loop: false,
        playbackRate: 1.0
      })

      // Connect to crossfade B input (will fade in when current fades out)
      if (crossFade) {
        nextPlayer.connect(crossFade.b) // Connect to input B of crossfade
      } else {
        nextPlayer.connect(padNode) // Fallback: direct connection
      }

      callbacks.setAudioNodes?.({ nextPlayer, nextBuffer })
      callbacks.setPlaylistState?.({ 
        isPreloading: false,
        nextTrack: nextFile 
      })

      console.log('✅ Pre-loaded next track:', nextFile.title || nextFile.fileName)
    } catch (error) {
      console.error('❌ Error pre-loading next track:', error)
      callbacks.setPlaylistState?.({ isPreloading: false })
    }
  }

  /**
   * Crossfade to next pre-loaded track
   */
  async function crossfadeToNext() {
    const nodes = callbacks.getAudioNodes?.()
    const playlistState = callbacks.getPlaylistState?.()
    
    if (!nodes?.nextPlayer || !nodes?.crossFade || !playlistState?.nextTrack) {
      console.log('⚠️ No next track pre-loaded, falling back to loading')
      return false
    }

    const { player, nextPlayer, crossFade, nextBuffer } = nodes
    const Tone = callbacks.getTone?.()
    if (!Tone) return false

    try {
      // Start next player
      nextPlayer.start()
      
      // Crossfade from A (current) to B (next) over 0.5 seconds
      crossFade.fade.rampTo(1, 0.5) // Fade to input B
      
      // Wait for crossfade to complete
      await new Promise(resolve => setTimeout(resolve, 500))
      
      // Stop and dispose old player
      if (player) {
        try {
          player.stop()
          player.disconnect()
          player.dispose()
        } catch (e) { }
      }

      // Swap: next becomes current
      callbacks.setAudioNodes?.({ 
        player: nextPlayer,
        currentAudioBuffer: nextBuffer,
        nextPlayer: null,
        nextBuffer: null
      })

      // Reset crossfade to input A for next transition
      crossFade.fade.value = 0

      // Update playlist index
      const newIndex = (playlistState.currentPlaylistIndex + 1) % playlistState.playlistFiles.length
      callbacks.setPlaylistState?.({ 
        currentPlaylistIndex: newIndex,
        manualStop: false 
      })

      // Update file name display
      const currentFile = playlistState.playlistFiles[newIndex]
      const trackName = currentFile.title || currentFile.fileName
      const trackDisplay = currentFile.artist ? `${currentFile.artist} - ${trackName}` : trackName
      const playlistName = playlistState.currentPlaylist.name
      callbacks.setState?.({ 
        fileName: `${playlistName} (${newIndex + 1}/${playlistState.playlistFiles.length}) - ${trackDisplay}`,
        fileId: currentFile.id
      })

      // Pre-load the next track in background
      setTimeout(() => preloadNextTrack(), 100)

      return true
    } catch (error) {
      console.error('❌ Crossfade error:', error)
      return false
    }
  }

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
          onstop: async () => {
            const currentPlaylistState = callbacks.getPlaylistState?.()
            if (!currentPlaylistState?.manualStop && currentPlaylistState?.nextTrack) {
              // Try seamless crossfade first
              const crossfaded = await crossfadeToNext()
              
              if (!crossfaded) {
                // Fallback: traditional loading (with gap)
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
                // Crossfade successful - just update UI state
                callbacks.setState?.({ isPlaying: true })
                const waveformRef = callbacks.getWaveformDisplayRef?.()
                waveformRef?.start()
                callbacks.getPlaybackTime?.()?.startPlaybackTimeTracking()
              }
            } else {
              callbacks.setPlaylistState?.({ manualStop: false })
            }
          },
        })
        
        // Connect to crossfade input A (current player)
        const { crossFade } = nodes
        if (crossFade) {
          player.connect(crossFade.a) // Connect to input A of crossfade
        } else {
          player.connect(padNode) // Fallback: direct connection if crossfade not available
        }
        
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
      
      // Set next track if in playlist mode and start pre-loading
      const currentPlaylistState = callbacks.getPlaylistState?.()
      if (currentPlaylistState?.currentPlaylist && currentPlaylistState.playlistFiles.length > 0) {
        const nextIndex = (currentPlaylistState.currentPlaylistIndex + 1) % currentPlaylistState.playlistFiles.length
        callbacks.setPlaylistState?.({ nextTrack: currentPlaylistState.playlistFiles[nextIndex] })
        
        // Pre-load next track in background for seamless playback
        setTimeout(() => preloadNextTrack(), 500)
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
        
        // Connect to crossfade input A (current player)
        const { crossFade } = nodes
        if (crossFade) {
          player.connect(crossFade.a) // Connect to input A of crossfade
        } else {
          player.connect(padNode) // Fallback: direct connection if crossfade not available
        }
        
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
