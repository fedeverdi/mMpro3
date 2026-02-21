<template>
    <div class="mixer-app min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-black flex flex-col">
        <!-- Header -->
        <header class="bg-black/50 backdrop-blur-sm border-b border-gray-700 px-4 py-2">
            <div class="flex items-center justify-between gap-4 flex-wrap">
                <div class="flex items-center gap-2">
                    <h1
                        class="text-xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
                        mMpro3
                    </h1>

                    <!-- Quick Scene Access -->
                    <template v-if="pinnedScenes.length > 0">
                        <div class="w-px h-6 bg-gray-600"></div>
                        <div class="flex gap-1 items-center">
                            <span class="text-[10px] text-gray-400 font-semibold uppercase">Quick scenes:</span>
                            <button v-for="scene in pinnedScenes" :key="scene.id" @click="handleLoadScene(scene.id)"
                                class="px-2 py-1 text-[0.5rem] rounded transition-colors uppercase" :class="scene.id === currentSceneId
                                    ? 'bg-green-600 hover:bg-green-500 text-white'
                                    : 'bg-yellow-600 hover:bg-yellow-500 text-white'"
                                :title="`Load scene: ${scene.name}`">
                                {{ scene.name }}
                            </button>
                        </div>
                    </template>
                </div>

                <div class="flex gap-2 items-center flex-wrap">
                    <button @click="showAudioFlowModal = true"
                        class="px-3 py-1 bg-purple-600 hover:bg-purple-500 rounded text-xs font-semibold transition-colors flex items-center gap-1">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                        </svg>
                        Signal Flow
                    </button>

                    <button @click="showScenesModal = true"
                        class="px-3 py-1 bg-green-600 hover:bg-green-500 rounded text-xs font-semibold transition-colors flex items-center gap-1">
                        🎬
                        Scenes
                    </button>

                    <button @click="handleClearScene"
                        class="px-3 py-1 bg-orange-600 hover:bg-orange-500 rounded text-xs font-semibold transition-colors"
                        title="Clear mixer - Reload page">
                        Clear
                    </button>

                    <div class="w-px h-6 bg-gray-600"></div>

                    <div class="relative -mt-[3px]">
                        <button @click="showAddTrackMenu = !showAddTrackMenu" :disabled="tracks.length >= 24"
                            class="px-3 h-full py-1 bg-blue-600 hover:bg-blue-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed rounded text-xs font-semibold transition-colors">
                            + Add
                        </button>

                        <!-- Dropdown Menu -->
                        <div v-if="showAddTrackMenu"
                            class="absolute top-full left-0 mt-1 w-32 bg-gray-800 border border-gray-600 rounded shadow-lg z-50">
                            <button @click="addTrackOfType('audio')"
                                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2">
                                🎵 Audio Track
                            </button>
                            <button @click="addTrackOfType('signal')"
                                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2">
                                📡 Signal Track
                            </button>
                        </div>
                    </div>

                    <button @click="removeTrack" :disabled="tracks.length <= 1"
                        class="px-3 py-1 bg-red-600 hover:bg-red-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed rounded text-xs font-semibold transition-colors">
                        - Remove
                    </button>

                    <div class="text-xs text-gray-400">
                        {{ tracks.length }}/24
                    </div>
                </div>
            </div>
        </header>

        <!-- Mixer Console -->
        <main class="flex-1 flex gap-2 p-2 overflow-hidden">
            <!-- Audio Tracks Section (flexible) -->
            <div class="tracks-scroll-wrap flex-1 overflow-hidden min-w-0 pb-[2px]">
                <div class="tracks-scroll overflow-x-auto overflow-y-hidden h-full">
                    <div class="flex gap-2 h-full min-w-max">
                        <!-- Loading Skeletons -->
                        <template v-if="!isReady">
                            <div v-for="n in 24" :key="'skeleton-' + n" class="w-[8rem] h-full">
                                <div
                                    class="bg-gray-800 rounded-lg border border-gray-700 p-1 h-full flex flex-col gap-2">
                                    <!-- Header -->
                                    <div class="h-4 bg-gray-700 rounded animate-pulse"></div>
                                    <!-- Controls -->
                                    <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="flex gap-1">
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                    </div>
                                    <!-- Knob area -->
                                    <div class="flex-1 flex items-center justify-center">
                                        <div class="w-16 h-16 bg-gray-700 rounded-full animate-pulse"></div>
                                    </div>
                                    <!-- Fader area -->
                                    <div class="flex-1 flex flex-col items-center justify-center gap-2 py-4">
                                        <!-- VU Meter skeleton -->
                                        <div class="w-12 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                        <!-- Fader skeleton -->
                                        <div class="w-8 h-32 bg-gray-700 rounded animate-pulse"></div>
                                    </div>
                                    <!-- Bottom controls -->
                                    <div class="flex gap-1">
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                        <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                    </div>
                                </div>
                            </div>
                        </template>

                        <!-- Audio Tracks -->
                        <template v-else>
                            <div v-for="track in tracks" :key="track.id" class="w-[8.5rem] h-full mixer-fade-in">
                                <SignalTrack v-if="track.type === 'signal'" :ref="el => setTrackRef(track.id, el)"
                                    :trackNumber="track.id" :master-channel="masterChannel"
                                    :subgroup-channel="subgroupChannel" @soloChange="handleSoloChange"
                                    @levelUpdate="handleLevelUpdate" />
                                <AudioTrack v-else :ref="el => setTrackRef(track.id, el)" :trackNumber="track.id"
                                    :master-channel="masterChannel" :subgroup-channel="subgroupChannel"
                                    @soloChange="handleSoloChange" @levelUpdate="handleLevelUpdate" />
                            </div>
                        </template>
                    </div>
                </div>
            </div>

            <!-- Right Section (fixed width) -->
            <div class="flex gap-2 flex-shrink-0">
                <!-- Loading Skeletons for Right Section -->
                <template v-if="!isReady">
                    <!-- Master EQ/Spectrum/FX Skeleton -->
                    <div class="flex flex-col h-full gap-2">
                        <div class="w-[36rem] flex flex-col flex-1 min-h-0 gap-2">
                            <div class="flex-1 min-h-0 bg-gray-800 rounded-lg border border-gray-700 animate-pulse"></div>
                            <div class="flex-1 min-h-0 bg-gray-800 rounded-lg border border-gray-700 animate-pulse"></div>
                        </div>
                        <div class="w-[36rem] bg-gray-800 rounded-lg border border-gray-700 p-4 animate-pulse" style="height: 200px;"></div>
                    </div>

                    <!-- Subgroups Skeleton -->
                    <div class="w-32 h-full">
                        <div class="bg-gray-800 rounded-lg border border-gray-700 h-full p-2 flex flex-col gap-2">
                            <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                            <div class="flex-1 flex items-center justify-center py-4">
                                <div class="flex gap-2">
                                    <div class="w-5 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="w-5 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                </div>
                            </div>
                            <div class="h-8 bg-gray-700 rounded animate-pulse"></div>
                        </div>
                    </div>

                    <!-- Master Skeleton -->
                    <div class="w-44 h-full">
                        <div class="bg-gray-800 rounded-lg border border-gray-700 h-full p-2 flex flex-col gap-2">
                            <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                            <div class="flex-1 flex items-center justify-center py-4">
                                <div class="flex gap-2">
                                    <div class="w-8 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="w-8 flex-1 bg-gray-700 rounded animate-pulse"></div>
                                </div>
                            </div>
                            <div class="h-8 bg-gray-700 rounded animate-pulse"></div>
                        </div>
                    </div>
                </template>

                <!-- Master EQ Display, Spectrum & FX (Draggable) -->
                <template v-else>
                    <div class="flex flex-col h-full gap-2">
                        <template v-for="component in rightSectionComponents" :key="component.id">
                            <!-- Master EQ Display -->
                            <div v-if="component.id === 'eq'"
                                :class="[component.size === 'flex' ? 'flex-1 min-h-0' : '', 'w-[36rem] mixer-fade-in relative group']"
                                draggable="true"
                                @dragstart="handleDragStart(component.id)"
                                @dragover="handleDragOver"
                                @drop="handleDrop($event, component.id)"
                                @dragend="handleDragEnd"
                                :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
                                <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <div class="bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1">
                                        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
                                            <path d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z"/>
                                        </svg>
                                        <span>{{ component.name }}</span>
                                    </div>
                                </div>
                                <MasterEQDisplay :filters-data="masterEqFiltersData"
                                    :master-channel="masterChannel"
                                    @update:filters-data="handleMasterEQFiltersUpdate"
                                    @output-node="handleMasterEqOutputNode" />
                            </div>
                            
                            <!-- Spectrum Meter -->
                            <div v-if="component.id === 'spectrum'"
                                :class="[component.size === 'flex' ? 'flex-1 min-h-0' : '', 'w-[36rem] mixer-fade-in relative group']"
                                draggable="true"
                                @dragstart="handleDragStart(component.id)"
                                @dragover="handleDragOver"
                                @drop="handleDrop($event, component.id)"
                                @dragend="handleDragEnd"
                                :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
                                <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <div class="bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1">
                                        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
                                            <path d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z"/>
                                        </svg>
                                        <span>{{ component.name }}</span>
                                    </div>
                                </div>
                                <SpectrumMeter :master-fx-output-node="masterFxOutputNode" />
                            </div>
                            
                            <!-- Master FX -->
                            <div v-if="component.id === 'fx'"
                                class="w-[36rem] mixer-fade-in relative group"
                                draggable="true"
                                @dragstart="handleDragStart(component.id)"
                                @dragover="handleDragOver"
                                @drop="handleDrop($event, component.id)"
                                @dragend="handleDragEnd"
                                :style="{ cursor: draggedComponent ? 'grabbing' : 'grab' }">
                                <div class="absolute top-2 left-2 z-10 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <div class="bg-gray-900/90 backdrop-blur-sm px-2 py-1 rounded text-xs text-gray-400 flex items-center gap-1">
                                        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
                                            <path d="M9 3h2v2H9V3zm4 0h2v2h-2V3zM9 7h2v2H9V7zm4 0h2v2h-2V7zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2zm-4 4h2v2H9v-2zm4 0h2v2h-2v-2z"/>
                                        </svg>
                                        <span>{{ component.name }}</span>
                                    </div>
                                </div>
                                <MasterFX :master-eq-output-node="masterEqOutputNode"
                                    :master-section="masterSectionRef"
                                    @output-node="handleMasterFxOutputNode"
                                    @component="handleMasterFxComponent" />
                            </div>
                        </template>
                    </div>

                    <!-- Subgroups Section -->
                    <div class="flex-shrink-0 h-full mixer-fade-in">
                        <SubgroupsSection ref="subgroupsSectionRef" />
                    </div>

                    <!-- Master Section -->
                    <div class="flex-shrink-0 h-full mixer-fade-in">
                        <MasterSection ref="masterSectionRef" :master-fx-output-node="masterFxOutputNode"
                            :master-fx-component="masterFxComponent" :loaded-tracks="loadedTracks" />
                    </div>
                </template>
            </div>
        </main>

        <!-- Footer Info -->
        <footer class="bg-black/50 backdrop-blur-sm border-t border-gray-700 px-6 py-3">
            <div class="flex justify-between items-center text-xs text-gray-500">
                <div>
                    Built with Nuxt 3, Tone.js & Tailwind CSS
                </div>
                <div>
                    Sample Rate: {{ sampleRate }}Hz | Buffer Size: {{ bufferSize }}
                </div>
            </div>
        </footer>

        <!-- Audio Flow Modal -->
        <AudioFlowModal v-model="showAudioFlowModal" />

        <!-- Scenes Modal -->
        <ScenesModal v-model="showScenesModal" :scenes="scenes" :current-scene-id="currentSceneId"
            @save="handleSaveScene" @load="handleLoadScene" @update="handleUpdateScene" @delete="handleDeleteScene"
            @rename="handleRenameScene" @toggle-pin="handleTogglePin" />

        <!-- Scene Loading Overlay -->
        <Transition enter-from-class="opacity-0 scale-90 -translate-y-12"
            enter-active-class="transition-all duration-500 ease-out"
            enter-to-class="opacity-100 scale-100 translate-y-0" leave-from-class="opacity-100 scale-100 translate-y-0"
            leave-active-class="transition-all duration-300 ease-in" leave-to-class="opacity-0 scale-90 -translate-y-8">
            <div v-if="isLoadingScene" class="fixed inset-0 flex items-start justify-center pt-20 z-[9999]">
                <div
                    class="bg-gradient-to-br from-gray-600 to-gray-700 border-2 border-blue-500/70 rounded-lg shadow-2xl px-6 py-3 flex items-center gap-3 whitespace-nowrap">
                    <div class="relative w-5 h-5">
                        <div class="absolute inset-0 border-2 border-blue-500/30 rounded-full"></div>
                        <div
                            class="absolute inset-0 border-2 border-transparent border-t-blue-500 rounded-full animate-spin">
                        </div>
                    </div>
                    <span class="text-sm font-semibold text-white">Loading Scene</span>
                    <div class="flex gap-1 pt-2">
                        <div class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 0ms">
                        </div>
                        <div class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 150ms">
                        </div>
                        <div class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 300ms">
                        </div>
                    </div>
                </div>
            </div>
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, toRaw, nextTick, inject, watch } from 'vue'
import AudioTrack from './components/AudioTrack.vue'
import SignalTrack from './components/SignalTrack.vue'
import AudioFlowModal from './components/layout/AudioFlowModal.vue'
import MasterEQDisplay from './components/master/MasterEQDisplay.vue'
import MasterFX from './components/master/MasterFX.vue'
import MasterSection from './components/MasterSection.vue'
import SubgroupsSection from './components/SubgroupsSection.vue'
import ScenesModal from './components/layout/ScenesModal.vue'
import SpectrumMeter from './components/master/SpectrumMeter.vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useScenes, type Scene, type TrackSnapshot } from '~/composables/useScenes'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'
import { channel } from 'diagnostics_channel'

