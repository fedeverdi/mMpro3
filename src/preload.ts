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
  setTrackSourceFile: (track: number, filePath: string) => 
    ipcRenderer.invoke('audio-engine:set-track-source-file', track, filePath),
  saveTempAudioFile: (arrayBuffer: ArrayBuffer, fileName: string) => 
    ipcRenderer.invoke('audio-engine:save-temp-audio-file', arrayBuffer, fileName),
  
  // File playback controls
  playFile: (track: number, fileId?: string) => ipcRenderer.invoke('audio-engine:play-file', track, fileId),
  pauseFile: (track: number) => ipcRenderer.invoke('audio-engine:pause-file', track),
  stopFile: (track: number) => ipcRenderer.invoke('audio-engine:stop-file', track),
  
  // Master controls
  setMasterGain: (gain: number) => ipcRenderer.invoke('audio-engine:set-master-gain', gain),
  setMasterMute: (mute: boolean) => ipcRenderer.invoke('audio-engine:set-master-mute', mute),
  setMasterParametricEQFilters: (filters: Array<{type: string, frequency: number, gain: number, q: number}>) => 
    ipcRenderer.invoke('audio-engine:set-master-parametric-eq-filters', filters),
  setMasterParametricEQEnabled: (enabled: boolean) => 
    ipcRenderer.invoke('audio-engine:set-master-parametric-eq-enabled', enabled),
  clearMasterParametricEQ: () => ipcRenderer.invoke('audio-engine:clear-master-parametric-eq'),
  setMasterOutputChannels: (leftChannel: number, rightChannel: number) => 
    ipcRenderer.invoke('audio-engine:set-master-output-channels', leftChannel, rightChannel),
  
  // Master FX controls
  setMasterCompressor: (enabled: boolean, threshold: number, ratio: number, attack: number, release: number) =>
    ipcRenderer.invoke('audio-engine:set-master-compressor', enabled, threshold, ratio, attack, release),
  setMasterLimiter: (enabled: boolean, ceiling: number, release: number) =>
    ipcRenderer.invoke('audio-engine:set-master-limiter', enabled, ceiling, release),
  setMasterDelay: (enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number) =>
    ipcRenderer.invoke('audio-engine:set-master-delay', enabled, timeL, timeR, feedback, mix),
  setMasterReverb: (enabled: boolean, roomSize: number, damping: number, wet: number, width: number) =>
    ipcRenderer.invoke('audio-engine:set-master-reverb', enabled, roomSize, damping, wet, width),
  
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
  }
})