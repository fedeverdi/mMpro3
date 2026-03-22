<template>
  <div class="mixer-app min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-black flex flex-col">
    <!-- Floating Particles Background -->
    <div class="particles-background">
      <div v-for="i in 40" :key="i" class="particle" :style="getParticleStyle(i)"></div>
    </div>
    
    <!-- Remote Mode Banner -->
    <RemoteModeBanner />
    
    <!-- Custom Title Bar (Electron only) -->
    <CustomTitleBar v-if="isElectronMode" :project-name="currentProjectName" />
    
    <!-- Header -->
    <AppHeader 
      :is-locked="isLocked"
      :build-limits="buildLimits"
      :tracks-count="tracks.length"
      :subgroups-count="subgroups.length"
      @show-scenes="showScenesModal = true"
      @show-file-manager="showFileManager = true"
      @lock-toggle="handleLockToggle"
      @add-track="addTrackOfType"
      @add-subgroup="addSubgroup"
      @remove-track="handleRemoveLastTrack"
      @remove-subgroup="handleRemoveLastSubgroup"
      @load-scene="handleLoadScene"
    />

    <!-- Mixer Console -->
    <main class="flex-1 flex gap-2 p-2 overflow-hidden relative z-[999]">
      <!-- Audio Tracks Section (flexible) -->
      <div ref="tracksContainerRef" class="tracks-scroll-wrap flex-1 overflow-hidden min-w-0 pb-[2px]">
        <div class="tracks-scroll overflow-x-auto overflow-y-hidden h-full">
          <div class="flex gap-2 h-full min-w-max">
            <!-- Audio Tracks -->
            <div v-for="track in sortedTracks" :key="track.id" class="w-[8.5rem] h-full mixer-fade-in track-wrapper"
              :class="{
                'dragging': draggedTrackId === track.id,
                'drag-over': dragOverTrackId === track.id
              }" @dragover="handleTrackDragOver(track.id, $event)" @dragleave="handleTrackDragLeave"
              @drop="handleTrackDrop(track.id)" @dragend="handleTrackDragEnd">
              <SignalTrack v-if="track.type === 'signal'" :ref="el => setTrackRef(track.id, el)" :trackNumber="track.id"
                :order="track.order" :master-channel="masterChannel" :subgroups="subgroups"
                :allow-subgroup-routing="buildLimits.allowSubgroupRouting" :is-dragging="draggedTrackId === track.id"
                @soloChange="handleSoloChange" @remove="removeTrack(track.id)"
                @drag-start="handleTrackDragStart(track.id)" />
                           <AudioTrack v-else :ref="el => setTrackRef(track.id, el)" :trackNumber="track.id"
                :master-channel="masterChannel" :subgroups="subgroups" :auxBuses="auxBuses"
                :allow-subgroup-routing="buildLimits.allowSubgroupRouting" @open-library="handleOpenLibrary"
                @remove="removeTrack(track.id)" />
            </div>
          </div>
        </div>
      </div>

      <!-- Right Section (fixed width) -->
      <div class="flex gap-2 flex-shrink-0">
        <!-- Master EQ Display, Spectrum & FX -->
        <RightSection ref="rightSectionRef" :master-channel="masterChannel" :master-section-ref="masterSectionRef"
          :master-fx-output-node="masterFxOutputNode" :aux-buses="auxBuses" :subgroups="subgroups"
          :master-eq-filters="masterEqFiltersData" :master-fx-effects="masterFxEffectsData"
          @master-fx-output-node="handleMasterFxOutputNode"
          @master-fx-component="handleMasterFxComponent"
          @update:master-eq-filters="handleMasterEQFiltersUpdate"
          @add-aux="addAux" @remove-aux="removeAux" @update-aux="updateAux" />

        <!-- Meters & Tools Section -->
        <div class="flex-shrink-0 h-full mixer-fade-in">
          <SlotSectionOne />
        </div>

        <!-- Subgroups Section -->
        <template v-for="subgroup in subgroups" :key="subgroup.id">
          <div class="flex-shrink-0 h-full mixer-fade-in">
            <SubgroupsSection :ref="el => setSubgroupRef(subgroup.id, el)" :master-channel="masterChannel"
              :subgroup-id="subgroup.id" :subgroup-name="subgroup.name" v-model:volume="subgroup.volume"
              v-model:route-to-master="subgroup.routeToMaster" v-model:selected-output="subgroup.selectedOutput"
              @remove="removeSubgroup(subgroup.id)" />
          </div>
        </template>

        <!-- Master Section -->
        <div class="flex-shrink-0 h-full mixer-fade-in">
          <MasterSection ref="masterSectionRef" :master-fx-output-node="masterFxOutputNode"
            :master-fx-component="masterFxComponent" :is-recording="isRecording" 
            @open-recorder="showRecorder = true"
            @open-ndi="showNDIModal = true" />
        </div>
      </div>
    </main>

    <!-- Audio Flow Modal -->
    <AudioFlowModal v-model="showAudioFlowModal" :subgroups="subgroups.map(s => ({ id: s.id, name: s.name }))"
      :aux-buses="auxBuses.map(a => ({ id: a.id, name: a.name }))" />

    <!-- Audio Settings Modal -->
    <AudioSettingsModal :is-open="showAudioSettings" @close="showAudioSettings = false"
      @apply="handleAudioConfigApply" @open-ndi="handleOpenNDI" />
    
    <!-- NDI Stream Modal -->
    <NDIStreamModal :is-open="showNDIModal" @close="showNDIModal = false" :subgroups="subgroups" />
    
    <Recorder v-model="showRecorder" :master-level-left="audioEngineState.masterLevels.left"
      :master-level-right="audioEngineState.masterLevels.right" :recording-time="recordingTime"
      :recording-file-size="recordingFileSize" :available-disk-space="availableDiskSpace"
      @recording-state="handleRecordingStateChange" />

    <!-- File Manager Modal -->
    <FileManagerModal v-model="showFileManager" @select-file="handleFileManagerSelect"
      @select-playlist="handlePlaylistSelect" />

    <!-- Scenes Modal -->
    <ScenesModal v-model="showScenesModal" @load-scene="handleLoadScene" />

    <!-- Lock System -->
    <SetLockPasswordModal :show="showSetPasswordModal" @close="showSetPasswordModal = false"
      @confirm="handleSetPassword" />
    <LockScreen ref="lockScreenRef" :show="isLocked" :level-left="audioEngineState.masterLevels.left"
      :level-right="audioEngineState.masterLevels.right" @unlock="handleUnlock" />

    <!-- Limit Reached Modal -->
    <Transition enter-from-class="opacity-0" enter-active-class="transition-opacity duration-200"
      enter-to-class="opacity-100" leave-from-class="opacity-100" leave-active-class="transition-opacity duration-200"
      leave-to-class="opacity-0">
      <div v-if="showLimitModal" @click="showLimitModal = false"
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[9999] p-4">
        <div @click.stop
          class="bg-gradient-to-br from-gray-800 to-gray-900 border-2 border-orange-500/70 rounded-lg shadow-2xl max-w-md w-full p-6">
          <div class="flex items-start gap-3 mb-4">
            <div class="flex-shrink-0 w-10 h-10 rounded-full bg-orange-500/20 flex items-center justify-center">
              <svg class="w-6 h-6 text-orange-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold text-orange-400 mb-2">Limit Reached</h3>
              <p class="text-gray-300 text-sm leading-relaxed" v-html="limitModalMessage"></p>
            </div>
          </div>
          <div class="flex gap-3 mt-6">
            <a href="https://www.mmpro.it" target="_blank"
              class="flex-1 px-4 py-2.5 bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 rounded-lg text-white font-semibold text-sm transition-all text-center">
              Download Full Version
            </a>
            <button @click="showLimitModal = false"
              class="px-4 py-2.5 bg-gray-700 hover:bg-gray-600 rounded-lg text-gray-300 font-semibold text-sm transition-all">
              Close
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Footer -->
    <Footer :performance-stats="audioEngineState.performanceStats" :is-recording="isRecording"
      :recording-time="recordingTime" :recording-file-size="recordingFileSize"
      :available-disk-space="availableDiskSpace" @open-audio-flow="showAudioFlowModal = true"
      @open-audio-config="showAudioSettings = true" />

    <!-- Notification Toast -->
    <NotificationToast />
    
    <!-- Remote Control Overlay (shown when browser remote is active) -->
    <RemoteControlOverlay v-if="!isRemoteMode" :is-active="isRemoteControlActive" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, toRaw, nextTick, inject, onUnmounted, provide, watch, type Ref } from 'vue'