const ToneRef = inject<any>('Tone')
let Tone: any = null
const toneReady = ref(false)
const masterChannel = ref<any>(null)
const subgroupChannel = ref<any>(null)

interface Track {
    id: number
    type: 'audio' | 'signal'
}

// App ready state
const isReady = ref(false)

// Audio Flow Modal
const showAudioFlowModal = ref(false)
const showScenesModal = ref(false)
const isLoadingScene = ref(false)

// Tracks management
const tracks = ref<Track[]>([
    { id: 1, type: 'audio' },
    { id: 2, type: 'audio' },
    { id: 3, type: 'audio' },
    { id: 4, type: 'audio' },
    { id: 5, type: 'audio' },
    { id: 6, type: 'audio' },
    { id: 7, type: 'signal' }
])

const showAddTrackMenu = ref(false)

function getNextAvailableId(): number {
    // Find the smallest available ID from 1 to 24
    for (let i = 1; i <= 24; i++) {
        if (!tracks.value.find(t => t.id === i)) {
            return i
        }
    }
    // If all 1-24 are taken, return the next number
    return Math.max(...tracks.value.map(t => t.id)) + 1
}

function addTrackOfType(type: 'audio' | 'signal') {
    if (tracks.value.length >= 24) return
    tracks.value.push({ id: getNextAvailableId(), type })
    showAddTrackMenu.value = false
}

