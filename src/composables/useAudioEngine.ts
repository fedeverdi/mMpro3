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
  const startListening = () => {
    if (isListening || !window.audioEngine) return
    
    window.audioEngine.onResponse((response: any) => {
      console.log('[useAudioEngine] Response:', response.type)
      
      switch (response.type) {
        case 'devices':
          state.value.devices = response.devices
          console.log('[useAudioEngine] Available devices:', response.devices.length)
          break
          
        case 'started':
          state.value.isRunning = true
          console.log('[useAudioEngine] Engine started')
          break
          
        case 'stopped':
          state.value.isRunning = false
          console.log('[useAudioEngine] Engine stopped')
          break
          
        case 'error':
          console.error('[useAudioEngine] Engine error:', response.message)
          break
      }
    })
    
    isListening = true
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
    
    console.log('[useAudioEngine] Starting audio engine...')
    await window.audioEngine.start()
  }
  
  const stop = async () => {
    if (!window.audioEngine) return
    
    console.log('[useAudioEngine] Stopping audio engine...')
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
    getInputDevices,
    getOutputDevices
  }
}
