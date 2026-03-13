<template>
  <SplashScreen v-if="!isAppReady" :engine-ready="engineReady" @start="handleUserStart" />
  <IndexPage :audio-engine="audioEngine" v-else />
</template>

<script setup lang="ts">
import { provide, ref, onMounted, onUnmounted } from 'vue'
import IndexPage from './index.vue'
import SplashScreen from './components/layout/SplashScreen.vue'
import { useAudioEngine } from './composables/useAudioEngine'
import { useAudioDevices } from './composables/useAudioDevices'
import { useNotifications } from './composables/useNotifications'

const isAppReady = ref(false)
const engineReady = ref(false)

// Initialize Rust audio engine
const audioEngine = useAudioEngine()

// Initialize audio devices composable
const { enumerateAudioInputs } = useAudioDevices()

// Notifications system
const { error: showError } = useNotifications()

// Initialize audio engine during splash screen
const initializeEngine = async () => {
  
  try {
    // Load available audio devices from Rust engine (outputs)
    const loadDevicesPromise = audioEngine.loadDevices()
    
    // Enumerate audio input devices (skip if in remote mode)
    const loadInputsPromise = enumerateAudioInputs()
    
    // Wait for both to complete (allow partial failures)
    await Promise.allSettled([loadDevicesPromise, loadInputsPromise])
    
    // If in remote mode, give WebSocket time to sync engine state
    const isRemoteMode = !(window as any).electronAPI
    if (isRemoteMode) {
      await new Promise(resolve => setTimeout(resolve, 500)) // Wait 500ms for state sync
    }
    
    // Pre-initialize engine (but don't start audio yet - requires user interaction)
    engineReady.value = true
  } catch (error) {
    console.error('[App] Failed to initialize engine:', error)
    
    // Check if it's a timeout error
    const errorMessage = error instanceof Error ? error.message : String(error)
    if (errorMessage.includes('Timeout')) {
      showError(
        'Avvio audio engine troppo lento. Chiudi e riapri l\'applicazione. Se il problema persiste, verifica i permessi audio nelle Impostazioni di Sistema.',
        8000
      )
    } else {
      showError(
        'Errore durante l\'inizializzazione dell\'audio engine. Riavvia l\'applicazione.',
        5000
      )
    }
  }
}

// Start audio after user clicks (required by browser autoplay policies)
const handleUserStart = async () => {
  console.log('[App] Starting audio engine...')
  
  try {
    // Start engine with user gesture (will check if already running internally)
    await audioEngine.start()
    
    // If in remote mode, notify server that this client is now actively controlling
    const isRemoteMode = !(window as any).electronAPI
    if (isRemoteMode && (window as any).audioEngine?.notifyRemoteControlStarted) {
      (window as any).audioEngine.notifyRemoteControlStarted()
      console.log('[App] Notified server: remote control started')
    }
    
    isAppReady.value = true
  } catch (error) {
    console.error('[App] Failed to start engine:', error)
    
    // Check if it's a timeout error
    const errorMessage = error instanceof Error ? error.message : String(error)
    if (errorMessage.includes('Timeout')) {
      showError(
        'Timeout durante l\'avvio dell\'audio engine. Chiudi e riapri l\'applicazione.',
        6000
      )
    } else {
      showError(
        'Errore durante l\'avvio dell\'audio engine. Riavvia l\'applicazione.',
        5000
      )
    }
  }
}

// Exit remote control (go back to splash screen)
const handleExitRemoteControl = (notifyServer = true) => {
  const isRemoteMode = !(window as any).electronAPI
  if (isRemoteMode && notifyServer && (window as any).audioEngine?.notifyRemoteControlStopped) {
    (window as any).audioEngine.notifyRemoteControlStopped()
    console.log('[App] Notified server: remote control stopped')
  }
  
  // Reset to splash screen
  isAppReady.value = false
  engineReady.value = false
  
  // Re-initialize after a short delay
  setTimeout(() => {
    initializeEngine()
  }, 100)
}

// Handle remote control disconnection (for remote mode)
const handleRemoteDisconnection = (event: any) => {
  console.log('[App] Remote control disconnected:', event.detail?.message)
  // Return to splash screen without notifying server (already disconnected)
  handleExitRemoteControl(false)
}

// Auto-initialize on mount (during splash screen)
onMounted(() => {
  initializeEngine()
  
  // Listen for remote control disconnection (only in remote mode)
  const isRemoteMode = !(window as any).electronAPI
  if (isRemoteMode) {
    window.addEventListener('remote-control-disconnected', handleRemoteDisconnection)
  }
})

// Cleanup on unmount
onUnmounted(() => {
  const isRemoteMode = !(window as any).electronAPI
  if (isRemoteMode) {
    window.removeEventListener('remote-control-disconnected', handleRemoteDisconnection)
  }
})

// Provide audio engine and app ready state to all child components
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
provide('exitRemoteControl', handleExitRemoteControl)
</script>