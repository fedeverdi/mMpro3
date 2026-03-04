<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="modelValue" class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/70" @click.self="close">
          <!-- Modal Content -->
          <div class="bg-gray-900 rounded-lg shadow-2xl max-w-[800px] w-full max-h-[90vh] border border-gray-700 flex flex-col">

            <!-- Header -->
            <div class="flex items-center justify-between p-4 border-b border-gray-700 flex-shrink-0">
              <h2 class="text-lg font-semibold text-white">🎬 Scene Manager</h2>
              <button @click="close" class="text-gray-400 hover:text-white transition-colors">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-y-auto p-6">
              
              <!-- Save Current Scene Section -->
              <div class="mb-6 p-4 bg-gray-800 rounded-lg border border-gray-700">
                <h3 class="text-sm font-semibold text-white mb-3">💾 Save Current State</h3>
                <div class="flex gap-2">
                  <input 
                    v-model="newSceneName" 
                    type="text" 
                    placeholder="Enter scene name..."
                    class="flex-1 px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none text-sm"
                    @keyup.enter="saveCurrentScene"
                  />
                  <button 
                    @click="saveCurrentScene"
                    :disabled="!newSceneName.trim()"
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded font-semibold transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                  >
                    Save Scene
                  </button>
                </div>
                <p class="text-xs text-gray-400 mt-2">
                  This will capture the current state of all tracks, effects, and routing.
                </p>
              </div>

              <!-- Saved Scenes List -->
              <div>
                <h3 class="text-sm font-semibold text-white mb-3">📋 Saved Scenes</h3>
                
                <div v-if="scenes.length === 0" class="text-center py-8 text-gray-400">
                  <p class="text-sm">No scenes saved yet.</p>
                  <p class="text-xs mt-1">Save your first scene above!</p>
                </div>

                <div v-else class="space-y-2">
                  <div 
                    v-for="scene in scenes" 
                    :key="scene.id"
                    class="flex items-center justify-between p-3 bg-gray-800 rounded border border-gray-700 hover:border-gray-600 transition-colors"
                  >
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <svg v-if="scene.pinned" class="w-3 h-3 text-yellow-400" fill="currentColor" viewBox="0 0 384 512">
                          <path d="M32 32C32 14.3 46.3 0 64 0H320c17.7 0 32 14.3 32 32s-14.3 32-32 32H290.5l11.4 148.2c36.7 19.9 65.7 53.2 79.5 94.7l1 3c3.3 9.8 1.6 20.5-4.4 28.8s-15.7 13.3-26 13.3H32c-10.3 0-19.9-5-26-13.3s-7.7-19.1-4.4-28.8l1-3c13.8-41.5 42.8-74.8 79.5-94.7L93.5 64H64C46.3 64 32 49.7 32 32zM160 384h64v96c0 17.7-14.3 32-32 32s-32-14.3-32-32V384z"/>
                        </svg>
                        <span class="text-sm font-medium text-white">{{ scene.name }}</span>
                        <span v-if="currentSceneId === scene.id" class="text-xs px-2 py-0.5 bg-green-600 text-white rounded">
                          LOADED
                        </span>
                      </div>
                      <div class="text-xs text-gray-400 mt-1">
                        {{ formatDate(scene.timestamp) }} • {{ scene.tracks.length }} tracks
                      </div>
                    </div>
                    
                    <div class="flex items-center gap-2 ml-4">
                      <button
                        @click="loadScene(scene)"
                        class="px-3 py-1 text-xs bg-green-600 hover:bg-green-500 text-white rounded font-semibold transition-colors"
                      >
                        Load
                      </button>
                      <button
                        @click="handleTogglePin(scene)"
                        :class="[
                          'px-3 py-1 flex items-center gap-1.5 text-xs rounded font-semibold transition-colors',
                          scene.pinned 
                            ? 'bg-yellow-600 hover:bg-yellow-500 text-white' 
                            : 'bg-gray-700 hover:bg-gray-600 text-gray-300 border border-gray-600'
                        ]"
                        :title="scene.pinned ? 'Unpin from quick access' : 'Pin to quick access'"
                      >
                        <svg class="w-2.5 h-2.5" fill="currentColor" viewBox="0 0 384 512">
                          <path d="M32 32C32 14.3 46.3 0 64 0H320c17.7 0 32 14.3 32 32s-14.3 32-32 32H290.5l11.4 148.2c36.7 19.9 65.7 53.2 79.5 94.7l1 3c3.3 9.8 1.6 20.5-4.4 28.8s-15.7 13.3-26 13.3H32c-10.3 0-19.9-5-26-13.3s-7.7-19.1-4.4-28.8l1-3c13.8-41.5 42.8-74.8 79.5-94.7L93.5 64H64C46.3 64 32 49.7 32 32zM160 384h64v96c0 17.7-14.3 32-32 32s-32-14.3-32-32V384z"/>
                        </svg>
                        Pin
                      </button>
                      <button
                        v-if="currentSceneId === scene.id"
                        @click="updateCurrentScene(scene)"
                        class="px-3 py-1 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded font-semibold transition-colors"
                      >
                        Update
                      </button>
                      <button
                        @click="confirmDeleteScene(scene)"
                        class="px-3 py-1 text-xs bg-red-600 hover:bg-red-500 text-white rounded font-semibold transition-colors"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
              </div>

            </div>

          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useScenes } from '~/composables/useScenes'
