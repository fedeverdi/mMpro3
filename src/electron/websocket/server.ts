import { WebSocketServer, WebSocket } from 'ws'
import { BrowserWindow, app } from 'electron'
import { ChildProcess } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { sendCommandAndWaitForResponse } from '../audio-engine/process'

const WS_PORT = 3001

export interface WebSocketServerDependencies {
  getAudioEngineProcess: () => ChildProcess | null
  getIsAudioEngineStarted: () => boolean
  getLastKnownState: () => any
  broadcastRemoteControlState: () => void
}

let wss: WebSocketServer | null = null
let remoteClientsCount = 0
let activeRemoteClientsCount = 0
const activeRemoteClients = new Set<WebSocket>()
const detachedWindowClients = new Set<WebSocket>()

/**
 * Start WebSocket server for remote browser control
 */
export const startWebSocketServer = (deps: WebSocketServerDependencies): void => {
  try {
    wss = new WebSocketServer({ port: WS_PORT })
    
    console.log(`[WebSocket] Server started on port ${WS_PORT}`)
    
    wss.on('connection', (ws: WebSocket) => {
      // Increment remote clients count (browser-based remotes only)
      remoteClientsCount++
      
      // Setup event handlers first (needed for all clients, even if asking for confirmation)
      // Handle client disconnect
      ws.on('close', () => {
        // If this client was actively controlling, decrement the count
        if (activeRemoteClients.has(ws)) {
          activeRemoteClients.delete(ws)
          activeRemoteClientsCount = Math.max(0, activeRemoteClientsCount - 1)
        }
        remoteClientsCount = Math.max(0, remoteClientsCount - 1)
        deps.broadcastRemoteControlState()
      })
      
      // Handle messages from remote clients
      ws.on('message', async (data: Buffer) => {
        try {
          const message = JSON.parse(data.toString())
          
          // Handle detached window registration (pop-out components from main Electron app)
          // These clients DO NOT block the mixer - they only receive updates
          if (message.type === 'detached-window-register') {
            handleDetachedWindowRegister(ws, message, deps)
            return
          }
          
          // Handle remote control lifecycle messages
          if (message.type === 'force-take-control') {
            handleForceTakeControl(ws, deps)
            return
          }
          
          if (message.type === 'remote-control-started') {
            handleRemoteControlStarted(ws, deps)
            return
          }
          
          if (message.type === 'remote-control-stopped') {
            handleRemoteControlStopped(ws, deps)
            return
          }
          
          // Handle library operations via IPC-like pattern for WebSocket clients
          if (message.type.startsWith('ipc:audio-engine:')) {
            await handleIpcMessage(ws, message)
            return
          }
          
          // If the remote sends 'start' but the engine is already running, skip the
          // forward to Rust (which would cause an unwanted restart/audio interruption)
          // and just confirm the running state directly.
          if (message.type === 'start' && deps.getIsAudioEngineStarted()) {
            console.log('[WebSocket] Client sent start but engine already running — confirming state')
            ws.send(JSON.stringify({ type: 'started' }))
            return
          }

          // Forward audio engine commands to Rust
          const audioEngineProcess = deps.getAudioEngineProcess()
          if (audioEngineProcess && audioEngineProcess.stdin) {
            audioEngineProcess.stdin.write(JSON.stringify(message) + '\n')
          } else {
            console.error('[WebSocket] Audio engine not running!')
            ws.send(JSON.stringify({ 
              type: 'error', 
              message: 'Audio engine not running' 
            }))
          }
        } catch (error) {
          console.error('[WebSocket] Error processing message:', error)
          ws.send(JSON.stringify({ 
            type: 'error', 
            message: 'Invalid message format' 
          }))
        }
      })
      
      ws.on('close', () => {
        console.log('[WebSocket] Client disconnected')
      })
      
      ws.on('error', (error) => {
        console.error('[WebSocket] Client error:', error)
      })
      
      // Send initial connection success message
      deps.broadcastRemoteControlState()
      ws.send(JSON.stringify({ type: 'connected', message: 'Connected to mMpro3 Audio Engine' }))
      
      // Send current audio engine state immediately to the new client
      const isAudioEngineStarted = deps.getIsAudioEngineStarted()
      if (isAudioEngineStarted) {
        ws.send(JSON.stringify({ type: 'started' }))
        console.log('[WebSocket] Sent current engine state (started) to new client')
      } else {
        ws.send(JSON.stringify({ type: 'stopped' }))
        console.log('[WebSocket] Sent current engine state (stopped) to new client')
      }
    })
    
    wss.on('error', (error) => {
      console.error('[WebSocket] Server error:', error)
    })
  } catch (error) {
    console.error('[WebSocket] Failed to start server:', error)
  }
}

