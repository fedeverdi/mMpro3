<template>
  <div v-if="isElectron" class="custom-title-bar bg-black/20 backdrop-blur-md flex items-center justify-between px-3 py-1 select-none relative z-[300]">
    <!-- Wave Effect Background -->
    <div class="wave-effect"></div>
    <!-- Shiny Reflection Effect -->
    <div class="reflection-effect"></div>
    
    <!-- Left: Traffic Lights Space (macOS) + App Title -->
    <div class="flex items-center gap-3 draggable-region">
      <!-- Spacer for macOS traffic lights (close/minimize/maximize buttons) -->
      <div class="w-16 draggable-region" v-if="isMac"></div>
    </div>

    <!-- Center: Window Title (optional, can show project name) -->
    <div class="flex-1 text-center text-xs text-gray-300 font-medium draggable-region">
      {{ projectName }}
    </div>

    <!-- Right: Window Controls (non-macOS only) -->
    <div class="flex items-center gap-1 no-drag" v-if="!isMac">
      <!-- Minimize -->
      <button 
        @click="minimize"
        class="w-8 h-6 flex items-center justify-center hover:bg-gray-700 rounded transition-colors"
        title="Minimize"
      >
        <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
        </svg>
      </button>

      <!-- Maximize / Restore -->
      <button 
        @click="toggleMaximize"
        class="w-8 h-6 flex items-center justify-center hover:bg-gray-700 rounded transition-colors"
        :title="isMaximized ? 'Restore' : 'Maximize'"
      >
        <svg v-if="!isMaximized" class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
        </svg>
        <svg v-else class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25" />
        </svg>
      </button>

      <!-- Close -->
      <button 
        @click="close"
        class="w-8 h-6 flex items-center justify-center hover:bg-red-600 hover:text-white rounded transition-colors"
        title="Close"
      >
        <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Spacer for macOS (right side) -->
    <div class="w-16 draggable-region" v-if="isMac"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

interface Props {
  projectName?: string
}

withDefaults(defineProps<Props>(), {
  projectName: 'Untitled Project'
})

const isMac = ref(false)
const isMaximized = ref(false)
const isElectron = ref(false)

onMounted(async () => {
  // Check if running in Electron
  isElectron.value = !!(window as any).electronAPI
  
  if (!isElectron.value) return
  
  // Detect platform
  isMac.value = await window.electronAPI?.getPlatform() === 'darwin'
  
  // Get initial maximized state
  isMaximized.value = await window.electronAPI?.isMaximized() || false
  
  // Listen for maximize/unmaximize events
  window.electronAPI?.onMaximized(() => {
    isMaximized.value = true
  })
  
  window.electronAPI?.onUnmaximized(() => {
    isMaximized.value = false
  })
})

const minimize = () => {
  window.electronAPI?.minimizeWindow()
}

const toggleMaximize = () => {
  if (isMaximized.value) {
    window.electronAPI?.unmaximizeWindow()
  } else {
    window.electronAPI?.maximizeWindow()
  }
}

const close = () => {
  window.electronAPI?.closeWindow()
}
</script>

<style scoped>
.custom-title-bar {
  -webkit-app-region: drag;
  height: 32px;
  min-height: 32px;
  overflow: hidden;
}

.wave-effect {
  position: absolute;
  top: 0;
  left: -100%;
  width: 300%;
  height: 100%;
  background: repeating-linear-gradient(
    90deg,
    transparent 0%,
    rgba(37, 99, 235, 0.15) 8.33%,
    rgba(29, 78, 216, 0.2) 16.67%,
    rgba(30, 64, 175, 0.2) 25%,
    rgba(29, 78, 216, 0.2) 33.33%,
    rgba(37, 99, 235, 0.15) 41.67%,
    transparent 50%
  );
  animation: wave-slide 8s linear infinite;
  pointer-events: none;
  z-index: 0;
}

.reflection-effect {
  position: absolute;
  top: -50%;
  left: -50%;
  width: 80%;
  height: 200%;
  background: linear-gradient(
    135deg,
    transparent 0%,
    transparent 43%,
    rgba(255, 255, 255, 0.15) 50%,
    transparent 57%,
    transparent 100%
  );
  filter: blur(10px);
  animation: reflection-slide 5s ease-in-out infinite;
  pointer-events: none;
  z-index: 1;
}

@keyframes wave-slide {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(33.333%);
  }
}

@keyframes reflection-slide {
  0% {
    transform: translateX(-150%) scaleX(1);
    opacity: 0;
  }
  15% {
    opacity: 1;
  }
  85% {
    opacity: 1;
  }
  100% {
    transform: translateX(450%) scaleX(1.5);
    opacity: 0;
  }
}

.no-drag {
  -webkit-app-region: no-drag;
  position: relative;
  z-index: 2;
}

.draggable-region {
  -webkit-app-region: drag;
  position: relative;
  z-index: 2;
}

button {
  -webkit-app-region: no-drag;
  position: relative;
  z-index: 2;
}
</style>
