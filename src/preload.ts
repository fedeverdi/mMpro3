// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge, ipcRenderer } from 'electron'

// Audio Engine API
contextBridge.exposeInMainWorld('audioEngine', {
  start: (inputDevice?: string, outputDevice?: string, sampleRate?: number, bufferSize?: number) => ipcRenderer.invoke('audio-engine:start', inputDevice, outputDevice, sampleRate, bufferSize),
  stop: () => ipcRenderer.invoke('audio-engine:stop'),
  
  // Track controls
  setGain: (track: number, gain: number) => ipcRenderer.invoke('audio-engine:set-gain', track, gain),
  setVolume: (track: number, volume: number) => ipcRenderer.invoke('audio-engine:set-volume', track, volume),
  setMute: (track: number, mute: boolean) => ipcRenderer.invoke('audio-engine:set-mute', track, mute),
  setRouteToMaster: (track: number, route: boolean) => ipcRenderer.invoke('audio-engine:set-route-to-master', track, route),
  setPan: (track: number, pan: number) => ipcRenderer.invoke('audio-engine:set-pan', track, pan),
  setTrackPad: (track: number, enabled: boolean) => ipcRenderer.invoke('audio-engine:set-track-pad', track, enabled),
  setTrackHPF: (track: number, enabled: boolean) => ipcRenderer.invoke('audio-engine:set-track-hpf', track, enabled),
  setTrackPhaseInvert: (track: number, enabled: boolean) => ipcRenderer.invoke('audio-engine:set-track-phase-invert', track, enabled),
  setEQ: (track: number, low: number, low_mid: number, high_mid: number, high: number) => 
    ipcRenderer.invoke('audio-engine:set-eq', track, low, low_mid, high_mid, high),
  setEQEnabled: (track: number, enabled: boolean) => ipcRenderer.invoke('audio-engine:set-eq-enabled', track, enabled),
  
  // Parametric EQ controls
  setParametricEQFilters: (track: number, filters: Array<{type: string, frequency: number, gain: number, q: number}>) => 
    ipcRenderer.invoke('audio-engine:set-parametric-eq-filters', track, filters),
  setParametricEQEnabled: (track: number, enabled: boolean) => 
    ipcRenderer.invoke('audio-engine:set-parametric-eq-enabled', track, enabled),
  clearParametricEQ: (track: number) => 
    ipcRenderer.invoke('audio-engine:clear-parametric-eq', track),
    
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => 
    ipcRenderer.invoke('audio-engine:set-compressor', track, enabled, threshold, ratio, attack, release),
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => 
    ipcRenderer.invoke('audio-engine:set-gate', track, enabled, threshold, range, attack, release),
  
  // Track source selection
  setTrackSourceInput: (track: number, leftChannel: number, rightChannel: number, deviceName?: string | null) => 
    ipcRenderer.invoke('audio-engine:set-track-source-input', track, leftChannel, rightChannel, deviceName),
  setTrackSourceSignal: (track: number, waveform: string, frequency: number) => 
    ipcRenderer.invoke('audio-engine:set-track-source-signal', track, waveform, frequency),
  setSignalFrequency: (track: number, frequency: number) => 
    ipcRenderer.invoke('audio-engine:set-signal-frequency', track, frequency),
  setSignalWaveform: (track: number, waveform: string) => 
    ipcRenderer.invoke('audio-engine:set-signal-waveform', track, waveform),
  clearTrackSource: (track: number) => 
    ipcRenderer.invoke('audio-engine:clear-track-source', track),
  setTrackSourceFile: (track: number, filePath: string, artist?: string|null, title?: string|null) => 
    ipcRenderer.invoke('audio-engine:set-track-source-file', track, filePath, artist, title),
  saveTempAudioFile: (arrayBuffer: ArrayBuffer, fileName: string) => 
    ipcRenderer.invoke('audio-engine:save-temp-audio-file', arrayBuffer, fileName),
  deleteTempFile: (filePath: string) => 
    ipcRenderer.invoke('audio-engine:delete-temp-file', filePath),
  
  // File playback controls
  playFile: (track: number, fileId?: string) => ipcRenderer.invoke('audio-engine:play-file', track, fileId),
  pauseFile: (track: number) => ipcRenderer.invoke('audio-engine:pause-file', track),
  stopFile: (track: number) => ipcRenderer.invoke('audio-engine:stop-file', track),
  
  // Master controls
  setMasterGain: (gain: number) => ipcRenderer.invoke('audio-engine:set-master-gain', gain),
  setMasterGainLeft: (gain: number) => ipcRenderer.invoke('audio-engine:set-master-gain-left', gain),
  setMasterGainRight: (gain: number) => ipcRenderer.invoke('audio-engine:set-master-gain-right', gain),
  setMasterMute: (mute: boolean) => ipcRenderer.invoke('audio-engine:set-master-mute', mute),
  setMasterLinked: (linked: boolean) => ipcRenderer.invoke('audio-engine:set-master-linked', linked),
  setMasterParametricEQFilters: (filters: Array<{type: string, frequency: number, gain: number, q: number}>) => 
    ipcRenderer.invoke('audio-engine:set-master-parametric-eq-filters', filters),
  setMasterParametricEQEnabled: (enabled: boolean) => 
    ipcRenderer.invoke('audio-engine:set-master-parametric-eq-enabled', enabled),
  clearMasterParametricEQ: () => ipcRenderer.invoke('audio-engine:clear-master-parametric-eq'),
  setMasterOutputChannels: (leftChannel: number, rightChannel: number) => 
    ipcRenderer.invoke('audio-engine:set-master-output-channels', leftChannel, rightChannel),
  setSelectedMasterOutput: (deviceId: string | null) =>
    ipcRenderer.invoke('audio-engine:set-selected-master-output', deviceId),
  
  // Master FX controls
  setMasterCompressor: (enabled: boolean, threshold: number, ratio: number, attack: number, release: number) =>
    ipcRenderer.invoke('audio-engine:set-master-compressor', enabled, threshold, ratio, attack, release),
  setMasterLimiter: (enabled: boolean, ceiling: number, release: number) =>
    ipcRenderer.invoke('audio-engine:set-master-limiter', enabled, ceiling, release),
  setMasterDelay: (enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number) =>
    ipcRenderer.invoke('audio-engine:set-master-delay', enabled, timeL, timeR, feedback, mix),
  setMasterReverb: (enabled: boolean, roomSize: number, damping: number, wet: number, width: number) =>
    ipcRenderer.invoke('audio-engine:set-master-reverb', enabled, roomSize, damping, wet, width),
  
  // Master FX management
  addMasterFxEffect: (effectType: string) =>
    ipcRenderer.invoke('audio-engine:add-master-fx-effect', effectType),
  removeMasterFxEffect: (effectType: string) =>
    ipcRenderer.invoke('audio-engine:remove-master-fx-effect', effectType),
  
  // Subgroup controls
  addSubgroup: () => ipcRenderer.invoke('audio-engine:add-subgroup'),
  removeSubgroup: (subgroup: number) => ipcRenderer.invoke('audio-engine:remove-subgroup', subgroup),
  setSubgroupGain: (subgroup: number, gain: number) => ipcRenderer.invoke('audio-engine:set-subgroup-gain', subgroup, gain),
  setSubgroupMute: (subgroup: number, mute: boolean) => ipcRenderer.invoke('audio-engine:set-subgroup-mute', subgroup, mute),
  setSubgroupOutputEnabled: (subgroup: number, enabled: boolean) => 
    ipcRenderer.invoke('audio-engine:set-subgroup-output-enabled', subgroup, enabled),
  setSubgroupRouteToMaster: (subgroup: number, route: boolean) => 
    ipcRenderer.invoke('audio-engine:set-subgroup-route-to-master', subgroup, route),
  setSubgroupOutputChannels: (subgroup: number, leftChannel: number, rightChannel: number) => 
    ipcRenderer.invoke('audio-engine:set-subgroup-output-channels', subgroup, leftChannel, rightChannel),
  setSelectedSubgroupOutput: (subgroup: number, deviceId: string | null) =>
    ipcRenderer.invoke('audio-engine:set-selected-subgroup-output', subgroup, deviceId),
  setTrackRouteToSubgroup: (track: number, subgroup: number, route: boolean) => 
    ipcRenderer.invoke('audio-engine:set-track-route-to-subgroup', track, subgroup, route),
  
  // Aux bus methods
  setTrackAuxSend: (track: number, aux: number, level: number, preFader: boolean, muted: boolean) =>
    ipcRenderer.invoke('audio-engine:set-track-aux-send', track, aux, level, preFader, muted),
  setAuxBusGain: (aux: number, gain: number) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-gain', aux, gain),
  setAuxBusMute: (aux: number, mute: boolean) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-mute', aux, mute),
  setAuxBusReverb: (aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-reverb', aux, enabled, roomSize, damping, wet, width),
  setAuxBusDelay: (aux: number, enabled: boolean, time: number, feedback: number, mix: number) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-delay', aux, enabled, time, feedback, mix),
  setAuxBusRouteToMaster: (aux: number, route: boolean) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-route-to-master', aux, route),
  setAuxBusOutputEnabled: (aux: number, enabled: boolean) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-output-enabled', aux, enabled),
  setAuxBusOutputChannels: (aux: number, leftChannel: number, rightChannel: number) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-output-channels', aux, leftChannel, rightChannel),
  setAuxBusRouteToSubgroup: (aux: number, subgroup: number, route: boolean) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-route-to-subgroup', aux, subgroup, route),
  setAuxBusSelectedOutput: (aux: number, deviceId: string | null) =>
    ipcRenderer.invoke('audio-engine:set-aux-bus-selected-output', aux, deviceId),
  setTrackSourceAuxReturn: (track: number, aux: number) =>
    ipcRenderer.invoke('audio-engine:set-track-source-aux-return', track, aux),
  
  // Device management
  listDevices: () => ipcRenderer.invoke('audio-engine:list-devices'),
  listAudioInputs: () => ipcRenderer.invoke('audio-engine:list-audio-inputs'),
  
  // NDI Streaming
  startNdi: (streamName: string, source: string) => ipcRenderer.invoke('audio-engine:start-ndi', streamName, source),
  stopNdi: () => ipcRenderer.invoke('audio-engine:stop-ndi'),
  setNdiSource: (source: string) => ipcRenderer.invoke('audio-engine:set-ndi-source', source),
  setNdiName: (name: string) => ipcRenderer.invoke('audio-engine:set-ndi-name', name),
  setNdiVideoText: (text: string) => ipcRenderer.invoke('audio-engine:set-ndi-video-text', text),
  
  // Loudness Metering (EBU R128)
  getLoudness: () => ipcRenderer.invoke('audio-engine:get-loudness'),
  resetLoudness: () => ipcRenderer.invoke('audio-engine:reset-loudness'),
  
  // Dynamic Range Metering
  getDynamicRange: () => ipcRenderer.invoke('audio-engine:get-dynamic-range'),
  resetDynamicRange: () => ipcRenderer.invoke('audio-engine:reset-dynamic-range'),
  
  // Phase Correlation Metering (Master)
  getPhaseCorrelation: () => ipcRenderer.invoke('audio-engine:get-phase-correlation'),
  resetPhaseCorrelation: () => ipcRenderer.invoke('audio-engine:reset-phase-correlation'),
  
  // Stereo Width Metering (Master)
  getStereoWidth: () => ipcRenderer.invoke('audio-engine:get-stereo-width'),
  resetStereoWidth: () => ipcRenderer.invoke('audio-engine:reset-stereo-width'),
  
  // Headroom Metering (Master)
  getHeadroom: () => ipcRenderer.invoke('audio-engine:get-headroom'),
  resetHeadroom: () => ipcRenderer.invoke('audio-engine:reset-headroom'),
  
  // Master Tap (Recording) - Rust saves WAV file directly
  enableMasterTap: (filePath: string, settings: {
    format: 'wav' | 'mp3' | 'opus'
    sampleRate: number
    bitDepth: number
    bitrate: number
  }) => ipcRenderer.invoke('audio-engine:enable-master-tap', filePath, settings),
  disableMasterTap: () => ipcRenderer.invoke('audio-engine:disable-master-tap'),
  generateRecordingPath: () => ipcRenderer.invoke('audio-engine:generate-recording-path'),
  getRecordingFileInfo: (filePath: string) => ipcRenderer.invoke('audio-engine:get-recording-file-info', filePath),
  showRecordingInFolder: (filePath: string) => ipcRenderer.invoke('audio-engine:show-recording-in-folder', filePath),
  deleteRecordingFile: (filePath: string) => ipcRenderer.invoke('audio-engine:delete-recording-file', filePath),
  listRecordings: () => ipcRenderer.invoke('audio-engine:list-recordings'),
  
  // License management
  saveLicense: (key: string, licenseType: string, expiresAt: string | null) =>
    ipcRenderer.invoke('audio-engine:save-license', key, licenseType, expiresAt),
  getLicense: () => ipcRenderer.invoke('audio-engine:get-license'),
  
  // Library files
  saveLibraryFile: (arrayBuffer: ArrayBuffer, fileName: string, metadata?: any) => 
    ipcRenderer.invoke('audio-engine:save-library-file', arrayBuffer, fileName, metadata),
  listLibraryFiles: () => ipcRenderer.invoke('audio-engine:list-library-files'),
  getLibraryFile: (fileId: string) => ipcRenderer.invoke('audio-engine:get-library-file', fileId),
  deleteLibraryFile: (fileId: string) => ipcRenderer.invoke('audio-engine:delete-library-file', fileId),
  
  // Playlists
  savePlaylist: (playlist: any) => ipcRenderer.invoke('audio-engine:save-playlist', playlist),
  listPlaylists: () => ipcRenderer.invoke('audio-engine:list-playlists'),
  getPlaylist: (playlistId: string) => ipcRenderer.invoke('audio-engine:get-playlist', playlistId),
  deletePlaylist: (playlistId: string) => ipcRenderer.invoke('audio-engine:delete-playlist', playlistId),
  
  // Scenes
  saveScene: (scene: any) => ipcRenderer.invoke('audio-engine:save-scene', scene),
  listScenes: () => ipcRenderer.invoke('audio-engine:list-scenes'),
  getScene: (sceneId: string) => ipcRenderer.invoke('audio-engine:get-scene', sceneId),
  deleteScene: (sceneId: string) => ipcRenderer.invoke('audio-engine:delete-scene', sceneId),
  
  // Response listener
  onResponse: (callback: (response: any) => void) => {
    ipcRenderer.on('audio-engine-response', (_, data) => callback(data))
  },
  
  // File dialog
  showOpenFileDialog: () => ipcRenderer.invoke('show-open-file-dialog'),
  readFileAsBuffer: (filePath: string) => ipcRenderer.invoke('read-file-as-buffer', filePath)
})

// Electron API for window controls
contextBridge.exposeInMainWorld('electronAPI', {
  // Platform detection
  getPlatform: () => ipcRenderer.invoke('get-platform'),
  getAppVersion: () => ipcRenderer.invoke('get-app-version'),
  
  // License verification
  verifyLicense: (licenseKey: string) => ipcRenderer.invoke('verify-license', licenseKey),
  
  // Network URL
  getLocalIp: () => ipcRenderer.invoke('get-local-ip'),
  
  // Window controls
  minimizeWindow: () => ipcRenderer.send('window-minimize'),
  maximizeWindow: () => ipcRenderer.send('window-maximize'),
  unmaximizeWindow: () => ipcRenderer.send('window-unmaximize'),
  closeWindow: () => ipcRenderer.send('window-close'),
  isMaximized: () => ipcRenderer.invoke('window-is-maximized'),
  
  // Window state listeners
  onMaximized: (callback: () => void) => {
    ipcRenderer.on('window-maximized', callback)
  },
  onUnmaximized: (callback: () => void) => {
    ipcRenderer.on('window-unmaximized', callback)
  },
  
  // Remote control state listener
  onRemoteControlState: (callback: (data: { active: boolean, clientsCount: number }) => void) => {
    ipcRenderer.on('remote-control-state', (_event, data) => callback(data))
  },
  
  // Disconnect remote clients and take control
  disconnectRemoteClients: () => ipcRenderer.invoke('disconnect-remote-clients'),
  
  // Detached windows (pop-out components)
  openDetachedWindow: (componentType: string) => ipcRenderer.invoke('open-detached-window', componentType),
  closeDetachedWindow: (componentType: string) => ipcRenderer.invoke('close-detached-window', componentType),
  isDetachedWindowOpen: (componentType: string) => ipcRenderer.invoke('is-detached-window-open', componentType),
  
  // Update state for detached windows
  updateAuxBusesState: (auxBuses: any[]) => ipcRenderer.invoke('update-aux-buses-state', auxBuses)
})