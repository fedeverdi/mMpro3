<template>
  <div class="mixer-app min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-black flex flex-col">
    <!-- Header -->
    <header class="bg-black/50 backdrop-blur-sm border-b border-gray-700 px-4 py-2 relative z-50">
      <div class="flex items-center justify-between gap-4 flex-wrap relative">
        <div class="flex items-center gap-2">
          <img src="./assets/logo_no_scritta.svg" alt="mMpro3" class="h-8" />
          
          <!-- Quick Scenes in Header -->
          <div class="w-px h-6 bg-gray-600"></div>
          <QuickScenes @load-scene="handleLoadScene" />
        </div>
        <div class="flex gap-2 items-center flex-wrap">
          <button @click="showScenesModal = true"
            class="px-3 py-1.5 border border-gray-600 hover:border-green-500 hover:bg-green-500/10 rounded text-xs font-semibold text-gray-300 hover:text-green-400 transition-all flex items-center gap-1.5">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" class="h-3.5 w-3.5" fill="currentColor">
              <path
                d="M149.333 216v80c0 13.255-10.745 24-24 24H24c-13.255 0-24-10.745-24-24v-80c0-13.255 10.745-24 24-24h101.333c13.255 0 24 10.745 24 24zM0 376v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H24c-13.255 0-24 10.745-24 24zM125.333 32H24C10.745 32 0 42.745 0 56v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24zm80 448H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24zm-24-424v80c0 13.255 10.745 24 24 24H488c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24zm24 264H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24z" />
            </svg>
            Scenes
          </button>

          <!-- File Manager Button -->
          <button @click="showFileManager = true"
            class="px-3 py-1.5 border border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 rounded text-xs font-semibold text-gray-300 hover:text-blue-400 transition-all flex items-center gap-1.5">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            Library
          </button>

          <div class="w-px h-6 bg-gray-600"></div>

          <!-- Lock Button -->
          <button @click="handleLockToggle"
            :class="[
              'px-3 py-1.5 border rounded text-xs font-semibold transition-all flex items-center gap-1.5',
              isLocked
                ? 'border-blue-600 bg-blue-600/20 text-blue-400 hover:bg-blue-600/30'
                : 'border-gray-600 hover:border-yellow-500 hover:bg-yellow-500/10 text-gray-300 hover:text-yellow-400'
            ]">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path v-if="isLocked" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
            </svg>
            {{ isLocked ? 'Locked' : 'Lock' }}
          </button>

          <div class="w-px h-6 bg-gray-600"></div>

          <div class="relative -mt-[3px] z-[100]">
            <button @click="handleAddButtonClick"
              class="mt-1 px-3 h-full py-1.5 border border-gray-600 hover:border-emerald-500 hover:bg-emerald-500/10 rounded text-xs font-semibold text-gray-300 hover:text-emerald-400 transition-all flex items-center gap-1.5">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Add
            </button>

            <!-- Dropdown Menu -->
            <div v-if="showAddTrackMenu"
              class="absolute top-full left-0 mt-1 w-36 bg-gray-800 border border-gray-600 rounded shadow-2xl z-[1000] overflow-visible">
              <button @click="addTrackOfType('audio')"
                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
                <div class="flex">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 256 512">
                    <path
                      d="M96 496V16c0-8.8-7.2-16-16-16H48c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16zm128 0V16c0-8.8-7.2-16-16-16h-32c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16z" />
                  </svg>
                </div>
                Audio Track
              </button>
              <button @click="addTrackOfType('signal')"
                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
                <div class="flex">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 640 512">
                    <path
                      d="M476 480H324a36 36 0 0 1-36-36V96h-96v156a36 36 0 0 1-36 36H16a16 16 0 0 1-16-16v-32a16 16 0 0 1 16-16h112V68a36 36 0 0 1 36-36h152a36 36 0 0 1 36 36v348h96V260a36 36 0 0 1 36-36h140a16 16 0 0 1 16 16v32a16 16 0 0 1-16 16H512v156a36 36 0 0 1-36 36z" />
                  </svg>
                </div>
                Signal Track
              </button>
              <template v-if="buildLimits.maxSubgroups > 0">
                <div class="h-px bg-gray-600 my-1"></div>
                <button @click="addSubgroup(); showAddTrackMenu = false"
                  class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
                  <div class="flex">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 512 512">
                      <path
                        d="M12.41 148.02l232.94 105.67c6.8 3.09 14.49 3.09 21.29 0l232.94-105.67c16.55-7.51 16.55-32.52 0-40.03L266.65 2.31a25.607 25.607 0 0 0-21.29 0L12.41 107.98c-16.55 7.51-16.55 32.53 0 40.04zm487.18 88.28l-58.09-26.33-161.64 73.27c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.51 209.97l-58.1 26.33c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 276.3c16.55-7.5 16.55-32.5 0-40zm0 127.8l-57.87-26.23-161.86 73.37c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.29 337.87 12.41 364.1c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 404.1c16.55-7.5 16.55-32.5 0-40z" />
                    </svg>
                  </div>
                  Subgroup
                </button>
              </template>
            </div>
          </div>

          <button @click="removeTrack(tracks[tracks.length - 1].id)" :disabled="tracks.length <= 1"
            class="px-3 py-1.5 border border-gray-600 hover:border-red-500 hover:bg-red-500/10 disabled:border-gray-700 disabled:bg-gray-800/50 disabled:cursor-not-allowed rounded text-xs font-semibold text-gray-300 hover:text-red-400 disabled:text-gray-600 transition-all flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
            </svg>
            Remove
          </button>

          <div class="text-xs text-gray-400">
            {{ tracks.length }}/{{ buildLimits.maxTracks }}
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
            <!-- Audio Tracks -->
            <div v-for="track in sortedTracks" :key="track.id" class="w-[8.5rem] h-full mixer-fade-in track-wrapper"
              :class="{
                'dragging': draggedTrackId === track.id,
                'drag-over': dragOverTrackId === track.id
              }" @dragover="handleTrackDragOver(track.id, $event)" @dragleave="handleTrackDragLeave"
              @drop="handleTrackDrop(track.id)" @dragend="handleTrackDragEnd">
              <SignalTrack v-if="track.type === 'signal'" :ref="el => setTrackRef(track.id, el)"
                :trackNumber="track.id" :order="track.order" :master-channel="masterChannel" :subgroups="subgroups"
                :allow-subgroup-routing="buildLimits.allowSubgroupRouting" :is-dragging="draggedTrackId === track.id"
                @soloChange="handleSoloChange" @remove="removeTrack(track.id)"
                @drag-start="handleTrackDragStart(track.id)" />
              <AudioTrack v-else :ref="el => setTrackRef(track.id, el)" :trackNumber="track.id"
                :master-channel="masterChannel" :subgroups="subgroups" :aux-buses="auxBuses"
                :allow-subgroup-routing="buildLimits.allowSubgroupRouting"
                @open-library="handleOpenLibrary" @remove="removeTrack(track.id)" />
            </div>
          </div>
        </div>
      </div>

      <!-- Right Section (fixed width) -->
      <div class="flex gap-2 flex-shrink-0">
        <!-- Master EQ Display, Spectrum & FX -->
        <RightSection ref="rightSectionRef" :master-channel="masterChannel"
          :master-section-ref="masterSectionRef" :master-fx-output-node="masterFxOutputNode" :aux-buses="auxBuses"
          :subgroups="subgroups" :master-eq-filters="masterEqFiltersData"
          @master-fx-output-node="handleMasterFxOutputNode"
          @master-fx-component="handleMasterFxComponent" @update:master-eq-filters="handleMasterEQFiltersUpdate"
          @add-aux="addAux" @remove-aux="removeAux" @update-aux="updateAux" />

        <!-- Subgroups Section -->
        <template v-for="subgroup in subgroups" :key="subgroup.id">
          <div class="flex-shrink-0 h-full mixer-fade-in">
            <SubgroupsSection :ref="el => setSubgroupRef(subgroup.id, el)" :master-channel="masterChannel"
              :subgroup-id="subgroup.id" :subgroup-name="subgroup.name" @remove="removeSubgroup(subgroup.id)" />
          </div>
        </template>

        <!-- Master Section -->
        <div class="flex-shrink-0 h-full mixer-fade-in">
          <MasterSection ref="masterSectionRef" :master-fx-output-node="masterFxOutputNode"
            :master-fx-component="masterFxComponent" :is-recording="isRecording"
            @open-recorder="showRecorder = true" />
        </div>
      </div>
    </main>

    <!-- Audio Flow Modal -->
    <AudioFlowModal v-model="showAudioFlowModal" :subgroups="subgroups.map(s => ({ id: s.id, name: s.name }))"
      :aux-buses="auxBuses.map(a => ({ id: a.id, name: a.name }))" />

    <!-- Audio Settings Modal -->
    <AudioSettingsModal :is-open="showAudioSettings" @close="showAudioSettings = false" @apply="handleAudioConfigApply" />
    <Recorder 
      v-model="showRecorder" 
      :master-level-left="audioEngineState.masterLevels.left"
      :master-level-right="audioEngineState.masterLevels.right"
      :recording-time="recordingTime"
      :recording-file-size="recordingFileSize"
      :available-disk-space="availableDiskSpace"
      @recording-state="handleRecordingStateChange"
    />

    <!-- File Manager Modal -->
    <FileManagerModal v-model="showFileManager" @select-file="handleFileManagerSelect"
      @select-playlist="handlePlaylistSelect" />

    <!-- Scenes Modal -->
    <ScenesModal 
      v-model="showScenesModal" 
      :tracks="tracks"
      :get-track-state="getTrackState"
      :get-master-state="getMasterState"
      :get-master-eq-filters="getMasterEqFilters"
      :get-master-fx="getMasterFx"
      :get-subgroups-state="getSubgroupsState"
      :get-aux-buses-state="getAuxBusesState"
      @load-scene="handleLoadScene"
    />

    <!-- Lock System -->
    <SetLockPasswordModal :show="showSetPasswordModal" @close="showSetPasswordModal = false"
      @confirm="handleSetPassword" />
    <LockScreen ref="lockScreenRef" :show="isLocked" 
      :level-left="audioEngineState.masterLevels.left"
      :level-right="audioEngineState.masterLevels.right"
      @unlock="handleUnlock" />

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
    <Footer 
      :performance-stats="audioEngineState.performanceStats" 
      :is-recording="isRecording"
      :recording-time="recordingTime"
      :recording-file-size="recordingFileSize"
      :available-disk-space="availableDiskSpace"
      @open-audio-flow="showAudioFlowModal = true"
      @open-audio-config="showAudioSettings = true"
    />

    <!-- Notification Toast -->
    <NotificationToast />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, toRaw, nextTick, inject, onUnmounted, provide, type Ref } from 'vue'
