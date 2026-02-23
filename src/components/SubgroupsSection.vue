<template>
    <div
        class="subgroups-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-gray-600 p-2 flex flex-col items-center gap-1 h-full w-full max-w-[5rem]">
        <!-- Subgroup Header -->
        <div class="w-full flex items-center justify-between gap-1">
            <div class="text-xs font-bold text-gray-400 flex-1 text-center">{{ subgroupName }}</div>
            <button 
                @click="$emit('remove')" 
                class="w-4 h-4 pb-[0.05rem] rounded-full bg-white/20 hover:bg-white/30 text-white/60 hover:text-white/80 text-xs flex items-center justify-center transition-all"
                title="Remove Subgroup"
            >
                ×
            </button>
        </div>

        <!-- Output Device Selector -->
        <div class="w-full bg-gray-900 rounded p-1.5 border border-gray-700">
            <OutputSelector title="Select Subgroup Output" :devices="audioOutputDevices"
                :selected-device-id="selectedOutput" default-label="Default" default-description="Default audio output"
                default-icon="🔊" :show-no-output="true" @select="handleOutputSelect" />
        </div>

        <!-- FX Section -->
        <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
            <div class="grid grid-cols-1 gap-1">
                <TrackCompressor ref="trackCompressorRef" :enabled="compressorEnabled" :compressor-node="compressor"
                    :meter="leftMeter" :track-number="0" @toggle="toggleCompressor" />
                <TrackReverb ref="trackReverbRef" :enabled="reverbEnabled" :reverb-node="reverb" :track-number="0"
                    @toggle="toggleReverb" />
                <TrackLimiter ref="trackLimiterRef" :enabled="limiterEnabled" :limiter-node="limiter" :meter="leftMeter"
                    :track-number="0" @toggle="toggleLimiter" />
                <TrackDelay ref="trackDelayRef" :enabled="delayEnabled" :delay-node="delay" :track-number="0"
                    @toggle="toggleDelay" />
            </div>
        </div>

        <!-- VU Meters and Faders -->
        <div ref="metersContainer" class="flex-1 w-full flex flex-col items-center justify-center gap-2 min-h-0">
            <!-- VU Meters Row -->
            <div v-if="vuMetersHeight > 0"
                class="flex flex-col items-center w-full justify-center bg-gray-900 rounded p-1 border border-gray-700">
                <div class="flex gap-0 relative">
                    <VuMeter :level="leftLevel" label="L" :height="vuMetersHeight" :width="10" class="-mr-3" :value-font-size="5" />
                    <VuMeter :level="rightLevel" label="R" :height="vuMetersHeight" :width="10" class="-ml-3" :value-font-size="5" />
                </div>
            </div>



            <!-- Fader -->
            <div v-if="fadersHeight > 0" class="flex gap-1 items-end mt-3">
                <SubgroupFader v-model="volume" label="SUB" :trackHeight="fadersHeight" />
            </div>
        </div>

        <!-- Route to Master Button -->
        <div class="w-full">
            <button @click="toggleRouteToMaster" class="w-full py-1 text-[0.5rem] font-bold rounded transition-all"
                :class="routeToMaster ? 'bg-blue-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
                {{ routeToMaster ? '→ MASTER' : '→ DIRECT' }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import VuMeter from './core/VuMeter.vue'
import OutputSelector from './master/OutputSelector.vue'
import TrackCompressor from './audioTrack/TrackCompressor.vue'
import TrackReverb from './audioTrack/TrackReverb.vue'
import TrackLimiter from './audioTrack/TrackLimiter.vue'
import TrackDelay from './audioTrack/TrackDelay.vue'
import { ref, watch, onMounted, onUnmounted, nextTick, inject, toRaw } from 'vue'
import SubgroupFader from './subgroups/SubgroupFader.vue'
import { useAudioDevices } from '../composables/useAudioDevices'

// Props
interface Props {
    masterChannel?: any
    subgroupId?: number
    subgroupName?: string
}

const props = withDefaults(defineProps<Props>(), {
    subgroupName: 'SUBGROUP'
})

defineEmits<{
    remove: []
}>()

// Inject Tone.js from App.vue
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Subgroup volume
const volume = ref(0)
const routeToMaster = ref(false)

// FX refs and state
const trackCompressorRef = ref<any>(null)
const trackReverbRef = ref<any>(null)
const trackLimiterRef = ref<any>(null)
const trackDelayRef = ref<any>(null)
const compressorEnabled = ref(false)
const reverbEnabled = ref(false)
const limiterEnabled = ref(false)
const delayEnabled = ref(false)

// VU meter levels
const leftLevel = ref(-60)
const rightLevel = ref(-60)

// Audio outputs
const { audioOutputDevices, enumerateAudioOutputs } = useAudioDevices()
const selectedOutput = ref<string | null>(null)

// Container and dynamic height
const metersContainer = ref<HTMLElement | null>(null)
const vuMetersHeight = ref(0)
const fadersHeight = ref(0)
let resizeObserver: ResizeObserver | null = null

// Tone.js nodes
let inputGainNode: any = null // Input buffer node
let compressor: any = null
let reverb: any = null
let limiter: any = null
let delay: any = null
let leftMeter: any = null
let rightMeter: any = null
let splitNode: any = null
let leftGain: any = null
let rightGain: any = null
let outputMerge: any = null // Output merge node

// Output routing (direct connection in main context for perfect sync)
let outputGain: any = null
let outputStreamDest: MediaStreamAudioDestinationNode | null = null
let outputAudioContext: AudioContext | null = null

// Calculate meters height based on container
function updateMetersHeight() {
    if (metersContainer.value) {
        const height = metersContainer.value.clientHeight
        const availableHeight = Math.max(160, height - 80)
        fadersHeight.value = Math.max(90, Math.floor(availableHeight * 0.49))
        vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.46))
    }
}

