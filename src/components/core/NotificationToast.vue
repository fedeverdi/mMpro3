<template>
  <TransitionGroup name="notification">
    <div
      v-for="notification in notifications"
      :key="notification.id"
      class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
      @click="notification.type !== 'confirm' ? remove(notification.id) : null"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/70 backdrop-blur-sm"></div>
      
      <!-- Modal -->
      <div
        @click.stop
        class="notification-modal relative bg-gradient-to-br from-gray-800 to-gray-900 border rounded-xl shadow-2xl w-full max-w-md"
        :class="{
          'border-blue-500/70': notification.type === 'info',
          'border-green-500/70': notification.type === 'success',
          'border-yellow-500/70': notification.type === 'warning',
          'border-red-500/70': notification.type === 'error',
          'border-purple-500/70': notification.type === 'confirm'
        }"
      >
        <div class="p-6">
          <div class="flex items-start gap-4">
            <!-- Icon -->
            <div class="flex-shrink-0">
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center text-2xl"
                :class="{
                  'bg-blue-500/20 text-blue-400': notification.type === 'info',
                  'bg-green-500/20 text-green-400': notification.type === 'success',
                  'bg-yellow-500/20 text-yellow-400': notification.type === 'warning',
                  'bg-red-500/20 text-red-400': notification.type === 'error',
                  'bg-purple-500/20 text-purple-400': notification.type === 'confirm'
                }"
              >
                <span v-if="notification.type === 'info'">ℹ</span>
                <span v-else-if="notification.type === 'success'">✓</span>
                <span v-else-if="notification.type === 'warning'">⚠</span>
                <span v-else-if="notification.type === 'error'">✕</span>
                <span v-else-if="notification.type === 'confirm'">?</span>
              </div>
            </div>

            <!-- Content -->
            <div class="flex-1 min-w-0">
              <h3 
                class="text-lg font-semibold mb-2"
                :class="{
                  'text-blue-400': notification.type === 'info',
                  'text-green-400': notification.type === 'success',
                  'text-yellow-400': notification.type === 'warning',
                  'text-red-400': notification.type === 'error',
                  'text-purple-400': notification.type === 'confirm'
                }"
              >
                <template v-if="notification.type === 'info'">Info</template>
                <template v-else-if="notification.type === 'success'">Success</template>
                <template v-else-if="notification.type === 'warning'">Warning</template>
                <template v-else-if="notification.type === 'error'">Error</template>
                <template v-else-if="notification.type === 'confirm'">Confirm</template>
              </h3>
              <p class="text-gray-300 text-sm leading-relaxed">
                {{ notification.message }}
              </p>
            </div>

            <!-- Close button for non-confirm notifications -->
            <button
              v-if="notification.type !== 'confirm'"
              @click="remove(notification.id)"
              class="flex-shrink-0 text-gray-500 hover:text-gray-300 transition-colors -mt-1 -mr-1"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Action buttons -->
          <div class="flex gap-3 mt-6">
            <template v-if="notification.type === 'confirm'">
              <button
                @click="notification.onCancel?.()"
                class="flex-1 px-4 py-2.5 text-sm font-semibold text-gray-300 bg-gray-700 hover:bg-gray-600 rounded-lg transition-all"
              >
                Cancel
              </button>
              <button
                @click="notification.onConfirm?.()"
                class="flex-1 px-4 py-2.5 text-sm font-semibold text-white bg-purple-600 hover:bg-purple-500 rounded-lg transition-all"
              >
                Confirm
              </button>
            </template>
            <template v-else>
              <button
                @click="remove(notification.id)"
                class="flex-1 px-4 py-2.5 text-sm font-semibold rounded-lg transition-all"
                :class="{
                  'text-white bg-blue-600 hover:bg-blue-500': notification.type === 'info',
                  'text-white bg-green-600 hover:bg-green-500': notification.type === 'success',
                  'text-white bg-yellow-600 hover:bg-yellow-500': notification.type === 'warning',
                  'text-white bg-red-600 hover:bg-red-500': notification.type === 'error'
                }"
              >
                OK
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>
  </TransitionGroup>
</template>

<script setup lang="ts">
import { useNotifications } from '../../composables/useNotifications'

const { notifications, remove } = useNotifications()
</script>

<style scoped>
/* Wrapper transition - only opacity for backdrop fade */
.notification-enter-active,
.notification-leave-active {
  transition: opacity 0.25s ease;
}

.notification-enter-from,
.notification-leave-to {
  opacity: 0;
}

/* Modal transition - scale animation */
.notification-enter-active .notification-modal,
.notification-leave-active .notification-modal {
  transition: transform 0.25s ease;
}

.notification-enter-from .notification-modal {
  transform: scale(0.9);
}

.notification-leave-to .notification-modal {
  transform: scale(0.9);
}

.notification-move {
  transition: transform 0.25s ease;
}
</style>
