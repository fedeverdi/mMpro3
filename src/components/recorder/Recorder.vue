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
          <button @click="closeModal" class="text-gray-400 hover:text-white transition-colors">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Recording Controls -->
        <div class="bg-gray-900/50 rounded-lg p-6 mb-6 border border-gray-700">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4">
              <button @click="toggleRecording"
                class="w-16 h-16 rounded-full flex items-center justify-center text-white font-bold transition-all shadow-lg"
                :class="isRecording ? 'bg-red-700 animate-pulse' : 'bg-red-600 hover:bg-red-700'">
                <svg v-if="!isRecording" class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="8" />
                </svg>
                <svg v-else class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                  <rect x="6" y="6" width="12" height="12" />
                </svg>
              </button>
              
              <div>
                <div class="text-xs text-gray-400 uppercase tracking-wider mb-1">
                  {{ isRecording ? 'Recording' : 'Ready' }}
                </div>
                <div class="text-3xl font-mono font-bold text-white">
                  {{ recordingTime }}
                </div>
              </div>

              <!-- Quality Selector -->
              <QualitySelector 
                v-model="recordingQuality"
                :disabled="isRecording"
                class="ml-4"
              />
            </div>

            <!-- Level Meters -->
            <div class="flex-1 pl-6">
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
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import HorizontalStereoMeter from './components/HorizontalStereoMeter.vue'
import QualitySelector from './components/QualitySelector.vue'

interface Props {
  modelValue: boolean
  masterLevelLeft?: number  // Subscribe to master levels from parent
  masterLevelRight?: number
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
  masterLevelRight: -60
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'recording-state': [isRecording: boolean]
}>()

// Recording state
const isRecording = ref(false)
const recordingTime = ref('00:00')
const recordingStartTime = ref(0)
const recordings = ref<Recording[]>([])
const recordingQuality = ref<string>('192') // Default: High quality
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

// Recording internals
let recordingInterval: number | null = null

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
    
    // Start recording in Rust (it will save the file)
    await window.audioEngine.enableMasterTap(filePath)
    console.log('[Recorder] Recording started, will save to:', filePath)
    
    isRecording.value = true
    recordingStartTime.value = Date.now()
    recordingTime.value = '00:00'

    // Start timer
    recordingInterval = window.setInterval(() => {
      const elapsed = Math.floor((Date.now() - recordingStartTime.value) / 1000)
      const minutes = Math.floor(elapsed / 60)
      const seconds = elapsed % 60
      recordingTime.value = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
    }, 1000)
  } catch (error) {
    console.error('[Recorder] Error starting recording:', error)
    isRecording.value = false
  }
}

async function stopRecording() {
  console.log('[Recorder] Stopping recording...')
  
  if (recordingInterval) {
    clearInterval(recordingInterval)
    recordingInterval = null
  }

  const duration = recordingTime.value
  const filePath = currentRecordingPath.value
  
  isRecording.value = false

  // Stop recording in Rust (it will save the file)
  if (window.audioEngine) {
    try {
      await window.audioEngine.disableMasterTap()
      console.log('[Recorder] Recording stopped and saved')
      
      // Add to recordings list (file is already saved by Rust)
      if (filePath) {
        // Get file info via IPC
        const fileInfo = await window.audioEngine.getRecordingFileInfo(filePath)
        
        recordings.value.unshift({
          id: Date.now().toString(),
          name: fileInfo.name,
          filePath,
          duration,
          size: fileInfo.size,
          timestamp: Date.now()
        })
      }
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
    try {
      await window.audioEngine.deleteRecordingFile(recording.filePath)
      recordings.value = recordings.value.filter(r => r.id !== id)
      console.log('[Recorder] Recording deleted:', recording.filePath)
    } catch (e) {
      console.error('[Recorder] Failed to delete file:', e)
    }
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

// Cleanup on unmount
onUnmounted(async () => {
  if (recordingInterval) {
    clearInterval(recordingInterval)
  }
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
