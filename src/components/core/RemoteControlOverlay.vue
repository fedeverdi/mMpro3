<template>
  <Transition name="fade">
    <div v-if="isActive" class="remote-control-overlay">
      <div class="overlay-content">
        <div class="icon">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
            <line x1="8" y1="21" x2="16" y2="21"></line>
            <line x1="12" y1="17" x2="12" y2="21"></line>
          </svg>
        </div>
        <h2>Remote Control Enabled</h2>
        <p>This mixer is being controlled from a remote device</p>
        <p class="hint">Local controls are temporarily disabled</p>
        
        <button @click="handleTakeControl" class="take-control-btn">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
              d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
          Take Control
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
defineProps<{
  isActive: boolean
}>()

const handleTakeControl = async () => {
  // Call Electron API to disconnect remote clients
  if ((window as any).electronAPI?.disconnectRemoteClients) {
    await (window as any).electronAPI.disconnectRemoteClients()
    console.log('[RemoteControlOverlay] Disconnected remote clients and took control')
  }
}
</script>

<style scoped>
.remote-control-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(10px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: all;
  user-select: none;
}

.overlay-content {
  text-align: center;
  color: white;
  padding: 3rem;
  max-width: 500px;
}

.icon {
  width: 80px;
  height: 80px;
  margin: 0 auto 2rem;
  color: #22c55e;
  animation: pulse 2s ease-in-out infinite;
}

.icon svg {
  width: 100%;
  height: 100%;
}

h2 {
  font-size: 2rem;
  font-weight: 700;
  margin-bottom: 1rem;
  color: white;
}

p {
  font-size: 1.125rem;
  color: rgba(255, 255, 255, 0.8);
  margin-bottom: 0.5rem;
}

.hint {
  font-size: 0.875rem;
  color: rgba(255, 255, 255, 0.5);
  margin-top: 1.5rem;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.05);
  }
}

.take-control-btn {
  margin: 2rem auto 0 auto;
  padding: 0.875rem 2rem;
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
  border: none;
  border-radius: 0.5rem;
  font-size: 1rem;
  font-weight: 700;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  transition: all 0.2s ease;
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
}

.take-control-btn:hover {
  background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%);
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(239, 68, 68, 0.5);
}

.take-control-btn:active {
  transform: translateY(0);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
