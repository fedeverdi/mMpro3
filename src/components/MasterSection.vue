<template>
  <div
    class="master-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-2 flex flex-col items-center gap-1 h-full w-full w-[12rem] max-w-[12rem]">
    <!-- Master Header -->
    <div class="w-full text-center">
      <div class="text-xs font-bold text-blue-400">MASTER</div>
    </div>

    <!-- Master Output Selector -->
    <div class="w-full bg-gray-900 rounded p-1.5 border border-gray-700">
      <OutputSelector 
        title="Master Output" 
        :devices="audioOutputDevices"
        :selected-device-id="selectedMasterOutput" 
        default-label="Default" 
        default-description="Default audio output"
        icon="🔊"
        default-icon="🔊"
        :show-no-output="false"
        @select="onMasterOutputSelect" 
      />
    </div>

    <!-- Headphones Control -->
    <HeadphonesControl
      :devices="audioOutputDevices"
      :selected-device-id="selectedHeadphonesOutput"
      :volume="headphonesVolume"
      :level="headphonesLevel"
      @select="onHeadphonesOutputSelect"
      @update:volume="headphonesVolume = $event"
    />

    <!-- VU Meters and Faders -->
    <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-4 min-h-0 mt-2">
      <!-- VU Meters Row -->
      <MasterMeter 
        :left-level="leftLevel" 
        :right-level="rightLevel" 
        :vu-meters-height="vuMetersHeight" 
      />

      <!-- Faders Row -->
      <div v-if="fadersHeight > 0" class="flex gap-2 items-end mb-6">
        <MasterFader v-model="leftVolume" label="L" :trackHeight="fadersHeight" />
        <MasterFader v-model="rightVolume" label="R" :trackHeight="fadersHeight" />
      </div>
    </div>

    <!-- Master Controls -->
    <div class="w-full mt-2 flex gap-1">
      <!-- Master Mute Button -->
      <button @click="toggleMasterMute" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="masterMuted ? 'bg-red-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'"
        title="Mute master output (headphones still active)">
        <div class="flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 576 512">
            <path v-if="masterMuted"
              d="M215.03 71.05L126.06 160H24c-13.26 0-24 10.74-24 24v144c0 13.25 10.74 24 24 24h102.06l88.97 88.95c15.03 15.03 40.97 4.47 40.97-16.97V88.02c0-21.46-25.96-31.98-40.97-16.97zM461.64 256l45.64-45.64c6.3-6.3 6.3-16.52 0-22.82l-22.82-22.82c-6.3-6.3-16.52-6.3-22.82 0L416 210.36l-45.64-45.64c-6.3-6.3-16.52-6.3-22.82 0l-22.82 22.82c-6.3 6.3-6.3 16.52 0 22.82L370.36 256l-45.63 45.63c-6.3 6.3-6.3 16.52 0 22.82l22.82 22.82c6.3 6.3 16.52 6.3 22.82 0L416 301.64l45.64 45.64c6.3 6.3 16.52 6.3 22.82 0l22.82-22.82c6.3-6.3 6.3-16.52 0-22.82L461.64 256z" />
            <path v-else
              d="M215.03 71.05L126.06 160H24c-13.26 0-24 10.74-24 24v144c0 13.25 10.74 24 24 24h102.06l88.97 88.95c15.03 15.03 40.97 4.47 40.97-16.97V88.02c0-21.46-25.96-31.98-40.97-16.97zm233.32-51.08c-14.17-8.18-32.06-3.34-40.24 10.82-8.18 14.17-3.34 32.06 10.82 40.24 65.09 37.54 105.76 107.59 105.76 184.97 0 77.38-40.67 147.43-105.76 184.97-14.17 8.18-19.01 26.07-10.82 40.24 8.18 14.17 26.07 19.01 40.24 10.82 77.62-44.79 126.34-128.31 126.34-236.03s-48.72-191.24-126.34-236.03zm-63.58 79.13c-14.17-8.19-32.06-3.34-40.24 10.82-8.19 14.17-3.34 32.06 10.82 40.24 34.44 19.87 55.89 57.1 55.89 96.84 0 39.74-21.45 76.97-55.89 96.84-14.17 8.19-19.01 26.07-10.82 40.24 8.19 14.17 26.07 19.01 40.24 10.82 50.68-29.23 87.16-84.25 87.16-147.9s-36.48-118.67-87.16-147.9z" />
          </svg>
        </div>
      </button>

      <!-- Link Button -->
      <button @click="toggleLink" class="flex-1 py-1 text-xs font-bold rounded transition-all"
        :class="isLinked ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
        <div class="flex items-center justify-center">
          <svg v-if="isLinked" xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 512 512">
            <path
              d="M326.612 185.391c59.747 59.809 58.927 155.698.36 214.59-.11.12-.24.25-.36.37l-67.2 67.2c-59.27 59.27-155.699 59.262-214.96 0-59.27-59.26-59.27-155.7 0-214.96l37.106-37.106c9.84-9.84 26.786-3.3 27.294 10.606.648 17.722 3.826 35.527 9.69 52.721 1.986 5.822.567 12.262-3.783 16.612l-13.087 13.087c-28.026 28.026-28.905 73.66-1.155 101.96 28.024 28.579 74.086 28.749 102.325.51l67.2-67.19c28.191-28.191 28.073-73.757 0-101.83-3.701-3.694-7.429-6.564-10.341-8.569a16.037 16.037 0 0 1-6.947-12.606c-.396-10.567 3.348-21.456 11.698-29.806l21.054-21.055c5.521-5.521 14.182-6.199 20.584-1.731a152.482 152.482 0 0 1 20.522 17.197zM467.547 44.449c-59.261-59.262-155.69-59.27-214.96 0l-67.2 67.2c-.12.12-.25.25-.36.37-58.566 58.892-59.387 154.781.36 214.59a152.454 152.454 0 0 0 20.521 17.196c6.402 4.468 15.064 3.789 20.584-1.731l21.054-21.055c8.35-8.35 12.094-19.239 11.698-29.806a16.037 16.037 0 0 0-6.947-12.606c-2.912-2.005-6.64-4.875-10.341-8.569-28.073-28.073-28.191-73.639 0-101.83l67.2-67.19c28.239-28.239 74.3-28.069 102.325.51 27.75 28.3 26.872 73.934-1.155 101.96l-13.087 13.087c-4.35 4.35-5.769 10.79-3.783 16.612 5.864 17.194 9.042 34.999 9.69 52.721.509 13.906 17.454 20.446 27.294 10.606l37.106-37.106c59.271-59.259 59.271-155.699.001-214.959z" />
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 448 512">
            <path
              d="M400 224h-24v-72C376 68.2 307.8 0 224 0S72 68.2 72 152v72H48c-26.5 0-48 21.5-48 48v192c0 26.5 21.5 48 48 48h352c26.5 0 48-21.5 48-48V272c0-26.5-21.5-48-48-48zm-104 0H152v-72c0-39.7 32.3-72 72-72s72 32.3 72 72v72z" />
          </svg>
        </div>
      </button>

      <!-- Recording Button -->
      <RecorderButton :is-recording="isRecording" @open="showRecorder = true" />
    </div>

    <!-- Recorder Modal -->
    <Recorder v-model="showRecorder" v-model:is-recording="isRecording" :audio-node="mergeNodeRef" :tone="ToneRef" :loaded-tracks="props.loadedTracks" />
  </div>
