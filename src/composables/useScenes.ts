import { ref } from 'vue'

export interface SceneTrack {
  // Audio source
  audioFile?: {
    id: string
    fileName: string
  }
  
  // Basic controls
  gain: number
  volume: number
  pan: number
  mute: boolean
  solo: boolean
  phaseInvert: boolean
  padEnabled: boolean
  hpfEnabled: boolean
  
  // Routing
  routeToMaster: boolean
  routedSubgroups: number[]
  
  // Effects enabled state (detailed params stored in Rust or child components)
  gateEnabled: boolean
  compressorEnabled: boolean
  
  // EQ (4-band)
  eqEnabled: boolean
  eqLow: number
  eqLowMid: number
  eqHighMid: number
  eqHigh: number
  
  // Parametric EQ
  parametricEQFilters: any[]
}

export interface Scene {
  id: string
  name: string
  timestamp: number
  pinned: boolean
  tracks: SceneTrack[]
  // Future: master settings, subgroup settings, etc.
}

// Shared state (singleton pattern)
const scenes = ref<Scene[]>([])
const currentSceneId = ref<string | null>(null)

export function useScenes() {
  
  /**
   * Save a scene to filesystem
   */
  async function saveScene(scene: Scene): Promise<void> {
    try {
      // Serialize scene to ensure it's IPC-compatible (no Vue refs, circular refs, etc.)
      const serializedScene = JSON.parse(JSON.stringify(scene))
      
      await window.audioEngine.saveScene(serializedScene)
      
      // Update local list
      const existingIndex = scenes.value.findIndex(s => s.id === scene.id)
      if (existingIndex >= 0) {
        scenes.value[existingIndex] = scene
      } else {
        scenes.value.push(scene)
      }
    } catch (error) {
      console.error('[useScenes] Error saving scene:', error)
      throw error
    }
  }
  
  /**
   * Load all scenes from filesystem
   */
  async function loadAllScenes(): Promise<void> {
    try {
      const loadedScenes = await window.audioEngine.listScenes()
      scenes.value = loadedScenes || []
    } catch (error) {
      console.error('[useScenes] Error loading scenes:', error)
      scenes.value = []
    }
  }
  
  /**
   * Get a specific scene by ID
   */
  async function getScene(sceneId: string): Promise<Scene | null> {
    try {
      return await window.audioEngine.getScene(sceneId)
    } catch (error) {
      console.error('[useScenes] Error getting scene:', error)
      return null
    }
  }
  
  /**
   * Delete a scene
   */
  async function deleteScene(sceneId: string): Promise<void> {
    try {
      await window.audioEngine.deleteScene(sceneId)
      scenes.value = scenes.value.filter(s => s.id !== sceneId)
      
      if (currentSceneId.value === sceneId) {
        currentSceneId.value = null
      }
    } catch (error) {
      console.error('[useScenes] Error deleting scene:', error)
      throw error
    }
  }
  
  /**
   * Update an existing scene with current state
   */
  async function updateScene(sceneId: string, tracksData: SceneTrack[]): Promise<void> {
    try {
      const existingScene = scenes.value.find(s => s.id === sceneId)
      if (!existingScene) {
        throw new Error(`Scene not found: ${sceneId}`)
      }
      
      // Create updated scene with same ID, name, pinned state, new timestamp
      const updatedScene: Scene = {
        id: sceneId,
        name: existingScene.name,
        timestamp: Date.now(),
        pinned: existingScene.pinned || false,
        tracks: tracksData
      }
      
      await saveScene(updatedScene)
      console.log(`[useScenes] Updated scene: ${existingScene.name}`)
    } catch (error) {
      console.error('[useScenes] Error updating scene:', error)
      throw error
    }
  }
  
  /**
   * Toggle pin state for a scene
   */
  async function togglePinScene(sceneId: string): Promise<void> {
    try {
      const scene = scenes.value.find(s => s.id === sceneId)
      if (!scene) {
        throw new Error(`Scene not found: ${sceneId}`)
      }
      
      // Toggle pinned state
      scene.pinned = !scene.pinned
      
      // Save updated scene
      await saveScene(scene)
      console.log(`[useScenes] Toggled pin for scene: ${scene.name} (pinned: ${scene.pinned})`)
    } catch (error) {
      console.error('[useScenes] Error toggling pin:', error)
      throw error
    }
  }
  
  /**
   * Create a new scene from current state
   */
  function createNewScene(name: string, tracksData: SceneTrack[]): Scene {
    return {
      id: `scene_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
      name,
      timestamp: Date.now(),
      pinned: false,
      tracks: tracksData
    }
  }
  
  /**
   * Set the current active scene ID
   */
  function setCurrentSceneId(sceneId: string | null): void {
    currentSceneId.value = sceneId
  }
  
  return {
    scenes,
    currentSceneId,
    saveScene,
    updateScene,
    loadAllScenes,
    getScene,
    deleteScene,
    togglePinScene,
    createNewScene,
    setCurrentSceneId
  }
}