import AudioTrack from './components/AudioTrack.vue'
import SignalTrack from './components/SignalTrack.vue'
import AudioFlowModal from './components/layout/AudioFlowModal.vue'
import AudioSettingsModal from './components/layout/AudioSettingsModal.vue'
import NDIStreamModal from './components/master/NDIStreamModal.vue'
import FileManagerModal from './components/layout/FileManagerModal.vue'
import RightSection from './components/master/RightSection.vue'
import SlotSectionOne from './components/SlotSectionOne.vue'
import MasterSection from './components/MasterSection.vue'
import SubgroupsSection from './components/SubgroupsSection.vue'
import ScenesModal from './components/layout/ScenesModal.vue'
import QuickScenes from './components/layout/QuickScenes.vue'
import Footer from './components/layout/Footer.vue'
import NotificationToast from './components/core/NotificationToast.vue'
import RemoteControlOverlay from './components/core/RemoteControlOverlay.vue'
import CustomTitleBar from './components/layout/CustomTitleBar.vue'
import AppHeader from './components/layout/AppHeader.vue'
import RemoteModeBanner from './components/layout/RemoteModeBanner.vue'
import { useAudioDevices } from '~/composables/useAudioDevices'
import { useAudioEngine } from '~/composables/useAudioEngine'
import { useNotifications } from '~/composables/useNotifications'
import { useScenes } from '~/composables/useScenes'
import { getBuildLimits, canAddTrack, getTrackCounts, getBuildMode } from '~/config/buildLimits'
import { channel } from 'diagnostics_channel'
import Recorder from './components/recorder/Recorder.vue'
import SetLockPasswordModal from './components/layout/SetLockPasswordModal.vue'
import LockScreen from './components/layout/LockScreen.vue'

const { audioOutputDevices, audioInputDevices, refreshAudioOutputs, refreshAudioInputs } = useAudioDevices()

// Import audio engine from context
const audioEngine = inject('audioEngine') as any
const audioEngineState = audioEngine.state
const isRemoteControlActive = audioEngine.isRemoteControlActive
const notify = useNotifications()

// Check if we're in remote mode (WebSocket browser)
const isRemoteMode = !(window as any).electronAPI

// Project name for title bar
const currentProjectName = ref('mMpro3 - Your Multitrack Mixer')

const masterChannel = ref<any>(null)

// Subgroups system
interface Subgroup {
  id: number
  name: string
  volume: number
  routeToMaster: boolean
  selectedOutput: string | null
  channel: any
  ref: any
}



const subgroups = ref<Subgroup[]>([])
let nextSubgroupId = 1

// Build limits
const buildLimits = computed(() => getBuildLimits())
const buildMode = computed(() => getBuildMode())

// Aux buses system (Rust backend with zero latency direct output)
interface AuxBus {
  id: string
  name: string
  volume: number
  muted: boolean
  soloed: boolean
  routeToMaster: boolean
  routeToSubgroups?: number[]
  selectedOutputDevice?: string | null
  node?: any  // Input node (Channel)
  outputNode?: any  // Output node (final node of FX chain)
  outputStreamDest?: MediaStreamAudioDestinationNode | null
  // FX Chain
  reverbNode?: any
  reverbEnabled?: boolean
  reverbParams?: { roomSize: number, damping: number, wet: number, width: number, preDelay: number }
  delayNode?: any
  delayEnabled?: boolean
  delayParams?: { delayTime: number, feedback: number, wet: number }
}

const auxBuses = ref<AuxBus[]>([])
let nextAuxId = 0

interface Track {
  id: number
  type: 'audio' | 'signal'
  order: number
}

// App ready state - not needed anymore since splash screen handles initialization
const isAppReady = inject<Ref<boolean>>('isAppReady', ref(false))

// Centralized resize trigger for all tracks
// Instead of N ResizeObservers (one per track), have one that notifies all tracks
const resizeTrigger = ref(0)
const tracksContainerRef = ref<HTMLElement | null>(null)
provide('resizeTrigger', resizeTrigger)

// Audio Flow Modal
const showAudioFlowModal = ref(false)
const showScenesModal = ref(false)
const showFileManager = ref(false)
const showAudioSettings = ref(false)
const showNDIModal = ref(false)
const showRecorder = ref(false)
const isRecording = ref(false)
const showLimitModal = ref(false)

