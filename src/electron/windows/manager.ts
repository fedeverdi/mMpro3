import { app, BrowserWindow, screen, ipcMain } from 'electron'
import * as path from 'path'
import * as fs from 'fs'

// Declare Vite environment variables (set by Vite at build time)
declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string | undefined
declare const MAIN_WINDOW_VITE_NAME: string

// Window state interface
interface WindowState {
  x?: number
  y?: number
  width: number
  height: number
  isMaximized?: boolean
}

// Main application window
let mainWindow: BrowserWindow | null = null

// Splash window
let splashWindow: BrowserWindow | null = null
let splashStartTime: number = 0
const MINIMUM_SPLASH_DURATION = 3000 // 3 seconds

// Track open detached windows
const detachedWindows = new Map<string, BrowserWindow>()

/**
 * Get the main window instance
 */
export const getMainWindow = (): BrowserWindow | null => {
  return mainWindow
}

/**
 * Get the splash window instance
 */
export const getSplashWindow = (): BrowserWindow | null => {
  return splashWindow
}

/**
 * Get the detached windows map
 */
export const getDetachedWindows = (): Map<string, BrowserWindow> => {
  return detachedWindows
}

/**
 * Get the file path for window state storage
 */
const getWindowStateFilePath = (): string => {
  return path.join(app.getPath('userData'), 'window-state.json')
}

/**
 * Load window state from disk
 */
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

/**
 * Save window state to disk
 */
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

/**
 * Create the splash screen window
 */
export const createSplashWindow = (): void => {
  const { width: screenWidth, height: screenHeight } = screen.getPrimaryDisplay().workAreaSize
  const splashWidth = 800
  const splashHeight = 500
  
  // Record splash creation time for minimum display duration
  splashStartTime = Date.now()
  
  splashWindow = new BrowserWindow({
    x: Math.floor((screenWidth - splashWidth) / 2),
    y: Math.floor((screenHeight - splashHeight) / 2),
    width: splashWidth,
    height: splashHeight,
    transparent: true,
    frame: false,
    alwaysOnTop: true,
    resizable: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    }
  })

  // Load splash.html from app root in both dev and production
  const splashPath = app.isPackaged
    ? path.join(process.resourcesPath, 'splash.html')
    : path.join(app.getAppPath(), 'splash.html')
  
  splashWindow.loadFile(splashPath)
  
  splashWindow.on('closed', () => {
    splashWindow = null
  })
}

/**
 * Create the main application window
 * @param onRemoteStateUpdate - Callback to send remote control state updates
 * @param getActiveRemoteClientsCount - Function to get active remote client count
 */
