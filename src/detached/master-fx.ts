import '../index.css'
import { createApp } from 'vue'
import { DetachedWindowClient } from '../lib/detachedWindowClient'
import { DetachedAudioEngineProxy } from '../lib/detachedAudioEngineProxy'
import DetachedMasterFX from './components/DetachedMasterFX.vue'

// Create WebSocket client for this detached window
const wsClient = new DetachedWindowClient('master-fx')

// Create audio engine proxy that sends commands via WebSocket
const audioEngineProxy = new DetachedAudioEngineProxy(wsClient)

// Create Vue app
const app = createApp(DetachedMasterFX, {
  wsClient,
  audioEngineProxy
})

// Provide audio engine to components
app.provide('audioEngine', audioEngineProxy)

app.mount('#detached-app')

// Connect to WebSocket server
wsClient.connect().catch(err => {
  console.error('[DetachedMasterFX] Failed to connect:', err)
})

// Clean up on window unload
window.addEventListener('beforeunload', () => {
  wsClient.disconnect()
})
