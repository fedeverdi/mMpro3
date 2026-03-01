<template>
    <div
        class="aux-master bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-teal-600/60 p-2 flex flex-col gap-2">
        <div class="text-left flex-shrink-0">
            <p class="text-[0.7rem] font-bold text-teal-200 tracking-wide uppercase">Auxiliary Buses</p>
            <p class="text-[0.5rem] text-gray-400">Click + to add aux send</p>
        </div>

        <div v-if="auxBuses.length === 0" class="flex-1 flex items-center justify-center text-gray-500 text-xs">
            No aux buses yet. Click + to add one.
        </div>

        <div v-else class="flex-1 grid gap-1.5 min-h-0 overflow-y-auto overflow-x-auto custom-scrollbar"
            style="grid-template-columns: repeat(6, 1fr); grid-auto-rows: max-content;">
            <div v-for="(aux, index) in auxBuses" :key="aux.id"
                class="relative flex flex-col gap-0.5 bg-gray-800/50 rounded-lg p-1.5 border border-teal-700/50">
                <!-- Remove button (hidden for now) -->
                <button v-if="false" @click="removeAux(index)"
                    class="absolute -top-2 -right-2 w-6 h-6 bg-gray-600/70 hover:bg-gray-500/80 rounded-full text-gray-300 hover:text-gray-100 font-bold flex items-center justify-center transition-colors shadow-lg border-2 border-gray-700"
                    style="z-index: 50;" title="Remove aux">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
                        <path fill-rule="evenodd"
                            d="M6.225 4.811a.75.75 0 011.06 0L12 9.525l4.715-4.714a.75.75 0 111.06 1.06L13.06 10.586l4.715 4.715a.75.75 0 11-1.06 1.06L12 11.646l-4.715 4.715a.75.75 0 11-1.06-1.06l4.715-4.715-4.715-4.715a.75.75 0 010-1.06z"
                            clip-rule="evenodd" />
                    </svg>
                </button>

                <!-- Aux name -->
                <div class="text-center -mt-2.5">
                    <input v-model="aux.name" @blur="updateAuxName(index, aux.name)"
                        class="bg-transparent text-[10px] font-bold text-teal-300 text-center border-b border-transparent hover:border-teal-500 focus:border-teal-500 outline-none w-full"
                        maxlength="10" />
                </div>

                <!-- Volume Knob -->
                <div class="flex items-center justify-center scale-[0.65] -my-5">
                    <Knob :modelValue="aux.volume" @update:modelValue="(val) => updateAuxVolume(index, val)" :min="-60"
                        :max="10" label="Vol" unit="dB" color="#14b8a6" />
                </div>

                <!-- FX Buttons -->
                <div class="grid grid-cols-2 gap-1">
                    <div :class="[
                        'flex items-center justify-center gap-0.5 px-1 py-0.5 rounded transition-colors overflow-hidden',
                        aux.reverbEnabled ? 'bg-green-600 text-white' : 'bg-gray-700 text-gray-400'
                    ]">
                        <button @click="toggleAuxReverb(index)" 
                            class="flex-1 text-[0.5rem] font-bold hover:opacity-80"
                            title="Toggle Reverb">
                            RV
                        </button>
                        <button v-if="aux.reverbEnabled" @click.stop="showReverbModal(index)" 
                            class="px-0.5 hover:bg-green-700 rounded"
                            title="Reverb Settings">
                            <svg class="w-2.5 h-2.5" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd"
                                    d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z"
                                    clip-rule="evenodd" />
                            </svg>
                        </button>
                    </div>
                    <div :class="[
                        'flex items-center justify-center gap-0.5 px-1 py-0.5 rounded transition-colors overflow-hidden',
                        aux.delayEnabled ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400'
                    ]">
                        <button @click="toggleAuxDelay(index)" 
                            class="flex-1 text-[0.5rem] font-bold hover:opacity-80"
                            title="Toggle Delay">
                            DL
                        </button>
                        <button v-if="aux.delayEnabled" @click.stop="showDelayModal(index)" 
                            class="px-0.5 hover:bg-blue-700 rounded"
                            title="Delay Settings">
                            <svg class="w-2.5 h-2.5" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd"
                                    d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z"
                                    clip-rule="evenodd" />
                            </svg>
                        </button>
                    </div>
                </div>

                <!-- Output Device Selector with Routing (unified modal) -->
                <div class="w-full bg-gray-900 rounded mt-0.5">
                    <AuxOutputSelector 
                        title="Aux Output & Routing" 
                        :devices="audioOutputDevices" 
                        :selected-device-id="aux.selectedOutputDevice || 'no-output'"
                        :route-to-master="auxRouting[index]?.toMaster || false"
                        :routed-subgroups="auxRouting[index]?.toSubgroups || new Set()"
                        :subgroups="subgroups"
                        @select-device="(deviceId) => selectOutputDevice(index, deviceId)"
                        @toggle-master-routing="toggleRouteToMaster(index)"
                        @toggle-subgroup-routing="(sgId) => toggleSubgroupRoute(index, sgId)"
                    />
                </div>

                <!-- Mute Button -->
                <button @click="toggleAuxMute(index)" :class="[
                    'w-full py-0.5 text-[0.45rem] font-bold rounded transition-colors',
                    aux.muted ? 'bg-yellow-600 text-white' : 'bg-gray-700 text-gray-400'
                ]">
                    M
                </button>
            </div>

            <!-- Add Aux button inside grid -->
            <button v-if="auxBuses.length < 6" @click="addAux"
                class="flex flex-col items-center justify-center border-2 border-dashed border-teal-600/50 rounded-lg hover:border-teal-500 hover:bg-teal-900/20 transition-colors text-teal-400 font-bold">
                <span class="text-2xl leading-none">+</span>
                <span class="text-[0.6rem] mt-1">ADD</span>
            </button>
        </div>

        <!-- Reverb FX Modal -->
        <Teleport to="body">
            <div v-if="selectedReverbAux !== null"
                class="fixed inset-0 bg-black/70 flex items-center justify-center z-[9999]"
                @mousedown.self="selectedReverbAux = null">
                <div class="bg-gray-900 rounded-lg border-2 border-green-600 p-6 max-w-md w-full mx-4" @click.stop>
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-lg font-bold text-green-300">
                            {{ auxBuses[selectedReverbAux]?.name }} - Reverb
                        </h3>
                        <button @click="selectedReverbAux = null"
                            class="text-gray-400 hover:text-white text-2xl">&times;</button>
                    </div>
                    <div class="flex flex-wrap gap-4 justify-center">
                        <Knob :modelValue="auxBuses[selectedReverbAux]?.reverbParams?.decay || 2.5"
                            @update:modelValue="(val) => updateAuxReverbParam(selectedReverbAux!, 'decay', val)"
                            :min="0.1" :max="10" :step="0.1" label="Decay" unit="s" color="#10b981" />
                        <Knob :modelValue="auxBuses[selectedReverbAux]?.reverbParams?.preDelay || 0.01"
                            @update:modelValue="(val) => updateAuxReverbParam(selectedReverbAux!, 'preDelay', val)"
                            :min="0" :max="0.1" :step="0.001" label="Pre-Delay" unit="s" color="#f59e0b" />
                        <Knob :modelValue="auxBuses[selectedReverbAux]?.reverbParams?.wet ?? 1.0"
                            @update:modelValue="(val) => updateAuxReverbParam(selectedReverbAux!, 'wet', val)" :min="0"
                            :max="1" :step="0.01" label="Wet" unit="%" color="#06b6d4" />
                    </div>
                    <div class="mt-4 text-xs text-gray-400 text-center">
                        <p><strong>Decay:</strong> Reverb tail length</p>
                        <p><strong>Pre-Delay:</strong> Time before reverb starts</p>
                        <p><strong>Wet:</strong> Effect amount (100% for aux send)</p>
                    </div>
                </div>
            </div>
        </Teleport>

        <!-- Delay FX Modal -->
        <Teleport to="body">
            <div v-if="selectedDelayAux !== null"
                class="fixed inset-0 bg-black/70 flex items-center justify-center z-[9999]"
                @mousedown.self="selectedDelayAux = null">
                <div class="bg-gray-900 rounded-lg border-2 border-blue-600 p-6 max-w-md w-full mx-4" @click.stop>
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-lg font-bold text-blue-300">
                            {{ auxBuses[selectedDelayAux]?.name }} - Delay
                        </h3>
                        <button @click="selectedDelayAux = null"
                            class="text-gray-400 hover:text-white text-2xl">&times;</button>
                    </div>
                    <div class="flex flex-wrap gap-4 justify-center">
                        <Knob :modelValue="auxBuses[selectedDelayAux]?.delayParams?.delayTime || 0.25"
                            @update:modelValue="(val) => updateAuxDelayParam(selectedDelayAux!, 'delayTime', val)"
                            :min="0.01" :max="2" :step="0.01" label="Time" unit="s" color="#3b82f6" />
                        <Knob :modelValue="auxBuses[selectedDelayAux]?.delayParams?.feedback || 0.3"
                            @update:modelValue="(val) => updateAuxDelayParam(selectedDelayAux!, 'feedback', val)"
                            :min="0" :max="0.95" :step="0.01" label="Feedback" unit="%" color="#8b5cf6" />
                        <Knob :modelValue="auxBuses[selectedDelayAux]?.delayParams?.wet !== undefined ? auxBuses[selectedDelayAux].delayParams.wet : 1.0"
                            @update:modelValue="(val) => updateAuxDelayParam(selectedDelayAux!, 'wet', val)" :min="0"
                            :max="1" :step="0.01" label="Wet" unit="%" color="#06b6d4" />
                    </div>
                    
                    <!-- Tap Tempo Button -->
                    <div class="mt-4 flex flex-col items-center gap-2">
                        <button @click="handleTapTempo"
                            class="px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg transition-all active:scale-95 shadow-lg">
                            TAP TEMPO
                        </button>
                        <div v-if="tapBpm !== null" class="text-sm text-blue-300 font-bold">
                            {{ tapBpm }} BPM ({{ (auxBuses[selectedDelayAux]?.delayParams?.delayTime || 0).toFixed(3) }}s)
                        </div>
                        <div v-else class="text-xs text-gray-500">
                            Click button repeatedly at your desired tempo
                        </div>
                    </div>
                    
                    <div class="mt-4 text-xs text-gray-400 text-center">
                        <p><strong>Time:</strong> Delay time (ms to seconds)</p>
                        <p><strong>Feedback:</strong> Number of repeats</p>
                        <p><strong>Wet:</strong> Effect amount (100% for aux send)</p>
                    </div>
                </div>
            </div>
        </Teleport>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, inject, toRaw, onMounted } from 'vue'
