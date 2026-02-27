import { app, BrowserWindow, screen, ipcMain } from 'electron'
import { spawn, ChildProcess } from 'node:child_process'
import path from 'node:path'
import fs from 'node:fs'
import started from 'electron-squirrel-startup'

if (started) {
  app.quit()
}

// Audio Engine Process
let audioEngineProcess: ChildProcess | null = null

const startAudioEngine = () => {
  // If audio engine is already running, stop it first
  if (audioEngineProcess) {
    console.log('[Main] Audio engine already running, stopping it first...')
    stopAudioEngine()
    
    // Wait a bit for the stop to complete, then start fresh
    setTimeout(() => {
      startAudioEngineInternal()
    }, 150)
    return
  }
  
  startAudioEngineInternal()
}

const startAudioEngineInternal = () => {
  const enginePath = app.isPackaged
    ? path.join(process.resourcesPath, 'audio-engine', 'mmpro3-engine')
    : path.join(app.getAppPath(), 'audio-engine', 'target', 'release', 'mmpro3-engine')

  console.log('[Main] App path:', app.getAppPath())
  console.log('[Main] Starting audio engine:', enginePath)
  console.log('[Main] Engine exists:', fs.existsSync(enginePath))
  
  if (!fs.existsSync(enginePath)) {
    console.error('[Main] Engine not found! Skipping audio engine startup.')
    return
  }
  
  audioEngineProcess = spawn(enginePath, [], {
    stdio: ['pipe', 'pipe', 'pipe']
  })

  audioEngineProcess.stdout?.on('data', (data) => {
    const output = data.toString().trim()
    
    // Process each line separately (engine may send multiple JSON responses)
    const lines = output.split('\n').filter((line: string) => line.trim())
    
    for (const line of lines) {
      // Try to parse as JSON
      try {
        const response = JSON.parse(line)
        
        // Check if there's a pending handler for this response type
        const responseType = response.type
        if (responseType && responseHandlers.has(responseType)) {
          const handler = responseHandlers.get(responseType)
          if (handler) {
            handler(response)
          }
        }
        
        // Forward to renderer
        BrowserWindow.getAllWindows().forEach(win => {
          win.webContents.send('audio-engine-response', response)
        })
      } catch (err) {
        // Not JSON, just log as plain text
        console.log('[Engine Output]', line)
      }
    }
  })

  audioEngineProcess.stderr?.on('data', (data) => {
    console.error('[Engine Error]', data.toString())
  })

  audioEngineProcess.on('close', (code) => {
    console.log('[Main] Audio engine closed with code:', code)
    audioEngineProcess = null
  })
}

const stopAudioEngine = () => {
  if (audioEngineProcess) {
    console.log('[Main] Stopping audio engine')
    
    // First, stop all file players gracefully
    try {
      if (audioEngineProcess.stdin) {
        const stopCommand = JSON.stringify({ type: 'stop_all_files' }) + '\n'
        audioEngineProcess.stdin.write(stopCommand)
        console.log('[Main] Sent stop_all_files command')
      }
    } catch (err) {
      console.error('[Main] Error sending stop_all_files command:', err)
    }
    
    // Give it a moment to process, then kill
    setTimeout(() => {
      if (audioEngineProcess) {
        audioEngineProcess.kill()
        audioEngineProcess = null
      }
    }, 100)
  }
}

const sendCommandToEngine = (command: any): Promise<void> => {
  return new Promise((resolve, reject) => {
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      reject(new Error('Audio engine not running'))
      return
    }

    try {
      audioEngineProcess.stdin.write(JSON.stringify(command) + '\n')
      resolve()
    } catch (err) {
      reject(err)
    }
  })
}

// Store pending response handlers
const responseHandlers: Map<string, (response: any) => void> = new Map()

// Send command and wait for specific response type
const sendCommandAndWaitForResponse = (command: any, responseType: string, timeout = 5000): Promise<any> => {
  return new Promise((resolve, reject) => {
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      reject(new Error('Audio engine not running'))
      return
    }

    const timeoutId = setTimeout(() => {
      responseHandlers.delete(responseType)
      reject(new Error(`Timeout waiting for ${responseType} response`))
    }, timeout)

    // Register response handler
    responseHandlers.set(responseType, (response: any) => {
      clearTimeout(timeoutId)
      responseHandlers.delete(responseType)
      resolve(response)
    })

    try {
      audioEngineProcess.stdin.write(JSON.stringify(command) + '\n')
    } catch (err) {
      clearTimeout(timeoutId)
      responseHandlers.delete(responseType)
      reject(err)
    }
  })
}

// IPC Handlers
ipcMain.handle('audio-engine:start', async (_, inputDevice?: string, outputDevice?: string) => {
  const command: any = { type: 'start' }
  if (inputDevice) command.input_device = inputDevice
  if (outputDevice) command.output_device = outputDevice
  await sendCommandToEngine(command)
})

