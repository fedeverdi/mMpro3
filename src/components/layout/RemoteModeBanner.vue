<template>
  <div v-if="isRemoteMode && !dismissed" class="fixed top-16 left-1/2 transform -translate-x-1/2 z-[9999] animate-fade-in-down">
    <div class="bg-gradient-to-r from-purple-900/95 to-blue-900/95 backdrop-blur-sm border border-purple-500/50 rounded-lg shadow-2xl p-4 max-w-xl">
      <div class="flex items-start gap-3">
        <!-- Icon -->
        <div class="flex-shrink-0">
          <svg class="w-6 h-6 text-purple-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        
        <!-- Content -->
        <div class="flex-1 text-white">
          <h3 class="font-bold text-sm mb-1 text-purple-100">🌐 Remote Control Mode</h3>
          <p class="text-xs text-purple-200 leading-relaxed mb-2">
            You're controlling a remote audio engine. Some features are limited:
          </p>
          <ul class="text-xs text-purple-200 space-y-0.5 mb-2">
            <li>• Audio devices managed on host only</li>
            <li>• Library and file operations not available</li>
            <li>• Recording features disabled</li>
            <li>• All audio processing happens on the host machine</li>
          </ul>
          <p class="text-xs text-purple-300 italic">
            Perfect for remote mixing sessions!
          </p>
        </div>
        
        <!-- Close button -->
        <button
          @click="dismiss"
          class="flex-shrink-0 text-purple-300 hover:text-white transition-colors p-1"
          title="Dismiss"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      
      <!-- Progress indicator -->
      <div class="mt-3 h-1 bg-purple-950/50 rounded-full overflow-hidden">
        <div 
          class="h-full bg-gradient-to-r from-purple-400 to-blue-400 transition-all duration-200"
          :style="{ width: connectionStrength + '%' }"
        ></div>
      </div>
      <p class="text-xs text-purple-300 mt-1 text-right">
        Connection: {{ connectionStrength }}%
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const isRemoteMode = ref(false)
const dismissed = ref(false)
const connectionStrength = ref(100)

// Check if we're in remote mode (browser, not Electron)
onMounted(() => {
  isRemoteMode.value = !window.electronAPI
  
  // Check if already dismissed in this session
  const wasDismissed = sessionStorage.getItem('remoteBannerDismissed')
  if (wasDismissed === 'true') {
    dismissed.value = true
  }
})

const dismiss = () => {
  dismissed.value = true
  sessionStorage.setItem('remoteBannerDismissed', 'true')
}
</script>

<style scoped>
@keyframes fade-in-down {
  from {
    opacity: 0;
    transform: translate(-50%, -20px);
  }
  to {
    opacity: 1;
    transform: translate(-50%, 0);
  }
}

.animate-fade-in-down {
  animation: fade-in-down 0.3s ease-out;
}
</style>