import Knob from '../core/Knob.vue'
import AuxOutputSelector from './AuxOutputSelector.vue'
import { useAudioDevices } from '~/composables/useAudioDevices'

interface AuxBus {
    id: string
    name: string
    volume: number
    muted: boolean
    soloed: boolean
    routeToMaster: boolean
    selectedOutputDevice?: string | null
    node?: any  // Input node
    outputNode?: any  // Output node (final node of FX chain)
    outputStreamDest?: MediaStreamAudioDestinationNode | null
    // FX Chain
    reverbNode?: any
    reverbEnabled?: boolean
    reverbParams?: { decay: number, preDelay: number, wet: number }
    delayNode?: any
    delayEnabled?: boolean
    delayParams?: { delayTime: number, feedback: number, wet: number }
}

interface Props {
    auxBuses?: AuxBus[]
    masterChannel?: any
    subgroups?: Array<{ id: number, name: string }>
}

const props = defineProps<Props>()
const emit = defineEmits<{
    (e: 'add-aux'): void
    (e: 'remove-aux', index: number): void
    (e: 'update-aux', index: number, aux: AuxBus): void
}>()

const { audioOutputDevices } = useAudioDevices()
const audioEngine = inject('audioEngine') as any

const selectedReverbAux = ref<number | null>(null)
const selectedDelayAux = ref<number | null>(null)
const auxBuses = ref<AuxBus[]>(props.auxBuses || [])