// Detect if running in Electron (vs browser)
const isElectronMode = ref(!!window.electronAPI)

// Lock system
const isLocked = ref(false)
const lockPassword = ref<string | null>(null)
const showSetPasswordModal = ref(false)
const lockScreenRef = ref<any>(null)
const limitModalMessage = ref('')

// Generate random styles for particles background - cached at mount time
const particleStyles = ref<any[]>(
  Array.from({ length: 40 }, (_, index) => {
    const size = Math.random() * 3 + 1.5
    const left = Math.random() * 100
    const animationDuration = Math.random() * 18 + 15  // 15-33 secondi (molto molto lento)
    const animationDelay = Math.random() * 12
    
    return {
      width: `${size}px`,
      height: `${size}px`,
      left: `${left}%`,
      animationDuration: `${animationDuration}s`,
      animationDelay: `${animationDelay}s`
    }
  })
)

const getParticleStyle = (index: number) => {
  return particleStyles.value[index - 1] || {}
}

// Recording state handler - just updates the recording flag
function handleRecordingStateChange(state: boolean) {
  isRecording.value = state

  // Reset recording stats when stopping
  if (!state) {
    audioEngineState.value.recordingStats = null
  }
  // All recording stats (time, size, disk space) come from Rust via RecordingStats events
}

// Format recording stats for Footer display
const recordingTime = computed(() => {
  if (!audioEngineState.value.recordingStats) return '00:00'
  const elapsed = audioEngineState.value.recordingStats.elapsedSeconds
  const minutes = Math.floor(elapsed / 60)
  const seconds = elapsed % 60
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
})

const recordingFileSize = computed(() => {
  if (!audioEngineState.value.recordingStats) return '0 MB'
  const bytes = audioEngineState.value.recordingStats.fileSizeBytes

  if (bytes < 1024 * 1024) {
    return (bytes / 1024).toFixed(1) + ' KB'
  } else if (bytes < 1024 * 1024 * 1024) {
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  } else {
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
  }
})

const availableDiskSpace = computed(() => {
  if (!audioEngineState.value.recordingStats) return 'Waiting...'
  const gb = audioEngineState.value.recordingStats.availableSpaceGb
  return gb > 0 ? gb.toFixed(2) + ' GB' : 'Waiting...'
})

// Watch for configuration errors and offer to switch to Auto mode
// This must be outside onMounted to catch errors during initial startup
watch(() => audioEngineState.value.lastConfigError, async (error) => {
  if (!error) return
  
  // Clear the error immediately to avoid re-triggering
  audioEngineState.value.lastConfigError = null
  
  // Show confirmation modal
  const shouldSwitchToAuto = await notify.confirm(
    '❌ Audio configuration not supported by this device.\n\n' +
    'Would you like to switch to Auto mode?\n' +
    '(Auto mode will automatically adapt to your device\'s native settings)'
  )
  
  if (shouldSwitchToAuto) {
    // Apply Auto configuration (0 = Auto for both sample rate and buffer size)
    await handleAudioConfigApply({ sampleRate: 0, bufferSize: 0 })
    notify.success('✅ Switched to Auto mode')
  }
})

// Lock system functions
function handleLockToggle() {
  if (isLocked.value) {
    // Already locked, the lock screen is showing
    return
  } else {
    // Show set password modal
    showSetPasswordModal.value = true
  }
}

function handleSetPassword(password: string) {
  lockPassword.value = password
  isLocked.value = true
  showSetPasswordModal.value = false
}

function handleUnlock(password: string) {
  if (password === lockPassword.value) {
    isLocked.value = false
    lockPassword.value = null
  } else {
    // Show error in lock screen
    if (lockScreenRef.value && lockScreenRef.value.showError) {
      lockScreenRef.value.showError('Incorrect password')
    }
  }
}

// File Manager for tracks (Electron only)
const fileManagerTargetTrackId = ref<number | null>(null)

function openFileManagerForTrack(trackId: number) {
  fileManagerTargetTrackId.value = trackId
  showFileManager.value = true
}

function handleOpenLibrary(trackId: number) {
  fileManagerTargetTrackId.value = trackId
  showFileManager.value = true
}

function handleFileManagerSelect(file: any) {
  let targetTrackId = fileManagerTargetTrackId.value

  // If no specific track was selected (opened from top bar), find first free audio track
  if (targetTrackId === null) {
    targetTrackId = findFirstFreeAudioTrack()
    if (targetTrackId === null) {
      notify.warning('No free audio tracks available. All tracks have files loaded.')
      showFileManager.value = false
      return
    }
  }

  const trackRef = trackRefs.value.get(targetTrackId)
  if (trackRef && trackRef.loadFileFromLibrary) {
    trackRef.loadFileFromLibrary(file)
  }

  fileManagerTargetTrackId.value = null
  showFileManager.value = false
}

function handlePlaylistSelect(playlist: any) {
  let targetTrackId = fileManagerTargetTrackId.value

  // If no specific track was selected (opened from top bar), find first free audio track
  if (targetTrackId === null) {
    targetTrackId = findFirstFreeAudioTrack()
    if (targetTrackId === null) {
      notify.warning('No free audio tracks available. All tracks have files loaded.')
      showFileManager.value = false
      return
    }
  }

  const trackRef = trackRefs.value.get(targetTrackId)
  if (trackRef && trackRef.loadPlaylistFromLibrary) {
    trackRef.loadPlaylistFromLibrary(playlist)
  }

  fileManagerTargetTrackId.value = null
  showFileManager.value = false
}

// Find first audio track without a file loaded
function findFirstFreeAudioTrack(): number | null {
  for (const track of sortedTracks.value) {
    if (track.type === 'audio') {
      const trackRef = trackRefs.value.get(track.id)
      if (trackRef && trackRef.isAudioLoaded && !trackRef.isAudioLoaded()) {
        return track.id
      }
    }
  }
  return null
}

// Provide file manager API to child components
provide('fileManager', {
  openFileManager: openFileManagerForTrack
})

// Tracks management
// Initialize tracks based on build limits
function initializeTracks(): Track[] {
  const tracks: Track[] = []
  const limits = getBuildLimits()

  let trackId = 1

  // Add default audio tracks
  for (let i = 0; i < limits.defaultAudioTracks; i++) {
    tracks.push({ id: trackId, type: 'audio', order: trackId })
    trackId++
  }

  // Add default signal tracks
  for (let i = 0; i < limits.defaultSignalTracks; i++) {
    tracks.push({ id: trackId, type: 'signal', order: trackId })
    trackId++
  }

  return tracks
}

