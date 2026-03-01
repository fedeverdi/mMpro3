import { ref } from 'vue'

// Shared state for audio input devices
const audioInputDevices = ref<RustAudioDevice[]>([])
let devicesEnumerated = false
let enumerationPromise: Promise<void> | null = null

// Shared state for audio output devices
const audioOutputDevices = ref<RustAudioDevice[]>([])
let outputDevicesEnumerated = false
let outputEnumerationPromise: Promise<void> | null = null

export function useAudioDevices() {
  async function enumerateAudioInputs() {
    // If already enumerating, return the same promise
    if (enumerationPromise) {
      return enumerationPromise
    }
    
    // If already enumerated, return immediately
    if (devicesEnumerated) {
      return
    }

    // Start enumeration from Rust audio engine
    enumerationPromise = (async () => {
      try {
        console.log('[useAudioDevices] Fetching audio inputs from Rust engine...')
        
        // Get input devices from Rust engine (already expanded with channels)
        const devices = await window.audioEngine.listAudioInputs()
        
        console.log(`[useAudioDevices] Received ${devices.length} audio inputs from Rust`)
        
        audioInputDevices.value = devices
        devicesEnumerated = true
      } catch (error) {
        console.error('[useAudioDevices] Error enumerating audio inputs:', error)
      } finally {
        enumerationPromise = null
      }
    })()
    
    return enumerationPromise
  }
  
  function refreshAudioInputs() {
    devicesEnumerated = false
    return enumerateAudioInputs()
  }

  async function enumerateAudioOutputs() {
    // If already enumerating, return the same promise
    if (outputEnumerationPromise) {
      return outputEnumerationPromise
    }
    
    // If already enumerated, return immediately
    if (outputDevicesEnumerated) {
      return
    }
    
    // Start enumeration from Rust audio engine
    outputEnumerationPromise = (async () => {
      try {
        console.log('[useAudioDevices] Fetching audio outputs from Rust engine...')
        
        // Get all devices from Rust engine and filter only outputs
        const allDevices = await window.audioEngine.listDevices()
        const outputDevices = allDevices.filter((device: RustAudioDevice) => device.output_channels > 0)
        
        console.log(`[useAudioDevices] Received ${outputDevices.length} audio outputs from Rust`)
        
        audioOutputDevices.value = outputDevices
        outputDevicesEnumerated = true
      } catch (error) {
        console.error('[useAudioDevices] Error enumerating audio outputs:', error)
      } finally {
        outputEnumerationPromise = null
      }
    })()
    
    return outputEnumerationPromise
  }
  
  function refreshAudioOutputs() {
    outputDevicesEnumerated = false
    return enumerateAudioOutputs()
  }
  
  return {
    audioInputDevices,
    enumerateAudioInputs,
    refreshAudioInputs,
    audioOutputDevices,
    enumerateAudioOutputs,
    refreshAudioOutputs
  }
}