/**
 * Handle detached window registration
 */
function handleDetachedWindowRegister(ws: WebSocket, message: any, deps: WebSocketServerDependencies): void {
  detachedWindowClients.add(ws)
  
  // Remove from detached set on close
  const originalOnClose = ws.listeners('close')[0] as Function
  ws.removeAllListeners('close')
  ws.on('close', () => {
    detachedWindowClients.delete(ws)
    // Call original close handler
    if (originalOnClose) originalOnClose()
  })
  
  // Send confirmation
  ws.send(JSON.stringify({ 
    type: 'detached-window-registered', 
    componentType: message.componentType 
  }))
  
  // Send current audio engine state
  const isAudioEngineStarted = deps.getIsAudioEngineStarted()
  if (isAudioEngineStarted) {
    ws.send(JSON.stringify({ type: 'started' }))
  } else {
    ws.send(JSON.stringify({ type: 'stopped' }))
  }
  
  // Send cached state to newly connected detached window
  const lastKnownState = deps.getLastKnownState()
  if (Object.keys(lastKnownState).length > 0) {    
    // Send as parameters_changed message so components can handle it
    ws.send(JSON.stringify({
      type: 'parameters_changed',
      master: lastKnownState.masterParameters,
      auxes: lastKnownState.auxParameters,
      subgroups: lastKnownState.subgroupParameters
    }))
    
    // Send frontend aux buses state (separate from backend auxParameters)
    if (lastKnownState.auxBuses) {
      ws.send(JSON.stringify({
        type: 'aux_buses_state',
        auxBuses: lastKnownState.auxBuses
      }))
    }
    
    console.log(`[WebSocket] Sent cached state to detached window: ${message.componentType}`)
  } else {
    console.log('[WebSocket] No cached state available for detached window')
  }
}

/**
 * Handle force take control
 */
function handleForceTakeControl(ws: WebSocket, deps: WebSocketServerDependencies): void {
  console.log(`[WebSocket] Force take control requested, disconnecting ${activeRemoteClients.size} active clients`)
  
  // Disconnect all currently active remote clients
  const clientsToDisconnect = Array.from(activeRemoteClients)
  activeRemoteClients.clear()
  activeRemoteClientsCount = 0
  
  clientsToDisconnect.forEach(client => {
    if (client !== ws && client.readyState === WebSocket.OPEN) {
      client.send(JSON.stringify({ type: 'disconnected', reason: 'taken-over' }))
      client.close()
    }
  })
  
  // Now add this client as the new active client
  activeRemoteClients.add(ws)
  activeRemoteClientsCount = 1
  console.log(`[WebSocket] New client took control and is now active`)
  
  // Send confirmation that control was taken
  ws.send(JSON.stringify({ type: 'control-taken', message: 'You are now in control' }))
  
  // Send current audio engine state
  const isAudioEngineStarted = deps.getIsAudioEngineStarted()
  if (isAudioEngineStarted) {
    ws.send(JSON.stringify({ type: 'started' }))
    console.log('[WebSocket] Sent current engine state (started) to new client')
  } else {
    ws.send(JSON.stringify({ type: 'stopped' }))
    console.log('[WebSocket] Sent current engine state (stopped) to new client')
  }
  
  deps.broadcastRemoteControlState()
}

