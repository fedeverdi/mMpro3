<template>
  <div class="space-y-2">
    <div v-if="isLoading" class="flex items-center justify-center py-4">
      <svg class="animate-spin h-6 w-6 text-blue-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
        </path>
      </svg>
    </div>

    <div v-else-if="playlistFiles.length === 0" class="text-center py-4 text-gray-500 text-sm">
      <p>No tracks in this playlist</p>
      <p class="text-xs mt-1">Use the library view to add tracks</p>
    </div>

    <div v-else v-for="(file, index) in playlistFiles" :key="file.id"
      class="flex items-center gap-3 p-2 bg-gray-900 rounded border border-gray-700 hover:border-gray-600 transition-colors">
      <!-- Track number -->
      <div class="text-gray-500 text-xs w-6 text-right">{{ index + 1 }}</div>

      <!-- Artwork or icon -->
      <div v-if="file.artwork" class="w-10 h-10 flex-shrink-0 rounded overflow-hidden bg-gray-800">
        <img :src="file.artwork" :alt="file.title || file.fileName" class="w-full h-full object-cover" />
      </div>
      <svg v-else class="w-10 h-10 text-blue-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
      </svg>

      <!-- Track info -->
      <div class="flex-1 min-w-0">
        <p class="text-white text-sm font-medium truncate">{{ file.title || file.fileName }}</p>
        <p class="text-gray-500 text-xs truncate">
          <span v-if="file.artist" class="text-blue-400">{{ file.artist }}</span>
        </p>
      </div>

      <!-- Actions -->
      <div class="flex gap-1">
        <button @click="$emit('load-file', file)"
          class="p-1.5 border border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 rounded text-gray-300 hover:text-blue-400 transition-all"
          title="Load this track">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
        </button>
        <button @click="handleRemoveFile(file.id)"
          class="p-1.5 border border-gray-600 hover:border-red-500 hover:bg-red-500/10 rounded text-gray-300 hover:text-red-400 transition-all"
          title="Remove from playlist">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { usePlaylist } from '~/composables/usePlaylist'
import type { StoredAudioFile } from '~/composables/useAudioFileStorage'

const props = defineProps<{
  playlistId: string
}>()

const emit = defineEmits<{
  'load-file': [file: StoredAudioFile]
  'remove-file': [fileId: string]
}>()

const { getPlaylistFiles, removeFilesFromPlaylist } = usePlaylist()

const playlistFiles = ref<StoredAudioFile[]>([])
const isLoading = ref(false)

async function loadPlaylistFiles() {
  isLoading.value = true
  try {
    playlistFiles.value = await getPlaylistFiles(props.playlistId)
  } catch (error) {
    console.error('Failed to load playlist files:', error)
  } finally {
    isLoading.value = false
  }
}

async function handleRemoveFile(fileId: string) {
  try {
    await removeFilesFromPlaylist(props.playlistId, [fileId])
    await loadPlaylistFiles()
  } catch (error) {
    console.error('Failed to remove file from playlist:', error)
  }
}

onMounted(() => {
  loadPlaylistFiles()
})

watch(() => props.playlistId, () => {
  loadPlaylistFiles()
})
</script>
