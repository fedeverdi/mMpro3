<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/70 backdrop-blur-sm"
      @click.self="$emit('close')">
      <div class="bg-gray-900 border border-gray-700 rounded-lg shadow-2xl p-6 w-96 max-w-[90vw]">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-bold text-white flex items-center gap-2">
            <svg class="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            Set Lock Password
          </h2>
          <button @click="$emit('close')"
            class="text-gray-400 hover:text-white transition-colors">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <p class="text-sm text-gray-400 mb-4">
          Set a password to lock the mixer interface. You'll need this password to unlock it.
        </p>

        <div class="space-y-4">
          <div>
            <label class="block text-xs font-semibold text-gray-300 mb-2">Password</label>
            <input v-model="password" type="password" placeholder="Enter password"
              @keyup.enter="handleConfirm"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded text-white text-sm focus:outline-none focus:border-yellow-500 transition-colors" />
          </div>

          <div>
            <label class="block text-xs font-semibold text-gray-300 mb-2">Confirm Password</label>
            <input v-model="confirmPassword" type="password" placeholder="Confirm password"
              @keyup.enter="handleConfirm"
              class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded text-white text-sm focus:outline-none focus:border-yellow-500 transition-colors" />
          </div>

          <div v-if="error" class="text-red-400 text-xs bg-red-500/10 border border-red-500/30 rounded px-3 py-2">
            {{ error }}
          </div>
        </div>

        <div class="flex gap-2 mt-6">
          <button @click="$emit('close')"
            class="flex-1 px-4 py-2 border border-gray-600 hover:border-gray-500 rounded text-sm font-semibold text-gray-300 hover:text-white transition-all">
            Cancel
          </button>
          <button @click="handleConfirm"
            class="flex-1 px-4 py-2 bg-yellow-600 hover:bg-yellow-500 rounded text-sm font-semibold text-white transition-all flex items-center justify-center gap-2">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            Lock Interface
          </button>
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
  close: []
  confirm: [password: string]
}>()

const password = ref('')
const confirmPassword = ref('')
const error = ref('')

watch(() => props.show, (newVal) => {
  if (newVal) {
    // Reset form when modal opens
    password.value = ''
    confirmPassword.value = ''
    error.value = ''
  }
})

const handleConfirm = () => {
  error.value = ''

  if (!password.value) {
    error.value = 'Please enter a password'
    return
  }

  if (password.value.length < 4) {
    error.value = 'Password must be at least 4 characters'
    return
  }

  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }

  emit('confirm', password.value)
}
</script>