/**
 * Handle remote control started
 */
function handleRemoteControlStarted(ws: WebSocket, deps: WebSocketServerDependencies): void {
  // Check if there's already an active remote client
  if (activeRemoteClientsCount > 0 && !activeRemoteClients.has(ws)) {
    console.log(`[WebSocket] Remote already active, sending confirmation request`)
    ws.send(JSON.stringify({ 
      type: 'remote-already-active', 
      message: 'C\'è già un client remoto attivo. Vuoi prendere il controllo?',
      activeClientsCount: activeRemoteClientsCount
    }))
    return
  }
  
  if (!activeRemoteClients.has(ws)) {
    activeRemoteClients.add(ws)
    activeRemoteClientsCount++
    deps.broadcastRemoteControlState()

    // Send current engine running state so the remote doesn't need to restart it
    const isAudioEngineStarted = deps.getIsAudioEngineStarted()
    if (isAudioEngineStarted) {
      ws.send(JSON.stringify({ type: 'started' }))
    } else {
      ws.send(JSON.stringify({ type: 'stopped' }))
    }

    // Send cached parameter state so faders/gains sync immediately
    const lastKnownState = deps.getLastKnownState()
    if (Object.keys(lastKnownState).length > 0) {
      ws.send(JSON.stringify({
        type: 'parameters_changed',
        master: lastKnownState.masterParameters,
        auxes: lastKnownState.auxParameters,
        subgroups: lastKnownState.subgroupParameters
      }))
      if (lastKnownState.auxBuses) {
        ws.send(JSON.stringify({
          type: 'aux_buses_state',
          auxBuses: lastKnownState.auxBuses
        }))
      }
      console.log('[WebSocket] Sent cached state to new remote client')
    }

    // Send confirmation that control was accepted LAST so the remote shows the UI
    // only after it has already received the state above
    ws.send(JSON.stringify({ 
      type: 'remote-control-accepted', 
      message: 'Control accepted' 
    }))
  }
}

/**
 * Handle remote control stopped
 */
function handleRemoteControlStopped(ws: WebSocket, deps: WebSocketServerDependencies): void {
  if (activeRemoteClients.has(ws)) {
    activeRemoteClients.delete(ws)
    activeRemoteClientsCount = Math.max(0, activeRemoteClientsCount - 1)
    console.log(`[WebSocket] Remote client stopped control (active: ${activeRemoteClientsCount})`)
    deps.broadcastRemoteControlState()
  }
}

/**
 * Handle IPC-like messages (library, playlists, scenes)
 */
async function handleIpcMessage(ws: WebSocket, message: any): Promise<void> {
  try {
    switch (message.type) {
      case 'ipc:audio-engine:get-license':
        await handleGetLicense(ws, message)
        break
      case 'ipc:audio-engine:save-license':
        await handleSaveLicense(ws, message)
        break
      case 'ipc:audio-engine:get-audio-config':
        await handleGetAudioConfig(ws, message)
        break
      case 'ipc:audio-engine:save-audio-config':
        await handleSaveAudioConfig(ws, message)
        break
      case 'ipc:audio-engine:list-library-files':
        await handleListLibraryFiles(ws, message)
        break
      case 'ipc:audio-engine:get-library-file':
        await handleGetLibraryFile(ws, message)
        break
      case 'ipc:audio-engine:save-library-file':
        await handleSaveLibraryFile(ws, message)
        break
      case 'ipc:audio-engine:delete-library-file':
        await handleDeleteLibraryFile(ws, message)
        break
      case 'ipc:audio-engine:save-playlist':
        await handleSavePlaylist(ws, message)
        break
      case 'ipc:audio-engine:list-playlists':
        await handleListPlaylists(ws, message)
        break
      case 'ipc:audio-engine:get-playlist':
        await handleGetPlaylist(ws, message)
        break
      case 'ipc:audio-engine:delete-playlist':
        await handleDeletePlaylist(ws, message)
        break
      default:
        console.warn('[WebSocket] Unknown IPC message type:', message.type)
    }
  } catch (error) {
    console.error('[WebSocket] Error handling IPC message:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Internal error' }))
  }
}