function removeTrack() {
    if (tracks.value.length <= 1) return
    const removedTrack = tracks.value.pop()

    // Remove the track ref from the map
    if (removedTrack) {
        trackRefs.value.delete(removedTrack.id)
        // Also remove from solo tracks if it was soloed
        soloTracks.value.delete(removedTrack.id)
    }
}

// Track refs management (only for tracks, not for master components)
const trackRefs = ref<Map<number, any>>(new Map())
const masterSectionRef = ref<any>(null) // Keep only for getSnapshot/restoreSnapshot
const subgroupsSectionRef = ref<any>(null)

// Audio nodes received from components via emit
const masterEqOutputNode = ref<any>(null)
const masterFxOutputNode = ref<any>(null)
const masterFxComponent = ref<any>(null) // For getSnapshot only

// Handlers for output node updates
function handleMasterEqOutputNode(node: any) {
    masterEqOutputNode.value = node
}

function handleMasterFxOutputNode(node: any) {
    masterFxOutputNode.value = node
}

function handleMasterFxComponent(component: any) {
    masterFxComponent.value = component
}

function setTrackRef(trackId: number, el: any | null) {
    if (el) {
        trackRefs.value.set(trackId, el)
    } else {
        // Remove ref when component is unmounted
        trackRefs.value.delete(trackId)
    }
}

