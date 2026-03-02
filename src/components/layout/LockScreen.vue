<template>
  <Transition name="fade">
    <div v-if="show" class="lock-screen">
      <!-- Animated Background Effects -->
      <div class="bg-gradient-animated"></div>
      <div class="wave wave-1"></div>
      <div class="wave wave-2"></div>
      <div class="wave wave-3"></div>
      
      <!-- Floating Particles -->
      <div class="particles">
        <div v-for="i in 20" :key="i" class="particle" :style="getParticleStyle(i)"></div>
      </div>
      
      <div class="lock-content">
        <!-- Lock Icon -->
        <div class="lock-icon-container">
          <svg class="lock-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
        </div>
        
        <h1 class="lock-title">Interface Locked</h1>
        <p class="lock-subtitle">Enter password to unlock the mixer</p>
        
        <!-- Unlock Form -->
        <div class="unlock-form">
          <input v-model="password" type="password" placeholder="Enter password" 
            @keyup.enter="handleUnlock"
            class="password-input"
            autofocus />
          
          <div v-if="error" class="error-message">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>{{ error }}</span>
          </div>
          
          <button @click="handleUnlock" class="unlock-button">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
            </svg>
            <span>Unlock Interface</span>
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  show: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  unlock: [password: string]
}>()

const password = ref('')
const error = ref('')

watch(() => props.show, (newVal) => {
  if (newVal) {
    password.value = ''
    error.value = ''
  }
})

const handleUnlock = () => {
  if (!password.value) {
    error.value = 'Please enter password'
    return
  }
  
  emit('unlock', password.value)
}

// Expose method to show error
defineExpose({
  showError: (message: string) => {
    error.value = message
    password.value = ''
  }
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
</script>

<style scoped>
.lock-screen {
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
  background: radial-gradient(ellipse at center, rgba(239, 68, 68, 0.15) 0%, rgba(220, 38, 38, 0.1) 25%, transparent 50%);
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
  background: radial-gradient(ellipse at 30% 50%, rgba(239, 68, 68, 0.1) 0%, transparent 50%);
  animation: wave 8s ease-in-out infinite;
}

.wave-2 {
  background: radial-gradient(ellipse at 70% 50%, rgba(220, 38, 38, 0.08) 0%, transparent 50%);
  animation: wave 6s ease-in-out infinite reverse;
  animation-delay: 1s;
}

.wave-3 {
  background: radial-gradient(ellipse at 50% 80%, rgba(239, 68, 68, 0.06) 0%, transparent 50%);
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
  background: radial-gradient(circle, rgba(239, 68, 68, 0.8) 0%, rgba(220, 38, 38, 0.4) 100%);
  border-radius: 50%;
  animation: float-up linear infinite;
  opacity: 0;
}

.lock-content {
  position: relative;
  z-index: 10;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
  max-width: 500px;
  width: 90%;
}

.lock-icon-container {
  width: 120px;
  height: 120px;
  background: rgba(239, 68, 68, 0.1);
  border: 3px solid rgba(239, 68, 68, 0.3);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: pulse-lock 2s ease-in-out infinite;
}

.lock-icon {
  width: 60px;
  height: 60px;
  color: #ef4444;
  filter: drop-shadow(0 0 20px rgba(239, 68, 68, 0.5));
}

.lock-title {
  font-size: 2.5rem;
  font-weight: 800;
  color: white;
  text-align: center;
  letter-spacing: 0.05em;
}

.lock-subtitle {
  color: #9ca3af;
  font-size: 1rem;
  text-align: center;
  margin-top: -1rem;
}

.unlock-form {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  margin-top: 1rem;
}

.password-input {
  width: 100%;
  padding: 1rem 1.5rem;
  background: rgba(17, 24, 39, 0.8);
  border: 2px solid rgba(239, 68, 68, 0.3);
  border-radius: 12px;
  color: white;
  font-size: 1.125rem;
  text-align: center;
  outline: none;
  transition: all 0.3s ease;
}

.password-input:focus {
  border-color: #ef4444;
  background: rgba(17, 24, 39, 0.95);
  box-shadow: 0 0 20px rgba(239, 68, 68, 0.2);
}

.password-input::placeholder {
  color: #6b7280;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  color: #ef4444;
  font-size: 0.875rem;
}

.unlock-button {
  width: 100%;
  padding: 1rem 1.5rem;
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  border: none;
  border-radius: 12px;
  color: white;
  font-size: 1rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  box-shadow: 0 4px 20px rgba(239, 68, 68, 0.3);
}

.unlock-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 30px rgba(239, 68, 68, 0.5);
}

.unlock-button:active {
  transform: translateY(0);
}

@keyframes pulse-lock {
  0%, 100% {
    transform: scale(1);
    box-shadow: 0 0 20px rgba(239, 68, 68, 0.3);
  }
  50% {
    transform: scale(1.05);
    box-shadow: 0 0 40px rgba(239, 68, 68, 0.5);
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
