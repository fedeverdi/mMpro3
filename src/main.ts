// Load environment variables from .env file (only in development)
import { app, BrowserWindow, screen, ipcMain, shell, dialog, powerSaveBlocker } from 'electron'
import { spawn, ChildProcess } from 'node:child_process'
import path from 'node:path'
import fs from 'node:fs'
import started from 'electron-squirrel-startup'
import { createClient } from '@vercel/edge-config'


// Lazy initialization of Edge Config client
let edgeConfigClient: ReturnType<typeof createClient> | null = null
const getEdgeConfigClient = () => {
  if (!edgeConfigClient) {
    const connectionString = process.env.EDGE_CONFIG
    if (!connectionString) {
      throw new Error('EDGE_CONFIG environment variable is not set')
    }
    edgeConfigClient = createClient(connectionString)
  }
  return edgeConfigClient
}

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

// Audio Engine Process
let audioEngineProcess: ChildProcess | null = null

// Splash Window
let splashWindow: BrowserWindow | null = null
let splashStartTime: number = 0
const MINIMUM_SPLASH_DURATION = 3000 // 3 seconds

// Power Save Blocker - prevent system from throttling audio
let powerSaveBlockerId: number | null = null

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
  // Get the binary name based on platform
  const binaryName = process.platform === 'win32' ? 'mmpro3-engine.exe' : 'mmpro3-engine'
  
  const enginePath = app.isPackaged
    ? path.join(process.resourcesPath, binaryName)
    : path.join(app.getAppPath(), 'audio-engine', 'target', 'release', binaryName)
  
  if (!fs.existsSync(enginePath)) {
    console.error('[Main] Engine not found! Skipping audio engine startup.')
    console.error('[Main] Searched at:', enginePath)
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
ipcMain.handle('audio-engine:start', async (_, inputDevice?: string | null, outputDevice?: string | null, sampleRate?: number | null, bufferSize?: number | null) => {
  const command: any = { type: 'start' }
  if (inputDevice) command.input_device = inputDevice
  if (outputDevice) command.output_device = outputDevice
  if (sampleRate) command.sample_rate = sampleRate
  if (bufferSize) command.buffer_size = bufferSize
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
ipcMain.handle('audio-engine:set-track-source-input', async (_, track: number, leftChannel: number, rightChannel: number, deviceName?: string | null) => {
  const command: any = { type: 'set_track_source_input', track, left_channel: leftChannel, right_channel: rightChannel }
  if (deviceName) {
    command.device_name = deviceName
  }
  await sendCommandToEngine(command)
})

ipcMain.handle('audio-engine:set-track-source-signal', async (_, track: number, waveform: string, frequency: number) => {
  await sendCommandToEngine({ type: 'set_track_source_signal', track, waveform, frequency })
})

ipcMain.handle('audio-engine:set-signal-frequency', async (_, track: number, frequency: number) => {
  await sendCommandToEngine({ type: 'set_signal_frequency', track, frequency })
})

ipcMain.handle('audio-engine:set-signal-waveform', async (_, track: number, waveform: string) => {
  await sendCommandToEngine({ type: 'set_signal_waveform', track, waveform })
})

ipcMain.handle('audio-engine:clear-track-source', async (_, track: number) => {
  await sendCommandToEngine({ type: 'clear_track_source', track })
})

ipcMain.handle('audio-engine:set-track-source-file', async (_, track: number, filePath: string) => {
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
    
    return tempFilePath
  } catch (error) {
    console.error('[Main] Error saving temp audio file:', error)
    throw error
  }
})

// File playback controls
ipcMain.handle('audio-engine:play-file', async (_, track: number, fileId?: string) => {
  let filePath: string | undefined
  
  // If fileId is provided, resolve it to full path
  if (fileId) {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    filePath = path.join(libraryDir, fileId)
    
    // Verify file exists
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found in library: ${fileId}`)
    }
  }
  
  await sendCommandToEngine({ type: 'play_file', track, file_path: filePath })
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
ipcMain.handle('audio-engine:set-track-phase-invert', async (_, track: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_track_phase_invert', track, enabled })
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

ipcMain.handle('audio-engine:set-master-parametric-eq-filters', async (_, filters: Array<{type: string, frequency: number, gain: number, q: number}>) => {
  await sendCommandToEngine({ type: 'set_master_parametric_eq_filters', filters })
})

ipcMain.handle('audio-engine:set-master-parametric-eq-enabled', async (_, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_master_parametric_eq_enabled', enabled })
})

ipcMain.handle('audio-engine:clear-master-parametric-eq', async () => {
  await sendCommandToEngine({ type: 'clear_master_parametric_eq' })
})

ipcMain.handle('audio-engine:set-master-output-channels', async (_, leftChannel: number, rightChannel: number) => {
  await sendCommandToEngine({ type: 'set_master_output_channels', left_channel: leftChannel, right_channel: rightChannel })
})

// Master FX handlers
ipcMain.handle('audio-engine:set-master-compressor', async (_, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
  await sendCommandToEngine({ type: 'set_master_compressor', enabled, threshold, ratio, attack, release })
})

ipcMain.handle('audio-engine:set-master-limiter', async (_, enabled: boolean, ceiling: number, release: number) => {
  await sendCommandToEngine({ type: 'set_master_limiter', enabled, ceiling, release })
})

// NDI Streaming handlers
ipcMain.handle('audio-engine:start-ndi', async (_, streamName: string, source: string) => {
  await sendCommandToEngine({ type: 'start_ndi', stream_name: streamName, source })
})

ipcMain.handle('audio-engine:stop-ndi', async () => {
  await sendCommandToEngine({ type: 'stop_ndi' })
})

ipcMain.handle('audio-engine:set-ndi-source', async (_, source: string) => {
  await sendCommandToEngine({ type: 'set_ndi_source', source })
})

ipcMain.handle('audio-engine:set-ndi-name', async (_, name: string) => {
  await sendCommandToEngine({ type: 'set_ndi_name', name })
})

ipcMain.handle('audio-engine:set-ndi-video-text', async (_, text: string) => {
  await sendCommandToEngine({ type: 'set_ndi_video_text', text })
})

// Loudness metering
ipcMain.handle('audio-engine:get-loudness', async () => {
  try {
    // Silently return null if engine not running (polling will retry)
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      return null
    }
    return await sendCommandAndWaitForResponse({ type: 'get_loudness' }, 'loudness')
  } catch (error) {
    // Silently ignore errors during polling
    return null
  }
})

ipcMain.handle('audio-engine:reset-loudness', async () => {
  await sendCommandToEngine({ type: 'reset_loudness' })
})

// Dynamic Range metering
ipcMain.handle('audio-engine:get-dynamic-range', async () => {
  try {
    // Silently return null if engine not running (polling will retry)
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      return null
    }
    return await sendCommandAndWaitForResponse({ type: 'get_dynamic_range' }, 'dynamic_range')
  } catch (error) {
    // Silently ignore errors during polling
    return null
  }
})

ipcMain.handle('audio-engine:reset-dynamic-range', async () => {
  await sendCommandToEngine({ type: 'reset_dynamic_range' })
})

// Phase Correlation metering (Master)
ipcMain.handle('audio-engine:get-phase-correlation', async () => {
  try {
    // Silently return null if engine not running (polling will retry)
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      return null
    }
    return await sendCommandAndWaitForResponse({ type: 'get_phase_correlation' }, 'phase_correlation')
  } catch (error) {
    // Silently ignore errors during polling
    return null
  }
})

ipcMain.handle('audio-engine:reset-phase-correlation', async () => {
  await sendCommandToEngine({ type: 'reset_phase_correlation' })
})

// Stereo Width metering (Master)
ipcMain.handle('audio-engine:get-stereo-width', async () => {
  try {
    // Silently return null if engine not running (polling will retry)
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      return null
    }
    return await sendCommandAndWaitForResponse({ type: 'get_stereo_width' }, 'stereo_width')
  } catch (error) {
    // Silently ignore errors during polling
    return null
  }
})

ipcMain.handle('audio-engine:reset-stereo-width', async () => {
  await sendCommandToEngine({ type: 'reset_stereo_width' })
})

// Headroom metering (Master)
ipcMain.handle('audio-engine:get-headroom', async () => {
  try {
    // Silently return null if engine not running (polling will retry)
    if (!audioEngineProcess || !audioEngineProcess.stdin) {
      return null
    }
    return await sendCommandAndWaitForResponse({ type: 'get_headroom' }, 'headroom')
  } catch (error) {
    // Silently ignore errors during polling
    return null
  }
})

ipcMain.handle('audio-engine:reset-headroom', async () => {
  await sendCommandToEngine({ type: 'reset_headroom' })
})

ipcMain.handle('audio-engine:set-master-limiter-old', async (_, enabled: boolean, ceiling: number, release: number) => {
  await sendCommandToEngine({ type: 'set_master_limiter', enabled, ceiling, release })
})

ipcMain.handle('audio-engine:set-master-delay', async (_, enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number) => {
  await sendCommandToEngine({ type: 'set_master_delay', enabled, time_l: timeL, time_r: timeR, feedback, mix })
})

ipcMain.handle('audio-engine:set-master-reverb', async (_, enabled: boolean, roomSize: number, damping: number, wet: number, width: number) => {
  await sendCommandToEngine({ type: 'set_master_reverb', enabled, room_size: roomSize, damping, wet, width })
})

// Subgroup handlers
ipcMain.handle('audio-engine:add-subgroup', async () => {
  try {
    const response = await sendCommandAndWaitForResponse({ type: 'add_subgroup' }, 'subgroup_created')
    return response.id
  } catch (error) {
    console.error('[Main] Failed to add subgroup:', error)
    return null
  }
})

ipcMain.handle('audio-engine:remove-subgroup', async (_, subgroup: number) => {
  await sendCommandToEngine({ type: 'remove_subgroup', subgroup })
})

ipcMain.handle('audio-engine:set-subgroup-gain', async (_, subgroup: number, gain: number) => {
  await sendCommandToEngine({ type: 'set_subgroup_gain', subgroup, gain })
})

ipcMain.handle('audio-engine:set-subgroup-mute', async (_, subgroup: number, mute: boolean) => {
  await sendCommandToEngine({ type: 'set_subgroup_mute', subgroup, mute })
})

ipcMain.handle('audio-engine:set-subgroup-output-enabled', async (_, subgroup: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_subgroup_output_enabled', subgroup, enabled })
})

ipcMain.handle('audio-engine:set-subgroup-route-to-master', async (_, subgroup: number, route: boolean) => {
  await sendCommandToEngine({ type: 'set_subgroup_route_to_master', subgroup, route })
})

ipcMain.handle('audio-engine:set-subgroup-output-channels', async (_, subgroup: number, leftChannel: number, rightChannel: number) => {
  await sendCommandToEngine({ type: 'set_subgroup_output_channels', subgroup, left_channel: leftChannel, right_channel: rightChannel })
})

ipcMain.handle('audio-engine:set-track-route-to-subgroup', async (_, track: number, subgroup: number, route: boolean) => {
  await sendCommandToEngine({ type: 'set_track_route_to_subgroup', track, subgroup, route })
})

// Aux bus IPC handlers
ipcMain.handle('audio-engine:set-track-aux-send', async (_, track: number, aux: number, level: number, preFader: boolean, muted: boolean) => {
  await sendCommandToEngine({ type: 'set_track_aux_send', track, aux, level, pre_fader: preFader, muted })
})

ipcMain.handle('audio-engine:set-aux-bus-gain', async (_, aux: number, gain: number) => {
  await sendCommandToEngine({ type: 'set_aux_bus_gain', aux, gain })
})

ipcMain.handle('audio-engine:set-aux-bus-mute', async (_, aux: number, mute: boolean) => {
  await sendCommandToEngine({ type: 'set_aux_bus_mute', aux, mute })
})

ipcMain.handle('audio-engine:set-aux-bus-reverb', async (_, aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number) => {
  await sendCommandToEngine({ type: 'set_aux_bus_reverb', aux, enabled, room_size: roomSize, damping, wet, width })
})

ipcMain.handle('audio-engine:set-aux-bus-delay', async (_, aux: number, enabled: boolean, time: number, feedback: number, mix: number) => {
  await sendCommandToEngine({ type: 'set_aux_bus_delay', aux, enabled, time, feedback, mix })
})

ipcMain.handle('audio-engine:set-aux-bus-route-to-master', async (_, aux: number, route: boolean) => {
  await sendCommandToEngine({ type: 'set_aux_bus_route_to_master', aux, route })
})

ipcMain.handle('audio-engine:set-aux-bus-output-enabled', async (_, aux: number, enabled: boolean) => {
  await sendCommandToEngine({ type: 'set_aux_bus_output_enabled', aux, enabled })
})

ipcMain.handle('audio-engine:set-aux-bus-output-channels', async (_, aux: number, leftChannel: number, rightChannel: number) => {
  await sendCommandToEngine({ type: 'set_aux_bus_output_channels', aux, left_channel: leftChannel, right_channel: rightChannel })
})

ipcMain.handle('audio-engine:set-aux-bus-route-to-subgroup', async (_, aux: number, subgroup: number, route: boolean) => {
  await sendCommandToEngine({ type: 'set_aux_bus_route_to_subgroup', aux, subgroup, route })
})

ipcMain.handle('audio-engine:set-track-source-aux-return', async (_, track: number, aux: number) => {
  await sendCommandToEngine({ type: 'set_track_source_aux_return', track, aux })
})

ipcMain.handle('audio-engine:list-devices', async () => {
  // Longer timeout for first launch (macOS may ask for permissions or verify signature)
  const response = await sendCommandAndWaitForResponse({ type: 'list_devices' }, 'devices', 60000)
  return response.devices
})

ipcMain.handle('audio-engine:list-audio-inputs', async () => {
  // Longer timeout for first launch (macOS may ask for permissions or verify signature)
  const response = await sendCommandAndWaitForResponse({ type: 'list_audio_inputs' }, 'audio_inputs', 60000)
  return response.inputs
})

// File picker dialog - non-blocking
ipcMain.handle('show-open-file-dialog', async () => {
  // Get the focused window to avoid blocking the main process
  const focusedWindow = BrowserWindow.getFocusedWindow()
  
  const dialogOptions = {
    properties: ['openFile', 'multiSelections'],
    filters: [
      { name: 'Audio Files', extensions: ['mp3', 'wav', 'flac', 'm4a', 'aac', 'ogg', 'wma', 'aiff'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  } as any
  
  const result = focusedWindow 
    ? await dialog.showOpenDialog(focusedWindow, dialogOptions)
    : await dialog.showOpenDialog(dialogOptions)
  
  if (result.canceled) {
    return null
  }
  
  // Return only paths - file reading will happen in chunks in renderer
  return result.filePaths.map(filePath => ({
    path: filePath,
    name: path.basename(filePath)
  }))
})

// Read file as array buffer (called after dialog closes to avoid blocking)
ipcMain.handle('read-file-as-buffer', async (_event, filePath: string) => {
  try {
    const buffer = await fs.promises.readFile(filePath)
    return {
      name: path.basename(filePath),
      buffer: buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength)
    }
  } catch (error) {
    console.error('Failed to read file:', error)
    throw error
  }
})

// Window control handlers
ipcMain.handle('get-platform', () => {
  return process.platform
})

ipcMain.handle('get-app-version', () => {
  return app.getVersion()
})

// License verification handler
ipcMain.handle('verify-license', async (_, licenseKey: string) => {
  try {
    // Get Edge Config client (lazy initialization)
    const client = getEdgeConfigClient()
    
    // Fetch license from Vercel Edge Config
    const license = await client.get<{
      type: 'demo' | 'medium' | 'full'
      expiresAt?: string
      active: boolean
    }>(`license_${licenseKey}`)

    // Check if license exists
    if (!license) {
      return { valid: false, message: 'Invalid license key' }
    }

    // Check if license is active
    if (!license.active) {
      return { valid: false, message: 'License has been deactivated' }
    }

    // Check if license has expired
    if (license.expiresAt) {
      const expiryDate = new Date(license.expiresAt)
      if (expiryDate < new Date()) {
        return { valid: false, message: 'License has expired' }
      }
    }

    // License is valid
    return {
      valid: true,
      type: license.type,
      expiresAt: license.expiresAt
    }
  } catch (error) {
    console.error('License verification error:', error)
    return {
      valid: false,
      message: error instanceof Error ? error.message : 'Failed to verify license'
    }
  }
})

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

// Master Tap (Recording) - Rust saves WAV file directly
ipcMain.handle('audio-engine:enable-master-tap', async (_event, filePath: string, settings: {
  format: 'wav' | 'mp3' | 'opus'
  sampleRate: number
  bitDepth: number
  bitrate: number
}) => {
  console.log('[Main] Enabling master tap with settings:', settings)
  await sendCommandToEngine({ 
    type: 'enable_master_tap', 
    file_path: filePath,
    sample_rate: settings.sampleRate,
    bit_depth: settings.bitDepth,
    format: settings.format
  })
})

ipcMain.handle('audio-engine:disable-master-tap', async () => {
  await sendCommandToEngine({ type: 'disable_master_tap' })
})

// Recording file management
ipcMain.handle('audio-engine:generate-recording-path', async () => {
  const recordingsDir = path.join(app.getPath('userData'), 'Recordings')
  
  // Create directory if it doesn't exist
  if (!fs.existsSync(recordingsDir)) {
    fs.mkdirSync(recordingsDir, { recursive: true })
  }
  
  const now = new Date()
  const timestamp = now.toISOString().replace(/[:.]/g, '-').slice(0, -5)
  const fileName = `Recording_${timestamp}.wav`
  
  return path.join(recordingsDir, fileName)
})

ipcMain.handle('audio-engine:get-recording-file-info', async (_event, filePath: string) => {
  try {
    const stats = fs.statSync(filePath)
    const name = path.basename(filePath, '.wav')
    
    let size = '0 KB'
    if (stats.size < 1024) {
      size = stats.size + ' B'
    } else if (stats.size < 1024 * 1024) {
      size = (stats.size / 1024).toFixed(1) + ' KB'
    } else {
      size = (stats.size / (1024 * 1024)).toFixed(1) + ' MB'
    }
    
    return { name, size }
  } catch (error) {
    console.error('[Main] Error getting file info:', error)
    return { name: 'Unknown', size: '0 KB' }
  }
})

ipcMain.handle('audio-engine:show-recording-in-folder', async (_event, filePath: string) => {
  shell.showItemInFolder(filePath)
})

ipcMain.handle('audio-engine:delete-recording-file', async (_event, filePath: string) => {
  try {
    fs.unlinkSync(filePath)
    console.log('[Main] Recording file deleted:', filePath)
  } catch (error) {
    console.error('[Main] Error deleting file:', error)
    throw error
  }
})

ipcMain.handle('audio-engine:list-recordings', async () => {
  try {
    const recordingsDir = path.join(app.getPath('userData'), 'Recordings')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(recordingsDir)) {
      fs.mkdirSync(recordingsDir, { recursive: true })
      return []
    }
    
    // Read all .wav files
    const files = fs.readdirSync(recordingsDir)
      .filter(file => file.endsWith('.wav'))
      .map(file => {
        const filePath = path.join(recordingsDir, file)
        const stats = fs.statSync(filePath)
        
        // Calculate size
        let size = '0 KB'
        if (stats.size < 1024) {
          size = stats.size + ' B'
        } else if (stats.size < 1024 * 1024) {
          size = (stats.size / 1024).toFixed(1) + ' KB'
        } else {
          size = (stats.size / (1024 * 1024)).toFixed(1) + ' MB'
        }
        
        return {
          id: file,
          name: file.replace('.wav', ''),
          path: filePath,
          size,
          created: stats.birthtime.toISOString()
        }
      })
      .sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())
    
    return files
  } catch (error) {
    console.error('[Main] Error listing recordings:', error)
    return []
  }
})

// ============================================================================
// Library Files IPC Handlers
// ============================================================================

ipcMain.handle('audio-engine:save-library-file', async (_event, arrayBuffer: ArrayBuffer, fileName: string, metadata?: any) => {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(libraryDir)) {
      fs.mkdirSync(libraryDir, { recursive: true })
    }
    
    // Generate unique file ID with original extension
    const ext = path.extname(fileName)
    const baseName = path.basename(fileName, ext)
    const timestamp = Date.now()
    const random = Math.random().toString(36).substring(2, 9)
    const fileId = `${timestamp}_${random}${ext}`
    const filePath = path.join(libraryDir, fileId)
    
    // Write audio file
    const buffer = Buffer.from(arrayBuffer)
    fs.writeFileSync(filePath, buffer)
    
    // Save metadata if provided (exclude arrayBuffer - file is already on disk)
    if (metadata) {
      const metadataPath = path.join(libraryDir, `${fileId}.meta.json`)
      const { arrayBuffer: _, ...metadataToSave } = metadata // Remove arrayBuffer if present
      fs.writeFileSync(metadataPath, JSON.stringify({
        id: fileId, // File ID for playlist references
        ...metadataToSave,
        originalFileName: fileName,
        timestamp
      }, null, 2))
    }
    
    console.log('[Main] Library file saved:', filePath)
    return fileId
  } catch (error) {
    console.error('[Main] Error saving library file:', error)
    throw error
  }
})

ipcMain.handle('audio-engine:list-library-files', async () => {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(libraryDir)) {
      fs.mkdirSync(libraryDir, { recursive: true })
      return []
    }
    
    // Read all audio files (exclude metadata files)
    const files = fs.readdirSync(libraryDir)
      .filter(file => !file.endsWith('.meta.json'))
      .map(file => {
        const filePath = path.join(libraryDir, file)
        const stats = fs.statSync(filePath)
        
        // Calculate size
        let size = '0 KB'
        if (stats.size < 1024) {
          size = stats.size + ' B'
        } else if (stats.size < 1024 * 1024) {
          size = (stats.size / 1024).toFixed(1) + ' KB'
        } else {
          size = (stats.size / (1024 * 1024)).toFixed(1) + ' MB'
        }
        
        // Load metadata if available
        let metadata: any = {}
        const metadataPath = path.join(libraryDir, `${file}.meta.json`)
        if (fs.existsSync(metadataPath)) {
          try {
            metadata = JSON.parse(fs.readFileSync(metadataPath, 'utf-8'))
          } catch (error) {
            console.error('[Main] Error parsing metadata:', error)
          }
        }
        
        return {
          id: metadata.id || file, // Use metadata ID if available, fallback to filename
          fileName: metadata.originalFileName || file,
          filePath,
          mimeType: metadata.mimeType || 'audio/mpeg',
          size,
          timestamp: metadata.timestamp || stats.birthtimeMs,
          artist: metadata.artist,
          title: metadata.title,
          artwork: metadata.artwork,
          created: stats.birthtime.toISOString()
        }
      })
      .sort((a, b) => b.timestamp - a.timestamp)
    
    return files
  } catch (error) {
    console.error('[Main] Error listing library files:', error)
    return []
  }
})

ipcMain.handle('audio-engine:get-library-file', async (_event, fileId: string) => {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    const filePath = path.join(libraryDir, fileId)
    
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found: ${fileId}`)
    }
    
    // Read file and metadata
    const buffer = fs.readFileSync(filePath)
    const arrayBuffer = buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength)
    
    // Load metadata if available
    let metadata: any = {}
    const metadataPath = path.join(libraryDir, `${fileId}.meta.json`)
    if (fs.existsSync(metadataPath)) {
      try {
        metadata = JSON.parse(fs.readFileSync(metadataPath, 'utf-8'))
      } catch (error) {
        console.error('[Main] Error parsing metadata:', error)
      }
    }
    
    return {
      id: fileId,
      fileName: metadata.originalFileName || fileId,
      arrayBuffer,
      mimeType: metadata.mimeType || 'audio/mpeg',
      timestamp: metadata.timestamp || Date.now(),
      artist: metadata.artist,
      title: metadata.title,
      artwork: metadata.artwork
    }
  } catch (error) {
    console.error('[Main] Error getting library file:', error)
    throw error
  }
})