// Master EQ filters data
const masterEqFiltersData = ref<any[]>([])

// Handle master EQ filters update from MasterEQDisplay
function handleMasterEQFiltersUpdate(filters: any[]) {
    masterEqFiltersData.value = filters
}

// Right section components order (draggable)
interface RightSectionComponent {
    id: string
    name: string
    size: 'flex' | 'fixed'
}

const rightSectionComponents = ref<RightSectionComponent[]>([
    { id: 'eq', name: 'Master EQ', size: 'flex' },
    { id: 'spectrum', name: 'Spectrum', size: 'flex' },
    { id: 'fx', name: 'Master FX', size: 'fixed' }
])

const draggedComponent = ref<string | null>(null)

function handleDragStart(componentId: string) {
    draggedComponent.value = componentId
}

function handleDragOver(event: DragEvent) {
    event.preventDefault()
}

function handleDrop(event: DragEvent, targetComponentId: string) {
    event.preventDefault()
    
    if (!draggedComponent.value || draggedComponent.value === targetComponentId) {
        return
    }
    
    const draggedIndex = rightSectionComponents.value.findIndex(c => c.id === draggedComponent.value)
    const targetIndex = rightSectionComponents.value.findIndex(c => c.id === targetComponentId)
    
    if (draggedIndex !== -1 && targetIndex !== -1) {
        const components = [...rightSectionComponents.value]
        const [removed] = components.splice(draggedIndex, 1)
        components.splice(targetIndex, 0, removed)
        rightSectionComponents.value = components
    }
    
    draggedComponent.value = null
}

