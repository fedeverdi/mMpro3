<template>
  <SplashScreen 
    ref="splashScreenRef"
    v-if="!isAppReady" 
    :engine-ready="engineReady" 
    @start="handleUserStart" 
  />
  <IndexPage :audio-engine="audioEngine" v-else />
  <TakeControlModal 
    :show="showTakeControlModal" 
    :message="takeControlMessage"
    @confirm="handleTakeControl"
    @cancel="handleCancelTakeControl"
  />
</template>

<script setup lang="ts">
import { provide, ref, onMounted, onUnmounted } from 'vue'
import IndexPage from './index.vue'
import SplashScreen from './components/layout/SplashScreen.vue'
import TakeControlModal from './components/core/TakeControlModal.vue'
import { useAudioEngine } from './composables/useAudioEngine'
import { useAudioDevices } from './composables/useAudioDevices'
import { useNotifications } from './composables/useNotifications'
import { useLicense } from './composables/useLicense'

const isAppReady = ref(false)
const engineReady = ref(false)
const showTakeControlModal = ref(false)
const takeControlMessage = ref('C\'è già un client remoto attivo. Vuoi prendere il controllo?')
const splashScreenRef = ref<InstanceType<typeof SplashScreen> | null>(null)

// Initialize Rust audio engine
const audioEngine = useAudioEngine()

// Initialize license system
const license = useLicense()

// Initialize audio devices composable
const { enumerateAudioInputs } = useAudioDevices()

// Notifications system
const { error: showError } = useNotifications()

// Initialize audio engine during splash screen
const initializeEngine = async () => {
  console.log('[App] Starting engine initialization...')
  
  try {
    // Load license from Rust engine FIRST (must complete before UI renders)
    console.log('[App] Waiting for license load...')
    await license.waitForLicenseLoad()
    console.log('[App] License load complete, license type:', license.licenseType.value)
    
    // Load available audio devices from Rust engine (outputs)
    const loadDevicesPromise = audioEngine.loadDevices()
    
    // Enumerate audio input devices (skip if in remote mode)
    const loadInputsPromise = enumerateAudioInputs()
    
    // Wait for both to complete (allow partial failures)
    await Promise.allSettled([loadDevicesPromise, loadInputsPromise])
    console.log('[App] Devices loaded')
    
    // If in remote mode, give WebSocket time to sync engine state
    const isRemoteMode = !(window as any).electronAPI
    if (isRemoteMode) {
      console.log('[App] Remote mode: waiting for state sync...')
      await new Promise(resolve => setTimeout(resolve, 500)) // Wait 500ms for state sync
    }
    
    // Pre-initialize engine (but don't start audio yet - requires user interaction)
    engineReady.value = true
    console.log('[App] Engine initialization complete')
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
      // Don't set isAppReady = true here - wait for server response
      // (either remote-already-active or remote-control-accepted)
    } else {
      // Electron mode - proceed immediately
      splashScreenRef.value?.hide()
      isAppReady.value = true
    }
  } catch (error) {
    console.error('[App] Failed to start engine:', error)
    
    // Reset splash screen on error
    splashScreenRef.value?.reset()
    
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
  
  // Reset splash screen to show button again
  splashScreenRef.value?.reset()
  
  // Return to splash screen without notifying server (already disconnected)
  handleExitRemoteControl(false)
}

const handleRemoteControlTaken = (event: any) => {
  console.log('[App] Remote control taken successfully:', event.detail?.message)
  // Close modal if open
  showTakeControlModal.value = false
  // Hide splash screen and show main view
  splashScreenRef.value?.hide()
  isAppReady.value = true
}

const handleRemoteControlAccepted = (event: any) => {
  console.log('[App] Remote control accepted:', event.detail?.message)
  // No other remote was active - proceed to show main view
  splashScreenRef.value?.hide()
  isAppReady.value = true
}

const handleRemoteAlreadyActive = (event: any) => {
  console.log('[App] Remote already active, showing take control modal')
  takeControlMessage.value = event.detail?.message || 'C\'è già un client remoto attivo. Vuoi prendere il controllo?'
  showTakeControlModal.value = true
}

const handleTakeControl = () => {
  console.log('[App] User confirmed take control')
  showTakeControlModal.value = false
  
  // Send force-take-control message via audioEngine
  const remoteEngine = (window as any).audioEngine
  if (remoteEngine?.forceTakeControl) {
    remoteEngine.forceTakeControl()
  }
}

const handleCancelTakeControl = () => {
  console.log('[App] User cancelled take control')
  showTakeControlModal.value = false
  
  // Reset splash screen to show button again
  splashScreenRef.value?.reset()
  
  // Cancel connection and return to splash screen
  const remoteEngine = (window as any).audioEngine
  if (remoteEngine?.cancelConnection) {
    remoteEngine.cancelConnection()
  }
  
  // Reset to splash screen (will be handled by remote-control-disconnected event)
}

// Auto-initialize on mount (during splash screen)
onMounted(() => {
  initializeEngine()
  
  // Listen for remote control events (only in remote mode)
  const isRemoteMode = !(window as any).electronAPI
  if (isRemoteMode) {
    window.addEventListener('remote-control-disconnected', handleRemoteDisconnection)
    window.addEventListener('remote-control-taken', handleRemoteControlTaken)
    window.addEventListener('remote-control-accepted', handleRemoteControlAccepted)
    window.addEventListener('remote-already-active', handleRemoteAlreadyActive)
  }
})

// Cleanup on unmount
onUnmounted(() => {
  const isRemoteMode = !(window as any).electronAPI
  if (isRemoteMode) {
    window.removeEventListener('remote-control-disconnected', handleRemoteDisconnection)
    window.removeEventListener('remote-control-taken', handleRemoteControlTaken)
    window.removeEventListener('remote-control-accepted', handleRemoteControlAccepted)
    window.removeEventListener('remote-already-active', handleRemoteAlreadyActive)
  }
})

// Provide audio engine and app ready state to all child components
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
provide('exitRemoteControl', handleExitRemoteControl)
</script>