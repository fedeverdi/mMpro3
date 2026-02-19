<template>
    <div class="mixer-app min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-black flex flex-col">
        <!-- Header -->
        <header class="bg-black/50 backdrop-blur-sm border-b border-gray-700 px-4 py-2">
            <div class="flex items-center justify-between">
                <div>
                    <h1
                        class="text-xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
                        Audio Mixer Pro
                    </h1>
                </div>

                <div class="flex gap-2 items-center">
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

                    <div class="w-px h-6 bg-gray-600"></div>

                    <button @click="addTrack" :disabled="tracks.length >= 24"
                        class="px-3 py-1 bg-blue-600 hover:bg-blue-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed rounded text-xs font-semibold transition-colors">
                        + Add
                    </button>

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
                            <div v-for="track in tracks" :key="track.id" class="w-[8rem] h-full mixer-fade-in">
                                <AudioTrack :ref="el => setTrackRef(track.id, el)" :trackNumber="track.id"
                                    :master-channel="masterChannel" @soloChange="handleSoloChange"
                                    @levelUpdate="handleLevelUpdate" />
                            </div>
                        </template>
                    </div>
                </div>
            </div>

            <!-- Right Section (fixed width) -->
            <div class="flex gap-2 flex-shrink-0">
                <!-- Loading Skeleton for Right Section -->
                <template v-if="!isReady">
                    <div class="w-[32rem] flex flex-col h-full gap-2">
                        <div class="flex-[2] bg-gray-800 rounded-lg border border-gray-700 animate-pulse"></div>
                        <div class="flex-[1.5] bg-gray-800 rounded-lg border border-gray-700 animate-pulse"></div>
                        <div class="flex-1 bg-gray-800 rounded-lg border border-gray-700 animate-pulse"></div>
                    </div>
                    <div class="w-44 h-full">
                        <div class="bg-gray-800 rounded-lg border border-gray-700 h-full p-2 flex flex-col gap-2">
                            <!-- Header -->
                            <div class="h-6 bg-gray-700 rounded animate-pulse"></div>
                            <!-- Buttons -->
                            <div class="flex gap-1">
                                <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                            </div>
                            <div class="flex gap-1">
                                <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                                <div class="flex-1 h-6 bg-gray-700 rounded animate-pulse"></div>
                            </div>
                            <!-- VU Meter area -->
                            <div class="flex-1 flex items-center justify-center py-4">
                                <div class="flex gap-2">
                                    <div class="w-8 h-48 bg-gray-700 rounded animate-pulse"></div>
                                    <div class="w-8 h-48 bg-gray-700 rounded animate-pulse"></div>
                                </div>
                            </div>
                            <!-- Fader area -->
                            <div class="flex items-center justify-center py-4">
                                <div class="w-12 h-32 bg-gray-700 rounded animate-pulse"></div>
                            </div>
                            <!-- Bottom controls -->
                            <div class="h-8 bg-gray-700 rounded animate-pulse"></div>
                        </div>
                    </div>
                </template>

                <!-- Master EQ Display, Spectrum & FX -->
                <template v-else>
                    <div class="w-[32rem] flex flex-col h-full gap-2 mixer-fade-in">
                        <div class="flex-[2] min-h-0">
                            <MasterEQDisplay :filters-data="masterEqFiltersData"
                                @filters-change="handleMasterEQFiltersChange" />
                        </div>
                        <div class="flex-[1.5] min-h-0">
                            <SpectrumMeter :master-channel="masterSectionRef?.analysisOutput" />
                        </div>
                        <div class="flex-1 min-h-0">
                            <MasterFX :master-section="masterSectionRef" />
                        </div>
                    </div>

                    <!-- Master Section -->
                    <div class="w-44 h-full mixer-fade-in">
                        <MasterSection ref="masterSectionRef" :master-channel="masterChannel" />
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
            @rename="handleRenameScene" />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, toRaw, nextTick, inject } from 'vue'
import AudioTrack from './components/AudioTrack.vue'
import AudioFlowModal from './components/AudioFlowModal.vue'
import MasterEQDisplay from './components/MasterEQDisplay.vue'
import MasterFX from './components/MasterFX.vue'
import MasterSection from './components/MasterSection.vue'
import ScenesModal from './components/ScenesModal.vue'
import SpectrumMeter from './components/SpectrumMeter.vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useScenes, type Scene, type TrackSnapshot } from '~/composables/useScenes'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'