ipcMain.handle('audio-engine:delete-library-file', async (_event, fileId: string) => {
  try {
    const libraryDir = path.join(app.getPath('userData'), 'Library')
    const filePath = path.join(libraryDir, fileId)
    const metadataPath = path.join(libraryDir, `${fileId}.meta.json`)
    
    // Delete audio file
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath)
    }
    
    // Delete metadata file
    if (fs.existsSync(metadataPath)) {
      fs.unlinkSync(metadataPath)
    }
    
    console.log('[Main] Library file deleted:', fileId)
  } catch (error) {
    console.error('[Main] Error deleting library file:', error)
    throw error
  }
})

// ============================================================================
// Playlist IPC Handlers
// ============================================================================

ipcMain.handle('audio-engine:save-playlist', async (_event, playlist: any) => {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(playlistsDir)) {
      fs.mkdirSync(playlistsDir, { recursive: true })
    }
    
    const playlistPath = path.join(playlistsDir, `${playlist.id}.json`)
    fs.writeFileSync(playlistPath, JSON.stringify(playlist, null, 2))
    
  } catch (error) {
    console.error('[Main] Error saving playlist:', error)
    throw error
  }
})

ipcMain.handle('audio-engine:list-playlists', async () => {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(playlistsDir)) {
      fs.mkdirSync(playlistsDir, { recursive: true })
      return []
    }
    
    // Read all playlist JSON files
    const files = fs.readdirSync(playlistsDir)
      .filter(file => file.endsWith('.json'))
      .map(file => {
        const filePath = path.join(playlistsDir, file)
        try {
          const content = fs.readFileSync(filePath, 'utf-8')
          return JSON.parse(content)
        } catch (error) {
          console.error('[Main] Error parsing playlist:', file, error)
          return null
        }
      })
      .filter(playlist => playlist !== null)
      .sort((a, b) => (b.updatedAt || b.createdAt) - (a.updatedAt || a.createdAt))
    
    return files
  } catch (error) {
    console.error('[Main] Error listing playlists:', error)
    return []
  }
})