// Handle output selection
function handleOutputSelect(deviceId: string | null) {
    selectedOutput.value = deviceId
    onOutputSelect()
}

async function onOutputSelect() {
    if (!outputStreamDest || !Tone) return

    try {
        // Close existing output context if any
        if (outputAudioContext) {
            console.log('[Subgroup Output] Closing old AudioContext, state:', outputAudioContext.state)
            try {
                if (outputAudioContext.state !== 'closed') {
                    await outputAudioContext.close()
                }
            } catch (e) {
                console.warn('[Subgroup Output] Error closing context:', e)
            }
            outputAudioContext = null
        }

        // Small delay to ensure cleanup is complete
        await new Promise(resolve => setTimeout(resolve, 50))

        // If "no-output" is selected, don't create any output context
        if (selectedOutput.value === 'no-output') {
            return
        }

        // Parse composite deviceId (format: "realDeviceId:channelIndex")
        let realDeviceId = selectedOutput.value || ''
        let targetChannel: number | null = null
        
        if (selectedOutput.value && selectedOutput.value.includes(':')) {
            const parts = selectedOutput.value.split(':')
            realDeviceId = parts[0]
            targetChannel = parseInt(parts[1], 10)
            console.log(`[Subgroup Output] Parsed composite deviceId: device="${realDeviceId}", channel=${targetChannel + 1}`)
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

        outputAudioContext = new AudioContext(contextOptions)
        
        // Log device info
        console.log('[Subgroup Output] Output AudioContext created')
        console.log('[Subgroup Output] Destination maxChannelCount:', outputAudioContext.destination.maxChannelCount)
        console.log('[Subgroup Output] SinkId:', (outputAudioContext as any).sinkId)
        
        // Detect number of output channels from device capabilities
        let deviceChannelCount = outputAudioContext.destination.maxChannelCount
        
        // If we have a target channel from composite ID, use that as indicator of multi-channel device
        if (targetChannel !== null) {
            // Target channel tells us the device has at least targetChannel+1 channels
            // For Rubix44 we know it has 4 channels
            deviceChannelCount = Math.max(4, targetChannel + 1)
        }
        
        console.log(`[Subgroup Output] Device channel count: ${deviceChannelCount}`)
        
        // Configure destination for multi-channel output
        try {
            outputAudioContext.destination.channelCount = deviceChannelCount
            outputAudioContext.destination.channelCountMode = 'explicit'
            outputAudioContext.destination.channelInterpretation = 'discrete'
            console.log(`[Subgroup Output] Set destination to ${deviceChannelCount} channels (discrete)`)
        } catch (e) {
            console.warn('[Subgroup Output] Could not configure destination:', e)
        }
        
        // Create audio routing
        const source = outputAudioContext.createMediaStreamSource(outputStreamDest.stream)
        
        // Check actual channel count from the source
        const actualChannelCount = source.channelCount
        console.log(`[Subgroup Output] Source has ${actualChannelCount} channels`)
        
        // If a specific channel was selected (from composite deviceId), route to that channel
        if (targetChannel !== null && deviceChannelCount > 2) {
            // Create a channel merger to route subgroup to specific output channels
            const channelMerger = outputAudioContext.createChannelMerger(deviceChannelCount)
            
            if (actualChannelCount === 2) {
                // Stereo source - split and route to consecutive channels
                const splitter = outputAudioContext.createChannelSplitter(2)
                source.connect(splitter)
                
                // Route left to target channel, right to target+1 (if stereo width allows)
                splitter.connect(channelMerger, 0, targetChannel)
                if (targetChannel + 1 < deviceChannelCount) {
                    splitter.connect(channelMerger, 1, targetChannel + 1)
                    console.log(`[Subgroup Output] Routing stereo to output channels ${targetChannel + 1}-${targetChannel + 2} of ${deviceChannelCount}`)
                } else {
                    console.log(`[Subgroup Output] Routing mono (left) to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
                }
            } else {
                // Mono source - route directly to target channel
                const monoGain = outputAudioContext.createGain()
                source.connect(monoGain)
                monoGain.connect(channelMerger, 0, targetChannel)
                console.log(`[Subgroup Output] Routing mono to output channel ${targetChannel + 1} of ${deviceChannelCount}`)
            }
            
            // Connect merger to destination
            channelMerger.connect(outputAudioContext.destination)
        } else {
            // Default routing (stereo output or no specific channel selected)
            source.connect(outputAudioContext.destination)
            console.log('[Subgroup Output] Default stereo routing')
        }

        // Resume if suspended
        if (outputAudioContext.state === 'suspended') {
            await outputAudioContext.resume()
        }

        console.log('[Subgroup Output] Changed to:', selectedOutput.value || 'default')
    } catch (error) {
        console.error('[Subgroup Output] Error changing device:', error)
    }
}

// Initialize subgroup channel
function initSubgroupChannel() {
    if (splitNode || !Tone) {
        return
    }

    // Create INPUT gain node (buffer for subgroupChannel to connect to)
    inputGainNode = new Tone.Gain(1)

    // Create FX nodes
    compressor = new Tone.Compressor({
        threshold: -24,
        ratio: 4,
        attack: 0.003,
        release: 0.25,
        knee: 10
    })

    reverb = new Tone.Reverb({
        decay: 1.5,
        preDelay: 0.01,
        wet: 0
    })

    limiter = new Tone.Compressor({
        threshold: -3,
        ratio: 20,
        attack: 0.003,
        release: 0.1,
        knee: 0
    })

    delay = new Tone.FeedbackDelay({
        delayTime: 0.25,
        feedback: 0.3,
        wet: 0
    })

    // Create stereo split/process/merge chain
    splitNode = new Tone.Split()
    leftGain = new Tone.Gain(1)
    rightGain = new Tone.Gain(1)
    outputMerge = new Tone.Merge()

    // Create meters
    leftMeter = new Tone.Meter()
    rightMeter = new Tone.Meter()

    // Create output routing in main context (perfect sync)
    const mainAudioContext = Tone.context.rawContext as AudioContext
    outputGain = new Tone.Gain(1)
    outputStreamDest = mainAudioContext.createMediaStreamDestination()

    // Build audio chain:
    // INPUT: inputGainNode (subgroupChannel connects here)
    //   ↓
    // compressor → reverb → limiter → delay (based on enable state)
    //   ↓
    // splitNode (split L/R)
    //   ↓           ↓
    // leftGain    rightGain
    //   ↓           ↓
    // leftMeter   rightMeter
    //   ↓           ↓
    // outputMerge → outputStreamDest → audio element

    // Connect input through FX chain to split (will be rebuilt by reconnectAudioChain based on FX state)
    reconnectAudioChain()

    // Left channel chain
    splitNode.connect(leftGain, 0)
    leftGain.connect(leftMeter)
    leftGain.connect(outputMerge, 0, 0)

    // Right channel chain
    splitNode.connect(rightGain, 1)
    rightGain.connect(rightMeter)
    rightGain.connect(outputMerge, 0, 1)

    // Output to MediaStream for device routing
    // Chain: inputGainNode → FX → split → faders → outputMerge → outputGain → streamDest + (master or destination)
    outputMerge.connect(outputGain)
    outputGain.connect(outputStreamDest as any)
    
    // Connect to master or direct output based on routeToMaster
    updateRouting()

    // Update initial volumes
    updateSubgroupVolume()
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
    }, 50)
}

// Update subgroup volume
function updateSubgroupVolume() {
    if (!leftGain || !rightGain || !Tone) {
        initSubgroupChannel()
        if (!leftGain || !rightGain) return
    }

    const gainValue = Tone.dbToGain(volume.value)

    leftGain.gain.value = gainValue
    rightGain.gain.value = gainValue
}

// Watch for volume changes
watch(volume, updateSubgroupVolume)

// Update audio routing (master vs direct)
function updateRouting() {
    if (!outputGain || !Tone) return

    try {
        // Disconnect any existing output connections (except stream dest)
        outputGain.disconnect()
    } catch (e) {
        // Already disconnected
    }

    // Always reconnect to stream dest
    try {
        outputGain.connect(outputStreamDest as any)
    } catch (e) {
        console.error('[Subgroup] Error connecting to stream dest:', e)
    }

    // Connect to master or direct destination or no output
    if (routeToMaster.value && props.masterChannel) {
        try {
            // Unwrap masterChannel from Vue reactivity with toRaw
            const rawMasterChannel = toRaw(props.masterChannel)
            // Verify masterChannel is a valid audio node
            if (rawMasterChannel && typeof rawMasterChannel.connect === 'function') {
                outputGain.connect(rawMasterChannel)
            } else {
                console.warn('[Subgroup] Master channel not ready yet')
            }
        } catch (e) {
            console.error('[Subgroup] Error connecting to master channel:', e)
        }
    } else if (selectedOutput.value === 'no-output') {
        // No output - meters work but no audio out
        console.log('[Subgroup] No output selected')
    } else if (!selectedOutput.value || selectedOutput.value === '') {
        // Default output when no specific device is selected
        try {
            outputGain.connect(Tone.getDestination())
            console.log('[Subgroup] Connecting to default Tone destination')
        } catch (e) {
            console.error('[Subgroup] Error connecting to destination:', e)
        }
    } else {
        // Specific device selected - audio will be routed via outputAudioContext
        // No need to connect to Tone destination
        console.log('[Subgroup] Using dedicated output device:', selectedOutput.value)
    }
}

// Toggle route to master
function toggleRouteToMaster() {
    routeToMaster.value = !routeToMaster.value
    updateRouting()
}

// Watch for route changes
watch(routeToMaster, updateRouting)

// Watch for output device changes to update routing
watch(selectedOutput, updateRouting)

// FX toggles
function toggleCompressor() {
    compressorEnabled.value = !compressorEnabled.value
}

function toggleReverb() {
    reverbEnabled.value = !reverbEnabled.value
}

function toggleLimiter() {
    limiterEnabled.value = !limiterEnabled.value
}

function toggleDelay() {
    delayEnabled.value = !delayEnabled.value
}

// Reconnect audio chain based on FX enable state
function reconnectAudioChain() {
    if (!inputGainNode || !splitNode || !compressor || !reverb || !limiter || !delay) return

    // Disconnect all nodes from their outputs (this doesn't affect inputs)
    try {
        inputGainNode.disconnect()
        compressor.disconnect()
        reverb.disconnect()
        limiter.disconnect()
        delay.disconnect()
    } catch (e) {
        // Already disconnected
    }

    // Build chain based on enabled FX: input → compressor → reverb → limiter → delay → split
    let currentNode = inputGainNode

    if (compressorEnabled.value) {
        currentNode.connect(compressor)
        currentNode = compressor
    }

    if (reverbEnabled.value) {
        currentNode.connect(reverb)
        currentNode = reverb
    }

    if (limiterEnabled.value) {
        currentNode.connect(limiter)
        currentNode = limiter
    }

    if (delayEnabled.value) {
        currentNode.connect(delay)
        currentNode = delay
    }

    // Connect final node to split
    currentNode.connect(splitNode)
}

// Watch for FX enable/disable
watch([compressorEnabled, reverbEnabled, limiterEnabled, delayEnabled], reconnectAudioChain)

// Initialize
onMounted(async () => {
    // Get Tone.js from inject
    if (ToneRef?.value) {
        Tone = ToneRef.value
        initSubgroupChannel()
    } else {
        const checkTone = setInterval(() => {
            if (ToneRef?.value) {
                Tone = ToneRef.value
                initSubgroupChannel()
                clearInterval(checkTone)
            }
        }, 100)
    }

    // Enumerate audio outputs
    await enumerateAudioOutputs()

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
})

// Get subgroup input node for tracks to connect to
function getInputNode() {
    return inputGainNode
}

// Scene Snapshot Support
function getSnapshot() {
    return {
        volume: volume.value,
        routeToMaster: routeToMaster.value,
        selectedOutput: selectedOutput.value,
        compressorEnabled: compressorEnabled.value,
        reverbEnabled: reverbEnabled.value,
        limiterEnabled: limiterEnabled.value,
        delayEnabled: delayEnabled.value,
        compressorParams: trackCompressorRef.value?.getSnapshot() || null,
        reverbParams: trackReverbRef.value?.getSnapshot() || null,
        limiterParams: trackLimiterRef.value?.getSnapshot() || null,
        delayParams: trackDelayRef.value?.getSnapshot() || null
    }
}

function restoreSnapshot(snapshot: any) {
    if (!snapshot) return

    if (snapshot.volume !== undefined) volume.value = snapshot.volume
    if (snapshot.routeToMaster !== undefined) routeToMaster.value = snapshot.routeToMaster
    if (snapshot.selectedOutput !== undefined) {
        selectedOutput.value = snapshot.selectedOutput
        nextTick(() => onOutputSelect())
    }

    // Restore FX state
    if (snapshot.compressorEnabled !== undefined) compressorEnabled.value = snapshot.compressorEnabled
    if (snapshot.reverbEnabled !== undefined) reverbEnabled.value = snapshot.reverbEnabled
    if (snapshot.limiterEnabled !== undefined) limiterEnabled.value = snapshot.limiterEnabled
    if (snapshot.delayEnabled !== undefined) delayEnabled.value = snapshot.delayEnabled

    // Restore FX parameters
    nextTick(() => {
        if (snapshot.compressorParams && trackCompressorRef.value?.restoreSnapshot) {
            trackCompressorRef.value.restoreSnapshot(snapshot.compressorParams)
        }
        if (snapshot.reverbParams && trackReverbRef.value?.restoreSnapshot) {
            trackReverbRef.value.restoreSnapshot(snapshot.reverbParams)
        }
        if (snapshot.limiterParams && trackLimiterRef.value?.restoreSnapshot) {
            trackLimiterRef.value.restoreSnapshot(snapshot.limiterParams)
        }
        if (snapshot.delayParams && trackDelayRef.value?.restoreSnapshot) {
            trackDelayRef.value.restoreSnapshot(snapshot.delayParams)
        }
    })
}

// Expose interface
defineExpose({
    getInputNode,
    getSnapshot,
    restoreSnapshot,
    resetToDefaults: () => {
        volume.value = 0
        routeToMaster.value = false
        leftLevel.value = -60
        rightLevel.value = -60
        selectedOutput.value = null
        compressorEnabled.value = false
        reverbEnabled.value = false
        limiterEnabled.value = false
        delayEnabled.value = false

        // Reset FX components
        if (trackCompressorRef.value?.resetToDefaults) {
            trackCompressorRef.value.resetToDefaults()
        }
        if (trackReverbRef.value?.resetToDefaults) {
            trackReverbRef.value.resetToDefaults()
        }
        if (trackLimiterRef.value?.resetToDefaults) {
            trackLimiterRef.value.resetToDefaults()
        }
        if (trackDelayRef.value?.resetToDefaults) {
            trackDelayRef.value.resetToDefaults()
        }
    }
})

// Cleanup
onUnmounted(() => {
    try {
        if (inputGainNode) inputGainNode.dispose()
        if (compressor) compressor.dispose()
        if (reverb) reverb.dispose()
        if (limiter) limiter.dispose()
        if (delay) delay.dispose()
        if (leftMeter) leftMeter.dispose()
        if (rightMeter) rightMeter.dispose()
        if (splitNode) splitNode.dispose()
        if (leftGain) leftGain.dispose()
        if (rightGain) rightGain.dispose()
        if (outputMerge) outputMerge.dispose()
        if (outputGain) outputGain.dispose()
    } catch (e) {
        console.warn('[Subgroup] Error disposing audio nodes:', e)
    }

    // Cleanup output context
    if (outputAudioContext) {
        try {
            outputAudioContext.close()
        } catch (e) {
            console.warn('[Subgroup] Error closing output context:', e)
        }
        outputAudioContext = null
    }
    if (outputStreamDest) {
        outputStreamDest = null
    }

    // Remove device change listener
    try {
        navigator.mediaDevices.removeEventListener('devicechange', enumerateAudioOutputs)
    } catch (e) {
        // Ignore
    }

    if (resizeObserver) {
        resizeObserver.disconnect()
    }

    if (levelMonitorInterval) {
        clearInterval(levelMonitorInterval)
    }
})
</script>
