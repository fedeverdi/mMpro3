import { app, BrowserWindow, screen } from 'electron'
import path from 'node:path'
import started from 'electron-squirrel-startup'

if (started) {
  app.quit()
}

const createWindow = () => {
  const { width, height } = screen.getPrimaryDisplay().workAreaSize
  
  const mainWindow = new BrowserWindow({
    width,
    height,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
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