const tracks = ref<Track[]>(initializeTracks())

// Computed per ordinare le tracce per order
const sortedTracks = computed(() => {
  return [...tracks.value].sort((a, b) => a.order - b.order)
})

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
  console.log(`[Index] Adding ${type} track. Current tracks:`, tracks.value.length)

  // Check if we can add this track type
  if (!canAddTrack(tracks.value, type)) {
    const limits = buildLimits.value
    const counts = getTrackCounts(tracks.value)
    const mode = buildMode.value

    if (type === 'audio' && counts.audio >= limits.maxAudioTracks) {
      limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxAudioTracks} audio track${limits.maxAudioTracks > 1 ? 's' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
    } else if (type === 'signal' && counts.signal >= limits.maxSignalTracks) {
      limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxSignalTracks} signal track${limits.maxSignalTracks > 1 ? 's' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
    } else {
      limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxTracks} total tracks</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
    }
    showLimitModal.value = true
    return
  }

  const newId = getNextAvailableId()
  const maxOrder = tracks.value.length > 0 ? Math.max(...tracks.value.map(t => t.order)) : 0
  tracks.value.push({ id: newId, type, order: maxOrder + 1 })
  console.log(`[Index] Added ${type} track with ID ${newId}. Total tracks: ${tracks.value.length}`)
}

async function removeTrack(trackId: number) {
  if (tracks.value.length <= 1) return

  const trackIndex = tracks.value.findIndex(t => t.id === trackId)
  if (trackIndex === -1) return

  // Ask for confirmation
  const trackType = tracks.value[trackIndex].type === 'audio' ? 'Audio Track' : 'Signal Track'
  const confirmed = await notify.confirm(`Remove ${trackType} ${trackId}?`)
  if (!confirmed) {
    return
  }

  const removedTrack = tracks.value.splice(trackIndex, 1)[0]

  // Remove the track ref from the map
  if (removedTrack) {
    trackRefs.value.delete(removedTrack.id)
    // Also remove from solo tracks if it was soloed
    soloTracks.value.delete(removedTrack.id)
  }
}

function handleRemoveLastTrack() {
  if (tracks.value.length > 0) {
    const lastTrack = tracks.value[tracks.value.length - 1]
    removeTrack(lastTrack.id)
  }
}

function handleRemoveLastSubgroup() {
  if (subgroups.value.length > 0) {
    const lastSubgroup = subgroups.value[subgroups.value.length - 1]
    removeSubgroup(lastSubgroup.id)
  }
}

// Drag and Drop for track reordering
const draggedTrackId = ref<number | null>(null)
const dragOverTrackId = ref<number | null>(null)

function handleTrackDragStart(trackId: number) {
  draggedTrackId.value = trackId
}

function handleTrackDragOver(trackId: number, event: DragEvent) {
  event.preventDefault() // Necessary to allow drop
  if (draggedTrackId.value === trackId) return
  dragOverTrackId.value = trackId
}

function handleTrackDragLeave() {
  dragOverTrackId.value = null
}

function handleTrackDrop(targetTrackId: number) {
  const draggedId = draggedTrackId.value
  if (draggedId === null || draggedId === targetTrackId) {
    draggedTrackId.value = null
    dragOverTrackId.value = null
    return
  }

  // Find tracks
  const draggedTrack = tracks.value.find(t => t.id === draggedId)
  const targetTrack = tracks.value.find(t => t.id === targetTrackId)

  if (!draggedTrack || !targetTrack) {
    draggedTrackId.value = null
    dragOverTrackId.value = null
    return
  }

  // Scambia gli order delle due tracce
  const tempOrder = draggedTrack.order
  draggedTrack.order = targetTrack.order
  targetTrack.order = tempOrder

  // Clear drag state
  draggedTrackId.value = null
  dragOverTrackId.value = null
}

function handleTrackDragEnd() {
  draggedTrackId.value = null
  dragOverTrackId.value = null
}

// Track refs management (only for tracks, not for master components)
const trackRefs = ref<Map<number, any>>(new Map())
const masterSectionRef = ref<any>(null) // Keep only for getSnapshot/restoreSnapshot
const rightSectionRef = ref<any>(null) // Ref to RightSection component

// Audio nodes received from components via emit
const masterFxOutputNode = ref<any>(null)
const masterFxComponent = ref<any>(null) // For getSnapshot only

// Master EQ state (source of truth)
const masterEqFiltersData = ref<any[]>([])
// Master FX effects state (source of truth from backend)
const masterFxEffectsData = ref<any[]>([])
// Flag to prevent watch updates during drag operations
const isDraggingMasterEQ = ref(false)

// Handlers for output node updates
function handleMasterFxOutputNode(node: any) {
  masterFxOutputNode.value = node
}

function handleMasterFxComponent(component: any) {
  masterFxComponent.value = component
  // Backend handles effects synchronization automatically
}

// Handle master EQ filters update from RightSection
async function handleMasterEQFiltersUpdate(filters: any[]) {
  
  // Set flag to prevent watch from updating during our own update
  isDraggingMasterEQ.value = true

  if (!filters || filters.length === 0) {
    // Clear master EQ if no filters
    await window.audioEngine?.clearMasterParametricEQ()
    // Allow watch to update after a short delay (wait for Rust response)
    setTimeout(() => { isDraggingMasterEQ.value = false }, 100)
    return
  }

  // Convert filters to backend format and send to Rust audio engine
  const backendFilters = filters.map(f => ({
    type: f.type, // 'peaking', 'lowshelf', 'highshelf', etc.
    frequency: f.frequency,
    gain: f.gain,
    q: f.Q ?? 1.0  // Use Q if present, otherwise default to 1.0
  }))

  try {
    await window.audioEngine?.setMasterParametricEQFilters(backendFilters)
  } catch (error) {
    console.error('[Master EQ] Failed to update filters:', error)
  }

  // Allow watch to update after a short delay (wait for Rust response)
  setTimeout(() => { isDraggingMasterEQ.value = false }, 100)
}

function setTrackRef(trackId: number, el: any | null) {
  if (el) {
    trackRefs.value.set(trackId, el)
  } else {
    // Remove ref when component is unmounted
    trackRefs.value.delete(trackId)
  }
}

// Scene management
function getTrackState(trackId: number): any {
  const trackRef = trackRefs.value.get(trackId)
  if (!trackRef || !trackRef.getState) {
    console.warn('[Scene] Track ref not found or no getState method:', trackId)
    return null
  }
  return trackRef.getState()
}