// Library file handlers
async function handleListLibraryFiles(ws: WebSocket, message: any): Promise<void> {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    if (!fs.existsSync(libraryDir)) {
      fs.mkdirSync(libraryDir, { recursive: true })
      ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: [] }))
      return
    }
    
    const files = fs.readdirSync(libraryDir)
      .filter(file => !file.endsWith('.meta.json'))
      .map(file => {
        const filePath = path.join(libraryDir, file)
        const stats = fs.statSync(filePath)
        
        let size = '0 KB'
        if (stats.size < 1024) {
          size = stats.size + ' B'
        } else if (stats.size < 1024 * 1024) {
          size = (stats.size / 1024).toFixed(1) + ' KB'
        } else {
          size = (stats.size / (1024 * 1024)).toFixed(1) + ' MB'
        }
        
        let metadata: any = {}
        const metadataPath = path.join(libraryDir, `${file}.meta.json`)
        if (fs.existsSync(metadataPath)) {
          try {
            metadata = JSON.parse(fs.readFileSync(metadataPath, 'utf-8'))
          } catch (error) {
            console.error('[WebSocket] Error parsing metadata:', error)
          }
        }
        
        return {
          id: metadata.id || file,
          fileName: metadata.originalFileName || file,
          filePath,
          mimeType: metadata.mimeType || 'audio/mpeg',
          size,
          timestamp: metadata.timestamp || stats.birthtimeMs,
          artist: metadata.artist,
          title: metadata.title,
          artwork: metadata.artwork
        }
      })
      .sort((a, b) => b.timestamp - a.timestamp)
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: files }))
  } catch (error) {
    console.error('[WebSocket] Error listing library files:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to list library files' }))
  }
}

async function handleGetLibraryFile(ws: WebSocket, message: any): Promise<void> {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    const filePath = path.join(libraryDir, message.fileId)
    
    if (!fs.existsSync(filePath)) {
      ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: `File not found: ${message.fileId}` }))
      return
    }
    
    const buffer = fs.readFileSync(filePath)
    const arrayBuffer = buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength)
    
    let metadata: any = {}
    const metadataPath = path.join(libraryDir, `${message.fileId}.meta.json`)
    if (fs.existsSync(metadataPath)) {
      try {
        metadata = JSON.parse(fs.readFileSync(metadataPath, 'utf-8'))
      } catch (error) {
        console.error('[WebSocket] Error parsing metadata:', error)
      }
    }
    
    ws.send(JSON.stringify({
      type: 'ipc:response',
      id: message.id,
      data: {
        id: message.fileId,
        fileName: metadata.originalFileName || message.fileId,
        filePath: filePath,
        arrayBuffer: Array.from(new Uint8Array(arrayBuffer)),
        mimeType: metadata.mimeType || 'audio/mpeg',
        timestamp: metadata.timestamp || Date.now(),
        artist: metadata.artist,
        title: metadata.title,
        artwork: metadata.artwork
      }
    }))
  } catch (error) {
    console.error('[WebSocket] Error getting library file:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to get library file' }))
  }
}

async function handleSaveLibraryFile(ws: WebSocket, message: any): Promise<void> {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    if (!fs.existsSync(libraryDir)) {
      fs.mkdirSync(libraryDir, { recursive: true })
    }
    
    const fileId = message.metadata?.id || `${Date.now()}-${message.fileName}`
    const filePath = path.join(libraryDir, fileId)
    
    const buffer = Buffer.from(message.arrayBuffer)
    fs.writeFileSync(filePath, buffer)
    
    const metadata = {
      id: fileId,
      originalFileName: message.fileName,
      timestamp: Date.now(),
      mimeType: message.metadata?.mimeType || 'audio/mpeg',
      artist: message.metadata?.artist,
      title: message.metadata?.title,
      artwork: message.metadata?.artwork
    }
    
    const metadataPath = path.join(libraryDir, `${fileId}.meta.json`)
    fs.writeFileSync(metadataPath, JSON.stringify(metadata, null, 2))
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: fileId }))
  } catch (error) {
    console.error('[WebSocket] Error saving library file:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to save library file' }))
  }
}