function handleDragEnd() {
    draggedComponent.value = null
}

// Load components order from localStorage
function loadComponentsOrder() {
    try {
        const saved = localStorage.getItem('rightSectionComponentsOrder')
        if (saved) {
            const order = JSON.parse(saved)
            // Validate that all required components are present
            const requiredIds = ['eq', 'spectrum', 'fx']
            const savedIds = order.map((c: RightSectionComponent) => c.id)
            
            if (requiredIds.every(id => savedIds.includes(id))) {
                rightSectionComponents.value = order
            }
        }
    } catch (err) {
        console.warn('Failed to load components order:', err)
    }
}

// Save components order to localStorage
function saveComponentsOrder() {
    try {
        localStorage.setItem('rightSectionComponentsOrder', JSON.stringify(rightSectionComponents.value))
    } catch (err) {
        console.warn('Failed to save components order:', err)
    }
}

// Watch for changes and save automatically
watch(rightSectionComponents, () => {
    saveComponentsOrder()
}, { deep: true })

// Loaded tracks for recorder waveform display
const loadedTracks = computed(() => {
    const tracks: Array<{
        trackNumber: number,
        fileName: string,
        fileId: string,
        isPlaying: boolean,
        currentTime: number,
        duration: number
    }> = []
    trackRefs.value.forEach((trackRef, trackId) => {
        if (trackRef && typeof trackRef.getSnapshot === 'function') {
            const snapshot = trackRef.getSnapshot()
            // Only show tracks that are playing AND are file-based
            if (snapshot.sourceType === 'file' && snapshot.fileName && snapshot.fileId && snapshot.isPlaying) {
                tracks.push({
                    trackNumber: snapshot.trackNumber,
                    fileName: snapshot.fileName,
                    fileId: snapshot.fileId,
                    isPlaying: snapshot.isPlaying,
                    currentTime: snapshot.currentTime || 0,
                    duration: snapshot.duration || 0
                })
            }
        }
    })
    return tracks.sort((a, b) => a.trackNumber - b.trackNumber)
})

// Solo handling
const soloTracks = ref<Set<number>>(new Set())

function handleSoloChange(data: { trackNumber: number, isSolo: boolean }) {
    if (data.isSolo) {
        soloTracks.value.add(data.trackNumber)
    } else {
        soloTracks.value.delete(data.trackNumber)
    }

    // Update all tracks based on solo state
    trackRefs.value.forEach((trackRef, trackId) => {
        if (soloTracks.value.size > 0) {
            // If any track is soloed, mute all others
            const shouldBeMuted = !soloTracks.value.has(trackId)
            trackRef.setMuted(shouldBeMuted && !trackRef.isSolo())
        } else {
            // If no tracks are soloed, unmute all
            trackRef.setMuted(false)
        }
    })
}

