/**
 * Composable for storing and retrieving audio files using filesystem
 */
import { parseBlob } from 'music-metadata'

export interface StoredAudioFile {
  id: string
  fileName: string
  filePath?: string // Path to the file on the server (used in remote mode and Electron)
  arrayBuffer?: ArrayBuffer // Optional when listing files
  mimeType: string
  timestamp: number
  size?: string // Pre-formatted size string from backend (e.g., "2.5 MB")
  artist?: string
  title?: string
  artwork?: string // base64 encoded image
}

export function useAudioFileStorage() {
  // Get API dynamically to ensure it's available (especially in remote mode)
  const getApi = () => (window as any).audioEngine

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

  // Save audio file to filesystem
  async function saveAudioFile(file: File): Promise<string> {
    const arrayBuffer = await file.arrayBuffer()
    const metadata = await extractMetadata(file)
    
    const fileId = await getApi().saveLibraryFile(arrayBuffer, file.name, {
      mimeType: file.type,
      artist: metadata.artist,
      title: metadata.title,
      artwork: metadata.artwork
    })
    
    return fileId
  }

  // Get audio file from filesystem
  async function getAudioFile(fileId: string): Promise<StoredAudioFile | null> {
    try {
      const file = await getApi().getLibraryFile(fileId)
      return file
    } catch (error) {
      console.error('[useAudioFileStorage] Error getting file:', error)
      return null
    }
  }

  // Delete audio file from filesystem
  async function deleteAudioFile(fileId: string): Promise<void> {
    try {
      await getApi().deleteLibraryFile(fileId)
    } catch (error) {
      console.error('[useAudioFileStorage] Error deleting file:', error)
      throw error
    }
  }

  // Get all audio files from filesystem
  async function getAllAudioFiles(): Promise<StoredAudioFile[]> {
    try {
      const files = await getApi().listLibraryFiles()
      return files
    } catch (error) {
      console.error('[useAudioFileStorage] Error listing files:', error)
      return []
    }
  }

  // Check if file already exists in library (by filename)
  async function checkIfDuplicate(newBuffer: ArrayBuffer, fileName: string, fileSize: number): Promise<string | null> {
    const allFiles = await getAllAudioFiles()
    
    // Check by file name only (simple and fast)
    const existingFile = allFiles.find(f => f.fileName === fileName)
    
    if (existingFile) {
      return existingFile.id // Return the ID of the existing file
    }
    
    return null
  }

  // Compare two Uint8Arrays for equality (private helper)
  function areArrayBuffersEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false
    
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false
    }
    
    return true
  }

  // Clean up old files (optional - can be called periodically)
  // Note: This is a no-op for now, but can be implemented later
  async function cleanupOldFiles(maxAge: number = 30 * 24 * 60 * 60 * 1000): Promise<void> {
    // For filesystem storage, we could implement automatic cleanup later
    // For now, users can manually delete files they don't need
    console.log('[useAudioFileStorage] cleanupOldFiles is not implemented for filesystem storage')
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
