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
  
  // Set lookAhead for better timing precision (500ms)
  context.lookAhead = 0.060
  
  // Log Tone.js state every 5 seconds
  setInterval(() => {
    if (!Tone.value) return
    
    const ctx = context
    const rawCtx = ctx.rawContext
    
    const stats = {
      // Audio Context
      state: rawCtx.state,
      sampleRate: rawCtx.sampleRate,
      currentTime: rawCtx.currentTime.toFixed(2),
      
      // Latency
      baseLatency: rawCtx.baseLatency ? (rawCtx.baseLatency * 1000).toFixed(2) + 'ms' : 'N/A',
      outputLatency: rawCtx.outputLatency ? (rawCtx.outputLatency * 1000).toFixed(2) + 'ms' : 'N/A',
      
      // Tone.js specific
      lookAhead: ctx.lookAhead,
      updateInterval: ctx.updateInterval,
      
      // Buffer
      bufferSize: rawCtx.baseLatency ? Math.round(rawCtx.baseLatency * rawCtx.sampleRate) : 'N/A',
      
      // Memory (Performance API)
      memory: (performance as any).memory ? {
        usedJSHeapSize: ((performance as any).memory.usedJSHeapSize / 1048576).toFixed(2) + 'MB',
        totalJSHeapSize: ((performance as any).memory.totalJSHeapSize / 1048576).toFixed(2) + 'MB',
        jsHeapSizeLimit: ((performance as any).memory.jsHeapSizeLimit / 1048576).toFixed(2) + 'MB'
      } : 'N/A'
    }
    
    console.log('📊 Tone.js Status:', stats)
  }, 5000)
})

// Provide Tone.js to all child components
provide('Tone', Tone)
</script>