// Level updates (for future visualizations)
function handleLevelUpdate(data: { trackNumber: number, level: number }) {
    // Can be used for additional visualizations if needed
}

// Audio context info (reactive)
const sampleRate = computed(() => {
    if (toneReady.value && Tone?.context?.sampleRate) {
        return Math.round(Tone.context.sampleRate)
    }
    return 0
})
const bufferSize = computed(() => {
    if (toneReady.value && Tone?.context?.rawContext) {
        const audioContext = Tone.context.rawContext

        return audioContext.baseLatency
            ? Math.round(audioContext.baseLatency * audioContext.sampleRate)
            : (audioContext.sampleRate ? 128 : 0) // Default buffer size estimate
    }
    return 0
})

// Audio devices enumeration (shared across all tracks)
const { enumerateAudioInputs } = useAudioDevices()

// Scene management
const {
    scenes,
    currentSceneId,
    loadScenesFromStorage,
    createScene,
    updateScene,
    deleteScene,
    renameScene,
    setCurrentScene,
    togglePinScene
} = useScenes()

const { deleteAudioFile } = useAudioFileStorage()

async function handleSaveScene(sceneName: string) {
    // Collect snapshots from all tracks
    const trackSnapshots: TrackSnapshot[] = []
    tracks.value.forEach(track => {
        const trackRef = trackRefs.value.get(track.id)
        if (trackRef && trackRef.getSnapshot) {
            trackSnapshots.push(trackRef.getSnapshot())
        }
    })

    // Collect master snapshot from MasterSection
    const masterSectionSnapshot = masterSectionRef.value?.getSnapshot?.() || {
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: 0,
        isLinked: true,
        compressorEnabled: false,
        reverbEnabled: false,
        delayEnabled: false,
        limiterEnabled: false
    }

    const masterSnapshot = {
        ...masterSectionSnapshot,
        masterEQFilters: masterEqFiltersData.value.map((filter: any) => ({
            type: filter.type,
            frequency: filter.frequency,
            gain: filter.gain,
            Q: filter.Q,
            color: filter.color
        }))
    }

    // Create and save scene
    const scene = await createScene(sceneName, trackSnapshots, masterSnapshot)
    setCurrentScene(scene.id)
}

function handleLoadScene(sceneId: string) {
    const scene = scenes.value.find((s: Scene) => s.id === sceneId)
    if (!scene) return

    // Close the modal first
    showScenesModal.value = false

    // Show loading overlay
    isLoadingScene.value = true

    // Small delay before starting the animation
    setTimeout(() => {
        // Reset all tracks to defaults before loading scene
        trackRefs.value.forEach((trackRef) => {
            if (trackRef && trackRef.resetToDefaults) {
                trackRef.resetToDefaults()
            }
        })

        // Reset master section to defaults
        if (masterSectionRef.value?.resetToDefaults) {
            masterSectionRef.value.resetToDefaults()
        }

        // Clear master EQ filters
        masterEqFiltersData.value = []

        // Animate faders to -∞ (mute) first (digital mixer effect)
        trackRefs.value.forEach((trackRef) => {
            if (trackRef && trackRef.getSnapshot) {
                const snapshot = trackRef.getSnapshot()
                // Create a temporary snapshot with volume at -90 (-∞ / muted)
                const muteSnapshot = { ...snapshot, volume: -90 }
                trackRef.restoreFromSnapshot(muteSnapshot)
            }
        })

        // Wait before restoring actual values (give time to see the animation)
        setTimeout(() => {
            // Restore each track's state
            scene.tracks.forEach((trackSnapshot: TrackSnapshot) => {
                const trackRef = trackRefs.value.get(trackSnapshot.trackNumber)
                if (trackRef && trackRef.restoreFromSnapshot) {
                    trackRef.restoreFromSnapshot(trackSnapshot)
                }
            })

            // Restore master EQ filters
            if (scene.master.masterEQFilters && scene.master.masterEQFilters.length > 0) {
                masterEqFiltersData.value = scene.master.masterEQFilters.map((filter: any) => ({
                    type: filter.type,
                    frequency: filter.frequency,
                    gain: filter.gain,
                    Q: filter.Q,
                    color: filter.color || '#3b82f6'
                }))
            } else {
                masterEqFiltersData.value = []
            }

            // Restore master section state (volumes, FX)
            if (masterSectionRef.value?.restoreSnapshot) {
                masterSectionRef.value.restoreSnapshot(scene.master)
            }

            // Set as current scene
            setCurrentScene(scene.id)

            // Hide loading overlay after scene is restored and audio files have time to load
            setTimeout(() => {
                isLoadingScene.value = false
            }, 1200) // Extra time for audio files to load silently
        }, 600) // Wait 0.6 seconds at -∞ before restoring values
    }, 200) // Initial delay after closing modal
}

