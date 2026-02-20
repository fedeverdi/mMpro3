<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="close">
          <!-- Backdrop -->
          <div class="absolute inset-0 bg-black/80 backdrop-blur-sm"></div>
          
          <!-- Modal Content -->
          <div class="relative bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-6 max-w-[800px] w-full max-h-[90vh] overflow-y-auto shadow-2xl">
            
            <!-- Header -->
            <div class="flex items-center justify-between mb-6">
              <h2 class="text-xl font-bold text-blue-400">🎬 Scene Manager</h2>
              <button
                @click="close"
                class="w-9 h-9 flex items-center justify-center rounded-full bg-gray-700/80 hover:bg-red-600 text-gray-300 hover:text-white transition-colors text-sm font-bold"
                title="Close"
              >
                X
              </button>
            </div>

            <!-- Save New Scene -->
            <div class="mb-6 p-4 bg-gray-900/50 rounded-lg border border-gray-700">
              <h3 class="text-sm font-bold text-gray-300 mb-3">💾 Save New Scene</h3>
              <div class="flex gap-2">
                <input
                  v-model="newSceneName"
                  type="text"
                  placeholder="Scene name..."
                  class="flex-1 px-3 py-2 text-sm bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                  @keyup.enter="handleSaveNew"
                />
                <button
                  @click="handleSaveNew"
                  :disabled="!newSceneName.trim()"
                  class="px-4 py-2 text-sm font-semibold rounded bg-blue-600 hover:bg-blue-500 text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Save
                </button>
              </div>
            </div>

            <!-- Scenes List -->
            <div class="space-y-2">
              <h3 class="text-sm font-bold text-gray-300 mb-3">📚 Saved Scenes</h3>
              
              <div v-if="scenes.length === 0" class="text-center py-8 text-gray-500">
                No scenes saved yet. Create your first scene above!
              </div>

              <div
                v-for="scene in scenes"
                :key="scene.id"
                class="p-3 bg-gray-900/50 rounded border transition-all"
                :class="scene.id === currentSceneId ? 'border-green-500' : 'border-gray-700 hover:border-gray-600'"
              >
                <div class="flex items-center justify-between gap-3">
                  <!-- Scene Info -->
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <input
                        v-if="editingSceneId === scene.id"
                        v-model="editingSceneName"
                        type="text"
                        class="flex-1 px-2 py-1 text-sm bg-gray-800 text-white rounded border border-blue-500 focus:outline-none"
                        @keyup.enter="saveRename(scene.id)"
                        @keyup.esc="cancelRename"
                      />
                      <div v-else class="flex items-center gap-2">
                        <span class="font-semibold text-white">{{ scene.name }}</span>
                        <span v-if="scene.id === currentSceneId" class="px-2 py-0.5 text-[10px] font-bold bg-green-600 text-white rounded">
                          CURRENT
                        </span>
                      </div>
                    </div>
                    <div class="text-xs text-gray-500 mt-1">
                      {{ scene.tracks.length }} tracks • Created {{ formatDate(scene.createdAt) }}
                      <span v-if="scene.updatedAt !== scene.createdAt"> • Updated {{ formatDate(scene.updatedAt) }}</span>
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="flex gap-1 flex-shrink-0">
                    <!-- Update Button (only for current scene) -->
                    <button
                      v-if="scene.id === currentSceneId"
                      @click="confirmUpdate(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded bg-green-600 hover:bg-green-500 text-white transition-colors"
                      title="Update current scene"
                    >
                      Update
                    </button>

                    <!-- Load Button -->
                    <button
                      @click="confirmLoad(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded bg-blue-600 hover:bg-blue-500 text-white transition-colors"
                      title="Load scene"
                    >
                      Load
                    </button>

                    <!-- Pin Button -->
                    <button
                      @click="emit('togglePin', scene.id)"
                      class="px-3 py-1 text-xs font-semibold rounded transition-colors"
                      :class="scene.pinned ? 'bg-yellow-600 hover:bg-yellow-500' : 'bg-gray-700 hover:bg-gray-600'"
                      :title="scene.pinned ? 'Unpin from quick access' : 'Pin to quick access'"
                    >
                      {{ scene.pinned ? '📌' : '📍' }}
                    </button>

                    <!-- Rename Button -->
                    <button
                      v-if="editingSceneId !== scene.id"
                      @click="startRename(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded bg-gray-700 hover:bg-gray-600 text-white transition-colors"
                      title="Rename scene"
                    >
                      ✏️
                    </button>
                    <button
                      v-else
                      @click="saveRename(scene.id)"
                      class="px-3 py-1 text-xs font-semibold rounded bg-blue-600 hover:bg-blue-500 text-white transition-colors"
                      title="Save rename"
                    >
                      ✓
                    </button>

                    <!-- Delete Button -->
                    <button
                      @click="confirmDelete(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded bg-red-600 hover:bg-red-500 text-white transition-colors"
                      title="Delete scene"
                    >
                      🗑️
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>
      
      <!-- Confirmation Modal -->
      <Transition name="modal">
        <div v-if="confirmationModal.show" class="fixed inset-0 z-[60] flex items-center justify-center p-4">
          <!-- Backdrop -->
          <div class="absolute inset-0 bg-black/90 backdrop-blur-sm" @click="cancelConfirmation"></div>
          
          <!-- Confirmation Dialog -->
          <div class="relative bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-gray-500 p-6 max-w-[500px] w-full shadow-2xl">
            <div class="mb-4">
              <div class="flex items-center gap-3 mb-3">
                <div class="text-3xl">{{ confirmationModal.icon }}</div>
                <h3 class="text-lg font-bold text-gray-200">{{ confirmationModal.title }}</h3>
              </div>
              <p class="text-sm text-gray-300">{{ confirmationModal.message }}</p>
            </div>
            
            <div class="flex gap-2 justify-end">
              <button
                @click="cancelConfirmation"
                class="px-4 py-2 text-sm font-semibold rounded bg-gray-700 hover:bg-gray-600 text-white transition-colors"
              >
                Cancel
              </button>
              <button
                @click="executeConfirmation"
                class="px-4 py-2 text-sm font-semibold rounded transition-colors"
                :class="confirmationModal.confirmClass"
              >
                {{ confirmationModal.confirmText }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Scene } from '~/composables/useScenes'

