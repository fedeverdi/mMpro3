import { ref, type Ref } from 'vue'

export interface PlaybackTimeState {
  currentPlaybackTime: Ref<number>
}

export interface PlaybackTimeCallbacks {
  getTone?: () => any
  getPlayer?: () => any
  getIsPlaying?: () => boolean
  getCurrentAudioBuffer?: () => any
}

export function useTrackPlaybackTime(callbacks: PlaybackTimeCallbacks = {}) {
  // State - exposed to parent
  const currentPlaybackTime = ref(0)

  // Internal state
  let playbackTimeInterval: number | null = null
  let playbackStartTime: number = 0

  /**
   * Start tracking playback time for waveform animation
   */
  function startPlaybackTimeTracking() {
    if (playbackTimeInterval !== null) return // Already running

    const Tone = callbacks.getTone?.()
    if (!Tone) return

    // Record start time
    playbackStartTime = Tone.now()

    playbackTimeInterval = window.setInterval(() => {
      const player = callbacks.getPlayer?.()
      const isPlaying = callbacks.getIsPlaying?.()
      const currentAudioBuffer = callbacks.getCurrentAudioBuffer?.()

      if (player && isPlaying && Tone) {
        // Calculate elapsed time since start
        const elapsed = Tone.now() - playbackStartTime

        // For looping player, use modulo of duration
        if (currentAudioBuffer && currentAudioBuffer.duration) {
          currentPlaybackTime.value = elapsed % currentAudioBuffer.duration
        }
      }
    }, 50) // Update every 50ms for smooth animation
  }

  /**
   * Stop tracking playback time
   */
  function stopPlaybackTimeTracking() {
    if (playbackTimeInterval !== null) {
      clearInterval(playbackTimeInterval)
      playbackTimeInterval = null
    }
    currentPlaybackTime.value = 0
    playbackStartTime = 0
  }

  /**
   * Cleanup function - stops tracking and resets
   */
  function cleanup() {
    stopPlaybackTimeTracking()
  }

  return {
    // State
    currentPlaybackTime,
    
    // Functions
    startPlaybackTimeTracking,
    stopPlaybackTimeTracking,
    cleanup
  }
}
