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
              <p class="text-xs text-gray-400">Record your master mix (Coming Soon)</p>
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
                :left-level="leftLevel" 
                :right-level="rightLevel"
                :height="8"
                :segments="45"
              />
            </div>
          </div>
        </div>

        <!-- Info Message -->
        <div class="bg-blue-900/30 border border-blue-600/50 rounded-lg p-4 mb-4">
          <div class="flex items-start gap-3">
            <svg class="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
            </svg>
            <div class="text-sm text-blue-200">
              <p class="font-semibold mb-1">Recorder Feature Status</p>
              <p class="text-xs text-blue-300">Master recording will capture audio directly from the Rust audio engine. This feature requires implementing a tap point in the master bus to extract stereo samples at the configured sample rate.</p>
            </div>
          </div>
        </div>

        <!-- Loaded Tracks Info -->
        <div class="mt-4">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-bold text-white uppercase tracking-wider">Playing Tracks</h3>
            <div class="text-xs text-gray-500">{{ props.loadedTracks?.length || 0 }} track{{ (props.loadedTracks?.length || 0) !== 1 ? 's' : '' }}</div>
          </div>
          <div v-if="props.loadedTracks && props.loadedTracks.length > 0" class="bg-gray-800/30 border border-gray-700 rounded-lg p-3">
            <div class="space-y-1">
              <div v-for="track in props.loadedTracks" :key="track.trackNumber" class="text-xs text-gray-400">
                Track {{ track.trackNumber }}: {{ track.fileName }}
              </div>
            </div>
          </div>
          <div v-else class="text-center py-6 text-gray-500">
            <p class="text-xs">No tracks currently playing</p>
          </div>
        </div>

        <!-- Recordings List -->
        <div class="flex-1 overflow-y-auto mt-4 custom-scrollbar">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-bold text-white uppercase tracking-wider">Recordings</h3>
            <div class="text-xs text-gray-500">{{ recordings.length }} track{{ recordings.length !== 1 ? 's' : '' }}</div>
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
                    Download
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
import { ref, inject } from 'vue'
import HorizontalStereoMeter from './components/HorizontalStereoMeter.vue'
import QualitySelector from './components/QualitySelector.vue'

interface Props {
  modelValue: boolean
  loadedTracks?: Array<{ 
    trackNumber: number, 
    fileName: string, 
    fileId: string
  }>
}

interface Recording {
  id: string
  name: string
  blob: Blob
  duration: string
  size: string
  timestamp: number
}

const props = withDefaults(defineProps<Props>(), {
  loadedTracks: () => []
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

// Inject Rust audio engine
const audioEngine = inject<any>('audioEngine', null)

// Recording state
const isRecording = ref(false)
const recordingTime = ref('00:00')
const recordingStartTime = ref(0)
const recordings = ref<Recording[]>([])
const recordingQuality = ref<string>('192') // Default: High quality

// Level monitoring (will be fed from Rust engine in future)
const leftLevel = ref(-60)
const rightLevel = ref(-60)

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
  console.log('[Recorder] Audio engine available:', !!audioEngine)
  
  // TODO: Implement recording with Rust audio engine
  // Will need to:
  // 1. Enable master tap in Rust engine
  // 2. Stream PCM samples from Rust to renderer
  // 3. Encode samples to WebM using MediaRecorder or FFmpeg
  
  isRecording.value = true
  recordingStartTime.value = Date.now()
  recordingTime.value = '00:00'

  // Start timer
  recordingInterval = window.setInterval(() => {
    const elapsed = Math.floor((Date.now() - recordingStartTime.value) / 1000)
    const minutes = Math.floor(elapsed / 60)
    const seconds = elapsed % 60
    recordingTime.value = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
    
    // Simulate level changes
    leftLevel.value = -20 + Math.random() * 15
    rightLevel.value = -20 + Math.random() * 15
  }, 100)
}

function stopRecording() {
  console.log('[Recorder] Stopping recording...')
  
  if (recordingInterval) {
    clearInterval(recordingInterval)
    recordingInterval = null
  }

  isRecording.value = false
  leftLevel.value = -60
  rightLevel.value = -60
  
  // TODO: Finalize recording and add to list
  console.log('[Recorder] Recording stopped at', recordingTime.value)
}

function downloadRecording(recording: Recording) {
  const url = URL.createObjectURL(recording.blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${recording.name}.webm`
  a.click()
  setTimeout(() => URL.revokeObjectURL(url), 100)
}

function deleteRecording(id: string) {
  recordings.value = recordings.value.filter(r => r.id !== id)
}
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