const ToneRef = inject<any>('Tone')
let Tone: any = null
const toneReady = ref(false)
const masterChannel = ref<any>(null)

interface Track {
    id: number
}

// App ready state
const isReady = ref(false)

// Audio Flow Modal
const showAudioFlowModal = ref(false)
const showScenesModal = ref(false)

// Tracks management
const tracks = ref<Track[]>([
    { id: 1 },
    { id: 2 },
    { id: 3 },
    { id: 4 },
    { id: 5 },
    { id: 6 },
    { id: 7 },
    { id: 8 }
])

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

function addTrack() {
    if (tracks.value.length >= 24) return
    tracks.value.push({ id: getNextAvailableId() })
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

// Track refs management
const trackRefs = ref<Map<number, any>>(new Map())

function setTrackRef(trackId: number, el: any | null) {
    if (el) {
        trackRefs.value.set(trackId, el)
    } else {
        // Remove ref when component is unmounted
        trackRefs.value.delete(trackId)
    }
}

// Master section ref
const masterSectionRef = ref<any>(null)
const masterEqFiltersData = ref<any[]>([])

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

function handleMasterEQFiltersChange(filters: any) {
    if (!filters) return

    const rawFilters = filters.filtersData ? toRaw(filters.filtersData) : []
    masterEqFiltersData.value = rawFilters.map((filter: any) => ({
        type: filter.type,
        frequency: filter.frequency,
        gain: filter.gain,
        Q: filter.Q,
        color: filter.color
    }))

    if (masterSectionRef.value?.applyMasterEQ) {
        masterSectionRef.value.applyMasterEQ(filters)
    }
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
    deleteScene: deleteSceneFromStorage,
    renameScene,
    setCurrentScene
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

    // Collect master snapshot
    const masterSnapshot = masterSectionRef.value?.getSnapshot() || {
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: 0,
        isLinked: true,
        masterEQFilters: [],
        compressorEnabled: false,
        reverbEnabled: false,
        limiterEnabled: false
    }

    // Create and save scene
    const scene = await createScene(sceneName, trackSnapshots, masterSnapshot)
    setCurrentScene(scene.id)
}

function handleLoadScene(sceneId: string) {
    const scene = scenes.value.find((s: Scene) => s.id === sceneId)
    if (!scene) return

    // Restore each track's state
    scene.tracks.forEach((trackSnapshot: TrackSnapshot) => {
        const trackRef = trackRefs.value.get(trackSnapshot.trackNumber)
        if (trackRef && trackRef.restoreFromSnapshot) {
            trackRef.restoreFromSnapshot(trackSnapshot)
        }
    })

    // Restore master section state
    if (masterSectionRef.value && masterSectionRef.value.restoreFromSnapshot) {
        masterSectionRef.value.restoreFromSnapshot(scene.master)

        // Update masterEqFiltersData to sync with MasterEQDisplay component
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
    }

    // Set as current scene
    setCurrentScene(scene.id)
}

async function handleUpdateScene(sceneId: string) {
    // Get old scene to clean up orphaned files
    const oldScene = scenes.value.find((s: Scene) => s.id === sceneId)

    // Collect current state
    const trackSnapshots: TrackSnapshot[] = []
    tracks.value.forEach(track => {
        const trackRef = trackRefs.value.get(track.id)
        if (trackRef && trackRef.getSnapshot) {
            trackSnapshots.push(trackRef.getSnapshot())
        }
    })

    const masterSnapshot = masterSectionRef.value?.getSnapshot() || {
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: 0,
        isLinked: true,
        masterEQFilters: [],
        compressorEnabled: false,
        reverbEnabled: false,
        limiterEnabled: false
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

    // Delete scene from storage
    await deleteSceneFromStorage(sceneId)

    // If deleted scene was current, clear current
    if (currentSceneId.value === sceneId) {
        setCurrentScene(null)
    }
}

async function handleRenameScene(sceneId: string, newName: string) {
    await renameScene(sceneId, newName)
}

// Initialize audio
onMounted(async () => {
    document.title = 'Audio Mixer Pro - Multi-Track Mixer'

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

    masterChannel.value = new Tone.Channel({
        volume: 0,      // dB
        mute: false,
    }).toDestination()

    // Enumerate audio devices ONCE for all tracks
    await enumerateAudioInputs()

    // Wait for next tick to ensure all components are ready
    await nextTick()

    // Delay to ensure all components are fully mounted and initialized
    setTimeout(() => {
        isReady.value = true
    }, 100)
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
</style>
