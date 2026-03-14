/**
 * Remote Audio Engine Client - WebSocket Proxy
 * Provides the same interface as the native audioEngine but communicates via WebSocket
 * for remote browser access
 */

export class RemoteAudioEngine {
  private ws: WebSocket | null = null
  private url: string
  private reconnectInterval: number = 5000
  private reconnectTimer: number | null = null
  private responseListeners: Array<(response: any) => void> = []
  private isConnecting: boolean = false
  private pendingLicenseRequest: boolean = false // Track if we're waiting for a license response
  
  // Throttling system for continuous controls
  private pendingUpdates: Map<string, any> = new Map()
  private throttleTimer: number | null = null
  private readonly THROTTLE_MS = 16 // ~60fps
  
  // Local state cache for UI synchronization
  private masterEqFilters: any[] = []
  private trackEqFilters: Map<number, any[]> = new Map()

  constructor(host: string, port: number = 3001) {
    this.url = `ws://${host}:${port}`
  }

  async connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return
    }

    return new Promise((resolve, reject) => {
      this.isConnecting = true

      this.ws = new WebSocket(this.url)

      this.ws.onopen = () => {
        this.isConnecting = false
        if (this.reconnectTimer) {
          clearTimeout(this.reconnectTimer)
          this.reconnectTimer = null
        }
        resolve()
      }

      this.ws.onmessage = (event) => {
        try {
          const response = JSON.parse(event.data)
          
          // Handle remote already active - emit event for UI to show modal
          if (response.type === 'remote-already-active') {
            console.log('[RemoteAudioEngine] Remote already active, emitting event for modal')
            const remoteActiveEvent = new CustomEvent('remote-already-active', {
              detail: { 
                message: response.message,
                activeClientsCount: response.activeClientsCount
              }
            })
            window.dispatchEvent(remoteActiveEvent)
            return
          }
          
          // Handle control taken confirmation - user successfully took control from another remote
          if (response.type === 'control-taken') {
            console.log('[RemoteAudioEngine] Control successfully taken, notifying app')
            // Dispatch event to hide splash screen and show main view
            const controlTakenEvent = new CustomEvent('remote-control-taken', {
              detail: { message: response.message }
            })
            window.dispatchEvent(controlTakenEvent)
            return
          }
          
          // Handle control accepted - no other remote was active, proceed normally
          if (response.type === 'remote-control-accepted') {
            console.log('[RemoteAudioEngine] Remote control accepted by server')
            const acceptedEvent = new CustomEvent('remote-control-accepted', {
              detail: { message: response.message }
            })
            window.dispatchEvent(acceptedEvent)
            return
          }
          
          // Handle disconnection from server (local user took control or taken over by another remote)
          if (response.type === 'disconnected') {
            const reason = response.reason || 'unknown'
            let message = response.message || 'Disconnesso dal server'
            
            if (reason === 'taken-over') {
              message = 'Un altro client remoto ha preso il controllo'
            }
            
            console.log('[RemoteAudioEngine] Disconnected by server:', message)
            
            // Clear any pending reconnect timer
            if (this.reconnectTimer) {
              clearTimeout(this.reconnectTimer)
              this.reconnectTimer = null
            }
            
            // Close the WebSocket
            if (this.ws) {
              this.ws.close()
              this.ws = null
            }
            
            // Dispatch event to tell app to return to splash screen
            const disconnectEvent = new CustomEvent('remote-control-disconnected', {
              detail: { message }
            })
            window.dispatchEvent(disconnectEvent)
            
            // Reconnect after a delay (ready for next attempt)
            setTimeout(() => {
              console.log('[RemoteAudioEngine] Reconnecting after forced disconnect...')
              this.connect().catch(err => {
                console.error('[RemoteAudioEngine] Failed to reconnect after disconnect:', err)
              })
            }, 500)
            
            return
          }
          
          // Handle license updates specifically
          if (response.type === 'license') {
            
            // Only dispatch license-updated event if this is a broadcast (not a response to our request)
            // If we have a pending request, let the listener handle it instead
            if (!this.pendingLicenseRequest) {
              // Dispatch custom event for license updates
              const licenseEvent = new CustomEvent('license-updated', {
                detail: {
                  key: response.key,
                  license_type: response.license_type,
                  expires_at: response.expires_at,
                  is_valid: response.is_valid
                }
              })
              window.dispatchEvent(licenseEvent)
            }
          }
          
          // Notify all response listeners
          this.responseListeners.forEach(listener => listener(response))
        } catch (error) {
          console.error('[RemoteAudioEngine] Error parsing message:', error)
        }
      }

      this.ws.onerror = (error) => {
        console.error('[RemoteAudioEngine] WebSocket error:', error)
        this.isConnecting = false
        reject(error)
      }

      this.ws.onclose = () => {
        console.log('[RemoteAudioEngine] Connection closed, will attempt to reconnect...')
        this.isConnecting = false
        this.ws = null
        
        // Auto-reconnect after delay
        if (!this.reconnectTimer) {
          this.reconnectTimer = window.setTimeout(() => {
            this.reconnectTimer = null
            this.connect().catch(err => {
              console.error('[RemoteAudioEngine] Reconnection failed:', err)
            })
          }, this.reconnectInterval)
        }
      }
    })
  }

  disconnect(): void {
    // Flush any pending throttled updates before disconnecting
    if (this.throttleTimer) {
      clearTimeout(this.throttleTimer)
      this.flushPendingUpdates()
    }

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  private send(command: any): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket not connected'))
        return
      }

      try {
        this.ws.send(JSON.stringify(command))
        resolve()
      } catch (error) {
        reject(error)
      }
    })
  }

  // Send command and wait for specific response type
  private sendAndWaitForResponse(command: any, expectedResponseType: string, timeout: number = 5000): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket not connected'))
        return
      }

      const timeoutId = setTimeout(() => {
        // Remove listener
        const index = this.responseListeners.indexOf(listener)
        if (index > -1) {
          this.responseListeners.splice(index, 1)
        }
        console.error(`[RemoteAudioEngine] Timeout waiting for ${expectedResponseType} response after ${timeout}ms`)
        reject(new Error(`Timeout waiting for ${expectedResponseType} response`))
      }, timeout)

      // Set up one-time listener for this response type
      const listener = (response: any) => {
        if (response.type === expectedResponseType) {
          clearTimeout(timeoutId)
          // Remove listener
          const index = this.responseListeners.indexOf(listener)
          if (index > -1) {
            this.responseListeners.splice(index, 1)
          }
          resolve(response)
        }
      }

      this.responseListeners.push(listener)

      try {
        this.ws.send(JSON.stringify(command))
      } catch (error) {
        clearTimeout(timeoutId)
        // Remove listener on send error
        const index = this.responseListeners.indexOf(listener)
        if (index > -1) {
          this.responseListeners.splice(index, 1)
        }
        console.error('[RemoteAudioEngine] Error sending command:', error)
        reject(error)
      }
    })
  }

  // Send IPC-like message and wait for response
  private sendIPC(type: string, payload: any = {}): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket not connected'))
        return
      }

      const messageId = `${type}-${Date.now()}-${Math.random()}`
      const message = { type, id: messageId, ...payload }

      // Set up one-time listener for this specific response
      const listener = (response: any) => {
        if (response.id === messageId) {
          // Remove listener
          const index = this.responseListeners.indexOf(listener)
          if (index > -1) {
            this.responseListeners.splice(index, 1)
          }

          if (response.type === 'ipc:error') {
            reject(new Error(response.error || 'IPC request failed'))
          } else if (response.type === 'ipc:response') {
            resolve(response.data)
          }
        }
      }

      this.responseListeners.push(listener)

      try {
        this.ws.send(JSON.stringify(message))
      } catch (error) {
        // Remove listener on send error
        const index = this.responseListeners.indexOf(listener)
        if (index > -1) {
          this.responseListeners.splice(index, 1)
        }
        reject(error)
      }
    })
  }

  /**
   * Throttled send for continuous controls (volume, pan, etc.)
   * Batches rapid updates and sends them at ~60fps to prevent WebSocket congestion
   */
  private throttleSend(key: string, command: any): Promise<void> {
    // Store the latest command for this key (overwrites previous if exists)
    this.pendingUpdates.set(key, command)

    // If no timer is active, start one
    if (!this.throttleTimer) {
      this.throttleTimer = window.setTimeout(() => {
        this.flushPendingUpdatesInternal()
      }, this.THROTTLE_MS)
    }

    return Promise.resolve()
  }

  /**
   * Manually flush pending throttled updates (called when user releases fader)
   */
  flushPendingUpdates(): void {
    if (this.throttleTimer) {
      clearTimeout(this.throttleTimer)
    }
    this.flushPendingUpdatesInternal()
  }

  private flushPendingUpdatesInternal(): void {
    this.throttleTimer = null

    if (this.pendingUpdates.size === 0) {
      return
    }

    // Send all pending updates
    const updates = Array.from(this.pendingUpdates.values())
    this.pendingUpdates.clear()

    updates.forEach(command => {
      this.send(command).catch(error => {
        console.error('[RemoteAudioEngine] Error sending throttled update:', error)
      })
    })
  }

  // Implement all audioEngine methods
  async start(inputDevice?: string | null, outputDevice?: string | null, sampleRate?: number | null, bufferSize?: number | null): Promise<void> {
    return this.send({ type: 'start', input_device: inputDevice, output_device: outputDevice, sample_rate: sampleRate, buffer_size: bufferSize })
  }

  async stop(): Promise<void> {
    return this.send({ type: 'stop' })
  }

  async setGain(track: number, gain: number): Promise<void> {
    return this.throttleSend(`gain_${track}`, { type: 'set_gain', track, gain })
  }

  async setVolume(track: number, volume: number): Promise<void> {
    return this.throttleSend(`volume_${track}`, { type: 'set_volume', track, volume })
  }

  async setMute(track: number, mute: boolean): Promise<void> {
    return this.send({ type: 'set_mute', track, mute })
  }

  async setRouteToMaster(track: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_route_to_master', track, route })
  }

  async setPan(track: number, pan: number): Promise<void> {
    return this.throttleSend(`pan_${track}`, { type: 'set_pan', track, pan })
  }

  async setTrackPad(track: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_track_pad', track, enabled })
  }

  async setTrackHPF(track: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_track_hpf', track, enabled })
  }

  async setTrackPhaseInvert(track: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_track_phase_invert', track, enabled })
  }

  async setEQ(track: number, low: number, low_mid: number, high_mid: number, high: number): Promise<void> {
    return this.send({ type: 'set_eq', track, low, low_mid, high_mid, high })
  }

  async setEQEnabled(track: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_eq_enabled', track, enabled })
  }

  async setParametricEQFilters(track: number, filters: Array<{type: string, frequency: number, gain: number, q: number}>): Promise<void> {
    // DON'T emit event locally - let the WebSocket broadcast handle it
    return this.send({ type: 'set_parametric_eq_filters', track, filters })
  }

  async setParametricEQEnabled(track: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_parametric_eq_enabled', track, enabled })
  }

  async clearParametricEQ(track: number): Promise<void> {
    // Don't emit events locally - wait for server broadcast
    return this.send({ type: 'clear_parametric_eq', track })
  }

  async setCompressor(track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number): Promise<void> {
    return this.send({ type: 'set_compressor', track, enabled, threshold, ratio, attack, release })
  }

  async setGate(track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number): Promise<void> {
    return this.send({ type: 'set_gate', track, enabled, threshold, range, attack, release })
  }

  async setSignalFrequency(track: number, frequency: number): Promise<void> {
    return this.send({ type: 'set_signal_frequency', track, frequency })
  }

  async setSignalWaveform(track: number, waveform: string): Promise<void> {
    return this.send({ type: 'set_signal_waveform', track, waveform })
  }

  async clearTrackSource(track: number): Promise<void> {
    return this.send({ type: 'clear_track_source', track })
  }

  async setTrackSourceFile(track: number, filePath: string, artist?: string|null, title?: string|null, playlistId?: string|null, playlistName?: string|null, playlistIndex?: number|null): Promise<void> {
    const payload: any = { type: 'set_track_source_file', track, file_path: filePath }
    if (artist !== undefined && artist !== null) payload.artist = artist
    if (title !== undefined && title !== null) payload.title = title
    if (playlistId !== undefined && playlistId !== null) payload.playlist_id = playlistId
    if (playlistName !== undefined && playlistName !== null) payload.playlist_name = playlistName
    if (playlistIndex !== undefined && playlistIndex !== null) payload.playlist_index = playlistIndex
    return this.send(payload)
  }

  // File operations not supported in remote mode
  async saveTempAudioFile(arrayBuffer: ArrayBuffer, fileName: string): Promise<string> {
    throw new Error('File operations not supported in remote mode')
  }

  async deleteTempFile(filePath: string): Promise<void> {
    throw new Error('File operations not supported in remote mode')
  }

  async playFile(track: number, fileId?: string): Promise<void> {
    // Note: fileId is kept for API compatibility but ignored for now
    // Remote clients play already-loaded files, file loading happens via setTrackSourceFile
    return this.send({ type: 'play_file', track, file_path: undefined })
  }

  async pauseFile(track: number): Promise<void> {
    return this.send({ type: 'pause_file', track })
  }

  async stopFile(track: number): Promise<void> {
    return this.send({ type: 'stop_file', track })
  }

  async setMasterGain(gain: number): Promise<void> {
    return this.throttleSend('master_gain', { type: 'set_master_gain', gain })
  }

  async setMasterMute(mute: boolean): Promise<void> {
    return this.send({ type: 'set_master_mute', mute })
  }

  async setMasterParametricEQFilters(filters: Array<{type: string, frequency: number, gain: number, q: number}>): Promise<void> {
    // DON'T emit event locally - let the WebSocket broadcast handle it
    // This prevents loops where we receive our own event back
    
    // Send to backend in backend format (with q)
    return this.send({ type: 'set_master_parametric_eq_filters', filters })
  }

  async setMasterParametricEQEnabled(enabled: boolean): Promise<void> {
    return this.send({ type: 'set_master_parametric_eq_enabled', enabled })
  }

  async clearMasterParametricEQ(): Promise<void> {
    // DON'T emit event locally - let the WebSocket broadcast handle it
    return this.send({ type: 'clear_master_parametric_eq' })
  }

  async setMasterOutputChannels(leftChannel: number, rightChannel: number): Promise<void> {
    return this.send({ type: 'set_master_output_channels', left_channel: leftChannel, right_channel: rightChannel })
  }

  async setSelectedMasterOutput(deviceId: string | null): Promise<void> {
    return this.send({ type: 'set_selected_master_output', device_id: deviceId })
  }

  async setMasterCompressor(enabled: boolean, threshold: number, ratio: number, attack: number, release: number): Promise<void> {
    return this.send({ type: 'set_master_compressor', enabled, threshold, ratio, attack, release })
  }

  async setMasterLimiter(enabled: boolean, ceiling: number, release: number): Promise<void> {
    return this.send({ type: 'set_master_limiter', enabled, ceiling, release })
  }

  async setMasterDelay(enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number): Promise<void> {
    return this.send({ type: 'set_master_delay', enabled, time_l: timeL, time_r: timeR, feedback, mix })
  }

  async setMasterReverb(enabled: boolean, roomSize: number, damping: number, wet: number, width: number): Promise<void> {
    return this.send({ type: 'set_master_reverb', enabled, room_size: roomSize, damping, wet, width })
  }

  async addMasterFxEffect(effectType: string): Promise<void> {
    return this.send({ type: 'add_master_fx_effect', effect_type: effectType })
  }

  async removeMasterFxEffect(effectType: string): Promise<void> {
    return this.send({ type: 'remove_master_fx_effect', effect_type: effectType })
  }

  async addSubgroup(): Promise<number | null> {
    try {
      const response = await this.sendAndWaitForResponse({ type: 'add_subgroup' }, 'subgroup_created', 5000)
      return response.id
    } catch (error) {
      console.error('[RemoteAudioEngine] Failed to add subgroup:', error)
      return null
    }
  }

  async removeSubgroup(subgroup: number): Promise<void> {
    return this.send({ type: 'remove_subgroup', subgroup })
  }

  async setSubgroupGain(subgroup: number, gain: number): Promise<void> {
    return this.throttleSend(`subgroup_gain_${subgroup}`, { type: 'set_subgroup_gain', subgroup, gain })
  }

  async setSubgroupMute(subgroup: number, mute: boolean): Promise<void> {
    return this.send({ type: 'set_subgroup_mute', subgroup, mute })
  }

  async setSubgroupOutputEnabled(subgroup: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_subgroup_output_enabled', subgroup, enabled })
  }

  async setSubgroupRouteToMaster(subgroup: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_subgroup_route_to_master', subgroup, route })
  }

  async setSubgroupOutputChannels(subgroup: number, leftChannel: number, rightChannel: number): Promise<void> {
    return this.send({ type: 'set_subgroup_output_channels', subgroup, left_channel: leftChannel, right_channel: rightChannel })
  }

  async setSelectedSubgroupOutput(subgroup: number, deviceId: string | null): Promise<void> {
    return this.send({ type: 'set_selected_subgroup_output', subgroup, device_id: deviceId })
  }

  async setTrackRouteToSubgroup(track: number, subgroup: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_track_route_to_subgroup', track, subgroup, route })
  }

  async setTrackAuxSend(track: number, aux: number, level: number, preFader: boolean, muted: boolean): Promise<void> {
    return this.throttleSend(`aux_send_${track}_${aux}`, { type: 'set_track_aux_send', track, aux, level, pre_fader: preFader, muted })
  }

  async setAuxBusGain(aux: number, gain: number): Promise<void> {
    return this.throttleSend(`aux_gain_${aux}`, { type: 'set_aux_bus_gain', aux, gain })
  }

  async setAuxBusMute(aux: number, mute: boolean): Promise<void> {
    return this.send({ type: 'set_aux_bus_mute', aux, mute })
  }

  async setAuxBusReverb(aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number): Promise<void> {
    return this.send({ type: 'set_aux_bus_reverb', aux, enabled, room_size: roomSize, damping, wet, width })
  }

  async setAuxBusDelay(aux: number, enabled: boolean, time: number, feedback: number, mix: number): Promise<void> {
    return this.send({ type: 'set_aux_bus_delay', aux, enabled, time, feedback, mix })
  }

  async setAuxBusRouteToMaster(aux: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_aux_bus_route_to_master', aux, route })
  }

  async setAuxBusOutputEnabled(aux: number, enabled: boolean): Promise<void> {
    return this.send({ type: 'set_aux_bus_output_enabled', aux, enabled })
  }

  async setAuxBusOutputChannels(aux: number, leftChannel: number, rightChannel: number): Promise<void> {
    return this.send({ type: 'set_aux_bus_output_channels', aux, left_channel: leftChannel, right_channel: rightChannel })
  }

  async setAuxBusRouteToSubgroup(aux: number, subgroup: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_aux_bus_route_to_subgroup', aux, subgroup, route })
  }

  async setTrackSourceAuxReturn(track: number, aux: number): Promise<void> {
    return this.send({ type: 'set_track_source_aux_return', track, aux })
  }

  async listDevices(): Promise<any[]> {
    // Devices only available on host
    return []
  }

  async listAudioInputs(): Promise<any[]> {
    // Audio inputs only available on host
    return []
  }

  async startNdi(streamName: string, source: string): Promise<void> {
    return this.send({ type: 'start_ndi', stream_name: streamName, source })
  }

  async stopNdi(): Promise<void> {
    return this.send({ type: 'stop_ndi' })
  }

  async setNdiSource(source: string): Promise<void> {
    return this.send({ type: 'set_ndi_source', source })
  }

  async setNdiName(name: string): Promise<void> {
    return this.send({ type: 'set_ndi_name', name })
  }

  async setNdiVideoText(text: string): Promise<void> {
    return this.send({ type: 'set_ndi_video_text', text })
  }

  async getLoudness(): Promise<any> {
    // Return empty/default values - real data comes via response listeners
    return null
  }

  async resetLoudness(): Promise<void> {
    return this.send({ type: 'reset_loudness' })
  }

  async getDynamicRange(): Promise<any> {
    return null
  }

  async resetDynamicRange(): Promise<void> {
    return this.send({ type: 'reset_dynamic_range' })
  }

  async getPhaseCorrelation(): Promise<any> {
    return null
  }

  async resetPhaseCorrelation(): Promise<void> {
    return this.send({ type: 'reset_phase_correlation' })
  }

  async getStereoWidth(): Promise<any> {
    return null
  }

  async resetStereoWidth(): Promise<void> {
    return this.send({ type: 'reset_stereo_width' })
  }

  async getHeadroom(): Promise<any> {
    return null
  }

  async resetHeadroom(): Promise<void> {
    return this.send({ type: 'reset_headroom' })
  }

  async enableMasterTap(filePath: string, settings: any): Promise<void> {
    throw new Error('Recording not supported in remote mode')
  }

  async disableMasterTap(): Promise<void> {
    throw new Error('Recording not supported in remote mode')
  }

  async generateRecordingPath(): Promise<string> {
    throw new Error('Recording not supported in remote mode')
  }

  async getRecordingFileInfo(filePath: string): Promise<any> {
    throw new Error('Recording not supported in remote mode')
  }

  async showRecordingInFolder(filePath: string): Promise<void> {
    throw new Error('Recording not supported in remote mode')
  }

  async deleteRecordingFile(filePath: string): Promise<void> {
    throw new Error('Recording not supported in remote mode')
  }

  async listRecordings(): Promise<any[]> {
    return []
  }

  async saveLibraryFile(arrayBuffer: ArrayBuffer, fileName: string, metadata?: any): Promise<string> {
    // Convert ArrayBuffer to Array for JSON serialization
    const arrayData = Array.from(new Uint8Array(arrayBuffer))
    return this.sendIPC('ipc:audio-engine:save-library-file', {
      arrayBuffer: arrayData,
      fileName,
      metadata
    })
  }

  async listLibraryFiles(): Promise<any[]> {
    try {
      return await this.sendIPC('ipc:audio-engine:list-library-files')
    } catch (error) {
      console.error('[RemoteAudioEngine] Error listing library files:', error)
      return []
    }
  }

  async getLibraryFile(fileId: string): Promise<any> {
    const result = await this.sendIPC('ipc:audio-engine:get-library-file', { fileId })
    // Convert array back to ArrayBuffer if present
    if (result.arrayBuffer && Array.isArray(result.arrayBuffer)) {
      result.arrayBuffer = new Uint8Array(result.arrayBuffer).buffer
    }
    return result
  }

  async deleteLibraryFile(fileId: string): Promise<void> {
    await this.sendIPC('ipc:audio-engine:delete-library-file', { fileId })
  }

  async savePlaylist(playlist: any): Promise<void> {
    await this.sendIPC('ipc:audio-engine:save-playlist', { playlist })
  }

  async listPlaylists(): Promise<any[]> {
    try {
      return await this.sendIPC('ipc:audio-engine:list-playlists')
    } catch (error) {
      console.error('[RemoteAudioEngine] Error listing playlists:', error)
      return []
    }
  }

  async getPlaylist(playlistId: string): Promise<any> {
    try {
      return await this.sendIPC('ipc:audio-engine:get-playlist', { playlistId })
    } catch (error) {
      console.error('[RemoteAudioEngine] Error getting playlist:', error)
      return null
    }
  }

  async deletePlaylist(playlistId: string): Promise<void> {
    await this.sendIPC('ipc:audio-engine:delete-playlist', { playlistId })
  }

  async saveScene(scene: any): Promise<void> {
    throw new Error('Scene operations not supported in remote mode')
  }

  async listScenes(): Promise<any[]> {
    return []
  }

  async getScene(sceneId: string): Promise<any> {
    throw new Error('Scene operations not supported in remote mode')
  }

  async deleteScene(sceneId: string): Promise<void> {
    throw new Error('Scene operations not supported in remote mode')
  }

  onResponse(callback: (response: any) => void): void {
    this.responseListeners.push(callback)
  }

  async showOpenFileDialog(): Promise<string[]> {
    throw new Error('File dialog not supported in remote mode')
  }

  async readFileAsBuffer(filePath: string): Promise<ArrayBuffer> {
    throw new Error('File operations not supported in remote mode')
  }

  // License management
  async getLicense(): Promise<any> {
    
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.error('[RemoteAudioEngine] WebSocket not connected!')
      return {
        key: 'DEMO',
        license_type: 'demo',
        expires_at: null,
        is_valid: true
      }
    }
    
    try {
      // Mark that we're waiting for a license response (don't broadcast this one)
      this.pendingLicenseRequest = true
      
      const response = await this.sendAndWaitForResponse({ type: 'get_license' }, 'license', 5000)
      
      // Clear the pending flag after receiving response
      this.pendingLicenseRequest = false
      
      return response
    } catch (error) {
      console.error('[RemoteAudioEngine] Failed to get license (timeout or error):', error)
      
      // Clear the pending flag on error
      this.pendingLicenseRequest = false
      
      // Return demo license on error
      return {
        key: 'DEMO',
        license_type: 'demo',
        expires_at: null,
        is_valid: true
      }
    }
  }

  async saveLicense(key: string, licenseType: string, expiresAt: string | null): Promise<boolean> {
    console.log('[RemoteAudioEngine] Saving license:', key, licenseType)
    try {
      await this.sendAndWaitForResponse(
        { 
          type: 'save_license', 
          key, 
          license_type: licenseType, 
          expires_at: expiresAt 
        }, 
        'ok', 
        3000
      )
      console.log('[RemoteAudioEngine] License saved successfully')
      return true
    } catch (error) {
      console.error('[RemoteAudioEngine] Failed to save license:', error)
      return false
    }
  }
  
  // Audio configuration management
  async getAudioConfig(): Promise<any> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.error('[RemoteAudioEngine] WebSocket not connected!')
      return { sample_rate: 0, buffer_size: 256 } // Default: Auto, 256 frames
    }
    
    try {
      const response = await this.sendAndWaitForResponse({ type: 'get_audio_config' }, 'audio_config', 5000)
      return response
    } catch (error) {
      console.error('[RemoteAudioEngine] Failed to get audio config (timeout or error):', error)
      return { sample_rate: 0, buffer_size: 256 } // Default: Auto, 256 frames
    }
  }

  async saveAudioConfig(sampleRate: number, bufferSize: number): Promise<boolean> {
    console.log('[RemoteAudioEngine] Saving audio config:', sampleRate, bufferSize)
    try {
      await this.sendAndWaitForResponse(
        { 
          type: 'save_audio_config', 
          sample_rate: sampleRate, 
          buffer_size: bufferSize 
        }, 
        'ok', 
        3000
      )
      console.log('[RemoteAudioEngine] Audio config saved successfully')
      return true
    } catch (error) {
      console.error('[RemoteAudioEngine] Failed to save audio config:', error)
      return false
    }
  }
  
  // Remote control lifecycle methods
  notifyRemoteControlStarted(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'remote-control-started' }))
      console.log('[RemoteAudioEngine] Notified server: remote control started')
    }
  }
  
  notifyRemoteControlStopped(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'remote-control-stopped' }))
      console.log('[RemoteAudioEngine] Notified server: remote control stopped')
    }
  }

  forceTakeControl(): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'force-take-control' }))
      console.log('[RemoteAudioEngine] Sent force-take-control to server')
    }
  }

  cancelConnection(): void {
    console.log('[RemoteAudioEngine] User cancelled connection, closing WebSocket')
    
    // Clear any pending reconnect timer
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    
    // Close current connection
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    
    // Dispatch event to return to splash screen
    const disconnectEvent = new CustomEvent('remote-control-disconnected', {
      detail: { message: 'Connessione annullata' }
    })
    window.dispatchEvent(disconnectEvent)
    
    // Reconnect after a short delay (ready for next attempt)
    setTimeout(() => {
      console.log('[RemoteAudioEngine] Reconnecting after cancel...')
      this.connect().catch(err => {
        console.error('[RemoteAudioEngine] Failed to reconnect after cancel:', err)
      })
    }, 500)
  }
}