function getMasterState(): any {
  if (masterSectionRef.value && masterSectionRef.value.getState) {
    return masterSectionRef.value.getState()
  }
  return null
}

function getMasterEqFilters(): any[] {
  return masterEqFiltersData.value || []
}

function getMasterFx(): any {
  if (masterFxComponent.value && masterFxComponent.value.getSnapshot) {
    return masterFxComponent.value.getSnapshot()
  }
  return null
}

function getSubgroupsState(): any[] {
  return subgroups.value.map(subgroup => ({
    id: subgroup.id,
    name: subgroup.name,
    volume: subgroup.volume,
    routeToMaster: subgroup.routeToMaster,
    selectedOutput: subgroup.selectedOutput
  }))
}

function getAuxBusesState(): any {
  // Get basic aux bus state
  const auxState = auxBuses.value.map(aux => ({
    id: aux.id,
    name: aux.name,
    volume: aux.volume,
    muted: aux.muted,
    routeToMaster: aux.routeToMaster,
    selectedOutputDevice: aux.selectedOutputDevice,
    reverbEnabled: aux.reverbEnabled,
    reverbParams: aux.reverbParams,
    delayEnabled: aux.delayEnabled,
    delayParams: aux.delayParams
  }))

  // Get routing state from AuxMaster component
  const routingState = rightSectionRef.value?.auxMasterRef?.getRoutingState?.()

  return {
    buses: auxState,
    routing: routingState || {}
  }
}

async function handleLoadScene(sceneName: string) {
  try {
    // Rust engine handles complete state load automatically
    // No need for manual reset or state application!
    console.log('[Scene] Loading scene:', sceneName)
    // Scene is already loaded by the modal, nothing to do here
    // The frontend will update automatically via parameter broadcasts
  } catch (error) {
    console.error('[Scene] Error in scene load handler:', error)
  }
}

// Subgroup management
function setSubgroupRef(subgroupId: number, el: any | null) {
  const subgroup = subgroups.value.find(s => s.id === subgroupId)
  if (subgroup && el) {
    subgroup.ref = el

    // Connect channel to subgroup input when ref is set
    nextTick(() => {
      if (subgroup.channel && el.getInputNode) {
        const inputNode = el.getInputNode()
        if (inputNode) {
          const rawChannel = toRaw(subgroup.channel)
          const rawInputNode = toRaw(inputNode)
          try {
            rawChannel.connect(rawInputNode)
          } catch (e) {
            console.error(`[Subgroup ${subgroup.name}] Connection error:`, e)
          }
        }
      }
    })
  }
}

async function addSubgroup() {
  // Check build limits
  if (subgroups.value.length >= buildLimits.value.maxSubgroups) {
    const limits = buildLimits.value
    const mode = buildMode.value
    limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxSubgroups} subgroup${limits.maxSubgroups > 1 ? 's' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for more subgroups.`
    showLimitModal.value = true
    return
  }

  // Send command to backend - the subgroup will be added when we receive the data from Rust
  await audioEngine.addSubgroup()
  // The subgroup will be created by the watch on subgroupLevels when Rust sends the data
}

async function removeSubgroup(subgroupId: number) {
  const index = subgroups.value.findIndex(s => s.id === subgroupId)
  if (index !== -1) {
    const subgroup = subgroups.value[index]

    // Ask for confirmation
    const confirmed = await notify.confirm(`Remove ${subgroup.name}?`)
    if (!confirmed) {
      return
    }

    // Disconnect all tracks from this subgroup
    trackRefs.value.forEach((trackRef, trackId) => {
      if (trackRef?.disconnectFromSubgroup) {
        trackRef.disconnectFromSubgroup(subgroupId)
      }
    })

    // Remove from backend
    audioEngine.removeSubgroup(subgroupId)

    // Remove from array - Vue will handle unmounting and cleanup via onUnmounted
    subgroups.value.splice(index, 1)

    console.log(`[Subgroup ${subgroupId}] Removed`)
  }
}

// Aux buses management
function addAux() {
  // Check build limits
  if (auxBuses.value.length >= buildLimits.value.maxAuxBuses) {
    const limits = buildLimits.value
    const mode = buildMode.value
    limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxAuxBuses} aux bus${limits.maxAuxBuses > 1 ? 'es' : ''}</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for more aux buses.`
    showLimitModal.value = true
    return
  }

  const id = `aux-${nextAuxId}`
  const name = `AUX ${nextAuxId + 1}`
  nextAuxId++

  // Aux buses now managed by Rust backend with zero latency direct output
  const newAux: AuxBus = {
    id,
    name,
    volume: 0,
    muted: false,
    soloed: false,
    routeToMaster: false,
    selectedOutputDevice: 'no-output',
    node: null,
    outputNode: null,
    outputStreamDest: null,
    // FX
    reverbNode: null,
    reverbEnabled: false,
    reverbParams: { roomSize: 0.5, damping: 0.5, wet: 1.0, width: 1.0, preDelay: 0.0 },
    delayNode: null,
    delayEnabled: false,
    delayParams: { delayTime: 0.25, feedback: 0.3, wet: 1.0 }
  }

  auxBuses.value.push(newAux)
}

async function removeAux(index: number) {
  if (index >= 0 && index < auxBuses.value.length) {
    const aux = auxBuses.value[index]

    // Ask for confirmation
    const confirmed = await notify.confirm(`Remove ${aux.name}?`)
    if (!confirmed) {
      return
    }

    // Remove from array (Rust backend handles cleanup)
    auxBuses.value.splice(index, 1)
  }
}