export const createWindow = (
  onRemoteStateUpdate: (active: boolean, clientsCount: number) => void,
  getActiveRemoteClientsCount: () => number
): void => {
  // Load saved window state
  const windowState = loadWindowState()
  
  // Determine icon path based on platform
  let iconPath: string | undefined
  if (process.platform === 'win32') {
    iconPath = path.join(__dirname, '../renderer/main_window/assets/windows/icon.ico')
  } else if (process.platform === 'linux') {
    iconPath = path.join(__dirname, '../renderer/main_window/assets/linux/icons/512x512.png')
  }
  // macOS uses icon.icns from the app bundle, set via packagerConfig
  
  mainWindow = new BrowserWindow({
    x: windowState.x,
    y: windowState.y,
    width: windowState.width,
    height: windowState.height,
    show: false, // Don't show until ready
    titleBarStyle: 'hiddenInset', // Hide native title bar but keep traffic lights on macOS
    ...(iconPath && { icon: iconPath }),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  })

  // Send maximize/unmaximize events to renderer
  mainWindow.on('maximize', () => {
    mainWindow?.webContents.send('window-maximized')
  })

  mainWindow.on('unmaximize', () => {
    mainWindow?.webContents.send('window-unmaximized')
  })

  // Track both conditions: window ready AND minimum splash time elapsed
  let isWindowReady = false
  let isMinimumTimeElapsed = false
  
  const showMainWindow = () => {
    // Only show if both conditions are met
    if (isWindowReady && isMinimumTimeElapsed) {
      if (splashWindow) {
        splashWindow.close()
      }
      // Restore maximized state before showing
      if (windowState.isMaximized && mainWindow) {
        mainWindow.maximize()
      }
      setTimeout(() => {
        mainWindow?.show()
      }, 100) // Small delay to ensure splash closes first
    }
  }

  // Minimum splash duration timer
  setTimeout(() => {
    isMinimumTimeElapsed = true
    showMainWindow()
  }, MINIMUM_SPLASH_DURATION)

  // When main window is ready
  mainWindow.once('ready-to-show', () => {
    isWindowReady = true
    showMainWindow()
    
    // Send initial remote control state to the window
    setTimeout(() => {
      if (mainWindow) {
        const activeRemoteClientsCount = getActiveRemoteClientsCount()
        onRemoteStateUpdate(activeRemoteClientsCount > 0, activeRemoteClientsCount)
      }
    }, 100) // Small delay to ensure renderer is ready
  })

  // Save window state on resize and move
  let saveStateTimeout: NodeJS.Timeout | null = null
  const debouncedSaveState = () => {
    if (saveStateTimeout) {
      clearTimeout(saveStateTimeout)
    }
    saveStateTimeout = setTimeout(() => {
      saveWindowState(mainWindow!)
    }, 500) // Save after 500ms of inactivity
  }

  mainWindow.on('resize', debouncedSaveState)
  mainWindow.on('move', debouncedSaveState)
  
  mainWindow.on('maximize', () => saveWindowState(mainWindow!))
  mainWindow.on('unmaximize', () => saveWindowState(mainWindow!))

  // Save state before closing
  mainWindow.on('close', () => {
    if (saveStateTimeout) {
      clearTimeout(saveStateTimeout)
    }
    saveWindowState(mainWindow!)
  })

  // Clean up reference when window is closed
  mainWindow.on('closed', () => {
    mainWindow = null
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

  // Set Content Security Policy
  mainWindow.webContents.session.webRequest.onHeadersReceived((details, callback) => {
    const isDev = !!MAIN_WINDOW_VITE_DEV_SERVER_URL
    
    // Different CSP for development and production
    const csp = isDev
      ? [
          "default-src 'self'",
          "script-src 'self' 'unsafe-eval'", // unsafe-eval needed for Vite HMR
          "style-src 'self' 'unsafe-inline'", // unsafe-inline needed for Vue style blocks
          "img-src 'self' data: blob:",
          "font-src 'self' data:",
          "connect-src 'self' ws://localhost:* http://localhost:*", // Vite dev server WebSocket
          "media-src 'self' blob:",
          "worker-src 'self' blob:"
        ].join('; ')
      : [
          "default-src 'self'",
          "script-src 'self'",
          "style-src 'self' 'unsafe-inline'", // Still needed for Vue in production
          "img-src 'self' data: blob:",
          "font-src 'self' data:",
          "connect-src 'self' ws://localhost:3001", // Allow WebSocket for detached windows
          "media-src 'self' blob:",
          "worker-src 'self' blob:"
        ].join('; ')

    callback({
      responseHeaders: {
        ...details.responseHeaders,
        'Content-Security-Policy': [csp]
      }
    })
  })

  // Load the app
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL)
  } else {
    mainWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`))
  }
}

/**
 * Create a detached window for a specific component
 */
export const createDetachedWindow = (
  componentType: 'master-eq' | 'spectrum' | 'aux-master' | 'master-fx',
  title: string,
  width: number,
  height: number
): BrowserWindow => {
  // Close existing window for this component if any
  const existingWindow = detachedWindows.get(componentType)
  if (existingWindow && !existingWindow.isDestroyed()) {
    existingWindow.close()
  }
  
  const detachedWindow = new BrowserWindow({
    width,
    height,
    title,
    backgroundColor: '#0f172a',
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
    autoHideMenuBar: true
  })
  
  // Store window reference
  detachedWindows.set(componentType, detachedWindow)
  
  // Remove from map when closed
  detachedWindow.on('closed', () => {
    detachedWindows.delete(componentType)
    console.log(`[Main] Detached window closed: ${componentType}`)
  })
  
  // Load the detached component page
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    // Development mode: use Vite dev server
    detachedWindow.loadURL(`${MAIN_WINDOW_VITE_DEV_SERVER_URL}/detached-${componentType}.html`)
  } else {
    // Production mode: load from built files
    detachedWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/detached-${componentType}.html`))
  }
  
  // Open DevTools in development
  // if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
  //   detachedWindow.webContents.openDevTools()
  // }
  
  console.log(`[Main] Created detached window: ${componentType}`)
  
  return detachedWindow
}

/**
 * Setup IPC handlers for window control
 */
export const setupWindowIpcHandlers = (): void => {
  // Window control handlers
  ipcMain.handle('window-is-maximized', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender)
    return window?.isMaximized() || false
  })

  ipcMain.on('window-minimize', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender)
    window?.minimize()
  })

  ipcMain.on('window-maximize', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender)
    window?.maximize()
  })

  ipcMain.on('window-unmaximize', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender)
    window?.unmaximize()
  })

  ipcMain.on('window-close', (event) => {
    const window = BrowserWindow.fromWebContents(event.sender)
    window?.close()
  })

  // Detached window handlers
  ipcMain.handle('open-detached-window', async (_event, componentType: string) => {
    console.log(`[Main] Opening detached window: ${componentType}`)
    
    switch (componentType) {
      case 'master-eq':
        createDetachedWindow('master-eq', 'Master EQ', 1400, 800)
        break
      case 'spectrum':
        createDetachedWindow('spectrum', 'Spectrum Analyzer', 600, 500)
        break
      case 'aux-master':
        createDetachedWindow('aux-master', 'Aux Buses', 900, 260)
        break
      case 'master-fx':
        createDetachedWindow('master-fx', 'Master FX Chain', 900, 220)
        break
      default:
        console.error(`[Main] Unknown component type: ${componentType}`)
        return false
    }
    
    return true
  })

  ipcMain.handle('close-detached-window', async (_event, componentType: string) => {
    console.log(`[Main] Closing detached window: ${componentType}`)
    
    const window = detachedWindows.get(componentType)
    if (window && !window.isDestroyed()) {
      window.close()
      return true
    }
    
    return false
  })

  ipcMain.handle('is-detached-window-open', async (_event, componentType: string) => {
    const window = detachedWindows.get(componentType)
    return window && !window.isDestroyed()
  })
}

/**
 * Broadcast a message to all windows (main + detached)
 */
export const broadcastToAllWindows = (channel: string, ...args: any[]): void => {
  BrowserWindow.getAllWindows().forEach(win => {
    win.webContents.send(channel, ...args)
  })
}
