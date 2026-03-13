import '../index.css'
import { createApp } from 'vue'
import { DetachedWindowClient } from '../lib/detachedWindowClient'
import DetachedSpectrum from './components/DetachedSpectrum.vue'

// Create WebSocket client for this detached window
const wsClient = new DetachedWindowClient('spectrum')

// Create Vue app
const app = createApp(DetachedSpectrum, {
  wsClient
})

app.mount('#detached-app')

// Connect to WebSocket server
wsClient.connect().catch(err => {
  console.error('[DetachedSpectrum] Failed to connect:', err)
})

// Clean up on window unload
window.addEventListener('beforeunload', () => {
  wsClient.disconnect()
})
