import { ref, onUnmounted } from 'vue'

export interface AudioDevice {
  id: string
  name: string
  input_channels: number
  output_channels: number
}

export interface AudioEngineState {
  isRunning: boolean
  devices: AudioDevice[]
  selectedInputDevice: string | null
  selectedOutputDevice: string | null
  trackLevels: Map<number, { left: number, right: number }>
  trackWaveforms: Map<number, number[]>
  masterLevels: { left: number, right: number }
}

const state = ref<AudioEngineState>({
  isRunning: false,
  devices: [],
  selectedInputDevice: null,
  selectedOutputDevice: null,
  trackLevels: new Map(),
  trackWaveforms: new Map(),
  masterLevels: { left: 0, right: 0 }
})

let isListening = false

export const useAudioEngine = () => {
  // Start listening immediately
  const startListening = () => {
    if (isListening || !window.audioEngine) return

    window.audioEngine.onResponse((response: any) => {

      switch (response.type) {
        case 'devices':
          state.value.devices = response.devices
          break

        case 'started':
          state.value.isRunning = true
          break

        case 'stopped':
          state.value.isRunning = false
          break

        case 'ok':
          // Command acknowledged successfully (silent)
          break

        case 'error':
          console.error('[useAudioEngine] Engine error:', response.message)
          break

        case 'levels':
          // Update track levels and waveforms
          if (response.tracks) {
            response.tracks.forEach((trackLevel: any) => {
              state.value.trackLevels.set(trackLevel.track, {
                left: trackLevel.level_l,
                right: trackLevel.level_r
              })

              // Update waveform data if present
              if (trackLevel.waveform) {
                state.value.trackWaveforms.set(trackLevel.track, trackLevel.waveform)
              }
            })
          }
          // Update master levels
          if (response.master_l !== undefined && response.master_r !== undefined) {
            state.value.masterLevels = {
              left: response.master_l,
              right: response.master_r
            }
          }
          break

        default:
          console.log('[useAudioEngine] Unhandled response type:', response.type)
      }
    })

    isListening = true
  }

  // Start listening as soon as the composable is created
  if (window.audioEngine && !isListening) {
    startListening()
  }

  const loadDevices = async () => {
    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
      return
    }

    startListening()
    await window.audioEngine.listDevices()
  }

  const start = async (inputDevice?: string, outputDevice?: string) => {
    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
      return
    }

    startListening()
    await window.audioEngine.start(inputDevice, outputDevice)
  }

  const stop = async () => {
    if (!window.audioEngine) return

    await window.audioEngine.stop()
  }

  const restartWithDevices = async (inputDevice?: string, outputDevice?: string) => {
    if (!window.audioEngine) return

    await stop()
    // Small delay to ensure streams are closed
    await new Promise(resolve => setTimeout(resolve, 100))
    await start(inputDevice, outputDevice)
  }

  const setTrackGain = async (track: number, gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setGain(track, gain)
  }

  const setTrackVolume = async (track: number, volume: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setVolume(track, volume)
  }

  const setTrackMute = async (track: number, mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setMute(track, mute)
  }

  const setTrackCompressor = async (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setCompressor(track, enabled, threshold, ratio, attack, release)
  }

  const setTrackGate = async (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setGate(track, enabled, threshold, range, attack, release)
  }

  // Track source selection
  const setTrackSourceInput = async (track: number, leftChannel: number, rightChannel: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setTrackSourceInput(track, leftChannel, rightChannel)
  }

  const setTrackSourceSignal = async (track: number, waveform: string, frequency: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setTrackSourceSignal(track, waveform, frequency)
  }

  const setTrackSourceFile = async (track: number, filePath: string) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setTrackSourceFile(track, filePath)
  }

  // File playback controls
  const playFile = async (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.playFile(track)
  }

  const pauseFile = async (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.pauseFile(track)
  }

  const stopFile = async (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.stopFile(track)
  }

  const setTrackPan = async (track: number, pan: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setPan(track, pan)
  }

  const setTrackPad = async (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setTrackPad(track, enabled)
  }

  const setTrackHPF = async (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setTrackHPF(track, enabled)
  }

  const setTrackEQ = async (track: number, low: number, lowMid: number, highMid: number, high: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setEQ(track, low, lowMid, highMid, high)
  }

  const setTrackEQEnabled = async (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setEQEnabled(track, enabled)
  }

  // Parametric EQ controls
  const setParametricEQFilters = async (track: number, filters: Array<{ type: string, frequency: number, gain: number, q: number }>) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setParametricEQFilters(track, filters)
  }

  const setParametricEQEnabled = async (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setParametricEQEnabled(track, enabled)
  }

  const clearParametricEQ = async (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.clearParametricEQ(track)
  }

  // Master controls
  const setMasterGain = async (gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setMasterGain(gain)
  }

  const setMasterMute = async (mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setMasterMute(mute)
  }

  const setMasterOutputChannels = async (leftChannel: number, rightChannel: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    await window.audioEngine.setMasterOutputChannels(leftChannel, rightChannel)
  }

  const getInputDevices = () => {
    return state.value.devices.filter(d => d.input_channels > 0)
  }

  const getOutputDevices = () => {
    return state.value.devices.filter(d => d.output_channels > 0)
  }

  onUnmounted(() => {
    if (state.value.isRunning) {
      stop()
    }
  })

  return {
    state,
    loadDevices,
    start,
    stop,
    restartWithDevices,
    setTrackGain,
    setTrackVolume,
    setTrackMute,
    setTrackEQ,
    setTrackEQEnabled,
    setParametricEQFilters,
    setParametricEQEnabled,
    clearParametricEQ,
    setTrackCompressor,
    setTrackGate,
    setTrackSourceInput,
    setTrackSourceSignal,
    setTrackSourceFile,
    playFile,
    pauseFile,
    stopFile,
    setTrackPan,
    setTrackPad,
    setTrackHPF,
    setMasterGain,
    setMasterMute,
    setMasterOutputChannels,
    getInputDevices,
    getOutputDevices
  }
}