async function handleDeleteLibraryFile(ws: WebSocket, message: any): Promise<void> {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    const filePath = path.join(libraryDir, message.fileId)
    const metadataPath = path.join(libraryDir, `${message.fileId}.meta.json`)
    
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath)
    }
    if (fs.existsSync(metadataPath)) {
      fs.unlinkSync(metadataPath)
    }
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: true }))
  } catch (error) {
    console.error('[WebSocket] Error deleting library file:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to delete library file' }))
  }
}

// Playlist handlers
async function handleSavePlaylist(ws: WebSocket, message: any): Promise<void> {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    
    if (!fs.existsSync(playlistsDir)) {
      fs.mkdirSync(playlistsDir, { recursive: true })
    }
    
    const playlistPath = path.join(playlistsDir, `${message.playlist.id}.json`)
    fs.writeFileSync(playlistPath, JSON.stringify(message.playlist, null, 2))
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: true }))
  } catch (error) {
    console.error('[WebSocket] Error saving playlist:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to save playlist' }))
  }
}

async function handleListPlaylists(ws: WebSocket, message: any): Promise<void> {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    
    if (!fs.existsSync(playlistsDir)) {
      fs.mkdirSync(playlistsDir, { recursive: true })
      ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: [] }))
      return
    }
    
    const files = fs.readdirSync(playlistsDir)
      .filter(file => file.endsWith('.json'))
      .map(file => {
        const filePath = path.join(playlistsDir, file)
        try {
          const content = fs.readFileSync(filePath, 'utf-8')
          return JSON.parse(content)
        } catch (error) {
          console.error('[WebSocket] Error parsing playlist:', file, error)
          return null
        }
      })
      .filter(playlist => playlist !== null)
      .sort((a, b) => (b.updatedAt || b.createdAt) - (a.updatedAt || a.createdAt))
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: files }))
  } catch (error) {
    console.error('[WebSocket] Error listing playlists:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to list playlists' }))
  }
}

async function handleGetPlaylist(ws: WebSocket, message: any): Promise<void> {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    const playlistPath = path.join(playlistsDir, `${message.playlistId}.json`)
    
    if (!fs.existsSync(playlistPath)) {
      ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: null }))
      return
    }
    
    const content = fs.readFileSync(playlistPath, 'utf-8')
    const playlist = JSON.parse(content)
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: playlist }))
  } catch (error) {
    console.error('[WebSocket] Error getting playlist:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to get playlist' }))
  }
}

async function handleDeletePlaylist(ws: WebSocket, message: any): Promise<void> {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    const playlistPath = path.join(playlistsDir, `${message.playlistId}.json`)
    
    if (fs.existsSync(playlistPath)) {
      fs.unlinkSync(playlistPath)
    }
    
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: true }))
  } catch (error) {
    console.error('[WebSocket] Error deleting playlist:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to delete playlist' }))
  }
}

// License handlers
async function handleGetLicense(ws: WebSocket, message: any): Promise<void> {
  try {
    const response = await sendCommandAndWaitForResponse({ type: 'get_license' }, 'license', 5000)
    ws.send(JSON.stringify({ 
      type: 'ipc:response', 
      id: message.id, 
      data: {
        key: response.key,
        license_type: response.license_type,
        expires_at: response.expires_at,
        is_valid: response.is_valid
      }
    }))
  } catch (error) {
    console.error('[WebSocket] Error getting license:', error)
    ws.send(JSON.stringify({ 
      type: 'ipc:response', 
      id: message.id, 
      data: {
        key: 'DEMO',
        license_type: 'demo',
        expires_at: null,
        is_valid: true
      }
    }))
  }
}