</template>

<script setup lang="ts">
import MasterFader from './master/MasterFader.vue'
import MasterMeter from './master/MasterMeter.vue'
import HeadphonesControl from './master/HeadphonesControl.vue'
import OutputSelector from './master/OutputSelector.vue'
import RecorderButton from './recorder/RecorderButton.vue'
import Recorder from './recorder/Recorder.vue'
import { ref, watch, onMounted, onUnmounted, nextTick, inject, toRaw } from 'vue'
import { useAudioDevices } from '../composables/useAudioDevices'

// Props
interface Props {
  masterFxOutputNode?: any  // Output node from MasterFX
  masterFxComponent?: any   // Component interface from MasterFX (for getSnapshot/restoreSnapshot)
  loadedTracks?: Array<{ trackNumber: number, fileName: string, fileId: string }>
}

const props = withDefaults(defineProps<Props>(), {
  loadedTracks: () => []
})

// Inject Tone.js from App.vue
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Master volumes
const leftVolume = ref(0)
const rightVolume = ref(0)
const headphonesVolume = ref(-60)
const isLinked = ref(true)
const masterMuted = ref(false)

// VU meter levels
const leftLevel = ref(-60)
const rightLevel = ref(-60)
const headphonesLevel = ref(-60)

// Recorder modal
const showRecorder = ref(false)
const isRecording = ref(false)