ipcMain.handle('audio-engine:get-playlist', async (_event, playlistId: string) => {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    const playlistPath = path.join(playlistsDir, `${playlistId}.json`)
    
    if (!fs.existsSync(playlistPath)) {
      return null
    }
    
    const content = fs.readFileSync(playlistPath, 'utf-8')
    return JSON.parse(content)
  } catch (error) {
    console.error('[Main] Error getting playlist:', error)
    return null
  }
})

ipcMain.handle('audio-engine:delete-playlist', async (_event, playlistId: string) => {
  try {
    const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
    const playlistPath = path.join(playlistsDir, `${playlistId}.json`)
    
    if (fs.existsSync(playlistPath)) {
      fs.unlinkSync(playlistPath)
      console.log('[Main] Playlist deleted:', playlistId)
    }
  } catch (error) {
    console.error('[Main] Error deleting playlist:', error)
    throw error
  }
})

// ============================================================================
// Scenes IPC Handlers
// ============================================================================

ipcMain.handle('audio-engine:save-scene', async (_event, scene: any) => {
  try {
    const scenesDir = path.join(app.getPath('userData'), 'Scenes')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(scenesDir)) {
      fs.mkdirSync(scenesDir, { recursive: true })
    }
    
    const scenePath = path.join(scenesDir, `${scene.id}.json`)
    fs.writeFileSync(scenePath, JSON.stringify(scene, null, 2))
    
    console.log('[Main] Scene saved:', scenePath)
  } catch (error) {
    console.error('[Main] Error saving scene:', error)
    throw error
  }
})

