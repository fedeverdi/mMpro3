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
    console.log('[Engine]', output)
    
    // Parse JSON responses and forward to renderer
    try {
      const response = JSON.parse(output)
      BrowserWindow.getAllWindows().forEach(win => {
        win.webContents.send('audio-engine-response', response)
      })
    } catch (err) {
      // Not JSON, just log
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
    audioEngineProcess.kill()
    audioEngineProcess = null
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

// IPC Handlers
ipcMain.handle('audio-engine:start', async () => {
  await sendCommandToEngine({ type: 'start' })
})

ipcMain.handle('audio-engine:stop', async () => {
  await sendCommandToEngine({ type: 'stop' })
})

ipcMain.handle('audio-engine:set-gain', async (_, track: number, gain: number) => {
  await sendCommandToEngine({ type: 'set_gain', track, gain })
})

ipcMain.handle('audio-engine:set-mute', async (_, track: number, mute: boolean) => {
  await sendCommandToEngine({ type: 'set_mute', track, mute })
})

ipcMain.handle('audio-engine:list-devices', async () => {
  await sendCommandToEngine({ type: 'list_devices' })
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