import AudioTrack from './components/AudioTrack.vue'
import SignalTrack from './components/SignalTrack.vue'
import AudioFlowModal from './components/layout/AudioFlowModal.vue'
import AudioSettingsModal from './components/layout/AudioSettingsModal.vue'
import FileManagerModal from './components/layout/FileManagerModal.vue'
import RightSection from './components/master/RightSection.vue'
import MasterSection from './components/MasterSection.vue'
import SubgroupsSection from './components/SubgroupsSection.vue'
import ScenesModal from './components/layout/ScenesModal.vue'
import QuickScenes from './components/layout/QuickScenes.vue'
import Footer from './components/layout/Footer.vue'
import NotificationToast from './components/core/NotificationToast.vue'
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
const audioEngine = useAudioEngine()
const audioEngineState = audioEngine.state
const notify = useNotifications()

const masterChannel = ref<any>(null)

// Subgroups system
interface Subgroup {
  id: number
  name: string
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
  selectedOutputDevice?: string | null
  node?: any  // Input node (Channel)
  outputNode?: any  // Output node (final node of FX chain)
  outputStreamDest?: MediaStreamAudioDestinationNode | null
  // FX Chain
  reverbNode?: any
  reverbEnabled?: boolean
  reverbParams?: { roomSize: number, damping: number, wet: number, width: number }
  delayNode?: any
  delayEnabled?: boolean
  delayParams?: { delayTime: number, feedback: number, wet: number }
}