ipcMain.handle('audio-engine:list-scenes', async () => {
  try {
    const scenesDir = path.join(app.getPath('userData'), 'Scenes')
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(scenesDir)) {
      fs.mkdirSync(scenesDir, { recursive: true })
      return []
    }
    
    // Read all scene JSON files
    const files = fs.readdirSync(scenesDir)
      .filter(file => file.endsWith('.json'))
      .map(file => {
        const filePath = path.join(scenesDir, file)
        try {
          const content = fs.readFileSync(filePath, 'utf-8')
          return JSON.parse(content)
        } catch (error) {
          console.error('[Main] Error parsing scene:', file, error)
          return null
        }
      })
      .filter(scene => scene !== null)
      .sort((a, b) => b.timestamp - a.timestamp) // Most recent first
    
    return files
  } catch (error) {
    console.error('[Main] Error listing scenes:', error)
    return []
  }
})

ipcMain.handle('audio-engine:get-scene', async (_event, sceneId: string) => {
  try {
    const scenesDir = path.join(app.getPath('userData'), 'Scenes')
    const scenePath = path.join(scenesDir, `${sceneId}.json`)
    
    if (!fs.existsSync(scenePath)) {
      return null
    }
    
    const content = fs.readFileSync(scenePath, 'utf-8')
    return JSON.parse(content)
  } catch (error) {
    console.error('[Main] Error getting scene:', error)
    return null
  }
})

