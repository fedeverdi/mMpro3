import { app, BrowserWindow, screen } from 'electron'
import path from 'node:path'
import fs from 'node:fs'
import started from 'electron-squirrel-startup'

if (started) {
  app.quit()
}

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

  // mainWindow.webContents.openDevTools()
}

app.whenReady().then(() => {
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})