const auxBuses = ref<AuxBus[]>([])
let nextAuxId = 1

interface Track {
  id: number
  type: 'audio' | 'signal'
  order: number
}

// App ready state - not needed anymore since splash screen handles initialization
const isAppReady = inject<Ref<boolean>>('isAppReady', ref(false))

// Audio Flow Modal
const showAudioFlowModal = ref(false)
const showScenesModal = ref(false)
const showFileManager = ref(false)
const showAudioSettings = ref(false)
const showRecorder = ref(false)
const isRecording = ref(false)
const showLimitModal = ref(false)

// Lock system
const isLocked = ref(false)
const lockPassword = ref<string | null>(null)
const showSetPasswordModal = ref(false)
const lockScreenRef = ref<any>(null)
const limitModalMessage = ref('')

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
// Initialize with 1 audio track by default
function initializeTracks(): Track[] {
  const tracks: Track[] = []

  // Start with 5 audio tracks
  tracks.push({ id: 1, type: 'audio', order: 1 })
  tracks.push({ id: 2, type: 'audio', order: 2 })
  tracks.push({ id: 3, type: 'audio', order: 3 })
  tracks.push({ id: 4, type: 'audio', order: 4 })
  
  // Add one signal track at the end
  tracks.push({ id: 5, type: 'signal', order: 5 })

  return tracks
}