async function updateAux(index: number, updatedAux: AuxBus) {
  if (index >= 0 && index < auxBuses.value.length) {
    const aux = auxBuses.value[index]
    
    // Extract aux ID from string (e.g., "aux-0" -> 0)
    const auxId = parseInt(aux.id.replace('aux-', ''))

    // Send updates to Rust engine
    if (audioEngine.state.value.isRunning) {
      // Update volume (gain)
      if (updatedAux.volume !== aux.volume) {
        const linearGain = Math.pow(10, updatedAux.volume / 20)
        audioEngine.setAuxBusGain(auxId, linearGain)
      }

      // Update mute
      if (updatedAux.muted !== aux.muted) {
        audioEngine.setAuxBusMute(auxId, updatedAux.muted)
      }

      // Update routing to master
      if (updatedAux.routeToMaster !== aux.routeToMaster) {
        audioEngine.setAuxBusRouteToMaster(auxId, updatedAux.routeToMaster)
      }

      // Update reverb enabled state
      if (updatedAux.reverbEnabled !== aux.reverbEnabled) {
        const enabled = updatedAux.reverbEnabled ?? false
        // Use reverb params from updated aux or defaults
        const reverbParams = updatedAux.reverbParams
        const roomSize = reverbParams?.roomSize ?? 0.5
        const damping = reverbParams?.damping ?? 0.5
        const wet = reverbParams?.wet ?? 1.0
        const width = reverbParams?.width ?? 1.0
        const preDelay = reverbParams?.preDelay ?? 0.0

        audioEngine.setAuxBusReverb(
          auxId,
          enabled,
          roomSize,
          damping,
          wet,
          width,
          preDelay
        )
      }

      // Update delay enabled state
      if (updatedAux.delayEnabled !== aux.delayEnabled) {
        const enabled = updatedAux.delayEnabled ?? false
        const delayParams = updatedAux.delayParams
        const time = delayParams?.delayTime ?? 0.5
        const feedback = delayParams?.feedback ?? 0.3
        const wet = delayParams?.wet ?? 0.5

        audioEngine.setAuxBusDelay(
          auxId,
          enabled,
          time * 1000,  // Convert seconds to milliseconds
          feedback,
          wet
        )
      }
    }

    // Handle output device selection via Rust backend (like subgroups)
    if (updatedAux.selectedOutputDevice !== aux.selectedOutputDevice) {
      const deviceId = updatedAux.selectedOutputDevice

      // Save selected output to backend (same as Master/Subgroup does)
      await audioEngine.setAuxBusSelectedOutput(auxId, deviceId)

      // Parse device ID (format: "deviceId" or "deviceId:channel" for mono aux)
      const parts = deviceId?.split(':') || []
      const actualDeviceId = parts[0]
      // UI channels are 1-based (Ch 1, Ch 2, etc), backend is 0-based
      // For mono aux, use same channel for both left and right
      const uiChannel = parts[1] ? parseInt(parts[1]) : 1 // Default to Ch 1
      const backendChannel = uiChannel - 1 // Convert to 0-based
      const leftChannel = backendChannel
      const rightChannel = backendChannel // Same channel for mono

      // If "no-output" is selected, disable direct output
      if (actualDeviceId === 'no-output' || actualDeviceId === null) {
        audioEngine.setAuxBusOutputEnabled(auxId, false)
      } else {
        // Enable direct output when a device is selected
        audioEngine.setAuxBusOutputEnabled(auxId, true)

        // Set channel selection (aux buses can be stereo)
        audioEngine.setAuxBusOutputChannels(auxId, leftChannel, rightChannel)
      }
    }

    // Update values (preserve audio nodes managed separately)
    auxBuses.value[index] = {
      ...updatedAux,
      node: auxBuses.value[index].node,
      outputNode: auxBuses.value[index].outputNode,
      reverbNode: auxBuses.value[index].reverbNode,
      delayNode: auxBuses.value[index].delayNode,
      outputStreamDest: auxBuses.value[index].outputStreamDest
    }
  }
}

// Aux output now handled by Rust backend (ZERO latency like subgroups)
// No Web Audio API needed - direct CPAL output with channel selection support

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

// Audio Configuration Handler
async function handleAudioConfigApply(config: { sampleRate: number; bufferSize: number }) {
  console.log('[App] Applying audio config:', config)

  // Save to Rust engine (file-based)
  try {
    await window.audioEngine.saveAudioConfig(config.sampleRate, config.bufferSize)
    console.log('[App] Audio config saved to file')
  } catch (e) {
    console.error('[App] Failed to save audio config:', e)
  }

  // Stop current audio
  if (window.audioEngine) {
    await window.audioEngine.stop()
  }

  // Wait a bit for cleanup
  await new Promise(resolve => setTimeout(resolve, 100))

  // Start with new configuration
  // If sampleRate is 0 (Auto), pass null to let device choose its native rate
  // If bufferSize is 0 (Auto), pass null to let device choose its optimal buffer size
  // Otherwise, force the selected values
  if (window.audioEngine) {
    const sampleRate = config.sampleRate === 0 ? null : config.sampleRate
    const bufferSize = config.bufferSize === 0 ? null : config.bufferSize
    await window.audioEngine.start(null, null, sampleRate, bufferSize)
  }

  console.log('[App] Audio config applied successfully')
}

// Handle NDI modal open from audio settings
function handleOpenNDI() {
  showNDIModal.value = true
}

// Refs for cleanup handlers (need to be accessible in onUnmounted)
let resizeObserver: ResizeObserver | null = null
let resizeTimeout: ReturnType<typeof setTimeout> | null = null

// Listen for subgroup creation events from backend (both Electron and remote)
const handleSubgroupCreated = (event: CustomEvent) => {
  const { id } = event.detail
  
  // In Electron mode, skip the broadcast since we handle it optimistically
  if (!isRemoteMode) {
    return
  }
  
  // Check if subgroup already exists (remote mode)
  if (subgroups.value.find(sg => sg.id === id)) {
    return
  }
  
  // Create new subgroup in frontend
  const name = `SUB ${id + 1}`
  subgroups.value.push({
    id,
    name,
    volume: 0,
    routeToMaster: false,
    selectedOutput: 'no-output',
    channel: null,
    ref: null
  })
  
  // Sort by ID to maintain order
  subgroups.value.sort((a, b) => a.id - b.id)
}

// Handle visibility changes for ResizeObserver
const handleVisibilityChange = () => {
  if (document.hidden) {
    resizeObserver?.disconnect()
  } else if (tracksContainerRef.value && resizeObserver) {
    resizeObserver.observe(tracksContainerRef.value)
    resizeTrigger.value++ // Force update when visible again
  }
}

