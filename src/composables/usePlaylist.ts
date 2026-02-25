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

const DB_NAME = 'MMpro3_Playlists'
const STORE_NAME = 'playlists'
const DB_VERSION = 1

let db: IDBDatabase | null = null

// Initialize IndexedDB
async function initDB(): Promise<IDBDatabase> {
  if (db) return db

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onerror = () => reject(request.error)
    request.onsuccess = () => {
      db = request.result
      resolve(db)
    }

    request.onupgradeneeded = (event) => {
      const database = (event.target as IDBOpenDBRequest).result

      // Create playlists store
      if (!database.objectStoreNames.contains(STORE_NAME)) {
        const store = database.createObjectStore(STORE_NAME, { keyPath: 'id' })
        store.createIndex('name', 'name', { unique: false })
        store.createIndex('updatedAt', 'updatedAt', { unique: false })
      }
    }
  })
}

export function usePlaylist() {
  const playlists = ref<Playlist[]>([])

  // Create a new playlist
  async function createPlaylist(name: string, fileIds: string[] = []): Promise<Playlist> {
    const database = await initDB()

    const playlist: Playlist = {
      id: `playlist_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      name,
      fileIds,
      createdAt: Date.now(),
      updatedAt: Date.now()
    }

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.add(playlist)

      request.onsuccess = () => {
        playlists.value.push(playlist)
        resolve(playlist)
      }
      request.onerror = () => reject(request.error)
    })
  }

  // Get all playlists
  async function getAllPlaylists(): Promise<Playlist[]> {
    const database = await initDB()

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.getAll()

      request.onsuccess = () => {
        const allPlaylists = request.result as Playlist[]
        playlists.value = allPlaylists.sort((a, b) => b.updatedAt - a.updatedAt)
        resolve(playlists.value)
      }
      request.onerror = () => reject(request.error)
    })
  }

  // Get a specific playlist by ID
  async function getPlaylist(id: string): Promise<Playlist | null> {
    const database = await initDB()

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.get(id)

      request.onsuccess = () => resolve(request.result || null)
      request.onerror = () => reject(request.error)
    })
  }

  // Update a playlist (rename or modify file list)
  async function updatePlaylist(id: string, updates: Partial<Omit<Playlist, 'id' | 'createdAt'>>): Promise<void> {
    const database = await initDB()
    const existing = await getPlaylist(id)
    
    if (!existing) {
      throw new Error('Playlist not found')
    }

    const updated: Playlist = {
      ...existing,
      ...updates,
      updatedAt: Date.now()
    }

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.put(updated)

      request.onsuccess = () => {
        const index = playlists.value.findIndex(p => p.id === id)
        if (index !== -1) {
          playlists.value[index] = updated
        }
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
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
    const database = await initDB()

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.delete(id)

      request.onsuccess = () => {
        playlists.value = playlists.value.filter(p => p.id !== id)
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
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