ipcMain.handle('audio-engine:stop', async () => {
  await sendCommandToEngine({ type: 'stop' })
})

ipcMain.handle('audio-engine:set-gain', async (_, track: number, gain: number) => {
  await sendCommandToEngine({ type: 'set_gain', track, gain })
})

ipcMain.handle('audio-engine:set-volume', async (_, track: number, volume: number) => {
  await sendCommandToEngine({ type: 'set_volume', track, volume })
})

ipcMain.handle('audio-engine:set-mute', async (_, track: number, mute: boolean) => {
  await sendCommandToEngine({ type: 'set_mute', track, mute })
})

ipcMain.handle('audio-engine:set-route-to-master', async (_, track: number, route: boolean) => {
  await sendCommandToEngine({ type: 'set_route_to_master', track, route })
})

ipcMain.handle('audio-engine:set-compressor', async (_, track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
  await sendCommandToEngine({ type: 'set_compressor', track, enabled, threshold, ratio, attack, release })
})

ipcMain.handle('audio-engine:set-gate', async (_, track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => {
  await sendCommandToEngine({ type: 'set_gate', track, enabled, threshold, range, attack, release })
})

// Track source selection
ipcMain.handle('audio-engine:set-track-source-input', async (_, track: number, leftChannel: number, rightChannel: number) => {
  await sendCommandToEngine({ type: 'set_track_source_input', track, left_channel: leftChannel, right_channel: rightChannel })
})

ipcMain.handle('audio-engine:set-track-source-signal', async (_, track: number, waveform: string, frequency: number) => {
  await sendCommandToEngine({ type: 'set_track_source_signal', track, waveform, frequency })
})

ipcMain.handle('audio-engine:set-track-source-file', async (_, track: number, filePath: string) => {
  console.log('[Main] Setting track source file:', { track, filePath })
  await sendCommandToEngine({ type: 'set_track_source_file', track, file_path: filePath })
})

// Save audio buffer to temp file and return path
ipcMain.handle('audio-engine:save-temp-audio-file', async (_, arrayBuffer: ArrayBuffer, fileName: string) => {
  try {
    const tempDir = app.getPath('temp')
    const mmpro3TempDir = path.join(tempDir, 'mmpro3-audio')
    
    // Create temp directory if it doesn't exist
    if (!fs.existsSync(mmpro3TempDir)) {
      fs.mkdirSync(mmpro3TempDir, { recursive: true })
    }
    
    // Generate unique filename
    const timestamp = Date.now()
    const ext = path.extname(fileName) || '.mp3'
    const baseName = path.basename(fileName, ext)
    const tempFileName = `${baseName}_${timestamp}${ext}`
    const tempFilePath = path.join(mmpro3TempDir, tempFileName)
    
    // Write buffer to file
    const buffer = Buffer.from(arrayBuffer)
    fs.writeFileSync(tempFilePath, buffer)
    
    console.log('[Main] Saved temp audio file:', tempFilePath)
    return tempFilePath
  } catch (error) {
    console.error('[Main] Error saving temp audio file:', error)
    throw error
  }
})

// File playback controls
ipcMain.handle('audio-engine:play-file', async (_, track: number) => {
  console.log('[Main] Playing file on track:', track)
  await sendCommandToEngine({ type: 'play_file', track })
})

ipcMain.handle('audio-engine:pause-file', async (_, track: number) => {
  await sendCommandToEngine({ type: 'pause_file', track })
})

ipcMain.handle('audio-engine:stop-file', async (_, track: number) => {
  await sendCommandToEngine({ type: 'stop_file', track })
})

ipcMain.handle('audio-engine:set-pan', async (_, track: number, pan: number) => {
  await sendCommandToEngine({ type: 'set_pan', track, pan })
})
ipcMain.handle('audio-engine:set-track-pad', async (_, track: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_track_pad', track, enabled })
})
ipcMain.handle('audio-engine:set-track-hpf', async (_, track: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_track_hpf', track, enabled })
})
ipcMain.handle('audio-engine:set-eq', async (_, track: number, low: number, low_mid: number, high_mid: number, high: number) => {
  await sendCommandToEngine({ type: 'set_eq', track, low, low_mid, high_mid, high })
})

ipcMain.handle('audio-engine:set-eq-enabled', async (_, track: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_eq_enabled', track, enabled })
})

// Parametric EQ controls
ipcMain.handle('audio-engine:set-parametric-eq-filters', async (_, track: number, filters: Array<{type: string, frequency: number, gain: number, q: number}>) => {
  await sendCommandToEngine({ type: 'set_parametric_eq_filters', track, filters })
})

ipcMain.handle('audio-engine:set-parametric-eq-enabled', async (_, track: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_parametric_eq_enabled', track, enabled })
})

ipcMain.handle('audio-engine:clear-parametric-eq', async (_, track: number) => {
  await sendCommandToEngine({ type: 'clear_parametric_eq', track })
})

// Master controls
ipcMain.handle('audio-engine:set-master-gain', async (_, gain: number) => {
  await sendCommandToEngine({ type: 'set_master_gain', gain })
})

ipcMain.handle('audio-engine:set-master-mute', async (_, mute: boolean) => {
  await sendCommandToEngine({ type: 'set_master_mute', mute })
})

ipcMain.handle('audio-engine:set-master-output-channels', async (_, leftChannel: number, rightChannel: number) => {
  await sendCommandToEngine({ type: 'set_master_output_channels', left_channel: leftChannel, right_channel: rightChannel })
})

ipcMain.handle('audio-engine:list-devices', async () => {
  const response = await sendCommandAndWaitForResponse({ type: 'list_devices' }, 'devices')
  return response.devices
})
// Window state management
interface WindowState {
  x?: number
  y?: number
  width: number
  height: number
  isMaximized?: boolean
}

const getWindowStateFilePath = (): string => {
  return path.join(app.getPath('userData'), 'window-state.json')
}

const loadWindowState = (): WindowState => {
  const defaultState: WindowState = {
    width: 1400,
    height: 900
  }

  try {
    const filePath = getWindowStateFilePath()
    if (fs.existsSync(filePath)) {
      const data = fs.readFileSync(filePath, 'utf-8')
      const savedState = JSON.parse(data) as WindowState
      
      // Validate that the window is visible on screen
      const { width: screenWidth, height: screenHeight } = screen.getPrimaryDisplay().workAreaSize
      
      // Ensure window fits on screen
      if (savedState.width > screenWidth) savedState.width = screenWidth
      if (savedState.height > screenHeight) savedState.height = screenHeight
      
      // Ensure window is not off-screen
      if (savedState.x !== undefined && savedState.y !== undefined) {
        if (savedState.x < 0 || savedState.x > screenWidth - 100) savedState.x = undefined
        if (savedState.y < 0 || savedState.y > screenHeight - 100) savedState.y = undefined
      }
      
      return { ...defaultState, ...savedState }
    }
  } catch (error) {
    console.error('Failed to load window state:', error)
  }

  return defaultState
}

const saveWindowState = (window: BrowserWindow): void => {
  try {
    const bounds = window.getBounds()
    const state: WindowState = {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
      isMaximized: window.isMaximized()
    }
    
    const filePath = getWindowStateFilePath()
    fs.writeFileSync(filePath, JSON.stringify(state, null, 2), 'utf-8')
  } catch (error) {
    console.error('Failed to save window state:', error)
  }
}

const createWindow = () => {
  // Load saved window state
  const windowState = loadWindowState()
  
  const mainWindow = new BrowserWindow({
    x: windowState.x,
    y: windowState.y,
    width: windowState.width,
    height: windowState.height,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  })

  // Restore maximized state if needed
  if (windowState.isMaximized) {
    mainWindow.maximize()
  }

  // Save window state on resize and move
  let saveStateTimeout: NodeJS.Timeout | null = null
  const debouncedSaveState = () => {
    if (saveStateTimeout) {
      clearTimeout(saveStateTimeout)
    }
    saveStateTimeout = setTimeout(() => {
      saveWindowState(mainWindow)
    }, 500) // Save after 500ms of inactivity
  }

  mainWindow.on('resize', debouncedSaveState)
  mainWindow.on('move', debouncedSaveState)
  mainWindow.on('maximize', () => saveWindowState(mainWindow))
  mainWindow.on('unmaximize', () => saveWindowState(mainWindow))

  // Save state before closing
  mainWindow.on('close', () => {
    if (saveStateTimeout) {
      clearTimeout(saveStateTimeout)
    }
    saveWindowState(mainWindow)
  })

  // Automatically grant permission for media access
  mainWindow.webContents.session.setPermissionRequestHandler((webContents, permission, callback) => {
    // Allow all media-related permissions
    callback(true)
  })

  // Also handle permission checks (for already granted permissions)
  mainWindow.webContents.session.setPermissionCheckHandler((webContents, permission, requestingOrigin, details) => {
    if (permission === 'media') {
      return true
    }
    return true
  })

  // Handle device permission requests (needed for getUserMedia on macOS)
  mainWindow.webContents.session.setDevicePermissionHandler((details) => {
    return true
  })

  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL)
  } else {
    mainWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`))
  }

  mainWindow.webContents.openDevTools()
}

app.whenReady().then(() => {
  startAudioEngine()
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  stopAudioEngine()
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('before-quit', () => {
  stopAudioEngine()
})