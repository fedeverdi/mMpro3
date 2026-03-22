// Load environment variables from .env file (only in development)
import { app, BrowserWindow, powerSaveBlocker } from 'electron'
import path from 'node:path'
import fs from 'node:fs'
import os from 'node:os'
import started from 'electron-squirrel-startup'
import { startHttpServer, stopHttpServer, HTTP_PORT } from './electron/http/server'
import { 
  startWebSocketServer, 
  stopWebSocketServer, 
  broadcastToWebSocketClients,
  disconnectAllRemoteClients,
  getActiveRemoteClientsCount,
  WS_PORT,
  detachedWindowClients,
  type WebSocketServerDependencies 
} from './electron/websocket/server'
import {
  initAudioEngine,
  startAudioEngine,
  stopAudioEngine,
  sendCommandToEngine,
  sendCommandAndWaitForResponse,
  getAudioEngineProcess,
  getIsAudioEngineStarted,
  type AudioEngineCallbacks
} from './electron/audio-engine/process'
import {
  createWindow,
  createSplashWindow,
  setupWindowIpcHandlers,
  broadcastToAllWindows,
  getMainWindow
} from './electron/windows/manager'
import { setupIpcHandlers, type IpcHandlerDependencies } from './electron/ipc/handlers'

// Disable Electron security warnings in development
// (unsafe-eval is required for Vite HMR)
if (process.env.NODE_ENV !== 'production') {
  process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = 'true'
}

if (started) {
  app.quit()
}

if (process.platform === 'darwin') {
  app.commandLine.appendSwitch('disable-smooth-scrolling')
}

// Power Save Blocker - prevent system from throttling audio
let powerSaveBlockerId: number | null = null

// Track active temp files for cleanup
const activeTempFiles = new Set<string>()

// Track last known state for new detached windows
let lastKnownState: {
  masterEqFilters?: any[]
  masterParameters?: any
  auxParameters?: any[]
  subgroupParameters?: any[]
  auxBuses?: any[] // Frontend aux buses state (separate from backend auxParameters)
} = {}

// WebSocket server dependencies (initialized in app.whenReady)
let wsServerDeps: WebSocketServerDependencies | null = null

/**
 * Broadcast remote control state to all Electron windows
 */
const broadcastRemoteControlState = () => {
  const activeRemoteClientsCount = getActiveRemoteClientsCount()
  const isRemoteActive = activeRemoteClientsCount > 0
  broadcastToAllWindows('remote-control-state', { active: isRemoteActive, clientsCount: activeRemoteClientsCount })
}

/**
 * Broadcast audio engine response to all WebSocket clients
 * Wrapper that updates local state and forwards to WebSocket module
 */
const broadcastToWebSocketClientsWrapper = (response: any) => {
  // Update cached state for new detached windows
  if (response.type === 'parameters' || response.type === 'levels' || response.type === 'parameters_changed') {
    if (response.master) {
      // Merge instead of overwrite so partial updates (e.g. SetMasterGain only sends gain)
      // don't wipe out other fields like gain_left, gain_right, linked, mute, etc.
      lastKnownState.masterParameters = { ...lastKnownState.masterParameters, ...response.master }
    }
    if (response.auxes) {
      lastKnownState.auxParameters = response.auxes
    }
    if (response.subgroups) {
      lastKnownState.subgroupParameters = response.subgroups
    }
  }
  
  // Forward to WebSocket module
  broadcastToWebSocketClients(response)
}

/**
 * Clean up temporary audio files from previous sessions
 */
const cleanupTempAudioFiles = () => {
  try {
    const tempDir = app.getPath('temp')
    const mmpro3TempDir = path.join(tempDir, 'mmpro3-audio')
    
    if (fs.existsSync(mmpro3TempDir)) {
      const files = fs.readdirSync(mmpro3TempDir)
      let deletedCount = 0
      
      for (const file of files) {
        try {
          const filePath = path.join(mmpro3TempDir, file)
          const stats = fs.statSync(filePath)
          
          // Delete files older than 1 hour or not in active set
          const oneHourAgo = Date.now() - (60 * 60 * 1000)
          if (stats.mtimeMs < oneHourAgo || !activeTempFiles.has(filePath)) {
            fs.unlinkSync(filePath)
            activeTempFiles.delete(filePath)
            deletedCount++
          }
        } catch (err) {
          console.error(`[Main] Failed to delete temp file ${file}:`, err)
        }
      }
      
      console.log(`[Main] Cleaned up ${deletedCount} temporary audio files`)
    }
  } catch (error) {
    console.error('[Main] Error cleaning up temp audio files:', error)
  }
}