const tracks = ref<Track[]>(initializeTracks())

// Computed per ordinare le tracce per order
const sortedTracks = computed(() => {
  return [...tracks.value].sort((a, b) => a.order - b.order)
})

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

function handleAddButtonClick() {
  // Check if we've reached the total track limit
  if (tracks.value.length >= buildLimits.value.maxTracks) {
    const limits = buildLimits.value
    const mode = buildMode.value
    limitModalMessage.value = `You've reached the maximum of <strong>${limits.maxTracks} total tracks</strong> in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
    showLimitModal.value = true
    return
  }

  // Check if we can add at least one type of track
  const canAddAudio = canAddTrack(tracks.value, 'audio')
  const canAddSignal = canAddTrack(tracks.value, 'signal')

  if (!canAddAudio && !canAddSignal) {
    // Can't add any type of track
    const limits = buildLimits.value
    const mode = buildMode.value
    limitModalMessage.value = `You've reached the limits for all track types in <strong>${mode}</strong> mode.<br/><br/>Upgrade to the full version for unlimited tracks.`
    showLimitModal.value = true
    return
  }

  // Open the menu
  showAddTrackMenu.value = !showAddTrackMenu.value
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
    showAddTrackMenu.value = false
    return
  }

  const newId = getNextAvailableId()
  const maxOrder = tracks.value.length > 0 ? Math.max(...tracks.value.map(t => t.order)) : 0
  tracks.value.push({ id: newId, type, order: maxOrder + 1 })
  console.log(`[Index] Added ${type} track with ID ${newId}. Total tracks: ${tracks.value.length}`)
  showAddTrackMenu.value = false
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

// Handlers for output node updates
function handleMasterFxOutputNode(node: any) {
  masterFxOutputNode.value = node
}

function handleMasterFxComponent(component: any) {
  masterFxComponent.value = component
}

// Handle master EQ filters update from RightSection
async function handleMasterEQFiltersUpdate(filters: any[]) {
  // Update local state (source of truth)
  masterEqFiltersData.value = filters
  
  if (!filters || filters.length === 0) {
    // Clear master EQ if no filters
    await window.audioEngine?.clearMasterParametricEQ()
    console.log('[Master EQ] Cleared filters')
    return
  }
  
  // Convert filters to backend format and send to Rust audio engine
  const backendFilters = filters.map(f => ({
    type: f.type, // 'peaking', 'lowshelf', 'highshelf', etc.
    frequency: f.frequency,
    gain: f.gain,
    q: f.Q
  }))
  
  try {
    await window.audioEngine?.setMasterParametricEQFilters(backendFilters)
    console.log('[Master EQ] Updated filters:', backendFilters.length, 'bands')
  } catch (error) {
    console.error('[Master EQ] Failed to update filters:', error)
  }
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
  return subgroups.value.map(subgroup => {
    const subgroupRef = subgroup.ref
    if (subgroupRef && subgroupRef.getState) {
      return {
        id: subgroup.id,
        name: subgroup.name,
        ...subgroupRef.getState()
      }
    }
    return {
      id: subgroup.id,
      name: subgroup.name,
      volume: 0,
      routeToMaster: false,
      selectedOutput: 'no-output'
    }
  })
}

function getAuxBusesState(): any[] {
  return auxBuses.value.map(aux => ({
    id: aux.id,
    name: aux.name,
    volume: aux.volume,
    reverbParams: aux.reverbParams,
    delayParams: aux.delayParams
  }))
}

