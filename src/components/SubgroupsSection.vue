<template>
    <div
        class="subgroups-section bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-gray-600 p-2 flex flex-col items-center gap-1 h-full w-full max-w-[8rem]">
        <!-- Subgroup Header -->
        <div class="w-full text-center">
            <div class="text-xs font-bold text-gray-400">SUBGROUP</div>
        </div>

        <!-- Output Device Selector -->
        <div class="w-full bg-gray-900 rounded p-1.5 border border-gray-700">
            <OutputSelector icon="🔊" title="Select Subgroup Output" :devices="audioOutputs"
                :selected-device-id="selectedOutput" default-label="Default" default-description="Default audio output"
                default-icon="🔊" @select="handleOutputSelect" />
        </div>

        <!-- FX Section -->
        <div class="w-full bg-gray-900 rounded p-1 border border-gray-700">
            <div class="grid grid-cols-2 gap-1">
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
                class="flex flex-col items-center gap-1 w-full justify-center bg-gray-900 rounded p-1 border border-gray-700">
                <div class="flex gap-1.5 relative">
                    <VuMeter :level="leftLevel" label="L" :height="vuMetersHeight" :width="20" />
                    <VuMeter :level="rightLevel" label="R" :height="vuMetersHeight" :width="20" />
                    <div
                        class="text-[6px] text-gray-500 uppercase tracking-wider absolute bottom-6 left-1/2 transform -translate-x-1/2">
                        RMS</div>
                </div>
            </div>



            <!-- Faders Row -->
            <div v-if="fadersHeight > 0" class="flex gap-1 items-end mt-3">
                <SubgroupFader v-model="leftVolume" label="L" :trackHeight="fadersHeight" />
                <SubgroupFader v-model="rightVolume" label="R" :trackHeight="fadersHeight" />
            </div>
        </div>

        <!-- Link Button -->
        <div class="w-full">
            <button @click="toggleLink" class="w-full py-1 text-xs font-bold rounded transition-all"
                :class="isLinked ? 'bg-gray-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
                <div class="flex items-center justify-center">
                    <svg v-if="isLinked" xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3"
                        viewBox="0 0 512 512">
                        <path
                            d="M326.612 185.391c59.747 59.809 58.927 155.698.36 214.59-.11.12-.24.25-.36.37l-67.2 67.2c-59.27 59.27-155.699 59.262-214.96 0-59.27-59.26-59.27-155.7 0-214.96l37.106-37.106c9.84-9.84 26.786-3.3 27.294 10.606.648 17.722 3.826 35.527 9.69 52.721 1.986 5.822.567 12.262-3.783 16.612l-13.087 13.087c-28.026 28.026-28.905 73.66-1.155 101.96 28.024 28.579 74.086 28.749 102.325.51l67.2-67.19c28.191-28.191 28.073-73.757 0-101.83-3.701-3.694-7.429-6.564-10.341-8.569a16.037 16.037 0 0 1-6.947-12.606c-.396-10.567 3.348-21.456 11.698-29.806l21.054-21.055c5.521-5.521 14.182-6.199 20.584-1.731a152.482 152.482 0 0 1 20.522 17.197zM467.547 44.449c-59.261-59.262-155.69-59.27-214.96 0l-67.2 67.2c-.12.12-.25.25-.36.37-58.566 58.892-59.387 154.781.36 214.59a152.454 152.454 0 0 0 20.521 17.196c6.402 4.468 15.064 3.789 20.584-1.731l21.054-21.055c8.35-8.35 12.094-19.239 11.698-29.806a16.037 16.037 0 0 0-6.947-12.606c-2.912-2.005-6.64-4.875-10.341-8.569-28.073-28.073-28.191-73.639 0-101.83l67.2-67.19c28.239-28.239 74.3-28.069 102.325.51 27.75 28.3 26.872 73.934-1.155 101.96l-13.087 13.087c-4.35 4.35-5.769 10.79-3.783 16.612 5.864 17.194 9.042 34.999 9.69 52.721.509 13.906 17.454 20.446 27.294 10.606l37.106-37.106c59.271-59.259 59.271-155.699.001-214.959z" />
                    </svg>
                    <svg v-else xmlns="http://www.w3.org/2000/svg" fill="white" class="h-3 w-3" viewBox="0 0 448 512">
                        <path
                            d="M400 224h-24v-72C376 68.2 307.8 0 224 0S72 68.2 72 152v72H48c-26.5 0-48 21.5-48 48v192c0 26.5 21.5 48 48 48h352c26.5 0 48-21.5 48-48V272c0-26.5-21.5-48-48-48zm-104 0H152v-72c0-39.7 32.3-72 72-72s72 32.3 72 72v72z" />
                    </svg>
                </div>
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
import { ref, watch, onMounted, onUnmounted, nextTick, inject } from 'vue'
import SubgroupFader from './subgroups/SubgroupFader.vue'