// Audio outputs
const { audioOutputDevices, enumerateAudioOutputs } = useAudioDevices()
const selectedHeadphonesOutput = ref<string | null>(null)
const selectedMasterOutput = ref<string | null>(null)

// Master output routing
let masterOutputGain: any = null
let masterOutputStreamDest: MediaStreamAudioDestinationNode | null = null
let masterOutputAudioContext: AudioContext | null = null

// Headphones output routing (direct connection in main context for perfect sync)
let headphonesGain: any = null
let headphonesStreamDest: MediaStreamAudioDestinationNode | null = null
let headphonesAudioContext: AudioContext | null = null

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let resizeObserver: ResizeObserver | null = null

// Tone.js nodes
let leftMeter: any = null
let rightMeter: any = null
let headphonesMeter: any = null
let splitNode: any = null
let leftGain: any = null
let rightGain: any = null
let mergeNode: any = null

// Ref for passing to child components
const mergeNodeRef = ref<any>(null)

// Calculate meters height based on container
function updateMetersHeight() {
  if (metersContainer.value) {
    const height = metersContainer.value.clientHeight
    const availableHeight = Math.max(160, height - 80)
    vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.4))
    fadersHeight.value = Math.max(100, Math.floor(availableHeight * 0.6))
  }
}

// Handle master output selection
async function onMasterOutputSelect(deviceId: string | null) {
  selectedMasterOutput.value = deviceId
  
  if (!mergeNode || !Tone) return
  
  try {
    // Disconnect from existing output
    if (masterOutputGain) {
      try {
        masterOutputGain.disconnect()
      } catch (e) {
        console.warn('[Master Output] Error disconnecting gain:', e)
      }
    }
    
    // Close existing master output context if any
    if (masterOutputAudioContext) {
      console.log('[Master Output] Closing old AudioContext, state:', masterOutputAudioContext.state)
      try {
        if (masterOutputAudioContext.state !== 'closed') {
          await masterOutputAudioContext.close()
        }
      } catch (e) {
        console.warn('[Master Output] Error closing context:', e)
      }
      masterOutputAudioContext = null
    }
    
    // Small delay to ensure cleanup is complete
    await new Promise(resolve => setTimeout(resolve, 50))
    
    // Create gain node for master output if it doesn't exist
    // This gain is used for master mute functionality (headphones bypass this)
    if (!masterOutputGain) {
      masterOutputGain = new Tone.Gain(1) as any
    }
    
    // Disconnect mergeNode and reconnect through masterOutputGain
    try {
      mergeNode.disconnect()
    } catch (e) { }
    
    // Connect mergeNode to masterOutputGain
    mergeNode.connect(masterOutputGain)
    
    // Apply mute state if needed
    if (masterMuted.value) {
      masterOutputGain.gain.value = 0
    } else {
      masterOutputGain.gain.value = 1
    }
    
    // If null is selected, use default Tone destination
    if (!deviceId) {
      console.log('[Master Output] Using default Tone.js destination')
      try {
        masterOutputGain.disconnect()
      } catch (e) { }
      masterOutputGain.toDestination()
      return
    }
    
    // Parse composite deviceId (format: \"realDeviceId:channelIndex\")
    let realDeviceId = deviceId
    let targetChannel: number | null = null
    
    if (deviceId.includes(':')) {
      const parts = deviceId.split(':')
      realDeviceId = parts[0]
      targetChannel = parseInt(parts[1], 10)
      console.log(`[Master Output] Parsed composite deviceId: device=\"${realDeviceId}\", channel=${targetChannel + 1}`)
    }
    
    // Create stream destination if not exists
    if (!masterOutputStreamDest) {
      masterOutputStreamDest = Tone.context.createMediaStreamDestination()
    }
    
    // Disconnect masterOutputGain and connect to stream destination
    try {
      masterOutputGain.disconnect()
    } catch (e) { }
    
    masterOutputGain.connect(masterOutputStreamDest as any)
    
    // Create new AudioContext targeting selected device
    const mainAudioContext = Tone.context.rawContext as AudioContext
    const contextOptions: any = {
      latencyHint: 'interactive',
      sampleRate: mainAudioContext.sampleRate
    }
    
    if (realDeviceId && realDeviceId !== '') {
      contextOptions.sinkId = realDeviceId
    }
    
    masterOutputAudioContext = new AudioContext(contextOptions)
    
    // Log device info
    console.log('[Master Output] Output AudioContext created')
    console.log('[Master Output] Destination maxChannelCount:', masterOutputAudioContext.destination.maxChannelCount)
    console.log('[Master Output] SinkId:', (masterOutputAudioContext as any).sinkId)
    
    // Detect number of output channels from device capabilities
    let deviceChannelCount = masterOutputAudioContext.destination.maxChannelCount
    
    // If we have a target channel from composite ID, use that as indicator of multi-channel device
    if (targetChannel !== null) {
      // For multi-channel devices like Rubix44
      deviceChannelCount = Math.max(4, targetChannel + 1)
    }
    
    console.log(`[Master Output] Device channel count: ${deviceChannelCount}`)
    
    // Configure destination for multi-channel output
    try {
      masterOutputAudioContext.destination.channelCount = deviceChannelCount
      masterOutputAudioContext.destination.channelCountMode = 'explicit'
      masterOutputAudioContext.destination.channelInterpretation = 'discrete'
      console.log(`[Master Output] Set destination to ${deviceChannelCount} channels (discrete)`)
    } catch (e) {
      console.warn('[Master Output] Could not configure destination:', e)
    }
    
    // Create audio routing from stream
    if (!masterOutputStreamDest) {
      throw new Error('Master output stream destination not initialized')
    }
    const source = masterOutputAudioContext.createMediaStreamSource(masterOutputStreamDest.stream)
    
    // If a specific channel was selected (from composite deviceId), route to that channel
    if (targetChannel !== null && deviceChannelCount > 2) {
      // Create a channel merger to route stereo master to specific output channels
      const channelMerger = masterOutputAudioContext.createChannelMerger(deviceChannelCount)
      
      // Split the stereo source
      const splitter = masterOutputAudioContext.createChannelSplitter(2)
      source.connect(splitter)
      
      // Route left to target channel, right to target+1 (if stereo width allows)
      splitter.connect(channelMerger, 0, targetChannel)
      if (targetChannel + 1 < deviceChannelCount) {
        splitter.connect(channelMerger, 1, targetChannel + 1)
        console.log(`[Master Output] Routing stereo to output channels ${targetChannel + 1}-${targetChannel + 2} of ${deviceChannelCount}`)
      } else {
        console.log(`[Master Output] Routing mono (left) to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
      }
      
      // Connect merger to destination
      channelMerger.connect(masterOutputAudioContext.destination)
    } else {
      // Default routing (stereo output or no specific channel selected)
      source.connect(masterOutputAudioContext.destination)
      console.log('[Master Output] Default stereo routing')
    }
    
    // Resume if suspended
    if (masterOutputAudioContext.state === 'suspended') {
      await masterOutputAudioContext.resume()
    }
    
    console.log('[Master Output] Changed to:', deviceId || 'default')
  } catch (error) {
    console.error('[Master Output] Error changing device:', error)
  }
}

// Handle headphones output selection
async function onHeadphonesOutputSelect(deviceId: string | null) {
  selectedHeadphonesOutput.value = deviceId
  
  if (!headphonesStreamDest || !Tone) return
  
  try {
    // Close existing headphones context if any
    if (headphonesAudioContext) {
      console.log('[Headphones Output] Closing old AudioContext, state:', headphonesAudioContext.state)
      try {
        if (headphonesAudioContext.state !== 'closed') {
          await headphonesAudioContext.close()
        }
      } catch (e) {
        console.warn('[Headphones Output] Error closing context:', e)
      }
      headphonesAudioContext = null
    }
    
    // Small delay to ensure cleanup is complete
    await new Promise(resolve => setTimeout(resolve, 50))
    
    // Parse composite deviceId (format: "realDeviceId:channelIndex")
    let realDeviceId = deviceId || ''
    let targetChannel: number | null = null
    
    if (deviceId && deviceId.includes(':')) {
      const parts = deviceId.split(':')
      realDeviceId = parts[0]
      targetChannel = parseInt(parts[1], 10)
      console.log(`[Headphones Output] Parsed composite deviceId: device="${realDeviceId}", channel=${targetChannel + 1}`)
    }
    
    // Create new AudioContext targeting selected device
    const mainAudioContext = Tone.context.rawContext as AudioContext
    const contextOptions: any = {
      latencyHint: 'interactive',
      sampleRate: mainAudioContext.sampleRate
    }
    
    if (realDeviceId && realDeviceId !== '') {
      contextOptions.sinkId = realDeviceId
    }
    
    headphonesAudioContext = new AudioContext(contextOptions)
    
    // Log device info
    console.log('[Headphones Output] Output AudioContext created')
    console.log('[Headphones Output] Destination maxChannelCount:', headphonesAudioContext.destination.maxChannelCount)
    console.log('[Headphones Output] SinkId:', (headphonesAudioContext as any).sinkId)
    
    // Detect number of output channels from device capabilities
    let deviceChannelCount = headphonesAudioContext.destination.maxChannelCount
    
    // If we have a target channel from composite ID, use that as indicator of multi-channel device
    if (targetChannel !== null) {
      deviceChannelCount = Math.max(4, targetChannel + 1)
    }
    
    console.log(`[Headphones Output] Device channel count: ${deviceChannelCount}`)
    
    // Configure destination for multi-channel output
    try {
      headphonesAudioContext.destination.channelCount = deviceChannelCount
      headphonesAudioContext.destination.channelCountMode = 'explicit'
      headphonesAudioContext.destination.channelInterpretation = 'discrete'
      console.log(`[Headphones Output] Set destination to ${deviceChannelCount} channels (discrete)`)
    } catch (e) {
      console.warn('[Headphones Output] Could not configure destination:', e)
    }
    
    // Create audio routing from stream
    const source = headphonesAudioContext.createMediaStreamSource(headphonesStreamDest.stream)
    
    // Check actual channel count from the source
    const actualChannelCount = source.channelCount
    console.log(`[Headphones Output] Source has ${actualChannelCount} channels`)
    
    // If a specific channel was selected (from composite deviceId), route to that channel
    if (targetChannel !== null && deviceChannelCount > 2) {
      // Create a channel merger to route stereo headphones to specific output channels
      const channelMerger = headphonesAudioContext.createChannelMerger(deviceChannelCount)
      
      if (actualChannelCount === 2) {
        // Stereo source - split and route to consecutive channels
        const splitter = headphonesAudioContext.createChannelSplitter(2)
        source.connect(splitter)
        
        // Route left to target channel, right to target+1 (if stereo width allows)
        splitter.connect(channelMerger, 0, targetChannel)
        if (targetChannel + 1 < deviceChannelCount) {
          splitter.connect(channelMerger, 1, targetChannel + 1)
          console.log(`[Headphones Output] Routing stereo to output channels ${targetChannel + 1}-${targetChannel + 2} of ${deviceChannelCount}`)
        } else {
          console.log(`[Headphones Output] Routing mono (left) to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
        }
      } else {
        // Mono source - route directly to target channel
        const monoGain = headphonesAudioContext.createGain()
        source.connect(monoGain)
        monoGain.connect(channelMerger, 0, targetChannel)
        console.log(`[Headphones Output] Routing mono to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
      }
      
      // Connect merger to destination
      channelMerger.connect(headphonesAudioContext.destination)
    } else {
      // Default routing (stereo output or no specific channel selected)
      source.connect(headphonesAudioContext.destination)
      console.log('[Headphones Output] Default stereo routing')
    }
    
    // Resume if suspended
    if (headphonesAudioContext.state === 'suspended') {
      await headphonesAudioContext.resume()
    }
    
    console.log('[Headphones Output] Changed to:', deviceId || 'default')
  } catch (error) {
    console.error('[Headphones Output] Error changing device:', error)
  }
}

// Initialize master channel
function initMasterChannel() {
  if (splitNode || !Tone) {
    return
  }

  // Create stereo routing: Split → [LeftGain, RightGain] → Merge
  // Input will come from MasterFX.outputNode via rebuildFXChain()
  splitNode = new Tone.Split()
  leftGain = new Tone.Gain(1)
  rightGain = new Tone.Gain(1)
  mergeNode = new Tone.Merge()
  
  // Sync to ref for child components
  mergeNodeRef.value = mergeNode

  // Create meters
  leftMeter = new Tone.Meter()
  rightMeter = new Tone.Meter()
  headphonesMeter = new Tone.Meter()

  // Create headphones routing in main context (perfect sync)
  const mainAudioContext = Tone.context.rawContext as AudioContext
  headphonesGain = new Tone.Gain(1)
  headphonesStreamDest = mainAudioContext.createMediaStreamDestination()

  // Build main chain - connection will be done by rebuildFXChain()
  // masterChannel will be connected to splitNode through FX chain or directly
  
  // Left channel: split → gain → meter → merge
  splitNode.connect(leftGain, 0)
  leftGain.connect(leftMeter)
  leftGain.connect(mergeNode, 0, 0)
  
  // Right channel: split → gain → meter → merge
  splitNode.connect(rightGain, 1)
  rightGain.connect(rightMeter)
  rightGain.connect(mergeNode, 0, 1)
  
  // Create master output gain for mute functionality
  // (headphones bypass this gain, so mute only affects main output)
  masterOutputGain = new Tone.Gain(1)
  mergeNode.connect(masterOutputGain)
  masterOutputGain.toDestination()
  
  // Headphones output: mergeNode → gain → meter → streamDest (all in main Tone context)
  mergeNode.connect(headphonesGain)
  headphonesGain.connect(headphonesMeter)
  headphonesGain.connect(headphonesStreamDest as any)

  // Update initial volumes
  updateMasterVolume()
  updateHeadphonesVolume()
  
  // Establish audio chain (FX → splitNode → faders → destination)
  rebuildFXChain()
}

// Level monitoring
let levelMonitorInterval: number | null = null

function startLevelMonitoring() {
  levelMonitorInterval = window.setInterval(() => {
    if (leftMeter && rightMeter && Tone) {
      const leftValue = leftMeter.getValue() as number
      const rightValue = rightMeter.getValue() as number

      leftLevel.value = Math.max(-60, leftValue)
      rightLevel.value = Math.max(-60, rightValue)
    }

    if (headphonesMeter && Tone) {
      const hpValue = headphonesMeter.getValue() as number
      headphonesLevel.value = Math.max(-60, hpValue)
    }
  }, 50)
}

// Update master volume
function updateMasterVolume() {
  if (!leftGain || !rightGain || !Tone) {
    initMasterChannel()
    if (!leftGain || !rightGain) return
  }

  leftGain.gain.value = Tone.dbToGain(leftVolume.value)
  rightGain.gain.value = Tone.dbToGain(rightVolume.value)
}

// Update headphones volume
function updateHeadphonesVolume() {
  if (!headphonesGain || !Tone) return

  headphonesGain.gain.value = Tone.dbToGain(headphonesVolume.value)
}

// Watch for volume changes
watch([leftVolume, rightVolume], updateMasterVolume)
watch(headphonesVolume, updateHeadphonesVolume)

// Watch master mute state
watch(masterMuted, (muted) => {
  if (masterOutputGain && Tone) {
    // Smoothly transition the gain to avoid clicking
    masterOutputGain.gain.rampTo(muted ? 0 : 1, 0.05)
    console.log(`[Master Output] ${muted ? 'Muted' : 'Unmuted'} (headphones still active)`)
  }
})

// When linked, sync right to left
watch(leftVolume, (newVal) => {
  if (isLinked.value) {
    rightVolume.value = newVal
  }
})

// Watch for masterFx output node to become available and rebuild chain
watch(() => props.masterFxOutputNode, (newVal) => {
  if (newVal && splitNode) {
    setTimeout(() => {
      rebuildFXChain()
    }, 100)
  }
}, { immediate: true })

// Link/unlink channels
function toggleLink() {
  isLinked.value = !isLinked.value
  if (isLinked.value) {
    rightVolume.value = leftVolume.value
  }
}

// Toggle master mute (headphones stay active)
function toggleMasterMute() {
  masterMuted.value = !masterMuted.value
}

// Initialize
onMounted(async () => {
  // Get Tone.js from inject
  if (ToneRef?.value) {
    Tone = ToneRef.value
    initMasterChannel()
  } else {
    const checkTone = setInterval(() => {
      if (ToneRef?.value) {
        Tone = ToneRef.value
        initMasterChannel()
        clearInterval(checkTone)
      }
    }, 100)
  }

  // Enumerate audio outputs
  await enumerateAudioOutputs()
  
  // Initialize headphones output with default device
  // Wait a bit to ensure headphonesStreamDest is created
  setTimeout(() => {
    if (headphonesStreamDest) {
      onHeadphonesOutputSelect(null)
    }
  }, 300)

  // Calculate initial height
  await nextTick()
  updateMetersHeight()

  // Watch for container size changes
  if (metersContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateMetersHeight()
    })
    resizeObserver.observe(metersContainer.value)
  }

  // Start level monitoring
  startLevelMonitoring()

  // Listen for device changes
  navigator.mediaDevices.addEventListener('devicechange', enumerateAudioOutputs)
  
  // Wait for MasterEQDisplay to be ready, then establish initial connection
  setTimeout(() => {
    rebuildFXChain()
  }, 200)
})

// Rebuild FX chain (now handled by MasterFX internally)
function rebuildFXChain() {
  if (!props.masterFxOutputNode || !Tone || !splitNode) return

  const fxOutput = toRaw(props.masterFxOutputNode)
  if (!fxOutput) return

  // NO disconnect! MasterFX output can have multiple destinations (SpectrumMeter + MasterSection)
  // Just connect (Tone.js allows multiple connections from same source)
  fxOutput.connect(splitNode)
}

// Expose meter values for FX visualization
function getMeterValues() {
  if (!leftMeter || !rightMeter) return { left: -60, right: -60 }
  const leftValue = leftMeter.getValue() as number
  const rightValue = rightMeter.getValue() as number
  return {
    left: Math.max(-60, leftValue),
    right: Math.max(-60, rightValue)
  }
}

function getPreLimiterValues() {
  // For now, same as regular meters (can add separate pre-limiter meters if needed)
  return getMeterValues()
}

// Scene Snapshot Support
function getSnapshot() {
  // Get FX snapshot from MasterFX
  const fxSnapshot = props.masterFxComponent?.getSnapshot?.() || {}
  
  return {
    leftVolume: leftVolume.value,
    rightVolume: rightVolume.value,
    headphonesVolume: headphonesVolume.value,
    isLinked: isLinked.value,
    masterMuted: masterMuted.value,
    selectedMasterOutput: selectedMasterOutput.value,
    selectedHeadphonesOutput: selectedHeadphonesOutput.value,
    ...fxSnapshot  // Merge FX snapshot
  }
}

function restoreSnapshot(snapshot: any) {
  if (!snapshot) return

  // Restore volumes
  if (snapshot.leftVolume !== undefined) leftVolume.value = snapshot.leftVolume
  if (snapshot.rightVolume !== undefined) rightVolume.value = snapshot.rightVolume
  if (snapshot.headphonesVolume !== undefined) headphonesVolume.value = snapshot.headphonesVolume
  if (snapshot.isLinked !== undefined) isLinked.value = snapshot.isLinked
  if (snapshot.masterMuted !== undefined) masterMuted.value = snapshot.masterMuted

  // Restore output devices
  if (snapshot.selectedMasterOutput !== undefined) {
    selectedMasterOutput.value = snapshot.selectedMasterOutput
    nextTick(() => {
      if (selectedMasterOutput.value !== null) {
        onMasterOutputSelect(selectedMasterOutput.value)
      }
    })
  }
  if (snapshot.selectedHeadphonesOutput !== undefined) {
    selectedHeadphonesOutput.value = snapshot.selectedHeadphonesOutput
    nextTick(() => {
      if (selectedHeadphonesOutput.value) {
        onHeadphonesOutputSelect(selectedHeadphonesOutput.value)
      }
    })
  }

  // Restore FX via MasterFX
  if (props.masterFxComponent?.restoreSnapshot) {
    props.masterFxComponent.restoreSnapshot(snapshot)
  }
}

// Expose minimal interface
defineExpose({
  getMeterValues,
  getPreLimiterValues,
  getSnapshot,
  restoreSnapshot,
  resetToDefaults: () => {
    // Reset volumes
    leftVolume.value = 0
    rightVolume.value = 0
    headphonesVolume.value = -60
    isLinked.value = true

    // Reset levels
    leftLevel.value = -60
    rightLevel.value = -60
    headphonesLevel.value = -60

    // Reset Master FX if available
    if (props.masterFxComponent?.resetToDefaults) {
      props.masterFxComponent.resetToDefaults()
    }
  }
})

// Cleanup
onUnmounted(() => {
  if (leftMeter) leftMeter.dispose()
  if (rightMeter) rightMeter.dispose()
  if (headphonesMeter) headphonesMeter.dispose()
  if (splitNode) splitNode.dispose()
  if (leftGain) leftGain.dispose()
  if (rightGain) rightGain.dispose()
  if (mergeNode) {
    mergeNode.dispose()
    mergeNodeRef.value = null
  }
  if (headphonesGain) headphonesGain.dispose()
  if (masterOutputGain) masterOutputGain.dispose()
  
  // Cleanup master output context
  if (masterOutputAudioContext) {
    masterOutputAudioContext.close()
    masterOutputAudioContext = null
  }
  if (masterOutputStreamDest) {
    masterOutputStreamDest = null
  }
  
  // Cleanup headphones context
  if (headphonesAudioContext) {
    headphonesAudioContext.close()
    headphonesAudioContext = null
  }
  if (headphonesStreamDest) {
    headphonesStreamDest = null
  }

  // Remove device change listener
  navigator.mediaDevices.removeEventListener('devicechange', enumerateAudioOutputs)

  if (resizeObserver) {
    resizeObserver.disconnect()
  }

  if (levelMonitorInterval) {
    clearInterval(levelMonitorInterval)
  }
})
</script>
