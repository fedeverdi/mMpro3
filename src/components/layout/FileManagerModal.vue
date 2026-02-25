<template>
  <div v-if="modelValue" class="fixed inset-0 bg-black bg-opacity-80 flex items-center justify-center z-50"
    @click.self="close">
    <div class="bg-gray-800 rounded-lg shadow-2xl w-[800px] max-h-[600px] flex flex-col border border-gray-700">
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-700">
        <div class="flex items-center gap-2">
          <svg class="w-6 h-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          <h2 class="text-xl font-bold text-white">Audio Library</h2>
        </div>
        <button @click="close"
          class="text-gray-400 hover:text-white transition-colors">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Upload Section -->
      <div class="p-4 border-b border-gray-700">
        <div class="flex items-center gap-3">
          <input type="file" ref="fileInput" @change="handleFileUpload" accept="audio/*" class="hidden" />
          <button @click="fileInput?.click()" :disabled="isUploading"
            class="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-gray-600 disabled:cursor-not-allowed rounded text-white font-semibold transition-colors">
            <svg v-if="!isUploading" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <svg v-else class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
              </path>
            </svg>
            <span>{{ isUploading ? 'Uploading...' : 'Upload Audio File' }}</span>
          </button>
          <div v-if="uploadProgress" class="text-sm text-gray-400">
            {{ uploadProgress }}
          </div>
        </div>
      </div>

      <!-- Files List -->
      <div class="flex-1 overflow-y-auto p-4">
        <div v-if="isLoading" class="flex items-center justify-center h-32">
          <svg class="animate-spin h-8 w-8 text-blue-500" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
            </path>
          </svg>
        </div>

        <div v-else-if="files.length === 0" class="text-center py-12">
          <svg class="w-16 h-16 mx-auto text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-gray-400 text-lg">No audio files in library</p>
          <p class="text-gray-500 text-sm mt-2">Upload your first audio file to get started</p>
        </div>

        <div v-else class="grid gap-2">
          <div v-for="file in sortedFiles" :key="file.id"
            class="flex items-center justify-between p-3 bg-gray-900 rounded-lg border border-gray-700 hover:border-blue-500 transition-colors group">
            <div class="flex items-center gap-3 flex-1 min-w-0">
              <svg class="w-8 h-8 text-blue-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
              </svg>
              <div class="flex-1 min-w-0">
                <p class="text-white font-medium truncate">{{ file.fileName }}</p>
                <p class="text-gray-500 text-xs">{{ formatDate(file.timestamp) }} • {{ formatSize(file) }}</p>
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button @click="$emit('select-file', file)"
                class="px-3 py-1 bg-green-600 hover:bg-green-500 rounded text-xs font-semibold text-white transition-colors"
                title="Load to track">
                Load
              </button>
              <button @click="confirmDelete(file)"
                class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded text-xs font-semibold text-white transition-colors"
                title="Delete file">
                Delete
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-gray-700 flex items-center justify-between">
        <div class="text-sm text-gray-400">
          {{ files.length }} file{{ files.length !== 1 ? 's' : '' }} in library
        </div>
        <button @click="close"
          class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-white font-semibold transition-colors">
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAudioFileStorage } from '~/composables/useAudioFileStorage'

interface StoredAudioFile {
  id: string
  fileName: string
  arrayBuffer: ArrayBuffer
  mimeType: string
  timestamp: number
}

interface Props {
  modelValue: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'select-file': [file: StoredAudioFile]
}>()

const { saveAudioFile, getAllAudioFiles, deleteAudioFile } = useAudioFileStorage()

const files = ref<StoredAudioFile[]>([])
const isLoading = ref(false)
const isUploading = ref(false)
const uploadProgress = ref('')
const fileInput = ref<HTMLInputElement | null>(null)

const sortedFiles = computed(() => {
  return [...files.value].sort((a, b) => b.timestamp - a.timestamp)
})

async function loadFiles() {
  isLoading.value = true
  try {
    files.value = await getAllAudioFiles()
  } catch (error) {
    console.error('Failed to load audio files:', error)
  } finally {
    isLoading.value = false
  }
}

async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  isUploading.value = true
  uploadProgress.value = `Checking ${file.name}...`

  try {
    // Check for duplicates
    const fileArrayBuffer = await file.arrayBuffer()
    const isDuplicate = await checkIfDuplicate(fileArrayBuffer, file.name, file.size)
    
    if (isDuplicate) {
      uploadProgress.value = 'File already exists in library!'
      
      // Clear input
      if (fileInput.value) {
        fileInput.value.value = ''
      }

      setTimeout(() => {
        uploadProgress.value = ''
        isUploading.value = false
      }, 3000)
      return
    }

    uploadProgress.value = `Uploading ${file.name}...`
    
    // Need to re-create File object from ArrayBuffer since we consumed it
    const newFile = new File([fileArrayBuffer], file.name, { type: file.type })
    const fileId = await saveAudioFile(newFile)
    uploadProgress.value = 'File uploaded successfully!'
    
    // Reload files list
    await loadFiles()
    
    // Clear input
    if (fileInput.value) {
      fileInput.value.value = ''
    }

    setTimeout(() => {
      uploadProgress.value = ''
    }, 2000)
  } catch (error) {
    console.error('Failed to upload file:', error)
    uploadProgress.value = 'Upload failed!'
    setTimeout(() => {
      uploadProgress.value = ''
    }, 3000)
  } finally {
    isUploading.value = false
  }
}

// Check if file is already in library by comparing ArrayBuffers
async function checkIfDuplicate(newBuffer: ArrayBuffer, fileName: string, fileSize: number): Promise<boolean> {
  // First check by file name and size for quick rejection
  const potentialDuplicates = files.value.filter(
    f => f.fileName === fileName && f.arrayBuffer.byteLength === fileSize
  )
  
  if (potentialDuplicates.length === 0) {
    return false
  }
  
  // If name and size match, compare ArrayBuffers byte by byte
  const newBytes = new Uint8Array(newBuffer)
  
  for (const existingFile of potentialDuplicates) {
    const existingBytes = new Uint8Array(existingFile.arrayBuffer)
    
    // Compare buffers
    if (areArrayBuffersEqual(newBytes, existingBytes)) {
      return true
    }
  }
  
  return false
}

// Compare two Uint8Arrays for equality
function areArrayBuffersEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false
  
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false
  }
  
  return true
}

async function confirmDelete(file: StoredAudioFile) {
  if (!confirm(`Delete "${file.fileName}"?`)) return

  try {
    await deleteAudioFile(file.id)
    await loadFiles()
  } catch (error) {
    console.error('Failed to delete file:', error)
  }
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function formatSize(file: StoredAudioFile): string {
  const bytes = file.arrayBuffer.byteLength
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function close() {
  emit('update:modelValue', false)
}

// Load files when modal opens
watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    loadFiles()
  }
})
</script>