async function handleSaveLicense(ws: WebSocket, message: any): Promise<void> {
  try {
    const response = await sendCommandAndWaitForResponse({
      type: 'save_license',
      key: message.key,
      license_type: message.license_type,
      expires_at: message.expires_at
    }, 'ok', 5000)
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: true }))
  } catch (error) {
    console.error('[WebSocket] Error saving license:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to save license' }))
  }
}

// Audio config handlers
async function handleGetAudioConfig(ws: WebSocket, message: any): Promise<void> {
  try {
    const response = await sendCommandAndWaitForResponse({ type: 'get_audio_config' }, 'audio_config', 5000)
    ws.send(JSON.stringify({ 
      type: 'ipc:response', 
      id: message.id, 
      data: {
        sample_rate: response.sample_rate,
        buffer_size: response.buffer_size
      }
    }))
  } catch (error) {
    console.error('[WebSocket] Error getting audio config:', error)
    ws.send(JSON.stringify({ 
      type: 'ipc:response', 
      id: message.id, 
      data: {
        sample_rate: 0,
        buffer_size: 256
      }
    }))
  }
}

async function handleSaveAudioConfig(ws: WebSocket, message: any): Promise<void> {
  try {
    const response = await sendCommandAndWaitForResponse({
      type: 'save_audio_config',
      sample_rate: message.sample_rate,
      buffer_size: message.buffer_size
    }, 'ok', 5000)
    ws.send(JSON.stringify({ type: 'ipc:response', id: message.id, data: true }))
  } catch (error) {
    console.error('[WebSocket] Error saving audio config:', error)
    ws.send(JSON.stringify({ type: 'ipc:error', id: message.id, error: 'Failed to save audio config' }))
  }
}

/**
 * Broadcast audio engine response to all WebSocket clients
 * (remote front clients + detached window clients)
 */
export const broadcastToWebSocketClients = (response: any): void => {
  if (!wss) return
  
  wss.clients.forEach((client) => {
    if (client.readyState === WebSocket.OPEN) {
      try {
        client.send(JSON.stringify(response))
      } catch (error) {
        console.error('[WebSocket] Error broadcasting to client:', error)
      }
    }
  })
}

/**
 * Stop WebSocket server
 */
export const stopWebSocketServer = (): Promise<void> => {
  return new Promise((resolve) => {
    if (wss) {
      wss.close(() => {
        console.log('[WebSocket] Server closed')
        wss = null
        resolve()
      })
    } else {
      resolve()
    }
  })
}

/**
 * Get active remote clients count
 */
export const getActiveRemoteClientsCount = (): number => {
  return activeRemoteClientsCount
}

/**
 * Disconnect all active remote clients (called when local user takes control)
 */
export const disconnectAllRemoteClients = (deps: WebSocketServerDependencies): number => {
  console.log('[WebSocket] Disconnecting all remote clients...')
  
  const clientsToDisconnect = Array.from(activeRemoteClients)
  
  clientsToDisconnect.forEach((client) => {
    if (client.readyState === WebSocket.OPEN) {
      // Send notification to client before closing
      try {
        client.send(JSON.stringify({ 
          type: 'disconnected', 
          message: 'Local user has taken control' 
        }))
      } catch (error) {
        console.error('[WebSocket] Error sending disconnect message:', error)
      }
      
      // Close the connection
      client.close(1000, 'Local user took control')
    }
  })
  
  // Clear the active clients set
  activeRemoteClients.clear()
  activeRemoteClientsCount = 0
  
  // Broadcast new state (no more active clients)
  deps.broadcastRemoteControlState()
  
  console.log(`[WebSocket] Disconnected ${clientsToDisconnect.length} remote client(s)`)
  
  return clientsToDisconnect.length
}

export { WS_PORT, detachedWindowClients }
