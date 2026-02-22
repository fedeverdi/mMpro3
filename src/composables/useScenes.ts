import { ref, toRaw } from 'vue'

export interface TrackSnapshot {
  trackNumber: number
  volume: number
  pan: number
  muted: boolean
  soloed: boolean
  padEnabled?: boolean
  hpfEnabled?: boolean
  sourceType: 'none' | 'youtube' | 'input' | 'file'
  youtubeURL?: string
  selectedInputDevice?: string
  fileName?: string
  fileId?: string
  parametricEQFilters?: any[]
  compressorEnabled?: boolean
  compressorThreshold?: number
  compressorRatio?: number
  compressorAttack?: number
  compressorRelease?: number
  gateEnabled?: boolean
  gateThreshold?: number
  gateAttack?: number
  gateRelease?: number
  gateRange?: number
  reverbEnabled?: boolean
  reverbDecay?: number
  reverbPreDelay?: number
  reverbWet?: number
  auxSends?: Record<string, { level: number, preFader: boolean, muted: boolean }>
}

export interface MasterSnapshot {
  leftVolume: number
  rightVolume: number
  headphonesVolume: number
  isLinked: boolean
  selectedMasterOutput?: string | null
  selectedHeadphonesOutput?: string | null
  masterEQFilters: any[]
  compressorEnabled: boolean
  compressorThreshold?: number
  compressorRatio?: number
  compressorAttack?: number
  compressorRelease?: number
  reverbEnabled: boolean
  reverbDecay?: number
  reverbPreDelay?: number
  reverbWet?: number
  delayEnabled: boolean
  delayTime?: number
  delayFeedback?: number
  delayWet?: number
  limiterEnabled: boolean
  limiterThreshold?: number
}

export interface SubgroupSnapshot {
  id: number
  name: string
  volume: number
  routeToMaster: boolean
  selectedOutput: string | null
  compressorEnabled: boolean
  reverbEnabled: boolean
  limiterEnabled: boolean
  delayEnabled: boolean
  compressorParams?: any
  reverbParams?: any
  limiterParams?: any
  delayParams?: any
}

export interface AuxSnapshot {
  id: string
  name: string
  volume: number
  muted: boolean
  soloed: boolean
  routeToMaster: boolean
  selectedOutputDevice?: string | null
  // FX Chain
  reverbEnabled?: boolean
  reverbParams?: { decay: number, preDelay: number, wet: number }
  delayEnabled?: boolean
  delayParams?: { delayTime: number, feedback: number, wet: number }
}

export interface Scene {
  id: string
  name: string
  tracks: TrackSnapshot[]
  master: MasterSnapshot
  subgroups?: SubgroupSnapshot[]
  auxBuses?: AuxSnapshot[]
  createdAt: number
  updatedAt: number
  pinned?: boolean
}

const DB_NAME = 'MMpro3_Scenes'
const DB_VERSION = 1
const SCENES_STORE = 'scenes'

const scenes = ref<Scene[]>([])
const currentSceneId = ref<string | null>(null)
let db: IDBDatabase | null = null

