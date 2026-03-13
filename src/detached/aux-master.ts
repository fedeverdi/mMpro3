import '../index.css'
import { createApp } from 'vue'
import { DetachedWindowClient } from '../lib/detachedWindowClient'
import { DetachedAudioEngineProxy } from '../lib/detachedAudioEngineProxy'
import DetachedAuxMaster from './components/DetachedAuxMaster.vue'

// Create WebSocket client for this detached window
const wsClient = new DetachedWindowClient('aux-master')

// Create audio engine proxy that sends commands via WebSocket
const audioEngineProxy = new DetachedAudioEngineProxy(wsClient)

// Create Vue app
const app = createApp(DetachedAuxMaster, {
  wsClient,
  audioEngineProxy
})

// Provide audio engine to components
app.provide('audioEngine', audioEngineProxy)

app.mount('#detached-app')

// Connect to WebSocket server
wsClient.connect().catch(err => {
  console.error('[DetachedAuxMaster] Failed to connect:', err)
})

// Clean up on window unload
window.addEventListener('beforeunload', () => {
  wsClient.disconnect()
})
