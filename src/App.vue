<template>
  <SplashScreen ref="splashScreenRef" @start="handleAudioStart" />
  <IndexPage />
  
  <!-- Debug: Audio Engine Test Panel -->
  <div v-if="audioEngine" class="fixed bottom-4 right-4 bg-gray-900 border border-gray-700 rounded-lg p-4 shadow-xl z-50">
    <div class="text-xs font-mono text-gray-300 mb-2">
      <div>Engine Status: <span :class="audioEngine.state.value.isRunning ? 'text-green-400' : 'text-red-400'">
        {{ audioEngine.state.value.isRunning ? 'RUNNING' : 'STOPPED' }}
      </span></div>
      <div>Devices: {{ audioEngine.state.value.devices.length }}</div>
    </div>
    <div class="flex gap-2">
      <button 
        @click="testStartEngine" 
        :disabled="audioEngine.state.value.isRunning"
        class="px-3 py-1 text-xs font-bold rounded transition-all"
        :class="audioEngine.state.value.isRunning ? 'bg-gray-700 text-gray-500 cursor-not-allowed' : 'bg-green-600 hover:bg-green-700 text-white'">
        START ENGINE
      </button>
      <button 
        @click="testStopEngine"
        :disabled="!audioEngine.state.value.isRunning"
        class="px-3 py-1 text-xs font-bold rounded transition-all"
        :class="!audioEngine.state.value.isRunning ? 'bg-gray-700 text-gray-500 cursor-not-allowed' : 'bg-red-600 hover:bg-red-700 text-white'">
        STOP ENGINE
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { provide, ref, watch } from 'vue'
import IndexPage from './index.vue'
import SplashScreen from './components/layout/SplashScreen.vue'
import { useAudioEngine } from './composables/useAudioEngine'

const isAppReady = ref(false)
const splashScreenRef = ref<InstanceType<typeof SplashScreen> | null>(null)

// Initialize Rust audio engine
const audioEngine = useAudioEngine()

// Initialize audio engine after user interaction
const handleAudioStart = async () => {
  console.log('[App] User clicked start - initializing Rust audio engine...')
  
  // Load available audio devices from Rust engine
  await audioEngine.loadDevices()
  
  console.log('[App] Rust audio engine initialized')
}

// Test functions for audio engine
const testStartEngine = async () => {
  console.log('[App] Starting Rust audio engine...')
  await audioEngine.start()
}

const testStopEngine = async () => {
  console.log('[App] Stopping Rust audio engine...')
  await audioEngine.stop()
}

// Watch for app ready state and hide splash screen after 3 seconds
watch(isAppReady, (ready) => {
  if (ready) {
    setTimeout(() => {
      splashScreenRef.value?.hide()
    }, 3000)
  }
})

// Provide audio engine and app ready state to all child components
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
</script>