interface Props {
  modelValue: boolean
  scenes: Scene[]
  currentSceneId: string | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', sceneName: string): void
  (e: 'load', sceneId: string): void
  (e: 'update', sceneId: string): void
  (e: 'delete', sceneId: string): void
  (e: 'rename', sceneId: string, newName: string): void
  (e: 'togglePin', sceneId: string): void
}>()

const newSceneName = ref('')
const editingSceneId = ref<string | null>(null)
const editingSceneName = ref('')

// Confirmation modal state
const confirmationModal = ref({
  show: false,
  title: '',
  message: '',
  icon: '',
  confirmText: '',
  confirmClass: '',
  action: null as (() => void) | null
})

function showConfirmation(
  title: string,
  message: string,
  icon: string,
  confirmText: string,
  confirmClass: string,
  action: () => void
) {
  confirmationModal.value = {
    show: true,
    title,
    message,
    icon,
    confirmText,
    confirmClass,
    action
  }
}

function executeConfirmation() {
  if (confirmationModal.value.action) {
    confirmationModal.value.action()
  }
  cancelConfirmation()
}

function cancelConfirmation() {
  confirmationModal.value.show = false
}

function close() {
  emit('update:modelValue', false)
  newSceneName.value = ''
  cancelRename()
}

function handleSaveNew() {
  if (newSceneName.value.trim()) {
    emit('save', newSceneName.value.trim())
    newSceneName.value = ''
  }
}

function startRename(sceneId: string, currentName: string) {
  editingSceneId.value = sceneId
  editingSceneName.value = currentName
}

function saveRename(sceneId: string) {
  if (editingSceneName.value.trim()) {
    emit('rename', sceneId, editingSceneName.value.trim())
  }
  cancelRename()
}

function cancelRename() {
  editingSceneId.value = null
  editingSceneName.value = ''
}

function confirmLoad(sceneId: string, sceneName: string) {
  showConfirmation(
    'Load Scene',
    `Load scene "${sceneName}"? This will replace your current mixer state with the saved scene configuration.`,
    '📂',
    'Load Scene',
    'bg-blue-600 hover:bg-blue-500 text-white',
    () => {
      emit('load', sceneId)
      close()
    }
  )
}

function confirmUpdate(sceneId: string, sceneName: string) {
  showConfirmation(
    'Update Scene',
    `Update scene "${sceneName}"? This will overwrite the saved scene with your current mixer state.`,
    '💾',
    'Update',
    'bg-green-600 hover:bg-green-500 text-white',
    () => {
      emit('update', sceneId)
      close()
    }
  )
}

function confirmDelete(sceneId: string, sceneName: string) {
  showConfirmation(
    'Delete Scene',
    `Delete scene "${sceneName}"? This action cannot be undone.`,
    '⚠️',
    'Delete',
    'bg-red-600 hover:bg-red-500 text-white',
    () => emit('delete', sceneId)
  )
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  
  return date.toLocaleDateString()
}

// Clear new scene name when modal closes
watch(() => props.modelValue, (isOpen) => {
  if (!isOpen) {
    newSceneName.value = ''
    cancelRename()
    cancelConfirmation()
  }
})
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