app.whenReady().then(() => {
  // Clean up old temporary audio files from previous sessions
  cleanupTempAudioFiles()
  
  // Prevent system from throttling audio playback
  powerSaveBlockerId = powerSaveBlocker.start('prevent-app-suspension')
  console.log('[Main] Power save blocker started:', powerSaveBlocker.isStarted(powerSaveBlockerId))
  
  // Initialize audio engine with callbacks
  const audioEngineCallbacks: AudioEngineCallbacks = {
    onResponse: (response: any) => {
      // Forward to renderer windows
      broadcastToAllWindows('audio-engine-response', response)
      
      // Broadcast to WebSocket clients
      broadcastToWebSocketClientsWrapper(response)
    },
    onStarted: () => {
      console.log('[Main] Audio engine started callback')
    },
    onStopped: () => {
      console.log('[Main] Audio engine stopped callback')
    },
    onError: (error: string) => {
      
    }
  }
  initAudioEngine(audioEngineCallbacks)
  
  // Start audio engine
  startAudioEngine()
  
  // Initialize WebSocket server dependencies BEFORE IPC handlers
  wsServerDeps = {
    getAudioEngineProcess: getAudioEngineProcess,
    getIsAudioEngineStarted: getIsAudioEngineStarted,
    getLastKnownState: () => lastKnownState,
    broadcastRemoteControlState: broadcastRemoteControlState
  }
  
  // Setup IPC handlers (now wsServerDeps is initialized)
  const ipcDeps: IpcHandlerDependencies = {
    activeTempFiles,
    lastKnownState,
    wsServerDeps
  }
  setupIpcHandlers(ipcDeps)
  
  // Start WebSocket server for remote control
  startWebSocketServer(wsServerDeps)
  
  // Start HTTP server for web interface (production only)
  startHttpServer()
  
  // Setup window IPC handlers
  setupWindowIpcHandlers()
  
  // Show splash screen first
  createSplashWindow()
  
  // Create main window (hidden until ready)
  // Add a small delay to ensure splash is visible first
  setTimeout(() => {
    createWindow(
      (active, clientsCount) => {
        // Send remote control state to main window
        const mainWin = getMainWindow()
        if (mainWin) {
          mainWin.webContents.send('remote-control-state', { active, clientsCount })
        }
      },
      getActiveRemoteClientsCount
    )
  }, 100)

  app.on('activate', () => {
    // If no windows, create new ones
    if (BrowserWindow.getAllWindows().length === 0) {
      createSplashWindow()
      setTimeout(() => {
        createWindow(
          (active, clientsCount) => {
            const mainWin = getMainWindow()
            if (mainWin) {
              mainWin.webContents.send('remote-control-state', { active, clientsCount })
            }
          },
          getActiveRemoteClientsCount
        )
      }, 100)
    } else {
      // On macOS, restore hidden window when clicking dock icon
      const mainWin = getMainWindow()
      if (mainWin && !mainWin.isVisible()) {
        mainWin.show()
      }
    }
  })
})

app.on('window-all-closed', () => {
  stopAudioEngine()
  cleanupTempAudioFiles() // Clean up temp files on exit
  // Stop WebSocket server
  stopWebSocketServer()
  // Stop HTTP server
  stopHttpServer()
  // Stop power save blocker
  if (powerSaveBlockerId !== null && powerSaveBlocker.isStarted(powerSaveBlockerId)) {
    powerSaveBlocker.stop(powerSaveBlockerId)
    console.log('[Main] Power save blocker stopped')
  }
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('before-quit', () => {
  stopAudioEngine()
  cleanupTempAudioFiles() // Clean up temp files before quit
  // Stop WebSocket server
  stopWebSocketServer()
  // Stop HTTP server
  stopHttpServer()
  // Stop power save blocker
  if (powerSaveBlockerId !== null && powerSaveBlocker.isStarted(powerSaveBlockerId)) {
    powerSaveBlocker.stop(powerSaveBlockerId)
  }
})