<template>
  <IndexPage />
</template>

<script setup lang="ts">
import { provide, onMounted, ref } from 'vue'
import IndexPage from './index.vue'

const Tone = ref<any>(null)

// Import Tone.js once for the entire app
onMounted(async () => {
  Tone.value = await import('tone')

  const context = Tone.value.getContext()
  
  // Set lookAhead for better timing precision
  context.lookAhead = 0.060
})

// Provide Tone.js to all child components
provide('Tone', Tone)
</script>