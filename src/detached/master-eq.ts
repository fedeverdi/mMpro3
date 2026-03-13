import '../index.css'
import { createApp } from 'vue'
import { DetachedWindowClient } from '../lib/detachedWindowClient'
import { DetachedAudioEngineProxy } from '../lib/detachedAudioEngineProxy'
import DetachedMasterEQ from './components/DetachedMasterEQ.vue'

// Create WebSocket client for this detached window
const wsClient = new DetachedWindowClient('master-eq')

// Create audio engine proxy that sends commands via WebSocket
const audioEngineProxy = new DetachedAudioEngineProxy(wsClient)

// Create Vue app
const app = createApp(DetachedMasterEQ, {
  wsClient,
  audioEngineProxy
})

// Provide audio engine to components (this is how child components access it)
app.provide('audioEngine', audioEngineProxy)

app.mount('#detached-app')

// Connect to WebSocket server
wsClient.connect().catch(err => {
  console.error('[DetachedMasterEQ] Failed to connect:', err)
})

// Clean up on window unload
window.addEventListener('beforeunload', () => {
  wsClient.disconnect()
})
