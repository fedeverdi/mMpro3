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
  
  // Local state cache for UI synchronization
  private masterEqFilters: any[] = []
  private trackEqFilters: Map<number, any[]> = new Map()

  constructor(host: string, port: number = 3001) {
    this.url = `ws://${host}:${port}`
    console.log('[RemoteAudioEngine] Initializing with URL:', this.url)
  }

  async connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return
    }

    return new Promise((resolve, reject) => {
      this.isConnecting = true
      console.log('[RemoteAudioEngine] Connecting to:', this.url)

      this.ws = new WebSocket(this.url)

      this.ws.onopen = () => {
        console.log('[RemoteAudioEngine] Connected successfully')
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

  // Implement all audioEngine methods
  async start(inputDevice?: string | null, outputDevice?: string | null, sampleRate?: number | null, bufferSize?: number | null): Promise<void> {
    return this.send({ type: 'start', input_device: inputDevice, output_device: outputDevice, sample_rate: sampleRate, buffer_size: bufferSize })
  }

  async stop(): Promise<void> {
    return this.send({ type: 'stop' })
  }

  async setGain(track: number, gain: number): Promise<void> {
    return this.send({ type: 'set_gain', track, gain })
  }

  async setVolume(track: number, volume: number): Promise<void> {
    return this.send({ type: 'set_volume', track, volume })
  }

  async setMute(track: number, mute: boolean): Promise<void> {
    return this.send({ type: 'set_mute', track, mute })
  }

  async setRouteToMaster(track: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_route_to_master', track, route })
  }

  async setPan(track: number, pan: number): Promise<void> {
    return this.send({ type: 'set_pan', track, pan })
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

  async setTrackSourceFile(track: number, filePath: string, artist?: string|null, title?: string|null): Promise<void> {
    const payload: any = { type: 'set_track_source_file', track, file_path: filePath }
    if (artist !== undefined && artist !== null) payload.artist = artist
    if (title !== undefined && title !== null) payload.title = title
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
    return this.send({ type: 'set_master_gain', gain })
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

  async addSubgroup(): Promise<void> {
    return this.send({ type: 'add_subgroup' })
  }

  async removeSubgroup(subgroup: number): Promise<void> {
    return this.send({ type: 'remove_subgroup', subgroup })
  }

  async setSubgroupGain(subgroup: number, gain: number): Promise<void> {
    return this.send({ type: 'set_subgroup_gain', subgroup, gain })
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

  async setTrackRouteToSubgroup(track: number, subgroup: number, route: boolean): Promise<void> {
    return this.send({ type: 'set_track_route_to_subgroup', track, subgroup, route })
  }

  async setTrackAuxSend(track: number, aux: number, level: number, preFader: boolean, muted: boolean): Promise<void> {
    return this.send({ type: 'set_track_aux_send', track, aux, level, pre_fader: preFader, muted })
  }

  async setAuxBusGain(aux: number, gain: number): Promise<void> {
    return this.send({ type: 'set_aux_bus_gain', aux, gain })
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
    throw new Error('Playlist operations not supported in remote mode')
  }

  async listPlaylists(): Promise<any[]> {
    return []
  }

  async getPlaylist(playlistId: string): Promise<any> {
    throw new Error('Playlist operations not supported in remote mode')
  }

  async deletePlaylist(playlistId: string): Promise<void> {
    throw new Error('Playlist operations not supported in remote mode')
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
}
