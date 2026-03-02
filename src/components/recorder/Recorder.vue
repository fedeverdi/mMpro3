<template>
  <Transition
    enter-from-class="opacity-0"
    enter-active-class="transition-opacity duration-300"
    enter-to-class="opacity-100"
    leave-from-class="opacity-100"
    leave-active-class="transition-opacity duration-300"
    leave-to-class="opacity-0">
    <div v-if="modelValue" @click="closeModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]">
      <div @click.stop class="recorder-modal bg-gradient-to-br from-gray-800 to-gray-900 rounded-xl border-2 border-red-600 shadow-2xl p-6 w-[1100px] max-h-[85vh] overflow-hidden flex flex-col">
        <!-- Header -->
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 bg-red-600 rounded-full flex items-center justify-center">
              <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="8" />
              </svg>
            </div>
            <div>
              <h2 class="text-xl font-bold text-white">Master Recorder</h2>
              <p class="text-xs text-gray-400">Record your master mix to WAV</p>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <button @click="showSettings = true" class="text-gray-400 hover:text-white transition-colors" title="Recording Settings">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
            <button @click="closeModal" class="text-gray-400 hover:text-white transition-colors">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Recording Controls -->
        <div class="bg-gray-900/50 rounded-lg p-6 mb-6 border border-gray-700">
          <div class="flex items-center gap-4">
            <!-- Left section with fixed width -->
            <div class="flex items-center gap-4 w-[520px] flex-shrink-0">
              <button @click="toggleRecording"
                class="w-16 h-16 rounded-full flex items-center justify-center text-white font-bold transition-all shadow-lg flex-shrink-0"
                :class="isRecording ? 'bg-red-700 animate-pulse' : 'bg-red-600 hover:bg-red-700'">
                <svg v-if="!isRecording" class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="8" />
                </svg>
                <svg v-else class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                  <rect x="6" y="6" width="12" height="12" />
                </svg>
              </button>
              
              <div class="flex-1 min-w-0">
                <div class="text-xs text-gray-400 uppercase tracking-wider mb-1">
                  {{ isRecording ? 'Recording' : 'Ready' }}
                </div>
                <div class="text-3xl font-mono font-bold text-white">
                  {{ props.recordingTime }}
                </div>
                <!-- Recording stats from Rust (shown when recording) -->
                <div v-if="isRecording" class="flex gap-3 mt-2 text-xs text-gray-400">
                  <div class="w-[110px] truncate">
                    <span class="font-semibold text-gray-300">SIZE:</span> {{ props.recordingFileSize }}
                  </div>
                  <div class="w-[110px] truncate">
                    <span class="font-semibold text-gray-300">FREE:</span> {{ props.availableDiskSpace }}
                  </div>
                </div>
              </div>
            </div>

            <!-- Level Meters - takes remaining space -->
            <div class="flex-1 min-w-0">
              <HorizontalStereoMeter 
                :left-level="safeLeftLevel" 
                :right-level="safeRightLevel"
                :height="8"
                :segments="45"
              />
            </div>
          </div>
        </div>

        <!-- Recordings List -->
        <div class="flex-1 overflow-y-auto mt-4 custom-scrollbar">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-bold text-white uppercase tracking-wider">Recordings</h3>
          </div>

          <div v-if="recordings.length === 0" class="text-center py-6 text-gray-500">
            <svg class="w-6 h-6 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
            </svg>
            <p class="text-sm">No recordings yet</p>
            <p class="text-xs mt-1">Start recording to capture your mix</p>
          </div>

          <div v-else class="space-y-2">
            <div v-for="recording in recordings" :key="recording.id"
              class="bg-gray-800/50 rounded-lg p-3 border border-gray-700 hover:border-gray-600 transition-colors">
              <div class="flex items-center justify-between">
                <div class="flex-1">
                  <div class="text-sm font-semibold text-white">{{ recording.name }}</div>
                  <div class="text-xs text-gray-500 mt-1">{{ recording.duration }} • {{ recording.size }}</div>
                </div>
                
                <div class="flex items-center gap-2">
                  <button @click="downloadRecording(recording)"
                    class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold rounded transition-colors">
                    View
                  </button>
                  <button @click="deleteRecording(recording.id)"
                    class="px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-xs font-semibold rounded transition-colors">
                    Delete
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Recording Settings Modal -->
  <RecordingSettingsModal 
    v-model="showSettings"
    :settings="recordingSettings"
    @update:settings="recordingSettings = $event"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import HorizontalStereoMeter from './components/HorizontalStereoMeter.vue'
import RecordingSettingsModal from './components/RecordingSettingsModal.vue'
import { useNotifications } from '~/composables/useNotifications'

interface Props {
  modelValue: boolean
  masterLevelLeft?: number  // Subscribe to master levels from parent
  masterLevelRight?: number
  recordingTime?: string    // Recording time from Rust
  recordingFileSize?: string // File size from Rust
  availableDiskSpace?: string // Available disk space from Rust
}

interface Recording {
  id: string
  name: string
  filePath: string  // Path on disk
  duration: string
  size: string
  timestamp: number
}