// Watch aux buses and sync to detached windows (for caching only)
watch(auxBuses, (newAuxBuses) => {
  // Only broadcast if in Electron mode (not remote mode) and after component is mounted
  if (isElectronMode && (window as any).electronAPI?.updateAuxBusesState && isAppReady.value) {
    // Convert to raw to avoid proxy issues and extract only serializable data
    const auxBusesData = toRaw(newAuxBuses).map(aux => ({
      id: aux.id,
      name: aux.name,
      volume: aux.volume,
      muted: aux.muted,
      soloed: aux.soloed,
      routeToMaster: aux.routeToMaster,
      reverbEnabled: aux.reverbEnabled,
      reverbParams: aux.reverbParams ? { ...aux.reverbParams } : undefined,
      delayEnabled: aux.delayEnabled,
      delayParams: aux.delayParams ? { ...aux.delayParams } : undefined,
      selectedOutputDevice: aux.selectedOutputDevice
    }));
    
    (window as any).electronAPI.updateAuxBusesState(auxBusesData).catch((error: any) => {
      // Silently ignore errors during initialization
      if (!error.message?.includes('No handler registered')) {
        console.error('[index] Failed to update aux buses state:', error)
      }
    })
  }
}, { deep: true })

// Initialize audio
onMounted(async () => {
  document.title = 'Audio Mixer Pro - Multi-Track Mixer'

  masterChannel.value = null
  
  // Master FX effects synchronization is now handled via audioEngineState.masterFxEffects watch

  // Add initial aux buses FIRST (before async operations) for immediate rendering
  // Skip in remote mode - remote clients will sync state from the host
  if (!isRemoteMode) {
    const limits = getBuildLimits()
    
    // Subgroups are now created automatically by Rust backend, no need to add them here

    // Add default aux buses (up to the build limit)
    const maxAuxToAdd = Math.min(6, limits.maxAuxBuses)
    for (let i = 0; i < maxAuxToAdd; i++) {
      addAux()
    }
  }

  // Then refresh audio outputs
  await refreshAudioOutputs()

  // Sync subgroups immediately from audioEngineState if available
  const initialSubgroups = audioEngineState.value.subgroupLevels
  if (initialSubgroups && initialSubgroups.size > 0) {
    for (const [subgroupId, subgroupData] of initialSubgroups.entries()) {
      if (!subgroups.value.find(sg => sg.id === subgroupId)) {
        const name = `SUB ${subgroupId + 1}`
        subgroups.value.push({
          id: subgroupId,
          name,
          volume: subgroupData.gain !== undefined ? 20 * Math.log10(Math.max(0.00001, subgroupData.gain)) : 0,
          routeToMaster: subgroupData.routeToMaster ?? false,
          selectedOutput: subgroupData.selectedOutput ?? 'no-output',
          channel: null,
          ref: null
        })
      }
    }
    subgroups.value.sort((a, b) => a.id - b.id)
  }

  // Don't start connection here - wait for component to mount

  // Audio input devices are now enumerated during app initialization (in App.vue)
  // No need to enumerate them again here

  // Wait for next tick to ensure all components are ready
  await nextTick()

  // NEW: Watch for master EQ filter changes from Rust Levels response
  watch(() => audioEngineState.value.masterEQFilters, (newFilters) => {
    // Skip update if user is currently dragging EQ controls
    if (isDraggingMasterEQ.value) {
      return
    }

    if (newFilters && newFilters.length >= 0) {  // Allow empty array to clear filters
      // Convert 'q' (lowercase) to 'Q' (uppercase) for frontend compatibility
      masterEqFiltersData.value = newFilters.map((f: any) => ({
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        Q: f.q  // Rust uses lowercase 'q', frontend uses uppercase 'Q'
      }))
    }
  }, { deep: true })

  // Watch for master FX effects changes from Rust backend
  watch(() => audioEngineState.value.masterFxEffects, (newEffects) => {
    if (newEffects && Array.isArray(newEffects)) {
      masterFxEffectsData.value = newEffects
    }
  }, { deep: true })

  // Sync subgroups from backend (both Electron and remote browser)
  // Watch only the Map size to avoid triggering on every levels update (every 50ms)
  watch(() => audioEngineState.value.subgroupLevels.size, () => {
    const subgroupLevels = audioEngineState.value.subgroupLevels
    // Get all subgroup IDs from the backend
    const backendSubgroupIds = new Set(Array.from(subgroupLevels.keys()))
    
    // Remove subgroups that no longer exist in backend
    subgroups.value = subgroups.value.filter(sg => backendSubgroupIds.has(sg.id))
    
    // Add new subgroups that exist in backend but not in frontend
    for (const [subgroupId, subgroupData] of subgroupLevels.entries()) {
      const existingSubgroup = subgroups.value.find(sg => sg.id === subgroupId)
      
      if (!existingSubgroup) {
        // Create new subgroup in frontend
        const name = `SUB ${subgroupId + 1}`
        subgroups.value.push({
          id: subgroupId,
          name,
          volume: subgroupData.gain !== undefined ? 20 * Math.log10(Math.max(0.00001, subgroupData.gain)) : 0,
          routeToMaster: subgroupData.routeToMaster ?? false,
          selectedOutput: subgroupData.selectedOutput ?? 'no-output',
          channel: null,
          ref: null
        })
      }
      // DON'T update existing subgroup volume/routeToMaster here!
      // SubgroupsSection component handles that through its own watcher
    }
    
    // Sort by ID to maintain order
    subgroups.value.sort((a, b) => a.id - b.id)
  })

  // Sync aux buses from backend (both Electron and remote browser)
  // Watch the auxLevels Map directly to detect changes
  watch(() => audioEngineState.value.auxLevels, (newAuxLevels) => {
    // Get all aux IDs from the backend
    const backendAuxIds = new Set(Array.from(newAuxLevels.keys()))
    
    // If backend has no aux buses, don't clear the frontend yet
    // (wait for data to arrive)
    if (backendAuxIds.size === 0) {
      return
    }
    
    // Remove aux buses that no longer exist in backend
    auxBuses.value = auxBuses.value.filter(aux => {
      const auxId = parseInt(aux.id.replace('aux-', ''))
      return backendAuxIds.has(auxId)
    })
    
    // Add or update aux buses from backend
    for (const [auxId, auxData] of newAuxLevels.entries()) {
      const auxIdStr = `aux-${auxId}`
      const existingAuxIndex = auxBuses.value.findIndex(aux => aux.id === auxIdStr)
      
      if (existingAuxIndex === -1) {
        // Create new aux bus in frontend
        const name = `AUX ${auxId + 1}`
        auxBuses.value.push({
          id: auxIdStr,
          name,
          volume: auxData.gain !== undefined ? 20 * Math.log10(Math.max(0.00001, auxData.gain)) : 0,
          muted: auxData.mute ?? false,
          soloed: false,
          routeToMaster: auxData.routeToMaster ?? true,
          routeToSubgroups: auxData.routeToSubgroups ?? [],
          selectedOutputDevice: auxData.selectedOutput ?? null,
          node: null,
          outputNode: null,
          outputStreamDest: null,
          reverbEnabled: auxData.reverb?.enabled ?? false,
          reverbParams: auxData.reverb ? {
            roomSize: auxData.reverb.roomSize ?? 0.3,
            damping: auxData.reverb.damping ?? 0.5,
            wet: auxData.reverb.wet ?? 0.3,
            width: auxData.reverb.width ?? 1.0,
            preDelay: auxData.reverb.preDelay ?? 0.0,
          } : { roomSize: 0.3, damping: 0.5, wet: 0.3, width: 1.0, preDelay: 0.0 },
          delayEnabled: auxData.delay?.enabled ?? false,
          delayParams: auxData.delay ? {
            delayTime: (auxData.delay.delayTimeLMs ?? 250.0) / 1000, // Convert ms to seconds
            feedback: auxData.delay.feedback ?? 0.3,
            wet: auxData.delay.mix ?? 0.3,
          } : { delayTime: 0.25, feedback: 0.3, wet: 0.3 }
        })
      } else {
        // Update existing aux bus from backend data
        const existingAux = auxBuses.value[existingAuxIndex]
        auxBuses.value[existingAuxIndex] = {
          ...existingAux,
          volume: auxData.gain !== undefined ? 20 * Math.log10(Math.max(0.00001, auxData.gain)) : existingAux.volume,
          muted: auxData.mute ?? existingAux.muted,
          routeToMaster: auxData.routeToMaster ?? existingAux.routeToMaster,
          routeToSubgroups: auxData.routeToSubgroups ?? existingAux.routeToSubgroups ?? [],
          selectedOutputDevice: auxData.selectedOutput ?? existingAux.selectedOutputDevice,
          reverbEnabled: auxData.reverb?.enabled ?? existingAux.reverbEnabled,
          reverbParams: auxData.reverb ? {
            roomSize: auxData.reverb.roomSize ?? existingAux.reverbParams?.roomSize ?? 0.3,
            damping: auxData.reverb.damping ?? existingAux.reverbParams?.damping ?? 0.5,
            wet: auxData.reverb.wet ?? existingAux.reverbParams?.wet ?? 0.3,
            width: auxData.reverb.width ?? existingAux.reverbParams?.width ?? 1.0,
            preDelay: auxData.reverb.preDelay ?? existingAux.reverbParams?.preDelay ?? 0.0,
          } : existingAux.reverbParams,
          delayEnabled: auxData.delay?.enabled ?? existingAux.delayEnabled,
          delayParams: auxData.delay ? {
            delayTime: (auxData.delay.delayTimeLMs ? auxData.delay.delayTimeLMs / 1000 : existingAux.delayParams?.delayTime ?? 0.25), // Convert ms to seconds
            feedback: auxData.delay.feedback ?? existingAux.delayParams?.feedback ?? 0.3,
            wet: auxData.delay.mix ?? existingAux.delayParams?.wet ?? 0.3,
          } : existingAux.delayParams,
          // Keep frontend-only state (name, nodes)
        }
      }
    }
    
    // Sort by ID to maintain order
    auxBuses.value.sort((a, b) => {
      const aId = parseInt(a.id.replace('aux-', ''))
      const bId = parseInt(b.id.replace('aux-', ''))
      return aId - bId
    })
  }, { deep: true })

  // Add event listener for subgroup creation
  window.addEventListener('subgroup-created', handleSubgroupCreated as EventListener)

  // Set up centralized ResizeObserver for all tracks
  // Throttled to prevent blocking during window animations
  if (tracksContainerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (resizeTimeout) return
      resizeTimeout = setTimeout(() => {
        resizeTrigger.value++
        resizeTimeout = null
      }, 16) // ~60fps
    })
    resizeObserver.observe(tracksContainerRef.value)

    // Disconnect observer when window is hidden to prevent blocking during minimize
    document.addEventListener('visibilitychange', handleVisibilityChange)
  }
})

