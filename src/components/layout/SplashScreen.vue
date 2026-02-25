<template>
  <Transition name="fade">
    <div v-if="visible" class="splash-screen">
      <!-- Animated Background Effects -->
      <div class="bg-gradient-animated"></div>
      <div class="wave wave-1"></div>
      <div class="wave wave-2"></div>
      <div class="wave wave-3"></div>
      
      <!-- Floating Particles -->
      <div class="particles">
        <div v-for="i in 20" :key="i" class="particle" :style="getParticleStyle(i)"></div>
      </div>
      
      <div class="splash-content">
        <!-- Logo -->
        <h1 class="logo">mMpro3</h1>
        
        <!-- Loading Animation or Start Button -->
        <div v-if="!readyToStart" class="loading-container">
          <div class="spinner"></div>
          <p class="loading-text">Initializing...</p>
        </div>
        
        <!-- Start Button (shown when ready) -->
        <div v-else class="start-container">
          <button @click="handleStart" class="start-button">
            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
            </svg>
            <span>Click to Start</span>
          </button>
          <p class="text-xs text-gray-500 mt-3">Audio will be enabled after you click</p>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const visible = ref(true)
const readyToStart = ref(false)

const emit = defineEmits<{
  start: []
}>()

const hide = () => {
  visible.value = false
}

const handleStart = () => {
  emit('start')
  hide()
}

// Show start button after a short delay
onMounted(() => {
  setTimeout(() => {
    readyToStart.value = true
  }, 1500)
})

// Generate random styles for particles
const getParticleStyle = (index: number) => {
  const size = Math.random() * 4 + 2
  const left = Math.random() * 100
  const animationDuration = Math.random() * 3 + 2
  const animationDelay = Math.random() * 2
  
  return {
    width: `${size}px`,
    height: `${size}px`,
    left: `${left}%`,
    animationDuration: `${animationDuration}s`,
    animationDelay: `${animationDelay}s`
  }
}

defineExpose({ hide })
</script>

<style scoped>
.splash-screen {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: #0a0a0a;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  overflow: hidden;
}

/* Animated gradient background */
.bg-gradient-animated {
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(ellipse at center, rgba(96, 165, 250, 0.15) 0%, rgba(167, 139, 250, 0.1) 25%, transparent 50%);
  animation: rotateGradient 20s linear infinite;
}

/* Waves */
.wave {
  position: absolute;
  width: 100%;
  height: 100%;
  opacity: 0.3;
}

.wave-1 {
  background: radial-gradient(ellipse at 30% 50%, rgba(96, 165, 250, 0.1) 0%, transparent 50%);
  animation: wave 8s ease-in-out infinite;
}

.wave-2 {
  background: radial-gradient(ellipse at 70% 50%, rgba(167, 139, 250, 0.08) 0%, transparent 50%);
  animation: wave 6s ease-in-out infinite reverse;
  animation-delay: 1s;
}

.wave-3 {
  background: radial-gradient(ellipse at 50% 80%, rgba(96, 165, 250, 0.06) 0%, transparent 50%);
  animation: wave 10s ease-in-out infinite;
  animation-delay: 2s;
}

/* Particles */
.particles {
  position: absolute;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.particle {
  position: absolute;
  bottom: -10px;
  background: radial-gradient(circle, rgba(96, 165, 250, 0.8) 0%, rgba(167, 139, 250, 0.4) 100%);
  border-radius: 50%;
  animation: float-up linear infinite;
  opacity: 0;
}

.splash-content {
  position: relative;
  z-index: 10;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3rem;
}

.logo {
  font-size: 4rem;
  font-weight: 800;
  background: linear-gradient(135deg, #60a5fa 0%, #a78bfa 100%);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  letter-spacing: 0.05em;
  animation: pulse 2s ease-in-out infinite;
}

.loading-container,
.start-container {
  margin-top: 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.start-button {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 2rem;
  background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-radius: 9999px;
  color: white;
  font-size: 1.125rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 0 20px rgba(96, 165, 250, 0.3);
}

.start-button:hover {
  transform: translateY(-2px) scale(1.05);
  box-shadow: 0 0 30px rgba(96, 165, 250, 0.5);
  border-color: rgba(255, 255, 255, 0.4);
}

.start-button:active {
  transform: translateY(0) scale(1);
}

.start-button svg {
  width: 2rem;
  height: 2rem;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid rgba(96, 165, 250, 0.1);
  border-top-color: #60a5fa;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.loading-text {
  color: #9ca3af;
  font-size: 0.875rem;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.8;
    transform: scale(1.02);
  }
}

@keyframes rotateGradient {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

@keyframes wave {
  0%, 100% {
    transform: translateY(0) scale(1);
    opacity: 0.3;
  }
  50% {
    transform: translateY(-20px) scale(1.1);
    opacity: 0.5;
  }
}

@keyframes float-up {
  0% {
    bottom: -10px;
    opacity: 0;
  }
  10% {
    opacity: 1;
  }
  90% {
    opacity: 1;
  }
  100% {
    bottom: 110vh;
    opacity: 0;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