const props = withDefaults(defineProps<Props>(), {
  masterLevelLeft: -60,
  masterLevelRight: -60,
  recordingTime: '00:00',
  recordingFileSize: '0 MB',
  availableDiskSpace: 'Waiting...'
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'recording-state': [isRecording: boolean]
}>()

const { confirm } = useNotifications()

// Recording state
const isRecording = ref(false)
const recordings = ref<Recording[]>([])

// Recording settings state
const showSettings = ref(false)
const recordingSettings = ref({
  format: 'wav' as 'wav' | 'mp3' | 'opus',
  sampleRate: 48000,
  bitDepth: 16,
  bitrate: 192
})

// Load settings from localStorage on mount
onMounted(() => {
  const savedSettings = localStorage.getItem('mmpro3-recording-settings')
  if (savedSettings) {
    try {
      recordingSettings.value = JSON.parse(savedSettings)
    } catch (e) {
      console.error('[Recorder] Failed to parse saved settings:', e)
    }
  }
})

// Save settings to localStorage when they change
watch(recordingSettings, (newSettings) => {
  localStorage.setItem('mmpro3-recording-settings', JSON.stringify(newSettings))
}, { deep: true })
const currentRecordingPath = ref<string>('')

// Watch isRecording and emit state changes
watch(isRecording, (newValue) => {
  emit('recording-state', newValue)
})

// Safe level values with fallback
const safeLeftLevel = computed(() => {
  const val = props.masterLevelLeft
  // Force to -60 if undefined, null, or suspicious high values
  if (val === undefined || val === null || val > 0 || val < -60) {
    return -60
  }
  return val
})

const safeRightLevel = computed(() => {
  const val = props.masterLevelRight
  // Force to -60 if undefined, null, or suspicious high values
  if (val === undefined || val === null || val > 0 || val < -60) {
    return -60
  }
  return val
})

// Recording internals (no local timer - stats come from Rust)

function closeModal() {
  emit('update:modelValue', false)
}

function toggleRecording() {
  if (isRecording.value) {
    stopRecording()
  } else {
    startRecording()
  }
}

async function startRecording() {
  console.log('[Recorder] Starting recording...')
  
  if (!window.audioEngine) {
    console.error('[Recorder] Audio engine not available')
    return
  }

  try {
    // Generate file path via IPC (main process has access to Node APIs)
    const filePath = await window.audioEngine.generateRecordingPath()
    currentRecordingPath.value = filePath
    
    // Start recording in Rust with settings
    await window.audioEngine.enableMasterTap(filePath, {
      format: recordingSettings.value.format,
      sampleRate: recordingSettings.value.sampleRate,
      bitDepth: recordingSettings.value.bitDepth,
      bitrate: recordingSettings.value.bitrate
    })
    console.log('[Recorder] Recording started with settings:', recordingSettings.value)
    
    isRecording.value = true
    // Recording stats (time, size, disk space) will come from Rust events
  } catch (error) {
    console.error('[Recorder] Error starting recording:', error)
    isRecording.value = false
  }
}

async function stopRecording() {
  console.log('[Recorder] Stopping recording...')
  
  const filePath = currentRecordingPath.value
  
  isRecording.value = false

  // Stop recording in Rust (it will save the file)
  if (window.audioEngine) {
    try {
      await window.audioEngine.disableMasterTap()
      console.log('[Recorder] Recording stopped and saved')
      
      // Wait a bit to ensure file is written to disk
      await new Promise(resolve => setTimeout(resolve, 200))
      
      // Reload recordings list to include the new file
      await loadRecordings()
    } catch (err) {
      console.error('[Recorder] Error stopping recording:', err)
    }
  }
  
  currentRecordingPath.value = ''
}

async function downloadRecording(recording: Recording) {
  // Reveal file in Finder/Explorer via IPC
  await window.audioEngine.showRecordingInFolder(recording.filePath)
}

async function deleteRecording(id: string) {
  const recording = recordings.value.find(r => r.id === id)
  if (recording) {
    // Show confirmation modal
    const confirmed = await confirm(
      `Delete recording "${recording.name}"? This action cannot be undone.`
    )
    
    if (confirmed) {
      try {
        await window.audioEngine.deleteRecordingFile(recording.filePath)
        await loadRecordings() // Reload list after deletion
        console.log('[Recorder] Recording deleted:', recording.filePath)
      } catch (e) {
        console.error('[Recorder] Failed to delete file:', e)
      }
    }
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

// Load recordings from disk
async function loadRecordings() {
  if (!window.audioEngine) return
  
  try {
    const files = await window.audioEngine.listRecordings()
    recordings.value = files.map(file => ({
      id: file.id,
      name: file.name,
      duration: '--:--', // Duration not calculated for now
      size: file.size,
      filePath: file.path,
      timestamp: new Date(file.created).getTime()
    }))
    console.log('[Recorder] Loaded recordings:', recordings.value.length)
  } catch (error) {
    console.error('[Recorder] Error loading recordings:', error)
  }
}

// Watch modal open/close to load recordings
watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    loadRecordings()
  }
})

// Cleanup on unmount
onUnmounted(async () => {
  if (window.audioEngine && isRecording.value) {
    try {
      await window.audioEngine.disableMasterTap()
    } catch (err) {
      console.error('[Recorder] Cleanup error:', err)
    }
  }
})
</script>

<style scoped>
/* Custom scrollbar styles */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: #1f2937;
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 4px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}

/* Firefox scrollbar */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: #4b5563 #1f2937;
}
</style>
