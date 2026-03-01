<template>
  <div>
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70" @click.self="close">
          <!-- Modal Content -->
          <div class="bg-gray-900 rounded-lg shadow-2xl max-w-[800px] w-full max-h-[90vh] border border-gray-700 overflow-y-auto">

            <!-- Header -->
            <div class="flex items-center justify-between p-4 border-b border-gray-700">
              <h2 class="text-lg font-semibold text-white">🎬 Scene Manager</h2>
              <button @click="close" class="text-gray-400 hover:text-white transition-colors">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Content -->
            <div class="p-6">
              <div class="text-center text-gray-400">
                <p class="text-sm">Scene management will be implemented with Rust backend.</p>
                <p class="text-xs mt-2 text-gray-500">Coming soon...</p>
              </div>
            </div>

          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

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
