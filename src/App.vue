<template>
  <SplashScreen ref="splashScreenRef" />
  <IndexPage />
</template>

<script setup lang="ts">
import { provide, onMounted, ref, watch } from 'vue'
import IndexPage from './index.vue'
import SplashScreen from './components/layout/SplashScreen.vue'

const Tone = ref<any>(null)
const isAppReady = ref(false)
const splashScreenRef = ref<InstanceType<typeof SplashScreen> | null>(null)

// Import Tone.js once for the entire app
onMounted(async () => {
  Tone.value = await import('tone')

  const context = Tone.value.getContext()
  
  // Set lookAhead for better timing precision
  context.lookAhead = 0.060
})

// Watch for app ready state and hide splash screen after 3 seconds
watch(isAppReady, (ready) => {
  if (ready) {
    setTimeout(() => {
      splashScreenRef.value?.hide()
    }, 3000)
  }
})

// Provide Tone.js and app ready state to all child components
provide('Tone', Tone)
provide('isAppReady', isAppReady)
</script>