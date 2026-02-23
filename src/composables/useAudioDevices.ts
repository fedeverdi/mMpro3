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
        const inputDevices = devices.filter(device => device.kind === 'audioinput')
        
        // Expand multi-channel input devices into separate channel entries
        const expandedDevices: MediaDeviceInfo[] = []
        
        for (const device of inputDevices) {
          const label = device.label.toLowerCase()
          let channelCount = 1 // Default mono
          
          // Detect multi-channel devices by name
          if (label.includes('rubix') || label.includes('4x4') || label.includes('quad')) {
            channelCount = 4
          } else if (label.includes('8x8') || label.includes('octo')) {
            channelCount = 8
          } else if (label.includes('audio interface') && label.includes('usb')) {
            // Some USB audio interfaces might be multi-channel
            channelCount = 4
          }
          
          if (channelCount > 1) {
            // Create a virtual device entry for each channel
            for (let ch = 0; ch < channelCount; ch++) {
              // Create a virtual MediaDeviceInfo-like object
              const virtualDevice = {
                deviceId: `${device.deviceId}:${ch}`, // Composite ID: realId:channelIndex
                groupId: device.groupId,
                kind: device.kind,
                label: `${device.label} - Channel ${ch + 1}`,
                toJSON: device.toJSON
              } as MediaDeviceInfo
              
              expandedDevices.push(virtualDevice)
            }
            console.log(`[useAudioDevices] Expanded ${device.label} into ${channelCount} input channels`)
          } else {
            // Regular mono device, add as-is
            expandedDevices.push(device)
          }
        }
        
        audioInputDevices.value = expandedDevices
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
        const outputDevices = devices.filter(device => device.kind === 'audiooutput')
        
        // Expand multi-channel devices into separate channel entries
        const expandedDevices: MediaDeviceInfo[] = []
        
        for (const device of outputDevices) {
          const label = device.label.toLowerCase()
          let channelCount = 2 // Default stereo
          
          // Detect multi-channel devices by name
          if (label.includes('rubix') || label.includes('4x4') || label.includes('quad')) {
            channelCount = 4
          } else if (label.includes('8x8') || label.includes('octo')) {
            channelCount = 8
          } else if (label.includes('audio interface') && label.includes('usb')) {
            // Some USB audio interfaces might be multi-channel
            channelCount = 4
          }
          
          if (channelCount > 2) {
            // Create a virtual device entry for each channel
            for (let ch = 0; ch < channelCount; ch++) {
              // Create a virtual MediaDeviceInfo-like object
              const virtualDevice = {
                deviceId: `${device.deviceId}:${ch}`, // Composite ID: realId:channelIndex
                groupId: device.groupId,
                kind: device.kind,
                label: `${device.label} - Channel ${ch + 1}`,
                toJSON: device.toJSON
              } as MediaDeviceInfo
              
              expandedDevices.push(virtualDevice)
            }
            console.log(`[useAudioDevices] Expanded ${device.label} into ${channelCount} channels`)
          } else {
            // Regular stereo/mono device, add as-is
            expandedDevices.push(device)
          }
        }
        
        audioOutputDevices.value = expandedDevices
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