async function handleUpdateScene(sceneId: string) {
    // Get old scene to clean up orphaned files
    const oldScene = scenes.value.find((s: Scene) => s.id === sceneId)

    // Collect current state from tracks
    const trackSnapshots: TrackSnapshot[] = []
    tracks.value.forEach(track => {
        const trackRef = trackRefs.value.get(track.id)
        if (trackRef && trackRef.getSnapshot) {
            trackSnapshots.push(trackRef.getSnapshot())
        }
    })

    // Collect master snapshot from MasterSection
    const masterSectionSnapshot = masterSectionRef.value?.getSnapshot?.() || {
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: 0,
        isLinked: true,
        compressorEnabled: false,
        reverbEnabled: false,
        delayEnabled: false,
        limiterEnabled: false
    }

    const masterSnapshot = {
        ...masterSectionSnapshot,
        masterEQFilters: masterEqFiltersData.value.map((filter: any) => ({
            type: filter.type,
            frequency: filter.frequency,
            gain: filter.gain,
            Q: filter.Q,
            color: filter.color
        }))
    }

    // Update scene
    await updateScene(sceneId, trackSnapshots, masterSnapshot)

    // Clean up old audio files that are no longer in the scene
    if (oldScene) {
        const oldFileIds = new Set(
            oldScene.tracks
                .filter((t: TrackSnapshot) => t.fileId)
                .map((t: TrackSnapshot) => t.fileId!)
        )
        const newFileIds = new Set(
            trackSnapshots
                .filter(t => t.fileId)
                .map(t => t.fileId!)
        )

        // Delete files that were in old scene but not in new scene
        const filesToDelete = Array.from(oldFileIds).filter(id => !newFileIds.has(id))

        for (const fileId of filesToDelete) {
            try {
                await deleteAudioFile(fileId)
            } catch (err) {
                console.warn(`Failed to delete orphaned file ${fileId}:`, err)
            }
        }
    }
}

async function handleDeleteScene(sceneId: string) {
    // Find scene to get file IDs before deletion
    const scene = scenes.value.find((s: Scene) => s.id === sceneId)

    if (scene) {
        // Delete all audio files associated with this scene from IndexedDB
        const fileDeletePromises: Promise<void>[] = []

        scene.tracks.forEach((track: TrackSnapshot) => {
            if (track.fileId) {
                fileDeletePromises.push(
                    deleteAudioFile(track.fileId).catch((err: unknown) => {
                        console.warn(`Failed to delete file ${track.fileId}:`, err)
                    })
                )
            }
        })

        // Wait for all file deletions to complete
        await Promise.all(fileDeletePromises)
    }

    // Delete scene from storage (this removes from both scenes array and IndexedDB)
    await deleteScene(sceneId)

    // If deleted scene was current, clear current
    if (currentSceneId.value === sceneId) {
        setCurrentScene(null)
    }
}

async function handleRenameScene(sceneId: string, newName: string) {
    await renameScene(sceneId, newName)
}

async function handleTogglePin(sceneId: string) {
    await togglePinScene(sceneId)
}

function handleClearScene() {
    // Reload the page to clear all mixer state
    window.location.reload()
}

// Computed for pinned scenes
const pinnedScenes = computed(() => {
    return scenes.value.filter(scene => scene.pinned)
})

