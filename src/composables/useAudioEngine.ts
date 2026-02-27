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
}

const state = ref<AudioEngineState>({
  isRunning: false,
  devices: [],
  selectedInputDevice: null,
  selectedOutputDevice: null
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
          console.log('[useAudioEngine] Available devices:', response.devices.length)
          break
          
        case 'started':
          state.value.isRunning = true
          console.log('[useAudioEngine] Engine started - state updated to RUNNING')
          break
          
        case 'stopped':
          state.value.isRunning = false
          console.log('[useAudioEngine] Engine stopped - state updated to STOPPED')
          break
          
        case 'ok':
          // Command acknowledged successfully (silent)
          break
          
        case 'error':
          console.error('[useAudioEngine] Engine error:', response.message)
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
  
  const start = async () => {
    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
      return
    }
    
    startListening()
    await window.audioEngine.start()
  }
  
  const stop = async () => {
    if (!window.audioEngine) return
    
    await window.audioEngine.stop()
  }
  
  const setTrackGain = async (track: number, gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    await window.audioEngine.setGain(track, gain)
  }
  
  const setTrackMute = async (track: number, mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    await window.audioEngine.setMute(track, mute)
  }
  
  const setTrackEQ = async (track: number, low: number, mid: number, high: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    await window.audioEngine.setEQ(track, low, mid, high)
  }
  
  const setTrackCompressor = async (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    await window.audioEngine.setCompressor(track, enabled, threshold, ratio, attack, release)
  }
  
  const setTrackGate = async (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    await window.audioEngine.setGate(track, enabled, threshold, range, attack, release)
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
    setTrackGain,
    setTrackMute,
    setTrackEQ,
    setTrackCompressor,
    setTrackGate,
    getInputDevices,
    getOutputDevices
  }
}