export function useScenes() {
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
        if (!database.objectStoreNames.contains(SCENES_STORE)) {
          database.createObjectStore(SCENES_STORE, { keyPath: 'id' })
        }
      }
    })
  }

  async function loadScenesFromStorage() {
    try {
      const database = await initDB()
      
      return new Promise<void>(async (resolve, reject) => {
        const transaction = database.transaction([SCENES_STORE], 'readonly')
        const store = transaction.objectStore(SCENES_STORE)
        const request = store.getAll()

        request.onsuccess = async () => {
          scenes.value = request.result || []
          
          // Migration: check if there are scenes in localStorage and migrate them
          if (scenes.value.length === 0) {
            await migrateFromLocalStorage()
          }
          
          resolve()
        }
        request.onerror = () => reject(request.error)
      })
    } catch (error) {
      console.error('Error loading scenes from IndexedDB:', error)
    }
  }

  // Migrate existing scenes from localStorage to IndexedDB
  async function migrateFromLocalStorage() {
    try {
      const stored = localStorage.getItem('mixer_scenes')
      if (stored) {
        const oldScenes: Scene[] = JSON.parse(stored)
        
        if (oldScenes.length > 0) {
          console.log(`Migrating ${oldScenes.length} scenes from localStorage to IndexedDB...`)
          
          // Save all scenes to IndexedDB
          for (const scene of oldScenes) {
            await saveSceneToStorage(scene)
            scenes.value.push(scene)
          }
          
          // Clear localStorage after successful migration
          localStorage.removeItem('mixer_scenes')
          console.log('Migration completed successfully')
        }
      }
    } catch (error) {
      console.error('Error migrating scenes from localStorage:', error)
    }
  }

  async function saveSceneToStorage(scene: Scene) {
    try {
      const database = await initDB()
      
      // Remove Vue reactivity before saving to IndexedDB
      const rawScene = JSON.parse(JSON.stringify(toRaw(scene)))
      
      return new Promise<void>((resolve, reject) => {
        const transaction = database.transaction([SCENES_STORE], 'readwrite')
        const store = transaction.objectStore(SCENES_STORE)
        const request = store.put(rawScene)

        request.onsuccess = () => resolve()
        request.onerror = () => reject(request.error)
      })
    } catch (error) {
      console.error('Error saving scene to IndexedDB:', error)
      throw error
    }
  }

  async function deleteSceneFromStorage(sceneId: string) {
    try {
      const database = await initDB()
      
      return new Promise<void>((resolve, reject) => {
        const transaction = database.transaction([SCENES_STORE], 'readwrite')
        const store = transaction.objectStore(SCENES_STORE)
        const request = store.delete(sceneId)

        request.onsuccess = () => resolve()
        request.onerror = () => reject(request.error)
      })
    } catch (error) {
      console.error('Error deleting scene from IndexedDB:', error)
      throw error
    }
  }

  async function createScene(name: string, tracks: TrackSnapshot[], master: MasterSnapshot, subgroups?: SubgroupSnapshot[], auxBuses?: AuxSnapshot[]): Promise<Scene> {
    const scene: Scene = {
      id: `scene_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
      name,
      tracks,
      master,
      subgroups,
      auxBuses,
      createdAt: Date.now(),
      updatedAt: Date.now()
    }
    
    await saveSceneToStorage(scene)
    scenes.value.push(scene)
    return scene
  }

  async function updateScene(sceneId: string, tracks: TrackSnapshot[], master: MasterSnapshot, subgroups?: SubgroupSnapshot[], auxBuses?: AuxSnapshot[]) {
    const index = scenes.value.findIndex(s => s.id === sceneId)
    if (index !== -1) {
      const scene = scenes.value[index]
      const updatedScene: Scene = {
        ...scene,
        tracks,
        master,
        subgroups,
        auxBuses,
        updatedAt: Date.now()
      }
      
      // Update in IndexedDB
      await saveSceneToStorage(updatedScene)
      
      // Update reactive array
      scenes.value[index] = updatedScene
    }
  }

  async function deleteScene(sceneId: string) {
    const index = scenes.value.findIndex(s => s.id === sceneId)
    if (index !== -1) {
      scenes.value.splice(index, 1)
      await deleteSceneFromStorage(sceneId)
    }
  }

  async function renameScene(sceneId: string, newName: string) {
    const index = scenes.value.findIndex(s => s.id === sceneId)
    if (index !== -1) {
      const scene = scenes.value[index]
      const updatedScene: Scene = {
        ...scene,
        name: newName,
        updatedAt: Date.now()
      }
      
      // Update in IndexedDB
      await saveSceneToStorage(updatedScene)
      
      // Update reactive array
      scenes.value[index] = updatedScene
    }
  }

  function getSceneById(sceneId: string): Scene | undefined {
    return scenes.value.find(s => s.id === sceneId)
  }

  function setCurrentScene(sceneId: string | null) {
    currentSceneId.value = sceneId
  }

  async function togglePinScene(sceneId: string) {
    const index = scenes.value.findIndex(s => s.id === sceneId)
    if (index !== -1) {
      const scene = scenes.value[index]
      const updatedScene: Scene = {
        ...scene,
        pinned: !scene.pinned,
        updatedAt: Date.now()
      }
      
      // Update in IndexedDB
      await saveSceneToStorage(updatedScene)
      
      // Update reactive array
      scenes.value[index] = updatedScene
    }
  }

  return {
    scenes,
    currentSceneId,
    loadScenesFromStorage,
    createScene,
    updateScene,
    deleteScene,
    renameScene,
    getSceneById,
    setCurrentScene,
    togglePinScene
  }
}