// Routing state for each aux
interface AuxRoutingState {
    toMaster: boolean
    toSubgroups: Set<number>
}
const auxRouting = ref<Record<number, AuxRoutingState>>({})

// Tap tempo state
const tapTimes = ref<number[]>([])
const tapBpm = ref<number | null>(null)

// Audio output devices are already enumerated during app initialization
// No need to enumerate them again here
onMounted(() => {
    // Initialization complete
})

// Watch for prop changes
watch(() => props.auxBuses, (newVal) => {
    if (newVal) {
        auxBuses.value = newVal
    }
}, { deep: true })

// Reset tap tempo when delay modal is closed
watch(selectedDelayAux, (newVal) => {
    if (newVal === null) {
        tapTimes.value = []
        tapBpm.value = null
    }
})

// Initialize routing state for each aux
watch(() => props.auxBuses, (newVal) => {
    if (newVal) {
        newVal.forEach((_, index) => {
            if (!auxRouting.value[index]) {
                auxRouting.value[index] = {
                    toMaster: false,
                    toSubgroups: new Set()
                }
            }
        })
    }
}, { immediate: true, deep: true })

// Routing functions
function toggleRouteToMaster(auxIndex: number) {
    const currentState = auxRouting.value[auxIndex]?.toMaster || false
    const newState = !currentState
    
    if (!auxRouting.value[auxIndex]) {
        auxRouting.value[auxIndex] = { toMaster: newState, toSubgroups: new Set() }
    } else {
        auxRouting.value[auxIndex].toMaster = newState
    }
    
    if (audioEngine) {
        audioEngine.setAuxBusRouteToMaster(auxIndex, newState)
    }
}

