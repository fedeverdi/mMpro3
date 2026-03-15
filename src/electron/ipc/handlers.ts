import { app, ipcMain, BrowserWindow, dialog, shell } from 'electron'
import * as path from 'path'
import * as fs from 'fs'
import * as os from 'os'
import { createClient } from '@vercel/edge-config'
import { 
  sendCommandToEngine, 
  sendCommandAndWaitForResponse,
  getAudioEngineProcess
} from '../audio-engine/process'
import { broadcastToWebSocketClients, disconnectAllRemoteClients, WS_PORT } from '../websocket/server'
import { HTTP_PORT } from '../http/server'
import type { WebSocketServerDependencies } from '../websocket/server'

// Edge Config client for license verification (lazy initialized)
let edgeConfigClient: ReturnType<typeof createClient> | null = null

const getEdgeConfigClient = () => {
  if (!edgeConfigClient) {
    edgeConfigClient = createClient(process.env.EDGE_CONFIG)
  }
  return edgeConfigClient
}

/**
 * Dependencies for IPC handlers
 */
export interface IpcHandlerDependencies {
  activeTempFiles: Set<string>
  lastKnownState: {
    masterEqFilters?: any[]
    masterParameters?: any
    auxParameters?: any[]
    subgroupParameters?: any[]
    auxBuses?: any[]
  }
  wsServerDeps: WebSocketServerDependencies | null
}

/**
 * Setup all IPC handlers
 */