import { useNotifications } from '~/composables/useNotifications'

const props = defineProps<{
  modelValue: boolean
  tracks: Array<{ id: number, type: string }>
  getTrackState: (trackId: number) => any
  getMasterState?: () => any
  getMasterEqFilters?: () => any[]
  getMasterFx?: () => any
  getSubgroupsState?: () => any[]
  getAuxBusesState?: () => any[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'loadScene': [scene: any]
}>()

const { scenes, currentSceneId, saveScene, updateScene, loadAllScenes, deleteScene, togglePinScene, createNewScene } = useScenes()
const notify = useNotifications()

const newSceneName = ref('')

// Load scenes when modal opens
watch(() => props.modelValue, async (isOpen) => {
  if (isOpen) {
    await loadAllScenes()
  }
}, { immediate: true })

async function saveCurrentScene() {
  if (!newSceneName.value.trim()) return
  
  try {
    // Collect track states from parent
    const tracksData = props.tracks.map(track => {
      const trackState = props.getTrackState(track.id)
      return trackState || {}
    })
    
    // Collect master, subgroups, and aux buses states
    const masterState = props.getMasterState?.() || undefined
    const masterEQFilters = props.getMasterEqFilters?.() || undefined
    const masterFX = props.getMasterFx?.() || undefined
    const subgroupsState = props.getSubgroupsState?.() || undefined
    const auxBusesState = props.getAuxBusesState?.() || undefined
    
    const scene = createNewScene(
      newSceneName.value, 
      tracksData,
      masterState,
      masterEQFilters,
      masterFX,
      subgroupsState,
      auxBusesState
    )
    
    await saveScene(scene)
    newSceneName.value = ''
  } catch (error) {
    console.error('[ScenesModal] Error saving scene:', error)
  }
}

function loadScene(scene: any) {
  currentSceneId.value = scene.id
  emit('loadScene', scene)
  close() // Close modal after loading scene
}

async function updateCurrentScene(scene: any) {
  // Confirm update with custom notification
  const confirmed = await notify.confirm(`Update "${scene.name}" with current track settings?`)
  if (!confirmed) return
  
  try {
    // Collect current track states from parent
    const tracksData = props.tracks.map(track => {
      const trackState = props.getTrackState(track.id)
      return trackState || {}
    })
    
    // Collect master, subgroups, and aux buses states
    const masterState = props.getMasterState?.() || undefined
    const masterEQFilters = props.getMasterEqFilters?.() || undefined
    const masterFX = props.getMasterFx?.() || undefined
    const subgroupsState = props.getSubgroupsState?.() || undefined
    const auxBusesState = props.getAuxBusesState?.() || undefined
    
    await updateScene(
      scene.id, 
      tracksData,
      masterState,
      masterEQFilters,
      masterFX,
      subgroupsState,
      auxBusesState
    )
    // Reload scenes to update QuickScenes
    await loadAllScenes()
    close() // Close modal after updating scene
  } catch (error) {
    console.error('[ScenesModal] Error updating scene:', error)
  }
}

async function confirmDeleteScene(scene: any) {
  const confirmed = await notify.confirm(`Are you sure you want to delete "${scene.name}"?`)
  if (!confirmed) return
  
  try {
    await deleteScene(scene.id)
  } catch (error) {
    console.error('[ScenesModal] Error deleting scene:', error)
  }
}

async function handleTogglePin(scene: any) {
  try {
    await togglePinScene(scene.id)
    // Reload scenes to update QuickScenes
    await loadAllScenes()
  } catch (error) {
    console.error('[ScenesModal] Error toggling pin:', error)
  }
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleString()
}

function close() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