function toggleSubgroupRoute(auxIndex: number, subgroupId: number) {
    if (!auxRouting.value[auxIndex]) {
        auxRouting.value[auxIndex] = { toMaster: false, toSubgroups: new Set() }
    }
    
    const routes = auxRouting.value[auxIndex].toSubgroups
    const isRouted = routes.has(subgroupId)
    
    if (isRouted) {
        routes.delete(subgroupId)
    } else {
        routes.add(subgroupId)
    }
    
    if (audioEngine) {
        audioEngine.setAuxBusRouteToSubgroup(auxIndex, subgroupId, !isRouted)
    }
}

// Add new aux
function addAux() {
    emit('add-aux')
}

// Remove aux
function removeAux(index: number) {
    emit('remove-aux', index)
}

// Update aux volume
function updateAuxVolume(index: number, volume: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = { ...props.auxBuses[index], volume }
    if (aux.node) {
        aux.node.volume.value = volume
    }
    emit('update-aux', index, aux)
}

// Toggle mute
function toggleAuxMute(index: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = { ...props.auxBuses[index], muted: !props.auxBuses[index].muted }
    if (aux.node) {
        aux.node.mute = aux.muted
    }
    emit('update-aux', index, aux)
}

// Toggle solo
function toggleAuxSolo(index: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = { ...props.auxBuses[index], soloed: !props.auxBuses[index].soloed }
    emit('update-aux', index, aux)
}

// Update aux name
function updateAuxName(index: number, name: string) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = { ...props.auxBuses[index], name }
    emit('update-aux', index, aux)
}

// Select output device (Rust backend - zero latency like subgroups)
function selectOutputDevice(index: number, deviceId: string | null) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = { ...props.auxBuses[index], selectedOutputDevice: deviceId }
    emit('update-aux', index, aux)
}

// Toggle Reverb FX
function toggleAuxReverb(index: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = props.auxBuses[index]
    const newEnabled = !aux.reverbEnabled
    
    // Toggle wet parameter (restore saved value when enabling, 0 when disabling)
    if (aux.reverbNode) {
        if (newEnabled) {
            // Restore saved wet value (default 100% for professional aux bus setup)
            const savedWet = aux.reverbParams?.wet ?? 1.0
            aux.reverbNode.wet.value = savedWet
        } else {
            // Disable: set to 0
            aux.reverbNode.wet.value = 0
        }
    }
    
    // Update state
    const updatedAux = { ...aux, reverbEnabled: newEnabled }
    emit('update-aux', index, updatedAux)
}

// Toggle Delay FX
function toggleAuxDelay(index: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = props.auxBuses[index]
    const newEnabled = !aux.delayEnabled
    
    // Toggle wet parameter (restore saved value when enabling, 0 when disabling)
    if (aux.delayNode) {
        if (newEnabled) {
            // Restore saved wet value (default 100% for professional aux bus setup)
            const savedWet = aux.delayParams?.wet ?? 1.0
            aux.delayNode.wet.value = savedWet
        } else {
            // Disable: set to 0
            aux.delayNode.wet.value = 0
        }
    }
    
    // Update state
    const updatedAux = { ...aux, delayEnabled: newEnabled }
    emit('update-aux', index, updatedAux)
}

// NOTE: reconnectAuxChain is no longer needed - FX are always connected, controlled via wet parameter

// Show reverb modal
function showReverbModal(index: number) {
    selectedReverbAux.value = index
}

// Show delay modal
function showDelayModal(index: number) {
    selectedDelayAux.value = index
    const aux = auxBuses.value[index]
}