export const setupIpcHandlers = (deps: IpcHandlerDependencies): void => {
  const { activeTempFiles, lastKnownState, wsServerDeps } = deps

  // ============================================================================
  // Audio Engine Control Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:start', async (_, inputDevice?: string | null, outputDevice?: string | null, sampleRate?: number | null, bufferSize?: number | null) => {
    const command: any = { type: 'start' }
    if (inputDevice) command.input_device = inputDevice
    if (outputDevice) command.output_device = outputDevice
    // Only add sample_rate/buffer_size if explicitly provided (not null/undefined)
    // This allows 0 (Auto) to be passed correctly
    if (sampleRate !== null && sampleRate !== undefined) command.sample_rate = sampleRate
    if (bufferSize !== null && bufferSize !== undefined) command.buffer_size = bufferSize
    await sendCommandToEngine(command)
  })

  ipcMain.handle('audio-engine:stop', async () => {
    await sendCommandToEngine({ type: 'stop' })
  })

  // ============================================================================
  // Track Control Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:set-gain', async (_, track: number, gain: number) => {
    await sendCommandToEngine({ type: 'set_gain', track, gain })
  })

  ipcMain.handle('audio-engine:set-volume', async (_, track: number, volume: number) => {
    await sendCommandToEngine({ type: 'set_volume', track, volume })
  })

  ipcMain.handle('audio-engine:set-mute', async (_, track: number, mute: boolean) => {
    await sendCommandToEngine({ type: 'set_mute', track, mute })
  })

  ipcMain.handle('audio-engine:set-solo', async (_, track: number, solo: boolean) => {
    await sendCommandToEngine({ type: 'set_solo', track, solo })
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

  // Track Insert Effects Chain handlers
  ipcMain.handle('audio-engine:add-track-insert', async (_, track: number, effectType: string, position?: number) => {
    await sendCommandToEngine({ type: 'add_track_insert', track, effect_type: effectType, position })
  })

  ipcMain.handle('audio-engine:remove-track-insert', async (_, track: number, insertId: number) => {
    await sendCommandToEngine({ type: 'remove_track_insert', track, insert_id: insertId })
  })

  ipcMain.handle('audio-engine:move-track-insert', async (_, track: number, insertId: number, newPosition: number) => {
    await sendCommandToEngine({ type: 'move_track_insert', track, insert_id: insertId, new_position: newPosition })
  })

  ipcMain.handle('audio-engine:set-track-insert-enabled', async (_, track: number, insertId: number, enabled: boolean) => {
    await sendCommandToEngine({ type: 'set_track_insert_enabled', track, insert_id: insertId, enabled })
  })

  ipcMain.handle('audio-engine:set-track-insert-compressor', async (_, track: number, insertId: number, threshold: number, ratio: number, attack: number, release: number) => {
    await sendCommandToEngine({ type: 'set_track_insert_compressor', track, insert_id: insertId, threshold, ratio, attack, release })
  })

  ipcMain.handle('audio-engine:set-track-insert-gate', async (_, track: number, insertId: number, threshold: number, range: number, attack: number, release: number) => {
    await sendCommandToEngine({ type: 'set_track_insert_gate', track, insert_id: insertId, threshold, range, attack, release })
  })

  ipcMain.handle('audio-engine:set-track-insert-reverb', async (_, track: number, insertId: number, roomSize: number, damping: number, wet: number, width: number) => {
    await sendCommandToEngine({ type: 'set_track_insert_reverb', track, insert_id: insertId, room_size: roomSize, damping, wet, width })
  })

  ipcMain.handle('audio-engine:set-track-insert-delay', async (_, track: number, insertId: number, timeL: number, timeR: number, feedback: number, mix: number) => {
    await sendCommandToEngine({ type: 'set_track_insert_delay', track, insert_id: insertId, time_l: timeL, time_r: timeR, feedback, mix })
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

  // ============================================================================
  // Track Source Handlers
  // ============================================================================

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

  ipcMain.handle('audio-engine:set-track-source-file', async (_, track: number, filePath: string, artist?: string|null, title?: string|null, playlistId?: string|null, playlistName?: string|null, playlistIndex?: number|null) => {
    const payload: any = { type: 'set_track_source_file', track, file_path: filePath }
    if (artist !== undefined && artist !== null) payload.artist = artist
    if (title !== undefined && title !== null) payload.title = title
    if (playlistId !== undefined && playlistId !== null) payload.playlist_id = playlistId
    if (playlistName !== undefined && playlistName !== null) payload.playlist_name = playlistName
    if (playlistIndex !== undefined && playlistIndex !== null) payload.playlist_index = playlistIndex
    await sendCommandToEngine(payload)
  })

  ipcMain.handle('audio-engine:set-track-source-aux-return', async (_, track: number, aux: number) => {
    await sendCommandToEngine({ type: 'set_track_source_aux_return', track, aux })
  })

  // ============================================================================
  // Temp File Management
  // ============================================================================

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
      
      // Track this file for cleanup
      activeTempFiles.add(tempFilePath)
      
      return tempFilePath
    } catch (error) {
      console.error('[Main] Error saving temp audio file:', error)
      throw error
    }
  })

  ipcMain.handle('audio-engine:delete-temp-file', async (_, filePath: string) => {
    try {
      if (fs.existsSync(filePath)) {
        fs.unlinkSync(filePath)
        activeTempFiles.delete(filePath)
        console.log('[Main] Deleted temp file:', path.basename(filePath))
      }
    } catch (error) {
      console.error('[Main] Error deleting temp file:', error)
    }
  })

  // ============================================================================
  // File Playback Controls
  // ============================================================================

  ipcMain.handle('audio-engine:play-file', async (_, track: number, fileId?: string) => {
    let filePath: string | undefined
    let artist: string | undefined
    let title: string | undefined
    
    // If fileId is provided, resolve it to full path and load metadata
    if (fileId) {
      const libraryDir = path.join(app.getPath('userData'), 'Library')
      filePath = path.join(libraryDir, fileId)
      
      // Verify file exists
      if (!fs.existsSync(filePath)) {
        throw new Error(`File not found in library: ${fileId}`)
      }
      
      // Load metadata if available
      const metadataPath = path.join(libraryDir, `${fileId}.meta.json`)
      if (fs.existsSync(metadataPath)) {
        try {
          const metadata = JSON.parse(fs.readFileSync(metadataPath, 'utf-8'))
          artist = metadata.artist
          title = metadata.title
        } catch (error) {
          console.error('[Main] Error reading metadata for playFile:', error)
        }
      }
    }
    
    // Send command with metadata if available
    const payload: any = { type: 'play_file', track, file_path: filePath }
    if (artist) payload.artist = artist
    if (title) payload.title = title
    
    await sendCommandToEngine(payload)
  })

  ipcMain.handle('audio-engine:pause-file', async (_, track: number) => {
    await sendCommandToEngine({ type: 'pause_file', track })
  })

  ipcMain.handle('audio-engine:stop-file', async (_, track: number) => {
    await sendCommandToEngine({ type: 'stop_file', track })
  })

  // ============================================================================
  // EQ Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:set-eq', async (_, track: number, low: number, low_mid: number, high_mid: number, high: number) => {
    await sendCommandToEngine({ type: 'set_eq', track, low, low_mid, high_mid, high })
  })

  ipcMain.handle('audio-engine:set-eq-enabled', async (_, track: number, enabled: boolean) => {
    await sendCommandToEngine({ type: 'set_eq_enabled', track, enabled })
  })

  ipcMain.handle('audio-engine:set-parametric-eq-filters', async (_, track: number, filters: Array<{type: string, frequency: number, gain: number, q: number}>) => {
    await sendCommandToEngine({ type: 'set_parametric_eq_filters', track, filters })
  })

  ipcMain.handle('audio-engine:set-parametric-eq-enabled', async (_, track: number, enabled: boolean) => {
    await sendCommandToEngine({ type: 'set_parametric_eq_enabled', track, enabled })
  })

  ipcMain.handle('audio-engine:clear-parametric-eq', async (_, track: number) => {
    await sendCommandToEngine({ type: 'clear_parametric_eq', track })
  })

  // ============================================================================
  // Master Control Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:set-master-gain', async (_, gain: number) => {
    await sendCommandToEngine({ type: 'set_master_gain', gain })
  })

  ipcMain.handle('audio-engine:set-master-gain-left', async (_, gain: number) => {
    await sendCommandToEngine({ type: 'set_master_gain_left', gain })
  })

  ipcMain.handle('audio-engine:set-master-gain-right', async (_, gain: number) => {
    await sendCommandToEngine({ type: 'set_master_gain_right', gain })
  })

  ipcMain.handle('audio-engine:set-master-mute', async (_, mute: boolean) => {
    await sendCommandToEngine({ type: 'set_master_mute', mute })
  })

  ipcMain.handle('audio-engine:set-master-linked', async (_, linked: boolean) => {
    await sendCommandToEngine({ type: 'set_master_linked', linked })
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

  ipcMain.handle('audio-engine:set-selected-master-output', async (_, deviceId: string | null) => {
    await sendCommandToEngine({ type: 'set_selected_master_output', device_id: deviceId })
  })

  ipcMain.handle('audio-engine:set-master-compressor', async (_, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
    await sendCommandToEngine({ type: 'set_master_compressor', enabled, threshold, ratio, attack, release })
  })

  ipcMain.handle('audio-engine:set-master-limiter', async (_, enabled: boolean, ceiling: number, release: number) => {
    await sendCommandToEngine({ type: 'set_master_limiter', enabled, ceiling, release })
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

  ipcMain.handle('audio-engine:add-master-fx-effect', async (_, effectType: string) => {
    await sendCommandToEngine({ type: 'add_master_fx_effect', effect_type: effectType })
  })

  ipcMain.handle('audio-engine:remove-master-fx-effect', async (_, effectType: string) => {
    await sendCommandToEngine({ type: 'remove_master_fx_effect', effect_type: effectType })
  })

  // ============================================================================
  // Aux Buses Handlers
  // ============================================================================

  ipcMain.handle('update-aux-buses-state', async (_, auxBuses: any[]) => {
    // Cache the aux buses state
    lastKnownState.auxBuses = auxBuses
    
    // Broadcast to all WebSocket clients (including detached windows)
    broadcastToWebSocketClients({
      type: 'aux_buses_state',
      auxBuses: auxBuses
    })
  })

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

  ipcMain.handle('audio-engine:set-aux-bus-selected-output', async (_, aux: number, deviceId: string | null) => {
    await sendCommandToEngine({ type: 'set_aux_bus_selected_output', aux, device_id: deviceId })
  })

  // ============================================================================
  // Subgroup Handlers
  // ============================================================================

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

  ipcMain.handle('audio-engine:set-selected-subgroup-output', async (_, subgroup: number, deviceId: string | null) => {
    await sendCommandToEngine({ type: 'set_selected_subgroup_output', subgroup, device_id: deviceId })
  })

  ipcMain.handle('audio-engine:set-track-route-to-subgroup', async (_, track: number, subgroup: number, route: boolean) => {
    await sendCommandToEngine({ type: 'set_track_route_to_subgroup', track, subgroup, route })
  })

  // ============================================================================
  // NDI Streaming Handlers
  // ============================================================================

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

  // ============================================================================
  // Metering Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:get-loudness', async () => {
    try {
      const process = getAudioEngineProcess()
      if (!process || !process.stdin) {
        return null
      }
      return await sendCommandAndWaitForResponse({ type: 'get_loudness' }, 'loudness')
    } catch (error) {
      return null
    }
  })

  ipcMain.handle('audio-engine:reset-loudness', async () => {
    await sendCommandToEngine({ type: 'reset_loudness' })
  })

  ipcMain.handle('audio-engine:get-dynamic-range', async () => {
    try {
      const process = getAudioEngineProcess()
      if (!process || !process.stdin) {
        return null
      }
      return await sendCommandAndWaitForResponse({ type: 'get_dynamic_range' }, 'dynamic_range')
    } catch (error) {
      return null
    }
  })

  ipcMain.handle('audio-engine:reset-dynamic-range', async () => {
    await sendCommandToEngine({ type: 'reset_dynamic_range' })
  })

  ipcMain.handle('audio-engine:get-phase-correlation', async () => {
    try {
      const process = getAudioEngineProcess()
      if (!process || !process.stdin) {
        return null
      }
      return await sendCommandAndWaitForResponse({ type: 'get_phase_correlation' }, 'phase_correlation')
    } catch (error) {
      return null
    }
  })

  ipcMain.handle('audio-engine:reset-phase-correlation', async () => {
    await sendCommandToEngine({ type: 'reset_phase_correlation' })
  })

  ipcMain.handle('audio-engine:get-stereo-width', async () => {
    try {
      const process = getAudioEngineProcess()
      if (!process || !process.stdin) {
        return null
      }
      return await sendCommandAndWaitForResponse({ type: 'get_stereo_width' }, 'stereo_width')
    } catch (error) {
      return null
    }
  })

  ipcMain.handle('audio-engine:reset-stereo-width', async () => {
    await sendCommandToEngine({ type: 'reset_stereo_width' })
  })

  ipcMain.handle('audio-engine:get-headroom', async () => {
    try {
      const process = getAudioEngineProcess()
      if (!process || !process.stdin) {
        return null
      }
      return await sendCommandAndWaitForResponse({ type: 'get_headroom' }, 'headroom')
    } catch (error) {
      return null
    }
  })

  ipcMain.handle('audio-engine:reset-headroom', async () => {
    await sendCommandToEngine({ type: 'reset_headroom' })
  })

  // ============================================================================
  // Device Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:list-devices', async () => {
    const response = await sendCommandAndWaitForResponse({ type: 'list_devices' }, 'devices', 60000)
    return response.devices
  })

  ipcMain.handle('audio-engine:list-audio-inputs', async () => {
    const response = await sendCommandAndWaitForResponse({ type: 'list_audio_inputs' }, 'audio_inputs', 60000)
    return response.inputs
  })

  // ============================================================================
  // File Dialog Handlers
  // ============================================================================

  ipcMain.handle('show-open-file-dialog', async () => {
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
    
    return result.filePaths.map(filePath => ({
      path: filePath,
      name: path.basename(filePath)
    }))
  })

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

  // ============================================================================
  // System Info Handlers
  // ============================================================================

  ipcMain.handle('get-platform', () => {
    return process.platform
  })

  ipcMain.handle('get-app-version', () => {
    return app.getVersion()
  })

  ipcMain.handle('get-local-ip', () => {
    try {
      const networkInterfaces = os.networkInterfaces()
      let localIp = 'localhost'
      
      for (const interfaceName in networkInterfaces) {
        const interfaces = networkInterfaces[interfaceName]
        if (!interfaces) continue
        
        for (const iface of interfaces) {
          if (iface.family === 'IPv4' && !iface.internal) {
            localIp = iface.address
            break
          }
        }
        
        if (localIp !== 'localhost') break
      }
      
      return {
        ip: localIp,
        port: HTTP_PORT,
        wsPort: WS_PORT
      }
    } catch (error) {
      console.error('[Main] Error getting local IP:', error)
      return {
        ip: 'localhost',
        port: HTTP_PORT,
        wsPort: WS_PORT
      }
    }
  })

  // ============================================================================
  // License Handlers
  // ============================================================================

  ipcMain.handle('verify-license', async (_, licenseKey: string) => {
    try {
      const client = getEdgeConfigClient()
      
      const license = await client.get<{
        type: 'demo' | 'medium' | 'full'
        expiresAt?: string
        active: boolean
      }>(`license_${licenseKey}`)

      if (!license) {
        return { valid: false, message: 'Invalid license key' }
      }

      if (!license.active) {
        return { valid: false, message: 'License has been deactivated' }
      }

      if (license.expiresAt) {
        const expiryDate = new Date(license.expiresAt)
        if (expiryDate < new Date()) {
          return { valid: false, message: 'License has expired' }
        }
      }

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

  ipcMain.handle('audio-engine:save-license', async (_, key: string, licenseType: string, expiresAt: string | null) => {
    const process = getAudioEngineProcess()
    if (!process || !process.stdin) {
      throw new Error('Audio engine not running')
    }
    
    try {
      const command = {
        type: 'save_license',
        key,
        license_type: licenseType,
        expires_at: expiresAt
      }
          
      const response = await sendCommandAndWaitForResponse(command, 'ok', 3000)
          
      if (response.message?.includes('License saved')) {
        try {
          const getLicenseCommand = { type: 'get_license' }
          await sendCommandAndWaitForResponse(getLicenseCommand, 'license', 2000)
        } catch (err) {
          console.warn('[Main.ts] Failed to broadcast license, but save was successful:', err)
        }
        
        return true
      } else {
        return true
      }
    } catch (error) {
      console.error('[Main.ts] ✗ Failed to save license:', error)
      return true
    }
  })

  ipcMain.handle('audio-engine:get-license', async () => {
    const process = getAudioEngineProcess()
    if (!process || !process.stdin) {
      return { key: 'DEMO', license_type: 'demo', expires_at: null, is_valid: true }
    }
    
    try {
      const command = { type: 'get_license' }    
      const response = await sendCommandAndWaitForResponse(command, 'license', 2000)
      
      return response
    } catch (error) {
      console.warn('[Main.ts] Timeout or error getting license from Rust:', error)
      return { key: 'DEMO', license_type: 'demo', expires_at: null, is_valid: true }
    }
  })

  // ============================================================================
  // Audio Config Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:save-audio-config', async (_, sampleRate: number, bufferSize: number) => {
    const process = getAudioEngineProcess()
    if (!process || !process.stdin) {
      throw new Error('Audio engine not running')
    }
    
    try {
      const command = {
        type: 'save_audio_config',
        sample_rate: sampleRate,
        buffer_size: bufferSize
      }
          
      const response = await sendCommandAndWaitForResponse(command, 'ok', 3000)
          
      if (response.message?.includes('Audio config saved')) {
        return true
      } else {
        return true
      }
    } catch (error) {
      console.error('[Main.ts] ✗ Failed to save audio config:', error)
      return true
    }
  })

  ipcMain.handle('audio-engine:get-audio-config', async () => {
    const process = getAudioEngineProcess()
    if (!process || !process.stdin) {
      return { sample_rate: 0, buffer_size: 256 }
    }
    
    try {
      const command = { type: 'get_audio_config' }    
      const response = await sendCommandAndWaitForResponse(command, 'audio_config', 2000)
      
      return response
    } catch (error) {
      console.warn('[Main.ts] Timeout or error getting audio config from Rust:', error)
      return { sample_rate: 0, buffer_size: 256 }
    }
  })

  // ============================================================================
  // Remote Control Handlers
  // ============================================================================

  ipcMain.handle('disconnect-remote-clients', async () => {
    if (!wsServerDeps) {
      console.warn('[Main] WebSocket server not initialized yet')
      return 0
    }
    const count = disconnectAllRemoteClients(wsServerDeps)
    return count
  })

  // ============================================================================
  // Recording Handlers
  // ============================================================================

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

  ipcMain.handle('audio-engine:generate-recording-path', async () => {
    const recordingsDir = path.join(app.getPath('userData'), 'Recordings')
    
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
      
      if (!fs.existsSync(recordingsDir)) {
        fs.mkdirSync(recordingsDir, { recursive: true })
        return []
      }
      
      const files = fs.readdirSync(recordingsDir)
        .filter(file => file.endsWith('.wav'))
        .map(file => {
          const filePath = path.join(recordingsDir, file)
          const stats = fs.statSync(filePath)
          
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
  // Library Files Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:save-library-file', async (_event, arrayBuffer: ArrayBuffer, fileName: string, metadata?: any) => {
    try {
      const libraryDir = path.join(app.getPath('userData'), 'Library')
      
      if (!fs.existsSync(libraryDir)) {
        fs.mkdirSync(libraryDir, { recursive: true })
      }
      
      const ext = path.extname(fileName)
      const baseName = path.basename(fileName, ext)
      const timestamp = Date.now()
      const random = Math.random().toString(36).substring(2, 9)
      const fileId = `${timestamp}_${random}${ext}`
      const filePath = path.join(libraryDir, fileId)
      
      const buffer = Buffer.from(arrayBuffer)
      fs.writeFileSync(filePath, buffer)
      
      if (metadata) {
        const metadataPath = path.join(libraryDir, `${fileId}.meta.json`)
        const { arrayBuffer: _, ...metadataToSave } = metadata
        fs.writeFileSync(metadataPath, JSON.stringify({
          id: fileId,
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
      
      if (!fs.existsSync(libraryDir)) {
        fs.mkdirSync(libraryDir, { recursive: true })
        return []
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
              console.error('[Main] Error parsing metadata:', error)
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
      
      const buffer = fs.readFileSync(filePath)
      const arrayBuffer = buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength)
      
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
        filePath: filePath,
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
      
      if (fs.existsSync(filePath)) {
        fs.unlinkSync(filePath)
      }
      
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
  // Playlist Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:save-playlist', async (_event, playlist: any) => {
    try {
      const playlistsDir = path.join(app.getPath('userData'), 'Playlists')
      
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
      
      if (!fs.existsSync(playlistsDir)) {
        fs.mkdirSync(playlistsDir, { recursive: true })
        return []
      }
      
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
  // Scenes Handlers
  // ============================================================================

  ipcMain.handle('audio-engine:save-scene', async (_event, scene: any) => {
    try {
      const scenesDir = path.join(app.getPath('userData'), 'Scenes')
      
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
      
      if (!fs.existsSync(scenesDir)) {
        fs.mkdirSync(scenesDir, { recursive: true })
        return []
      }
      
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
        .sort((a, b) => b.timestamp - a.timestamp)
      
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
}
