<template>
  <SplashScreen v-if="!isAppReady" :engine-ready="engineReady" @start="handleUserStart" />
  <IndexPage :audio-engine="audioEngine" v-else />
</template>

<script setup lang="ts">
import { provide, ref, onMounted } from 'vue'
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
  console.log('[App] Initializing Rust audio engine...')
  
  try {
    // Load available audio devices from Rust engine (outputs)
    const loadDevicesPromise = audioEngine.loadDevices()
    
    // Enumerate audio input devices
    const loadInputsPromise = enumerateAudioInputs()
    
    // Wait for both to complete
    await Promise.all([loadDevicesPromise, loadInputsPromise])
    
    // Pre-initialize engine (but don't start audio yet - requires user interaction)
    engineReady.value = true
    console.log('[App] Engine and devices ready')
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
    // Start engine with user gesture
    await audioEngine.start()
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

// Auto-initialize on mount (during splash screen)
onMounted(() => {
  initializeEngine()
})

// Provide audio engine and app ready state to all child components
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
</script>