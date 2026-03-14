import { spawn, ChildProcess } from 'node:child_process'
import { app, BrowserWindow } from 'electron'
import path from 'node:path'
import fs from 'node:fs'

export interface AudioEngineCallbacks {
  onResponse: (response: any) => void
  onStarted: () => void
  onStopped: () => void
  onError: (error: string) => void
}

let audioEngineProcess: ChildProcess | null = null
let isAudioEngineStarted: boolean = false
const responseHandlers: Map<string, (response: any) => void> = new Map()
let callbacks: AudioEngineCallbacks | null = null

/**
 * Initialize audio engine with callbacks
 */
export const initAudioEngine = (engineCallbacks: AudioEngineCallbacks): void => {
  callbacks = engineCallbacks
}

/**
 * Start audio engine process
 */
export const startAudioEngine = (): void => {
  // If audio engine is already running, stop it first
  if (audioEngineProcess) {
    console.log('[AudioEngine] Already running, stopping it first...')
    stopAudioEngine()
    
    // Wait a bit for the stop to complete, then start fresh
    setTimeout(() => {
      startAudioEngineInternal()
    }, 150)
    return
  }
  
  startAudioEngineInternal()
}

/**
 * Internal function to start the audio engine process
 */
const startAudioEngineInternal = (): void => {
  // Get the binary name based on platform
  const binaryName = process.platform === 'win32' ? 'mmpro3-engine.exe' : 'mmpro3-engine'
  
  const enginePath = app.isPackaged
    ? path.join(process.resourcesPath, binaryName)
    : path.join(app.getAppPath(), 'audio-engine', 'target', 'release', binaryName)
  
  if (!fs.existsSync(enginePath)) {
    console.error('[AudioEngine] Engine not found! Skipping audio engine startup.')
    console.error('[AudioEngine] Searched at:', enginePath)
    return
  }
  
  const licensePath = app.getPath('userData')
  console.log('[AudioEngine] Starting audio engine with LICENSE_PATH:', licensePath)
  
  audioEngineProcess = spawn(enginePath, [], {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: {
      ...process.env,
      LICENSE_PATH: licensePath // Rust will save license.json here
    }
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
        
        // Track audio engine started/stopped state
        if (responseType === 'started') {
          isAudioEngineStarted = true
          console.log('[AudioEngine] Started - state updated')
          if (callbacks) callbacks.onStarted()
        } else if (responseType === 'stopped') {
          isAudioEngineStarted = false
          console.log('[AudioEngine] Stopped - state updated')
          if (callbacks) callbacks.onStopped()
        }
        
        // Forward response to callback
        if (callbacks) {
          callbacks.onResponse(response)
        }
      } catch (err) {
        // Not JSON, just log as plain text
      }
    }
  })

  audioEngineProcess.stderr?.on('data', (data) => {
    const error = data.toString()
    console.error('[AudioEngine Error]', error)
    if (callbacks) callbacks.onError(error)
  })

  audioEngineProcess.on('close', (code) => {
    console.log('[AudioEngine] Closed with code:', code)
    audioEngineProcess = null
  })
}

/**
 * Stop audio engine process
 */
export const stopAudioEngine = (): void => {
  if (audioEngineProcess) {
    
    // First, stop all file players gracefully
    try {
      if (audioEngineProcess.stdin) {
        const stopCommand = JSON.stringify({ type: 'stop_all_files' }) + '\n'
        audioEngineProcess.stdin.write(stopCommand)
        console.log('[AudioEngine] Sent stop_all_files command')
      }
    } catch (err) {
      console.error('[AudioEngine] Error sending stop_all_files command:', err)
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

/**
 * Send command to audio engine
 */
export const sendCommandToEngine = (command: any): Promise<void> => {
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

/**
 * Send command and wait for specific response type
 */
export const sendCommandAndWaitForResponse = (command: any, responseType: string, timeout = 5000): Promise<any> => {
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

/**
 * Get audio engine process reference
 */
export const getAudioEngineProcess = (): ChildProcess | null => {
  return audioEngineProcess
}

/**
 * Check if audio engine is started
 */
export const getIsAudioEngineStarted = (): boolean => {
  return isAudioEngineStarted
}
