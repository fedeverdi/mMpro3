<template>
  <SplashScreen ref="splashScreenRef" @start="handleAudioStart" />
  <IndexPage />
</template>

<script setup lang="ts">
import { provide, ref, watch } from 'vue'
import IndexPage from './index.vue'
import SplashScreen from './components/layout/SplashScreen.vue'
import { useAudioEngine } from './composables/useAudioEngine'

const Tone = ref<any>(null)
const isAppReady = ref(false)
const splashScreenRef = ref<InstanceType<typeof SplashScreen> | null>(null)

// Initialize audio engine
const audioEngine = useAudioEngine()

// Import Tone.js and initialize audio context ONLY after user interaction
const handleAudioStart = async () => {
  console.log('[App] User clicked start - initializing audio...')
  
  // Import Tone.js
  Tone.value = await import('tone')

  const context = Tone.value.getContext()
  
  // Start the audio context (now safe because user clicked)
  if (context.state !== 'running') {
    await context.resume()
  }
  
  // Set lookAhead for better timing precision
  context.lookAhead = 0.060
  
  console.log('[App] Audio context initialized:', context.state)
  
  // Initialize Rust audio engine
  await audioEngine.loadDevices()
}

// Watch for app ready state and hide splash screen after 3 seconds
watch(isAppReady, (ready) => {
  if (ready) {
    setTimeout(() => {
      splashScreenRef.value?.hide()
    }, 3000)
  }
})

// Provide Tone.js, audio engine and app ready state to all child components
provide('Tone', Tone)
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
</script>