async function handleLoadScene(scene: any) {
  try {
    // RESET: First, reset all mixer state to defaults
    console.log('[Scene] Resetting mixer to defaults...')
    
    // Reset all tracks to default state
    for (const track of tracks.value) {
      const trackRef = trackRefs.value.get(track.id)
      if (trackRef && trackRef.setState) {
        await trackRef.setState({
          gain: 0,
          volume: 0,
          pan: 0,
          mute: false,
          solo: false,
          phaseInvert: false,
          padEnabled: false,
          hpfEnabled: false,
          routeToMaster: true,
          routedSubgroups: [],
          gateEnabled: false,
          compressorEnabled: false,
          eqEnabled: false,
          eqLow: 0,
          eqLowMid: 0,
          eqHighMid: 0,
          eqHigh: 0,
          parametricEQFilters: []
        })
      }
    }
    
    // Reset master section to defaults
    if (masterSectionRef.value && masterSectionRef.value.setState) {
      await masterSectionRef.value.setState({
        leftVolume: 0,
        rightVolume: 0,
        headphonesVolume: -60,
        isLinked: true,
        masterMuted: false,
        selectedMasterOutput: null,
        selectedHeadphonesOutput: null
      })
    }
    
    // Reset master EQ filters (clear all filters)
    console.log('[Scene] Resetting Master EQ filters')
    masterEqFiltersData.value = []
    await nextTick() // Force Vue to process the reactive change
    await window.audioEngine?.clearMasterParametricEQ()
    
    // Reset master FX (clear all effects)
    if (masterFxComponent.value && masterFxComponent.value.resetToDefaults) {
      masterFxComponent.value.resetToDefaults()
    }
    
    // Reset subgroups to defaults
    for (const subgroup of subgroups.value) {
      if (subgroup.ref && subgroup.ref.setState) {
        await subgroup.ref.setState({
          volume: 0,
          routeToMaster: false,
          selectedOutput: 'no-output'
        })
      }
    }
    
    // Reset aux buses to defaults
    for (const aux of auxBuses.value) {
      aux.volume = 0
      aux.reverbParams = undefined
      aux.delayParams = undefined
    }
    
    console.log('[Scene] Reset complete. Loading scene:', scene.name)
    
    // LOAD: Now load the scene state
    // Load each track's state
    for (let i = 0; i < scene.tracks.length; i++) {
      const trackState = scene.tracks[i]
      const track = tracks.value[i]
      if (!track) continue
      
      const trackRef = trackRefs.value.get(track.id)
      if (trackRef && trackRef.setState) {
        await trackRef.setState(trackState)
      }
    }
    
    // Load master state
    if (scene.master && masterSectionRef.value && masterSectionRef.value.setState) {
      await masterSectionRef.value.setState(scene.master)
    }
    
    // Load master EQ filters
    if (scene.masterEQFilters && scene.masterEQFilters.length > 0) {
      console.log('[Scene] Loading Master EQ filters:', scene.masterEQFilters.length, 'bands', scene.masterEQFilters)
      masterEqFiltersData.value = [...scene.masterEQFilters]
      await nextTick() // Force Vue to process the change
      
      // Update the backend
      const backendFilters = scene.masterEQFilters.map((f: any) => ({
        type: f.type,
        frequency: f.frequency,
        gain: f.gain,
        q: f.Q
      }))
      await window.audioEngine?.setMasterParametricEQFilters(backendFilters)
      console.log('[Master EQ] Backend updated with', backendFilters.length, 'bands')
    } else {
      // Scene has no EQ filters, ensure they're cleared
      console.log('[Scene] No Master EQ filters in scene')
      masterEqFiltersData.value = []
      await nextTick() // Force Vue to process the change
    }
    
    // Load master FX
    if (scene.masterFX && masterFxComponent.value && masterFxComponent.value.restoreSnapshot) {
      console.log('[Scene] Loading Master FX chain')
      masterFxComponent.value.restoreSnapshot(scene.masterFX)
    }
    
    // Load subgroups state
    if (scene.subgroups && Array.isArray(scene.subgroups)) {
      for (const subgroupState of scene.subgroups) {
        const subgroup = subgroups.value.find(s => s.id === subgroupState.id)
        if (subgroup && subgroup.ref && subgroup.ref.setState) {
          await subgroup.ref.setState(subgroupState)
        }
      }
    }
    
    // Load aux buses state
    if (scene.auxBuses && Array.isArray(scene.auxBuses)) {
      // Restore aux buses data
      for (const auxState of scene.auxBuses) {
        const aux = auxBuses.value.find(a => a.id === auxState.id)
        if (aux) {
          aux.volume = auxState.volume ?? 0
          aux.reverbParams = auxState.reverbParams
          aux.delayParams = auxState.delayParams
        }
      }
    }
    
    // Set current scene ID in the composable
    const { setCurrentSceneId } = useScenes()
    setCurrentSceneId(scene.id)
    
    console.log('[Scene] Scene loaded successfully:', scene.name)
  } catch (error) {
    console.error('[Scene] Error loading scene:', error)
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

  const name = `SUB ${subgroups.value.length + 1}`

  // Add to frontend state IMMEDIATELY (optimistic UI)
  const tempSubgroup = {
    id: 0, // Temporary id, will be updated when backend responds
    name,
    channel: null,
    ref: null
  }
  subgroups.value.push(tempSubgroup)

  // Create subgroup in Rust backend (async)
  const id = await audioEngine.addSubgroup()
  if (id === null) {
    console.error('[addSubgroup] Failed to create subgroup in backend')
    // Remove the optimistically added subgroup on failure
    const index = subgroups.value.indexOf(tempSubgroup)
    if (index > -1) {
      subgroups.value.splice(index, 1)
    }
    return
  }

  // Update the id when backend responds
  tempSubgroup.id = id
  console.log(`[Subgroup ${id}] Created: ${name}`)
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
    await audioEngine.removeSubgroup(subgroupId)

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

  const id = `aux-${nextAuxId++}`
  const name = `AUX ${nextAuxId - 1}`

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
    reverbParams: { roomSize: 0.5, damping: 0.5, wet: 1.0, width: 1.0 },
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

    // Send updates to Rust engine
    if (audioEngine.state.value.isRunning) {
      // Update volume (gain)
      if (updatedAux.volume !== aux.volume) {
        const linearGain = Math.pow(10, updatedAux.volume / 20)
        await audioEngine.setAuxBusGain(index, linearGain)
      }

      // Update mute
      if (updatedAux.muted !== aux.muted) {
        await audioEngine.setAuxBusMute(index, updatedAux.muted)
      }

      // Update routing to master
      if (updatedAux.routeToMaster !== aux.routeToMaster) {
        await audioEngine.setAuxBusRouteToMaster(index, updatedAux.routeToMaster)
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
        
        await audioEngine.setAuxBusReverb(
          index,
          enabled,
          roomSize,
          damping,
          wet,
          width
        )
      }

      // Update delay enabled state
      if (updatedAux.delayEnabled !== aux.delayEnabled) {
        const enabled = updatedAux.delayEnabled ?? false
        const delayParams = updatedAux.delayParams
        const time = delayParams?.delayTime ?? 0.5
        const feedback = delayParams?.feedback ?? 0.3
        const wet = delayParams?.wet ?? 0.5
        
        await audioEngine.setAuxBusDelay(
          index,
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
      
      // Parse device ID (format: "deviceId" or "deviceId:ch" for mono)
      const parts = deviceId?.split(':') || []
      const actualDeviceId = parts[0]
      const channel = parts[1] ? parseInt(parts[1]) : 0
      
      // If "no-output" is selected, disable direct output
      if (actualDeviceId === 'no-output' || actualDeviceId === null) {
        await audioEngine.setAuxBusOutputEnabled(index, false)
      } else {
        // Enable direct output when a device is selected
        await audioEngine.setAuxBusOutputEnabled(index, true)
        
        // Aux are mono: use same channel for both L and R
        await audioEngine.setAuxBusOutputChannels(index, channel, channel)
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
  
  // Save to localStorage
  localStorage.setItem('audioConfig', JSON.stringify(config))
  
  // Stop current audio
  if (window.audioEngine) {
    await window.audioEngine.stop()
  }
  
  // Wait a bit for cleanup
  await new Promise(resolve => setTimeout(resolve, 100))
  
  // Start with new configuration
  if (window.audioEngine) {
    await window.audioEngine.start(null, null, config.sampleRate, config.bufferSize)
  }
  
  console.log('[App] Audio config applied successfully')
}

// Initialize audio
onMounted(async () => {
  document.title = 'Audio Mixer Pro - Multi-Track Mixer'

  masterChannel.value = null

  // Add initial subgroup and aux buses FIRST (before async operations) for immediate rendering
  const limits = getBuildLimits()
  if (limits.maxSubgroups > 0) {
    addSubgroup()
  }

  // Add default aux buses (up to the build limit)
  const maxAuxToAdd = Math.min(6, limits.maxAuxBuses)
  for (let i = 0; i < maxAuxToAdd; i++) {
    addAux()
  }

  // Then refresh audio outputs
  await refreshAudioOutputs()

  // Don't start connection here - wait for component to mount

  // Audio input devices are now enumerated during app initialization (in App.vue)
  // No need to enumerate them again here

  // Wait for next tick to ensure all components are ready
  await nextTick()

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
</style>
