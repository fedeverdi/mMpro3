<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="close">
          <!-- Backdrop -->
          <div class="absolute inset-0 bg-black/80 backdrop-blur-sm"></div>

          <!-- Modal Content -->
          <div
            class="relative bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-blue-600 p-6 max-w-[800px] w-full max-h-[90vh] overflow-y-auto shadow-2xl">

            <!-- Header -->
            <div class="flex items-center justify-between mb-6">
              <h2 class="text-xl font-bold text-blue-400">
                <div class="flex">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" class="h-6 w-6 pr-2 pt-1" fill="currentColor">
                    <path
                      d="M149.333 216v80c0 13.255-10.745 24-24 24H24c-13.255 0-24-10.745-24-24v-80c0-13.255 10.745-24 24-24h101.333c13.255 0 24 10.745 24 24zM0 376v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H24c-13.255 0-24 10.745-24 24zM125.333 32H24C10.745 32 0 42.745 0 56v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24zm80 448H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24zm-24-424v80c0 13.255 10.745 24 24 24H488c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24zm24 264H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24z" />
                  </svg>
                  Scene Manager
                </div>
              </h2>
              <button @click="close"
                class="w-9 h-9 flex items-center justify-center rounded-full border border-gray-600 hover:border-red-500 hover:bg-red-500/10 text-gray-300 hover:text-red-400 transition-all text-sm font-bold"
                title="Close">
                X
              </button>
            </div>

            <!-- Save New Scene -->
            <div class="mb-6 p-4 bg-gray-900/50 rounded-lg border border-gray-700">
              <h3 class="text-sm font-bold text-gray-300 mb-3">Save New Scene</h3>
              <div class="flex gap-2">
                <input v-model="newSceneName" type="text" placeholder="Scene name..."
                  class="flex-1 px-3 py-2 text-sm bg-gray-800 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
                  @keyup.enter="handleSaveNew" />
                <button @click="handleSaveNew" :disabled="!newSceneName.trim()"
                  class="px-4 py-2 text-sm font-semibold rounded border transition-all flex items-center gap-1.5"
                  :class="newSceneName.trim() 
                    ? 'border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 text-gray-300 hover:text-blue-400' 
                    : 'border-gray-700 bg-gray-800/50 text-gray-600 cursor-not-allowed'">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                      d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                  </svg>
                  Save
                </button>
              </div>
            </div>

            <!-- Scenes List -->
            <div class="space-y-2">
              <h3 class="text-sm font-bold text-gray-300 mb-3">Saved Scenes</h3>

              <div v-if="scenes.length === 0" class="text-center py-8 text-gray-500">
                No scenes saved yet. Create your first scene above!
              </div>

              <div v-for="scene in scenes" :key="scene.id" class="p-3 bg-gray-900/50 rounded border transition-all"
                :class="scene.id === currentSceneId ? 'border-green-500' : 'border-gray-700 hover:border-gray-600'">
                <div class="flex items-center justify-between gap-3">
                  <!-- Scene Info -->
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <input v-if="editingSceneId === scene.id" v-model="editingSceneName" type="text"
                        class="flex-1 px-2 py-1 text-sm bg-gray-800 text-white rounded border border-blue-500 focus:outline-none"
                        @keyup.enter="saveRename(scene.id)" @keyup.esc="cancelRename" />
                      <div v-else class="flex items-center gap-2">
                        <span class="font-semibold text-white">{{ scene.name }}</span>
                        <span v-if="scene.id === currentSceneId"
                          class="px-2 py-0.5 text-[10px] font-bold border border-green-500 bg-green-500/20 text-green-400 rounded">
                          CURRENT
                        </span>
                      </div>
                    </div>
                    <div class="text-xs text-gray-500 mt-1">
                      {{ scene.tracks.length }} tracks • Created {{ formatDate(scene.createdAt) }}
                      <span v-if="scene.updatedAt !== scene.createdAt"> • Updated {{ formatDate(scene.updatedAt)
                        }}</span>
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="flex gap-1 flex-shrink-0">
                    <!-- Update Button (only for current scene) -->
                    <button v-if="scene.id === currentSceneId" @click="confirmUpdate(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded border border-gray-600 hover:border-green-500 hover:bg-green-500/10 text-gray-300 hover:text-green-400 transition-all flex items-center gap-1"
                      title="Update current scene">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                      Update
                    </button>

                    <!-- Load Button -->
                    <button @click="confirmLoad(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded border border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 text-gray-300 hover:text-blue-400 transition-all flex items-center gap-1"
                      title="Load scene">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                      </svg>
                      Load
                    </button>

                    <!-- Pin Button -->
                    <button @click="emit('togglePin', scene.id)"
                      class="px-3 py-1 text-xs font-semibold rounded border transition-all flex items-center gap-1"
                      :class="scene.pinned 
                        ? 'border-yellow-500 bg-yellow-500/20 text-yellow-400 hover:border-yellow-400 hover:bg-yellow-500/30' 
                        : 'border-gray-600 hover:border-gray-500 hover:bg-gray-500/10 text-gray-300 hover:text-gray-200'"
                      :title="scene.pinned ? 'Unpin from quick access' : 'Pin to quick access'">
                      <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                        <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clip-rule="evenodd" />
                      </svg>
                      {{ scene.pinned ? 'Unpin' : 'Pin' }}
                    </button>

                    <!-- Rename Button -->
                    <button v-if="editingSceneId !== scene.id" @click="startRename(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded border border-gray-600 hover:border-gray-500 hover:bg-gray-500/10 text-gray-300 hover:text-gray-200 transition-all flex items-center gap-1"
                      title="Rename scene">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                          d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                      </svg>
                      Edit
                    </button>
                    <button v-else @click="saveRename(scene.id)"
                      class="px-3 py-1 text-xs font-semibold rounded border border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 text-gray-300 hover:text-blue-400 transition-all flex items-center gap-1"
                      title="Save rename">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                      Save
                    </button>

                    <!-- Delete Button -->
                    <button @click="confirmDelete(scene.id, scene.name)"
                      class="px-3 py-1 text-xs font-semibold rounded border border-gray-600 hover:border-red-500 hover:bg-red-500/10 text-gray-300 hover:text-red-400 transition-all flex items-center gap-1"
                      title="Delete scene">
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                      Delete
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
          <div
            class="relative bg-gradient-to-b from-gray-800 to-gray-900 rounded-lg border-2 border-gray-500 p-6 max-w-[500px] w-full shadow-2xl">
            <div class="mb-4">
              <div class="flex items-center gap-3 mb-3">
                <div class="text-3xl">{{ confirmationModal.icon }}</div>
                <h3 class="text-lg font-bold text-gray-200">{{ confirmationModal.title }}</h3>
              </div>
              <p class="text-sm text-gray-300">{{ confirmationModal.message }}</p>
            </div>

            <div class="flex gap-2 justify-end">
              <button @click="cancelConfirmation"
                class="px-4 py-2 text-sm font-semibold rounded border border-gray-600 hover:border-gray-500 hover:bg-gray-500/10 text-gray-300 hover:text-gray-200 transition-all">
                Cancel
              </button>
              <button @click="executeConfirmation" class="px-4 py-2 text-sm font-semibold rounded border transition-all"
                :class="getConfirmButtonClasses()">
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
    'load',
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
    'update',
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
    'delete',
    () => emit('delete', sceneId)
  )
}

function getConfirmButtonClasses() {
  const type = confirmationModal.value.confirmClass
  if (type === 'load') {
    return 'border-gray-600 hover:border-blue-500 hover:bg-blue-500/10 text-gray-300 hover:text-blue-400'
  } else if (type === 'update') {
    return 'border-gray-600 hover:border-green-500 hover:bg-green-500/10 text-gray-300 hover:text-green-400'
  } else if (type === 'delete') {
    return 'border-gray-600 hover:border-red-500 hover:bg-red-500/10 text-gray-300 hover:text-red-400'
  }
  return 'border-gray-600 hover:border-gray-500 hover:bg-gray-500/10 text-gray-300 hover:text-gray-200'
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
