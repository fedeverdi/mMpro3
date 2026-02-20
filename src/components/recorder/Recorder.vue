<template>
  <Transition
    enter-from-class="opacity-0"
    enter-active-class="transition-opacity duration-300"
    enter-to-class="opacity-100"
    leave-from-class="opacity-100"
    leave-active-class="transition-opacity duration-300"
    leave-to-class="opacity-0">
    <div v-if="modelValue" @click="closeModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]">
      <div @click.stop class="bg-gradient-to-br from-gray-800 to-gray-900 rounded-xl border-2 border-red-600 shadow-2xl p-6 w-[900px] max-h-[80vh] overflow-hidden flex flex-col">
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
              <p class="text-xs text-gray-400">Record your master mix</p>
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
          <div class="flex items-center justify-between mb-4">
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
            </div>

            <!-- Level Meter -->
            <div class="flex items-center gap-2">
              <div class="text-xs text-gray-400">Level</div>
              <div class="w-32 h-2 bg-gray-700 rounded-full overflow-hidden">
                <div class="h-full bg-gradient-to-r from-green-500 via-yellow-500 to-red-500 transition-all duration-100"
                  :style="{ width: `${levelPercent}%` }"></div>
              </div>
            </div>
          </div>

          <div class="text-xs text-gray-500">
            {{ isRecording ? 'Click STOP to finish recording' : 'Click REC to start recording' }}
          </div>
        </div>

        <!-- Waveform Display -->
        <WaveformDisplay 
          ref="waveformDisplay"
          :analyser="analyserNode" 
          :is-recording="isRecording" 
          @level-update="levelPercent = $event" 
        />

        <!-- Recordings List -->
        <div class="flex-1 overflow-y-auto">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-bold text-white uppercase tracking-wider">Recordings</h3>
            <div class="text-xs text-gray-500">{{ recordings.length }} track{{ recordings.length !== 1 ? 's' : '' }}</div>
          </div>

          <div v-if="recordings.length === 0" class="text-center py-12 text-gray-500">
            <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
import { ref, computed, watch, onUnmounted, toRaw, nextTick, type Ref } from 'vue'
import WaveformDisplay from './components/WaveformDisplay.vue'

interface Props {
  modelValue: boolean
  isRecording?: boolean
  audioNode?: Ref<any> | any
  tone?: Ref<any> | any
}

interface Recording {
  id: string
  name: string
  blob: Blob
  duration: string
  size: string
  timestamp: number
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'update:isRecording': [value: boolean]
}>()

// Recording state
const isRecording = ref(false)
const recordingTime = ref('00:00')
const recordingStartTime = ref(0)
const recordings = ref<Recording[]>([])

// Level monitoring
const levelPercent = ref(0)

// Recording internals
let recorder: { mediaRecorder: MediaRecorder; dest: MediaStreamAudioDestinationNode } | null = null
let recordingInterval: number | null = null

// Waveform
const waveformDisplay = ref<InstanceType<typeof WaveformDisplay> | null>(null)
const analyserNode = ref<AnalyserNode | null>(null)

// Computed values
const toneValue = computed(() => props.tone?.value ?? props.tone)
const audioNodeValue = computed(() => props.audioNode?.value ?? props.audioNode)