ipcMain.handle('audio-engine:delete-scene', async (_event, sceneId: string) => {
  try {
    const scenesDir = path.join(app.getPath('userData'), 'Scenes')
    const scenePath = path.join(scenesDir, `${sceneId}.json`)
    
    if (fs.existsSync(scenePath)) {
      fs.unlinkSync(scenePath)
      console.log('[Main] Scene deleted:', sceneId)
    }
  } catch (error) {
    console.error('[Main] Error deleting scene:', error)
    throw error
  }
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

const createSplashWindow = () => {
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

const createWindow = () => {
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
  
  const mainWindow = new BrowserWindow({
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
    mainWindow.webContents.send('window-maximized')
  })

  mainWindow.on('unmaximize', () => {
    mainWindow.webContents.send('window-unmaximized')
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
      if (windowState.isMaximized) {
        mainWindow.maximize()
      }
      setTimeout(() => {
        mainWindow.show()
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
  })

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
          "connect-src 'self'",
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

  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL)
  } else {
    mainWindow.loadFile(path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`))
  }

 // mainWindow.webContents.openDevTools()
}

app.whenReady().then(() => {
  // Prevent system from throttling audio playback
  powerSaveBlockerId = powerSaveBlocker.start('prevent-app-suspension')
  console.log('[Main] Power save blocker started:', powerSaveBlocker.isStarted(powerSaveBlockerId))
  
  startAudioEngine()
  
  // Show splash screen first
  createSplashWindow()
  
  // Create main window (hidden until ready)
  // Add a small delay to ensure splash is visible first
  setTimeout(() => {
    createWindow()
  }, 100)

  app.on('activate', () => {
    // If no windows, create new ones
    if (BrowserWindow.getAllWindows().length === 0) {
      createSplashWindow()
      setTimeout(() => {
        createWindow()
      }, 100)
    } else {
      // On macOS, restore hidden window when clicking dock icon
      const windows = BrowserWindow.getAllWindows()
      const mainWin = windows.find(w => !w.isDestroyed() && w.webContents.getURL().includes('index.html'))
      if (mainWin && !mainWin.isVisible()) {
        mainWin.show()
      }
    }
  })
})

app.on('window-all-closed', () => {
  stopAudioEngine()
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
  // Stop power save blocker
  if (powerSaveBlockerId !== null && powerSaveBlocker.isStarted(powerSaveBlockerId)) {
    powerSaveBlocker.stop(powerSaveBlockerId)
  }
})