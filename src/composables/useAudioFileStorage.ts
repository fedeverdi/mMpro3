/**
 * Composable for storing and retrieving audio files using IndexedDB
 */
import { parseBlob } from 'music-metadata'

const DB_NAME = 'MMpro3_AudioFiles'
const DB_VERSION = 1
const STORE_NAME = 'audioFiles'

export interface StoredAudioFile {
  id: string
  fileName: string
  arrayBuffer: ArrayBuffer
  mimeType: string
  timestamp: number
  artist?: string
  title?: string
  artwork?: string // base64 encoded image
}

export function useAudioFileStorage() {
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
        if (!database.objectStoreNames.contains(STORE_NAME)) {
          database.createObjectStore(STORE_NAME, { keyPath: 'id' })
        }
      }
    })
  }

  // Extract metadata from audio file using music-metadata
  async function extractMetadata(file: File): Promise<{ artist: string; title: string; artwork?: string }> {
    try {
      const metadata = await parseBlob(file)
      const artist = metadata.common.artist || 'Unknown Artist'
      const title = metadata.common.title || file.name.replace(/\.[^/.]+$/, '')
      
      // Extract artwork if available
      let artwork: string | undefined
      if (metadata.common.picture && metadata.common.picture.length > 0) {
        const picture = metadata.common.picture[0]
        // Convert Buffer to base64
        const base64 = btoa(
          new Uint8Array(picture.data).reduce((data, byte) => data + String.fromCharCode(byte), '')
        )
        artwork = `data:${picture.format};base64,${base64}`
      }
      
      return { artist, title, artwork }
    } catch (error) {
      // Fallback to filename parsing if tags are not available
      const nameWithoutExt = file.name.replace(/\.[^/.]+$/, '')
      const dashPattern = /^([^-]+)\s*-\s*(.+)$/
      const dashMatch = nameWithoutExt.match(dashPattern)
      
      if (dashMatch && dashMatch[1] && dashMatch[2]) {
        return { 
          artist: dashMatch[1].trim(), 
          title: dashMatch[2].trim() 
        }
      } else {
        return { 
          artist: 'Unknown Artist', 
          title: nameWithoutExt 
        }
      }
    }
  }

  // Save audio file to IndexedDB
  async function saveAudioFile(file: File): Promise<string> {
    const database = await initDB()
    const arrayBuffer = await file.arrayBuffer()
    
    const fileId = `audio_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`
    const metadata = await extractMetadata(file)
    
    const storedFile: StoredAudioFile = {
      id: fileId,
      fileName: file.name,
      arrayBuffer,
      mimeType: file.type,
      timestamp: Date.now(),
      artist: metadata.artist,
      title: metadata.title,
      artwork: metadata.artwork
    }

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.add(storedFile)

      request.onsuccess = () => resolve(fileId)
      request.onerror = () => reject(request.error)
    })
  }

  // Get audio file from IndexedDB
  async function getAudioFile(fileId: string): Promise<StoredAudioFile | null> {
    const database = await initDB()

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.get(fileId)

      request.onsuccess = () => resolve(request.result || null)
      request.onerror = () => reject(request.error)
    })
  }

  // Delete audio file from IndexedDB
  async function deleteAudioFile(fileId: string): Promise<void> {
    const database = await initDB()

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.delete(fileId)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })
  }

  // Clean up old files (optional - can be called periodically)
  async function cleanupOldFiles(maxAge: number = 30 * 24 * 60 * 60 * 1000): Promise<void> {
    const database = await initDB()
    const cutoffTime = Date.now() - maxAge

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readwrite')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.openCursor()

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result
        if (cursor) {
          const file = cursor.value as StoredAudioFile
          if (file.timestamp < cutoffTime) {
            cursor.delete()
          }
          cursor.continue()
        } else {
          resolve()
        }
      }

      request.onerror = () => reject(request.error)
    })
  }

  // Get all audio files from IndexedDB
  async function getAllAudioFiles(): Promise<StoredAudioFile[]> {
    const database = await initDB()

    return new Promise((resolve, reject) => {
      const transaction = database.transaction([STORE_NAME], 'readonly')
      const store = transaction.objectStore(STORE_NAME)
      const request = store.getAll()

      request.onsuccess = () => resolve(request.result || [])
      request.onerror = () => reject(request.error)
    })
  }

  // Compare two Uint8Arrays for equality (private helper)
  function areArrayBuffersEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false
    
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false
    }
    
    return true
  }

  // Check if file already exists in library
  async function checkIfDuplicate(newBuffer: ArrayBuffer, fileName: string, fileSize: number): Promise<string | null> {
    const allFiles = await getAllAudioFiles()
    
    // First check by file name and size for quick rejection
    const potentialDuplicates = allFiles.filter(
      f => f.fileName === fileName && f.arrayBuffer.byteLength === fileSize
    )
    
    if (potentialDuplicates.length === 0) {
      return null
    }
    
    // If name and size match, compare ArrayBuffers byte by byte
    const newBytes = new Uint8Array(newBuffer)
    
    for (const existingFile of potentialDuplicates) {
      const existingBytes = new Uint8Array(existingFile.arrayBuffer)
      
      // Compare buffers
      if (areArrayBuffersEqual(newBytes, existingBytes)) {
        return existingFile.id // Return the ID of the existing file
      }
    }
    
    return null
  }

  return {
    saveAudioFile,
    getAudioFile,
    deleteAudioFile,
    cleanupOldFiles,
    getAllAudioFiles,
    checkIfDuplicate
  }
}
