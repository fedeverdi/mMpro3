import { ref } from 'vue'

// Simplified scene interface - Rust engine handles all state
export interface Scene {
  name: string
  timestamp: number
  version: number
  pinned: boolean
  track_count: number
  track_ids: number[]
  track_types: string[]
}

// Shared state (singleton pattern)
const scenes = ref<Scene[]>([])
const currentSceneName = ref<string | null>(null)

export function useScenes() {
  
  /**
   * Save current engine state as a scene snapshot
   */
  async function saveScene(name: string): Promise<void> {
    try {
      // Rust engine saves complete state automatically
      await window.audioEngine.saveScene(name)
      
      // Reload list to update UI
      await loadAllScenes()
      
      console.log('[useScenes] Scene saved:', name)
    } catch (error) {
      console.error('[useScenes] Error saving scene:', error)
      throw error
    }
  }
  
  /**
   * Load all scenes from Rust engine
   */
  async function loadAllScenes(): Promise<void> {
    try {
      const loadedScenes = await window.audioEngine.listScenes()
      scenes.value = loadedScenes || []      
    } catch (error) {
      console.error('[useScenes] Error loading scenes:', error)
    }
  }
  
  /**
   * Load a scene (applies all state to engine automatically)
   */
  async function loadScene(name: string): Promise<void> {
    try {
      await window.audioEngine.loadScene(name)
      currentSceneName.value = name
      
      console.log('[useScenes] Scene loaded:', name)
    } catch (error) {
      console.error('[useScenes] Error loading scene:', error)
      throw error
    }
  }
  
  /**
   * Delete a scene
   */
  async function deleteScene(name: string): Promise<void> {
    try {
      await window.audioEngine.deleteScene(name)
      
      // Update local list
      scenes.value = scenes.value.filter(s => s.name !== name)
      
      if (currentSceneName.value === name) {
        currentSceneName.value = null
      }
      
      console.log('[useScenes] Scene deleted:', name)
    } catch (error) {
      console.error('[useScenes] Error deleting scene:', error)
      throw error
    }
  }
  
  /**
   * Rename a scene
   */
  async function renameScene(oldName: string, newName: string): Promise<void> {
    try {
      await window.audioEngine.renameScene(oldName, newName)
      
      // Update local list
      const scene = scenes.value.find(s => s.name === oldName)
      if (scene) {
        scene.name = newName
      }
      
      if (currentSceneName.value === oldName) {
        currentSceneName.value = newName
      }
      
      console.log('[useScenes] Scene renamed:', oldName, '->', newName)
    } catch (error) {
      console.error('[useScenes] Error renaming scene:', error)
      throw error
    }
  }

  return {
    scenes,
    currentSceneName,
    saveScene,
    loadAllScenes,
    loadScene,
    deleteScene,
    renameScene,
    pinScene: async (name: string) => {
      await window.audioEngine.pinScene(name)
      // Refresh list so pinned flag is updated
      await loadAllScenes()
    },
  }
}