// Cleanup: Remove event listeners and disconnect observers
onUnmounted(() => {
  window.removeEventListener('subgroup-created', handleSubgroupCreated as EventListener)
  
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
  
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  
  if (resizeTimeout) {
    clearTimeout(resizeTimeout)
    resizeTimeout = null
  }
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

/* Track drag and drop styles */
.track-wrapper {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.track-wrapper.dragging {
  opacity: 0.5;
}

.track-wrapper.drag-over {
  transform: translateX(4px);
}

.track-wrapper.drag-over::before {
  content: '';
  position: absolute;
  left: -2px;
  top: 0;
  bottom: 0;
  width: 4px;
  background: linear-gradient(180deg, #3b82f6 0%, #8b5cf6 100%);
  border-radius: 2px;
  z-index: 10;
  animation: pulse-glow 1s ease-in-out infinite;
}

@keyframes pulse-glow {

  0%,
  100% {
    opacity: 1;
    box-shadow: 0 0 8px rgba(59, 130, 246, 0.6);
  }

  50% {
    opacity: 0.7;
    box-shadow: 0 0 12px rgba(59, 130, 246, 0.8);
  }
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

/* Particles Background */
.particles-background {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  pointer-events: none;
  z-index: 200;
  transform: translateZ(0);
  isolation: isolate;
  contain: layout style paint;
}

.particle {
  position: absolute;
  bottom: -10px;
  background: radial-gradient(circle, rgba(96, 165, 250, 0.9) 0%, rgba(167, 139, 250, 0.6) 100%);
  border-radius: 50%;
  animation: float-up linear infinite;
  opacity: 0;
  box-shadow: 0 0 15px rgba(96, 165, 250, 0.7), 0 0 30px rgba(96, 165, 250, 0.3);
  will-change: transform;
  transform: translate3d(0, 0, 0);
}

@keyframes float-up {
  0% {
    transform: translate3d(0, 0, 0);
    opacity: 0;
  }
  3% {
    opacity: 0.75;
  }
  99% {
    opacity: 0.7;
  }
  100% {
    transform: translate3d(25px, -110vh, 0);
    opacity: 0;
  }
}

/* Alternative animation for variation */
.particle:nth-child(even) {
  animation-name: float-up-left;
}

@keyframes float-up-left {
  0% {
    transform: translate3d(0, 0, 0);
    opacity: 0;
  }
  3% {
    opacity: 0.75;
  }
  99% {
    opacity: 0.7;
  }
  100% {
    transform: translate3d(-25px, -110vh, 0);
    opacity: 0;
  }
}
</style>
