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
          <input type="file" ref="fileInput" @change="handleFileUpload" accept="audio/*" multiple class="hidden" />
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
            <span>{{ isUploading ? 'Uploading...' : 'Upload Audio Files' }}</span>
          </button>
          <div v-if="uploadProgress" class="text-sm text-gray-400">
            {{ uploadProgress }}
          </div>
        </div>
      </div>

      <!-- Search Bar -->
      <div class="px-4 pb-3 border-b border-gray-700">
        <div class="flex gap-2 mb-2">
          <button @click="viewMode = 'all'"
            :class="[
              'flex-1 px-3 py-2 rounded text-sm font-medium transition-colors',
              viewMode === 'all' 
                ? 'bg-blue-600 text-white' 
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]">
            All Files
          </button>
          <button @click="viewMode = 'byArtist'"
            :class="[
              'flex-1 px-3 py-2 rounded text-sm font-medium transition-colors',
              viewMode === 'byArtist' 
                ? 'bg-blue-600 text-white' 
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]">
            By Artist
          </button>
        </div>
        <div class="flex gap-2">
          <div class="relative flex-1">
            <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input v-model="searchQuery" type="text" placeholder="Search by title, artist or file name..."
              class="w-full pl-10 pr-4 py-2 bg-gray-900 border border-gray-700 rounded text-white text-sm placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors" />
            <button v-if="searchQuery" @click="searchQuery = ''"
              class="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white transition-colors">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <!-- Layout Toggle -->
          <div class="flex gap-1 bg-gray-900 border border-gray-700 rounded p-1">
            <button @click="viewLayout = 'list'"
              :class="[
                'p-2 rounded transition-colors',
                viewLayout === 'list' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-white'
              ]"
              title="List view">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
            <button @click="viewLayout = 'grid'"
              :class="[
                'p-2 rounded transition-colors',
                viewLayout === 'grid' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-white'
              ]"
              title="Grid view">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1H5a1 1 0 01-1-1v-4zM14 15a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
              </svg>
            </button>
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

        <div v-else-if="filteredFiles.length === 0 && files.length > 0" class="text-center py-12">
          <svg class="w-16 h-16 mx-auto text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <p class="text-gray-400 text-lg">No files match your search</p>
          <p class="text-gray-500 text-sm mt-2">Try a different search term</p>
        </div>

        <div v-else-if="files.length === 0" class="text-center py-12">
          <svg class="w-16 h-16 mx-auto text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-gray-400 text-lg">No audio files in library</p>
          <p class="text-gray-500 text-sm mt-2">Upload your first audio file to get started</p>
        </div>

        <!-- View: All Files - List -->
        <div v-else-if="viewMode === 'all' && viewLayout === 'list'" class="grid gap-2">
          <div v-for="file in filteredFiles" :key="file.id"
            class="flex items-center justify-between p-3 bg-gray-900 rounded-lg border border-gray-700 hover:border-blue-500 transition-colors group">
            <div class="flex items-center gap-3 flex-1 min-w-0">
              <!-- Artwork or default icon -->
              <div v-if="file.artwork" class="w-12 h-12 flex-shrink-0 rounded overflow-hidden bg-gray-800">
                <img :src="file.artwork" :alt="file.title || file.fileName" class="w-full h-full object-cover" />
              </div>
              <svg v-else class="w-12 h-12 text-blue-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
              </svg>
              <div class="flex-1 min-w-0">
                <p class="text-white font-medium truncate">{{ file.title || file.fileName }}</p>
                <p class="text-gray-500 text-xs">
                  <span v-if="file.artist" class="text-blue-400">{{ file.artist }}</span>
                  <span v-if="file.artist"> • </span>
                  {{ formatDate(file.timestamp) }} • {{ formatSize(file) }}
                </p>
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

        <!-- View: All Files - Grid -->
        <div v-else-if="viewMode === 'all' && viewLayout === 'grid'" class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
          <div v-for="file in filteredFiles" :key="file.id"
            class="bg-gray-900 rounded-lg border border-gray-700 hover:border-blue-500 transition-all overflow-hidden group cursor-pointer">
            <!-- Artwork -->
            <div class="aspect-square bg-gray-800 relative overflow-hidden">
              <img v-if="file.artwork" :src="file.artwork" :alt="file.title || file.fileName" 
                class="w-full h-full object-cover group-hover:scale-105 transition-transform" />
              <div v-else class="w-full h-full flex items-center justify-center">
                <svg class="w-16 h-16 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                </svg>
              </div>
              <!-- Overlay buttons -->
              <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-60 transition-all flex items-center justify-center gap-2 opacity-0 group-hover:opacity-100">
                <button @click="$emit('select-file', file)"
                  class="px-3 py-1.5 bg-green-600 hover:bg-green-500 rounded text-xs font-semibold text-white transition-colors"
                  title="Load to track">
                  Load
                </button>
                <button @click.stop="confirmDelete(file)"
                  class="px-3 py-1.5 bg-red-600 hover:bg-red-500 rounded text-xs font-semibold text-white transition-colors"
                  title="Delete file">
                  Delete
                </button>
              </div>
            </div>
            <!-- Info -->
            <div class="p-2">
              <p class="text-white text-sm font-medium truncate" :title="file.title || file.fileName">
                {{ file.title || file.fileName }}
              </p>
              <p class="text-blue-400 text-xs truncate" :title="file.artist">
                {{ file.artist || 'Unknown Artist' }}
              </p>
              <p class="text-gray-500 text-xs mt-1">
                {{ formatSize(file) }}
              </p>
            </div>
          </div>
        </div>

        <!-- View: By Artist -->
        <div v-else-if="viewMode === 'byArtist'" class="grid gap-3">
          <div v-for="artistGroup in filesByArtist" :key="artistGroup.artist" class="border border-gray-700 rounded-lg overflow-hidden">
            <!-- Artist Header -->
            <button @click="toggleArtist(artistGroup.artist)"
              class="w-full flex items-center justify-between p-3 bg-gray-900 hover:bg-gray-800 transition-colors">
              <div class="flex items-center gap-3">
                <svg 
                  :class="[
                    'w-5 h-5 text-gray-400 transition-transform',
                    expandedArtists.has(artistGroup.artist) ? 'rotate-90' : ''
                  ]" 
                  fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
                <svg class="w-6 h-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
                <div class="text-left">
                  <p class="text-white font-semibold">{{ artistGroup.artist }}</p>
                  <p class="text-gray-500 text-xs">{{ artistGroup.count }} track{{ artistGroup.count !== 1 ? 's' : '' }}</p>
                </div>
              </div>
            </button>
            
            <!-- Artist Files - List View -->
            <div v-if="expandedArtists.has(artistGroup.artist) && viewLayout === 'list'" class="bg-gray-950 border-t border-gray-700">
              <div v-for="file in artistGroup.files" :key="file.id"
                class="flex items-center justify-between p-3 border-b border-gray-800 last:border-b-0 hover:bg-gray-900 transition-colors">
                <div class="flex items-center gap-3 flex-1 min-w-0 ml-8">
                  <!-- Artwork or default icon -->
                  <div v-if="file.artwork" class="w-10 h-10 flex-shrink-0 rounded overflow-hidden bg-gray-800">
                    <img :src="file.artwork" :alt="file.title || file.fileName" class="w-full h-full object-cover" />
                  </div>
                  <svg v-else class="w-10 h-10 text-blue-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                  </svg>
                  <div class="flex-1 min-w-0">
                    <p class="text-white text-sm truncate">{{ file.title || file.fileName }}</p>
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

            <!-- Artist Files - Grid View -->
            <div v-if="expandedArtists.has(artistGroup.artist) && viewLayout === 'grid'" class="bg-gray-950 border-t border-gray-700 p-3">
              <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
                <div v-for="file in artistGroup.files" :key="file.id"
                  class="bg-gray-900 rounded-lg border border-gray-800 hover:border-blue-500 transition-all overflow-hidden group cursor-pointer">
                  <!-- Artwork -->
                  <div class="aspect-square bg-gray-800 relative overflow-hidden">
                    <img v-if="file.artwork" :src="file.artwork" :alt="file.title || file.fileName" 
                      class="w-full h-full object-cover group-hover:scale-105 transition-transform" />
                    <div v-else class="w-full h-full flex items-center justify-center">
                      <svg class="w-12 h-12 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                      </svg>
                    </div>
                    <!-- Overlay buttons -->
                    <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-60 transition-all flex items-center justify-center gap-2 opacity-0 group-hover:opacity-100">
                      <button @click="$emit('select-file', file)"
                        class="px-3 py-1.5 bg-green-600 hover:bg-green-500 rounded text-xs font-semibold text-white transition-colors"
                        title="Load to track">
                        Load
                      </button>
                      <button @click.stop="confirmDelete(file)"
                        class="px-3 py-1.5 bg-red-600 hover:bg-red-500 rounded text-xs font-semibold text-white transition-colors"
                        title="Delete file">
                        Delete
                      </button>
                    </div>
                  </div>
                  <!-- Info -->
                  <div class="p-2">
                    <p class="text-white text-xs font-medium truncate" :title="file.title || file.fileName">
                      {{ file.title || file.fileName }}
                    </p>
                    <p class="text-gray-500 text-[0.65rem] mt-1">
                      {{ formatSize(file) }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-gray-700 flex items-center justify-between">
        <div class="text-sm text-gray-400">
          <span v-if="searchQuery && filteredFiles.length !== files.length">
            {{ filteredFiles.length }} of {{ files.length }} file{{ files.length !== 1 ? 's' : '' }}
          </span>
          <span v-else>
            {{ files.length }} file{{ files.length !== 1 ? 's' : '' }} in library
          </span>
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
  artist?: string
  title?: string
  artwork?: string
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
const searchQuery = ref('')
const viewMode = ref<'all' | 'byArtist'>('all')
const viewLayout = ref<'list' | 'grid'>('list')
const expandedArtists = ref<Set<string>>(new Set())

const sortedFiles = computed(() => {
  return [...files.value].sort((a, b) => b.timestamp - a.timestamp)
})

const filteredFiles = computed(() => {
  if (!searchQuery.value.trim()) {
    return sortedFiles.value
  }
  
  const query = searchQuery.value.toLowerCase()
  return sortedFiles.value.filter(file => 
    file.fileName.toLowerCase().includes(query) ||
    file.artist?.toLowerCase().includes(query) ||
    file.title?.toLowerCase().includes(query)
  )
})

// Group files by artist
const filesByArtist = computed(() => {
  const grouped = new Map<string, StoredAudioFile[]>()
  
  filteredFiles.value.forEach(file => {
    const artist = file.artist || 'Unknown Artist'
    if (!grouped.has(artist)) {
      grouped.set(artist, [])
    }
    grouped.get(artist)!.push(file)
  })
  
  // Convert to array and sort by artist name
  return Array.from(grouped.entries())
    .sort(([artistA], [artistB]) => artistA.localeCompare(artistB))
    .map(([artist, files]) => ({
      artist,
      files: files.sort((a, b) => b.timestamp - a.timestamp),
      count: files.length
    }))
})

function toggleArtist(artist: string) {
  if (expandedArtists.value.has(artist)) {
    expandedArtists.value.delete(artist)
  } else {
    expandedArtists.value.add(artist)
  }
}

async function loadFiles() {
  isLoading.value = true
  try {
    files.value = await getAllAudioFiles()
    // Auto-expand all artists when in byArtist mode
    if (viewMode.value === 'byArtist') {
      const artists = new Set(files.value.map(f => f.artist || 'Unknown Artist'))
      expandedArtists.value = new Set(artists)
    }
  } catch (error) {
    console.error('Failed to load audio files:', error)
  } finally {
    isLoading.value = false
  }
}

async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const selectedFiles = target.files
  if (!selectedFiles || selectedFiles.length === 0) return

  isUploading.value = true
  const totalFiles = selectedFiles.length
  let uploadedCount = 0
  let skippedCount = 0
  let failedCount = 0

  try {
    // Process each file
    for (let i = 0; i < selectedFiles.length; i++) {
      const file = selectedFiles[i]
      uploadProgress.value = `Checking ${file.name} (${i + 1}/${totalFiles})...`

      try {
        // Check for duplicates
        const fileArrayBuffer = await file.arrayBuffer()
        const isDuplicate = await checkIfDuplicate(fileArrayBuffer, file.name, file.size)
        
        if (isDuplicate) {
          uploadProgress.value = `Skipped: ${file.name} (already exists)`
          skippedCount++
          await new Promise(resolve => setTimeout(resolve, 500))
          continue
        }

        uploadProgress.value = `Uploading ${file.name} (${i + 1}/${totalFiles})...`
        
        // Need to re-create File object from ArrayBuffer since we consumed it
        const newFile = new File([fileArrayBuffer], file.name, { type: file.type })
        await saveAudioFile(newFile)
        uploadedCount++
        
      } catch (error) {
        console.error(`Failed to upload ${file.name}:`, error)
        failedCount++
      }
    }
    
    // Show summary
    const summaryParts = []
    if (uploadedCount > 0) summaryParts.push(`${uploadedCount} uploaded`)
    if (skippedCount > 0) summaryParts.push(`${skippedCount} skipped`)
    if (failedCount > 0) summaryParts.push(`${failedCount} failed`)
    
    uploadProgress.value = summaryParts.join(', ') + '!'
    
    // Reload files list
    await loadFiles()
    
    // Clear input
    if (fileInput.value) {
      fileInput.value.value = ''
    }

    setTimeout(() => {
      uploadProgress.value = ''
    }, 3000)
  } catch (error) {
    console.error('Failed to upload files:', error)
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

// Auto-expand all artists when switching to byArtist view
watch(viewMode, (newMode) => {
  if (newMode === 'byArtist') {
    const artists = new Set(files.value.map(f => f.artist || 'Unknown Artist'))
    expandedArtists.value = new Set(artists)
  }
})
</script>
