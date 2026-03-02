<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70" @click.self="close">
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
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'loadScene': [scene: any]
}>()

const { scenes, currentSceneId, saveScene, updateScene, loadAllScenes, deleteScene, createNewScene } = useScenes()
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
    
    const scene = createNewScene(newSceneName.value, tracksData)
    
    await saveScene(scene)
    newSceneName.value = ''
  } catch (error) {
    console.error('[ScenesModal] Error saving scene:', error)
  }
}

function loadScene(scene: any) {
  emit('loadScene', scene)
  currentSceneId.value = scene.id
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
    
    await updateScene(scene.id, tracksData)
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