function closeModal() {
  // Allow closing even while recording
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
  const tone = toneValue.value
  const audioNode = audioNodeValue.value

  if (!tone || !audioNode) {
    console.error('[Recorder] Tone.js or audioNode not available')
    return
  }

  try {
    // Ensure audio context is running
    if (tone.context.state !== 'running') {
      await tone.start()
    }

    // Create MediaStreamDestination from Web Audio context
    const audioContext = tone.context.rawContext as AudioContext
    const dest = audioContext.createMediaStreamDestination()

    // Remove Vue reactivity and get native audio node
    const rawNode = toRaw(audioNode)
    
    // Create analyser for waveform visualization
    const analyser = audioContext.createAnalyser()
    analyser.fftSize = 2048
    analyser.smoothingTimeConstant = 0.8
    analyserNode.value = analyser
    
    // Connect using Tone.js and native nodes properly
    // Get the native output node from Tone.js Merge
    try {
      // Tone.js wraps nodes - we need to get the actual Web Audio ChannelMergerNode
      const mergerOutput = rawNode.output ? toRaw(rawNode.output) : rawNode
      
      // Connect: Tone Merge output -> analyser -> destination
      mergerOutput.connect(analyser)
      analyser.connect(dest)
      
      // IMPORTANT: Also keep the Merge connected to Tone destination
      // so the audio graph stays active in Tone.js
      rawNode.toDestination()
      
    } catch (err) {
      console.error('[Recorder] Connection error:', err)
      throw err
    }

    // Create MediaRecorder
    const mediaRecorder = new MediaRecorder(dest.stream, {
      mimeType: 'audio/webm;codecs=opus'
    })

    const chunks: Blob[] = []

    mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) {
        chunks.push(e.data)
      }
    }
    
    mediaRecorder.onerror = (e) => {
      console.error('[Recorder] MediaRecorder error:', e)
    }

    mediaRecorder.onstop = () => {
      const blob = new Blob(chunks, { type: 'audio/webm' })
      const duration = recordingTime.value
      const size = formatFileSize(blob.size)
      
      // Add to recordings list
      const now = new Date()
      const timestamp = now.toISOString().replace(/[:.]/g, '-').slice(0, -5)
      
      recordings.value.unshift({
        id: Date.now().toString(),
        name: `Recording_${timestamp}`,
        blob,
        duration,
        size,
        timestamp: Date.now()
      })

      // Cleanup audio connections after recording is fully stopped
      const audioNode = audioNodeValue.value
      if (audioNode && recorder && recorder.dest && analyserNode.value) {
        try {
          const rawNode = toRaw(audioNode)
          const mergerOutput = rawNode.output ? toRaw(rawNode.output) : rawNode
          mergerOutput.disconnect(analyserNode.value)
          analyserNode.value.disconnect(recorder.dest)
        } catch (e) {
          console.warn('[Recorder] Cleanup connection error:', e)
        }
      }

      // Clear analyser and recorder references
      analyserNode.value = null
      recorder = null
    }

    mediaRecorder.start()
    recorder = { mediaRecorder, dest }

    isRecording.value = true
    emit('update:isRecording', true)
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
    emit('update:isRecording', false)
  }
}

function stopRecording() {
  if (!recorder) return

  try {
    if (recorder.mediaRecorder.state !== 'inactive') {
      recorder.mediaRecorder.stop()
    }

    // Clear timer
    if (recordingInterval) {
      clearInterval(recordingInterval)
      recordingInterval = null
    }

    isRecording.value = false
    emit('update:isRecording', false)
  } catch (error) {
    console.error('[Recorder] Error stopping recording:', error)
    isRecording.value = false
    emit('update:isRecording', false)
  }
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

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

// When modal reopens during active recording, restart waveform
watch(() => props.modelValue, async (isOpen) => {
  if (!isOpen) {
    // Reset level when modal closes
    levelPercent.value = 0
  } else if (isOpen && isRecording.value) {
    // Modal reopened during active recording - restart waveform
    // Wait for component to be mounted after transition
    await nextTick()
    waveformDisplay.value?.restart()
  }
})

// Cleanup
onUnmounted(() => {
  if (recorder) {
    try {
      if (recorder.mediaRecorder.state !== 'inactive') {
        recorder.mediaRecorder.stop()
      }
      const audioNode = audioNodeValue.value
      if (audioNode && recorder.dest && analyserNode.value) {
        const rawNode = toRaw(audioNode)
        const mergerOutput = rawNode.output ? toRaw(rawNode.output) : rawNode
        mergerOutput.disconnect(analyserNode.value)
        analyserNode.value.disconnect(recorder.dest)
      }
    } catch (e) {
      console.warn('[Recorder] Cleanup error:', e)
    }
  }

  if (recordingInterval) {
    clearInterval(recordingInterval)
  }
})
</script>
