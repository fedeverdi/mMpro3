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

            <!-- Empty Content - To be implemented with Rust backend -->
            <div class="p-8 text-center text-gray-400">
              <p class="text-sm">Scene management will be implemented with Rust backend.</p>
              <p class="text-xs mt-2 text-gray-500">Coming soon...</p>
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
