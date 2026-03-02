import { ref } from 'vue'
import { useAudioFileStorage, type StoredAudioFile } from './useAudioFileStorage'

const { getAllAudioFiles } = useAudioFileStorage()

// Playlist structure - stores only file references, not the actual audio data
export interface Playlist {
  id: string
  name: string
  fileIds: string[] // References to files in the audio library
  createdAt: number
  updatedAt: number
}

export function usePlaylist() {
  const playlists = ref<Playlist[]>([])
  const api = (window as any).audioEngine

  // Create a new playlist
  async function createPlaylist(name: string, fileIds: string[] = []): Promise<Playlist> {
    const playlist: Playlist = {
      id: `playlist_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      name,
      fileIds,
      createdAt: Date.now(),
      updatedAt: Date.now()
    }

    await api.savePlaylist(playlist)
    playlists.value.push(playlist)
    return playlist
  }

  // Get all playlists
  async function getAllPlaylists(): Promise<Playlist[]> {
    const allPlaylists = await api.listPlaylists()
    playlists.value = allPlaylists
    return playlists.value
  }

  // Get a specific playlist by ID
  async function getPlaylist(id: string): Promise<Playlist | null> {
    const playlist = await api.getPlaylist(id)
    return playlist
  }

  // Update a playlist (rename or modify file list)
  async function updatePlaylist(id: string, updates: Partial<Omit<Playlist, 'id' | 'createdAt'>>): Promise<void> {
    const existing = await getPlaylist(id)
    
    if (!existing) {
      throw new Error('Playlist not found')
    }

    const updated: Playlist = {
      ...existing,
      ...updates,
      updatedAt: Date.now()
    }

    await api.savePlaylist(updated)
    
    const index = playlists.value.findIndex(p => p.id === id)
    if (index !== -1) {
      playlists.value[index] = updated
    }
  }

  // Add files to a playlist
  async function addFilesToPlaylist(playlistId: string, fileIds: string[]): Promise<void> {
    const playlist = await getPlaylist(playlistId)
    if (!playlist) {
      throw new Error('Playlist not found')
    }

    // Add only unique file IDs
    const uniqueIds = [...new Set([...playlist.fileIds, ...fileIds])]
    await updatePlaylist(playlistId, { fileIds: uniqueIds })
  }

  // Remove files from a playlist
  async function removeFilesFromPlaylist(playlistId: string, fileIds: string[]): Promise<void> {
    const playlist = await getPlaylist(playlistId)
    if (!playlist) {
      throw new Error('Playlist not found')
    }

    const updatedFileIds = playlist.fileIds.filter(id => !fileIds.includes(id))
    await updatePlaylist(playlistId, { fileIds: updatedFileIds })
  }

  // Reorder files in a playlist
  async function reorderPlaylist(playlistId: string, newFileIds: string[]): Promise<void> {
    await updatePlaylist(playlistId, { fileIds: newFileIds })
  }

  // Delete a playlist
  async function deletePlaylist(id: string): Promise<void> {
    await api.deletePlaylist(id)
    playlists.value = playlists.value.filter(p => p.id !== id)
  }

  // Get actual audio files from a playlist (resolves file IDs to StoredAudioFile objects)
  async function getPlaylistFiles(playlistId: string): Promise<StoredAudioFile[]> {
    const playlist = await getPlaylist(playlistId)
    if (!playlist) {
      throw new Error('Playlist not found')
    }

    const allFiles = await getAllAudioFiles()
    const filesMap = new Map(allFiles.map((file: StoredAudioFile) => [file.id, file]))

    // Resolve fileIds to actual files, maintaining order and filtering out deleted files
    return playlist.fileIds
      .map(id => filesMap.get(id))
      .filter((file): file is StoredAudioFile => file !== undefined)
  }

  // Rename a playlist
  async function renamePlaylist(id: string, newName: string): Promise<void> {
    await updatePlaylist(id, { name: newName })
  }

  return {
    playlists,
    createPlaylist,
    getAllPlaylists,
    getPlaylist,
    updatePlaylist,
    addFilesToPlaylist,
    removeFilesFromPlaylist,
    reorderPlaylist,
    deletePlaylist,
    getPlaylistFiles,
    renamePlaylist
  }
}
