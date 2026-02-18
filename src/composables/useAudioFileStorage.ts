/**
 * Composable for storing and retrieving audio files using IndexedDB
 */

const DB_NAME = 'MMpro3_AudioFiles'
const DB_VERSION = 1
const STORE_NAME = 'audioFiles'

interface StoredAudioFile {
  id: string
  fileName: string
  arrayBuffer: ArrayBuffer
  mimeType: string
  timestamp: number
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

  // Save audio file to IndexedDB
  async function saveAudioFile(file: File): Promise<string> {
    const database = await initDB()
    const arrayBuffer = await file.arrayBuffer()
    
    const fileId = `audio_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`
    
    const storedFile: StoredAudioFile = {
      id: fileId,
      fileName: file.name,
      arrayBuffer,
      mimeType: file.type,
      timestamp: Date.now()
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

  return {
    saveAudioFile,
    getAudioFile,
    deleteAudioFile,
    cleanupOldFiles
  }
}