// Inject Tone.js from App.vue
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Subgroup volumes
const leftVolume = ref(0)
const rightVolume = ref(0)
const isLinked = ref(true)

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
const audioOutputs = ref<MediaDeviceInfo[]>([])
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

// Output routing
let outputStreamDest: MediaStreamAudioDestinationNode | null = null
let outputAudioElement: HTMLAudioElement | null = null

// Calculate meters height based on container
function updateMetersHeight() {
    if (metersContainer.value) {
        const height = metersContainer.value.clientHeight
        const availableHeight = Math.max(160, height - 80)
        fadersHeight.value = Math.max(90, Math.floor(availableHeight * 0.49))
        vuMetersHeight.value = Math.max(60, Math.floor(availableHeight * 0.46))
    }
}

// Enumerate audio output devices
async function enumerateAudioOutputs() {
    try {
        // Request microphone permission to unlock device labels
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
            stream.getTracks().forEach(track => track.stop())
        } catch (permError) {
            console.warn('[Subgroup Audio Outputs] Permission denied for device labels')
        }

        const devices = await navigator.mediaDevices.enumerateDevices()
        audioOutputs.value = devices.filter(device => device.kind === 'audiooutput')
    } catch (error) {
        console.error('[Subgroup Audio Outputs] Error enumerating devices:', error)
    }
}

// Handle output selection
function handleOutputSelect(deviceId: string | null) {
    selectedOutput.value = deviceId
    onOutputSelect()
}

async function onOutputSelect() {
    if (!outputAudioElement) return

    try {
        if (selectedOutput.value) {
            // Set device
            await (outputAudioElement as any).setSinkId(selectedOutput.value)

            // Ensure playback
            if (outputAudioElement.paused) {
                await outputAudioElement.play()
            }
        } else {
            // Default output
            await (outputAudioElement as any).setSinkId('')

            // Ensure playback
            if (outputAudioElement.paused) {
                await outputAudioElement.play()
            }
        }
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

    // Create output routing
    const audioContext = Tone.context.rawContext as AudioContext
    outputStreamDest = audioContext.createMediaStreamDestination()

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
    outputMerge.connect(outputStreamDest as any)

    // Create output audio element
    if (!outputAudioElement && outputStreamDest) {
        outputAudioElement = new Audio()
        outputAudioElement.srcObject = outputStreamDest.stream
        document.body.appendChild(outputAudioElement)

        // Start playback immediately
        outputAudioElement.play().catch(err => {
            console.warn('[Subgroup Output] Autoplay blocked, will start on first interaction:', err)
        })
    }

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

    const leftGainValue = Tone.dbToGain(leftVolume.value)
    const rightGainValue = Tone.dbToGain(rightVolume.value)

    leftGain.gain.value = leftGainValue
    rightGain.gain.value = rightGainValue
}

// Watch for volume changes
watch([leftVolume, rightVolume], updateSubgroupVolume)

// When linked, sync right to left
watch(leftVolume, (newVal) => {
    if (isLinked.value) {
        rightVolume.value = newVal
    }
})

// Link/unlink channels
function toggleLink() {
    isLinked.value = !isLinked.value
    if (isLinked.value) {
        rightVolume.value = leftVolume.value
    }
}

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
        leftVolume: leftVolume.value,
        rightVolume: rightVolume.value,
        isLinked: isLinked.value,
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

    if (snapshot.leftVolume !== undefined) leftVolume.value = snapshot.leftVolume
    if (snapshot.rightVolume !== undefined) rightVolume.value = snapshot.rightVolume
    if (snapshot.isLinked !== undefined) isLinked.value = snapshot.isLinked
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
        leftVolume.value = 0
        rightVolume.value = 0
        isLinked.value = true
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

    // Cleanup output audio element
    if (outputAudioElement) {
        outputAudioElement.pause()
        outputAudioElement.srcObject = null
        outputAudioElement.remove()
        outputAudioElement = null
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
