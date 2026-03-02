<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/80 backdrop-blur-md">
      <div class="bg-gray-900 border-2 border-red-600/50 rounded-lg shadow-2xl p-6 w-96 max-w-[90vw]">
        <div class="flex flex-col items-center mb-6">
          <div class="w-16 h-16 bg-red-500/20 rounded-full flex items-center justify-center mb-4">
            <svg class="w-8 h-8 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h2 class="text-xl font-bold text-white text-center">Interface Locked</h2>
          <p class="text-sm text-gray-400 mt-2 text-center">
            Enter password to unlock the mixer
          </p>
        </div>

        <div class="space-y-4">
          <div>
            <input v-model="password" type="password" placeholder="Enter password" autofocus
              @keyup.enter="handleUnlock"
              class="w-full px-4 py-3 bg-gray-800 border-2 border-gray-600 rounded text-white text-center text-lg focus:outline-none focus:border-red-500 transition-colors" />
          </div>

          <div v-if="error" class="text-red-400 text-sm bg-red-500/20 border border-red-500/50 rounded px-3 py-3 flex items-center gap-2">
            <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>{{ error }}</span>
          </div>

          <button @click="handleUnlock"
            class="w-full px-4 py-3 bg-red-600 hover:bg-red-500 rounded text-base font-bold text-white transition-all flex items-center justify-center gap-2">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
            </svg>
            Unlock
          </button>
        </div>

        <div class="mt-6 pt-4 border-t border-gray-700 text-center">
          <p class="text-xs text-gray-500">
            The interface is locked to prevent accidental changes
          </p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  show: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  unlock: [password: string]
}>()

const password = ref('')
const error = ref('')

watch(() => props.show, (newVal) => {
  if (newVal) {
    // Reset form when modal opens
    password.value = ''
    error.value = ''
  }
})

const handleUnlock = () => {
  if (!password.value) {
    error.value = 'Please enter password'
    return
  }

  emit('unlock', password.value)
}

// Expose method to show error from parent
defineExpose({
  showError: (message: string) => {
    error.value = message
    password.value = ''
  }
})
</script>
