<template>
  <IndexPage />
</template>

<script setup lang="ts">
import { provide, ref, onMounted } from 'vue'
import IndexPage from './index.vue'
import { useAudioEngine } from './composables/useAudioEngine'

const isAppReady = ref(false)

// Initialize Rust audio engine
const audioEngine = useAudioEngine()

// Initialize audio engine automatically on mount
const handleAudioStart = async () => {
  console.log('[App] Auto-starting Rust audio engine...')
  
  // Load available audio devices from Rust engine
  await audioEngine.loadDevices()
  
  // Start engine automatically
  await audioEngine.start()
  
  isAppReady.value = true
}

// Auto-start on mount
onMounted(() => {
  handleAudioStart()
})

// Provide audio engine and app ready state to all child components
provide('audioEngine', audioEngine)
provide('isAppReady', isAppReady)
</script>