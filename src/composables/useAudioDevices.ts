import { ref } from 'vue'

// Shared state for audio input devices
const audioInputDevices = ref<MediaDeviceInfo[]>([])
let devicesEnumerated = false
let enumerationPromise: Promise<void> | null = null

// Shared state for audio output devices
const audioOutputDevices = ref<MediaDeviceInfo[]>([])
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
    
    // Start enumeration
    enumerationPromise = (async () => {
      try {     
        // Request microphone permission to unlock device labels
        try {
          const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
          stream.getTracks().forEach(track => track.stop())
        } catch (permError) {
          console.warn('[useAudioDevices] Permission denied for device labels')
        }
        
        const devices = await navigator.mediaDevices.enumerateDevices()
        audioInputDevices.value = devices.filter(device => device.kind === 'audioinput')
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
    
    // Start enumeration
    outputEnumerationPromise = (async () => {
      try {
        const devices = await navigator.mediaDevices.enumerateDevices()
        audioOutputDevices.value = devices.filter(device => device.kind === 'audiooutput')
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