// Initialize audio
onMounted(async () => {
    document.title = 'Audio Mixer Pro - Multi-Track Mixer'

    // Load right section components order from localStorage
    loadComponentsOrder()

    // Load scenes from IndexedDB
    await loadScenesFromStorage()

    // Get Tone.js from inject
    if (ToneRef?.value) {
        Tone = ToneRef.value
    } else {
        // Fallback: wait for it
        await new Promise<void>((resolve) => {
            const checkTone = setInterval(() => {
                if (ToneRef?.value) {
                    Tone = ToneRef.value
                    clearInterval(checkTone)
                    resolve()
                }
            }, 100)
        })
    }

    // Mark Tone as ready immediately after import
    toneReady.value = true

    // Use Gain instead of Channel to preserve stereo
    // Tone.Channel converts stereo to mono!
    masterChannel.value = new Tone.Channel({
        volume: 0,
        pan: 0,
        channelCount: 2,
        channelCountMode: 'explicit',
        channelInterpretation: 'speakers'
    })

    // NOTE: masterChannel will be connected by MasterEQDisplay component
    console.log('[Audio] Master channel created (will be connected to EQ)')

    // Create subgroup channel (similar to master)
    subgroupChannel.value = new Tone.Channel({
        volume: 0,
        pan: 0,
        channelCount: 2,
        channelCountMode: 'explicit',
        channelInterpretation: 'speakers'
    })

    // Connect subgroup to SubgroupsSection input (with retry)
    let retryCount = 0
    const maxRetries = 10
    const connectSubgroup = () => {
        if (subgroupsSectionRef.value?.getInputNode) {
            const subgroupInput = subgroupsSectionRef.value.getInputNode()

            if (subgroupInput) {
                const rawSubgroupChannel = toRaw(subgroupChannel.value)
                const rawInputNode = toRaw(subgroupInput)
                rawSubgroupChannel.connect(rawInputNode)
                return true
            } else {
                console.error('[Subgroup] ✗ Input node is null')
            }
        } else {
            console.error('[Subgroup] ✗ SubgroupsSection not ready')
        }

        // Retry if not successful
        retryCount++
        if (retryCount < maxRetries) {
            setTimeout(connectSubgroup, 200)
        } else {
            console.error('[Subgroup] ✗ Failed to connect after', maxRetries, 'attempts')
        }
        return false
    }

    // Don't start connection here - wait for component to mount

    // Ensure audio context is running
    if (Tone.context.state !== 'running') {
        await Tone.context.resume()
    }

    // Enumerate audio devices ONCE for all tracks
    await enumerateAudioInputs()

    // Wait for next tick to ensure all components are ready
    await nextTick()

    // Delay to ensure all components are fully mounted and initialized
    setTimeout(() => {
        isReady.value = true

        // Connect subgroup AFTER components are mounted
        setTimeout(connectSubgroup, 500)
    }, 100)

    // Close add track menu when clicking outside
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement
        if (!target.closest('.relative')) {
            showAddTrackMenu.value = false
        }
    })
})
</script>

<style scoped>
.mixer-app {
    height: 100vh;
    overflow: hidden;
}

.mixer-fade-in {
    animation: fadeIn 0.2s ease-in;
}

@keyframes fadeIn {
    from {
        opacity: 0;
    }

    to {
        opacity: 1;
    }
}

/* Custom scrollbar */
::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

::-webkit-scrollbar-track {
    background: #1f2937;
}

::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
}

/* Tracks section horizontal scrollbar - thin 4px */
.tracks-scroll::-webkit-scrollbar {
    height: 4px;
}

.tracks-scroll::-webkit-scrollbar-track {
    background: rgba(31, 41, 55, 0.3);
    border-radius: 2px;
}

.tracks-scroll::-webkit-scrollbar-thumb {
    background: rgba(59, 130, 246, 0.5);
    border-radius: 2px;
}

.tracks-scroll::-webkit-scrollbar-thumb:hover {
    background: rgba(96, 165, 250, 0.8);
}

.tracks-scroll-wrap {
    position: relative;
}

.tracks-scroll-wrap::after {
    content: '';
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 36px;
    pointer-events: none;
    z-index: 5;
    background: linear-gradient(to left, rgba(0, 0, 0, 0.95), rgba(0, 0, 0, 0));
}

/* Draggable components */
[draggable="true"] {
    transition: all 0.3s ease;
    border-radius: 0.5rem;
}

[draggable="true"]:hover {
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

[draggable="true"]:active {
    opacity: 0.7;
    transform: scale(0.98);
}
</style>
