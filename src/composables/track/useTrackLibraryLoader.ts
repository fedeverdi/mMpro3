/**
 * DEPRECATED: This file is no longer used.
 * 
 * Audio playback is now handled directly by the Rust audio engine.
 * Playlist functionality has been migrated to AudioTrack.vue component.
 * 
 * All Tone.js dependencies have been removed in favor of the Rust-based audio engine.
 */

export interface LibraryLoaderCallbacks {
  // Kept for backwards compatibility, but no longer used
}

export function useTrackLibraryLoader(callbacks: LibraryLoaderCallbacks = {}) {
  // This composable is deprecated and no longer implements any functionality
  // See AudioTrack.vue for the current implementation
  
  return {
    loadFileFromLibrary: () => console.warn('useTrackLibraryLoader is deprecated'),
    loadPlaylistFromLibrary: () => console.warn('useTrackLibraryLoader is deprecated'),
    loadFileFromIndexedDB: () => console.warn('useTrackLibraryLoader is deprecated')
  }
}