// Update single reverb parameter
function updateAuxReverbParam(index: number, param: 'decay' | 'preDelay' | 'wet', value: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = props.auxBuses[index]
    
    // Update the audio node directly (no emit = no lag)
    if (aux.reverbNode) {
        if (param === 'decay') {
            aux.reverbNode.decay = value
        } else if (param === 'preDelay') {
            aux.reverbNode.preDelay = value
        } else if (param === 'wet') {
            aux.reverbNode.wet.value = value
        }
    }
    
    // Update internal params object (for UI sync)
    if (!aux.reverbParams) {
        aux.reverbParams = { decay: 2.5, preDelay: 0.01, wet: 1.0 }
    }
    aux.reverbParams[param] = value
}

// Update single delay parameter
function updateAuxDelayParam(index: number, param: 'delayTime' | 'feedback' | 'wet', value: number) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = props.auxBuses[index]
    
    // Update the audio node directly (no emit = no lag)
    if (aux.delayNode) {
        if (param === 'delayTime') {
            aux.delayNode.delayTime.value = value
        } else if (param === 'feedback') {
            aux.delayNode.feedback.value = value
        } else if (param === 'wet') {
            aux.delayNode.wet.value = value
        }
    }
    
    // Update internal params object (for UI sync)
    if (!aux.delayParams) {
        aux.delayParams = { delayTime: 0.25, feedback: 0.3, wet: 1.0 }
    }
    aux.delayParams[param] = value
}

// Tap tempo function
function handleTapTempo() {
    if (selectedDelayAux.value === null) return
    
    const now = Date.now()
    tapTimes.value.push(now)
    
    // Keep only last 4 taps
    if (tapTimes.value.length > 4) {
        tapTimes.value.shift()
    }
    
    // Need at least 2 taps to calculate interval
    if (tapTimes.value.length >= 2) {
        // Calculate average interval between taps
        let totalInterval = 0
        for (let i = 1; i < tapTimes.value.length; i++) {
            totalInterval += tapTimes.value[i] - tapTimes.value[i - 1]
        }
        const avgInterval = totalInterval / (tapTimes.value.length - 1)
        
        // Convert to seconds and set delay time
        const delayTime = avgInterval / 1000
        
        // Calculate BPM for display (60000ms per minute / interval in ms)
        tapBpm.value = Math.round(60000 / avgInterval)
        
        // Update delay time
        updateAuxDelayParam(selectedDelayAux.value, 'delayTime', delayTime)
    }
    
    // Reset tap times after 2 seconds of inactivity
    setTimeout(() => {
        if (tapTimes.value.length > 0 && Date.now() - tapTimes.value[tapTimes.value.length - 1] >= 2000) {
            tapTimes.value = []
            tapBpm.value = null
        }
    }, 2000)
}

// Update Reverb parameters
function updateAuxReverbParams(index: number, params: { decay: number, preDelay: number, wet: number }) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = props.auxBuses[index]
    
    if (aux.reverbNode) {
        aux.reverbNode.decay = params.decay
        aux.reverbNode.preDelay = params.preDelay
        aux.reverbNode.wet.value = params.wet
    }
    
    const updatedAux = { ...aux, reverbParams: params }
    emit('update-aux', index, updatedAux)
}

// Update Delay parameters
function updateAuxDelayParams(index: number, params: { delayTime: number, feedback: number, wet: number }) {
    if (!props.auxBuses || !props.auxBuses[index]) return
    const aux = props.auxBuses[index]
    
    if (aux.delayNode) {
        aux.delayNode.delayTime.value = params.delayTime
        aux.delayNode.feedback.value = params.feedback
        aux.delayNode.wet.value = params.wet
    }
    
    const updatedAux = { ...aux, delayParams: params }
    emit('update-aux', index, updatedAux)
}
</script>

<style scoped>
/* Vertical scrollbar */
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
    height: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(20, 184, 166, 0.5);
    border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(20, 184, 166, 0.7);
}

/* Horizontal scrollbar - very minimal */
.custom-scrollbar::-webkit-scrollbar:horizontal {
    height: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:horizontal {
    background: rgba(20, 184, 166, 0.4);
    border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:horizontal:hover {
    background: rgba(20, 184, 166, 0.6);
}

/* Modal scrollbar */
.max-h-\[80vh\]::-webkit-scrollbar {
    width: 6px;
}

.max-h-\[80vh\]::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
}

.max-h-\[80vh\]::-webkit-scrollbar-thumb {
    background: rgba(20, 184, 166, 0.5);
    border-radius: 3px;
}

.max-h-\[80vh\]::-webkit-scrollbar-thumb:hover {
    background: rgba(20, 184, 166, 0.7);
}
</style>
