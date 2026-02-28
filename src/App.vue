<template>
  <SplashScreen v-if="!isAppReady" :engine-ready="engineReady" @start="handleUserStart" />
  <IndexPage v-else />
</template>

<script setup lang="ts">
import { provide, ref, onMounted } from 'vue'
import IndexPage from './index.vue'
import SplashScreen from './components/layout/SplashScreen.vue'
import { useAudioEngine } from './composables/useAudioEngine'

const isAppReady = ref(false)
const engineReady = ref(false)

// Initialize Rust audio engine
const audioEngine = useAudioEngine()

// Initialize audio engine during splash screen
const initializeEngine = async () => {
  console.log('[App] Initializing Rust audio engine...')
  
  try {
    // Load available audio devices from Rust engine
    await audioEngine.loadDevices()
    
    // Pre-initialize engine (but don't start audio yet - requires user interaction)
    engineReady.value = true
    console.log('[App] Engine ready')
  } catch (error) {
    console.error('[App] Failed to initialize engine:', error)
  }
}

// Start audio after user clicks (required by browser autoplay policies)
const handleUserStart = async () => {
  console.log('[App] Starting audio engine...')
  
  try {
    // Start engine with user gesture
    await audioEngine.start()
    isAppReady.value = true
  } catch (error) {
    console.error('[App] Failed to start engine:', error)
  }
}

// Auto-initialize on mount (during splash screen)
onMounted(() => {
  initializeEngine()
})

// Provide audio engine and app ready state to all child components
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
</script>