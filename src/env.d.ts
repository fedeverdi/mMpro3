/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string
declare const MAIN_WINDOW_VITE_NAME: string

declare module 'electron-squirrel-startup'

// Rust Audio Device
interface RustAudioDevice {
  id: string
  name: string
  input_channels: number
  output_channels: number
  default_sample_rate: number
}

// Audio Engine API
interface AudioEngine {
  start: (inputDevice?: string | null, outputDevice?: string | null, sampleRate?: number | null, bufferSize?: number | null) => Promise<void>
  stop: () => Promise<void>
  setGain: (track: number, gain: number) => Promise<void>
  setVolume: (track: number, volume: number) => Promise<void>
  setMute: (track: number, mute: boolean) => Promise<void>
  setSolo: (track: number, solo: boolean) => Promise<void>
  setRouteToMaster: (track: number, route: boolean) => Promise<void>
  setEQ: (track: number, low: number, low_mid: number, high_mid: number, high: number) => Promise<void>
  setEQEnabled: (track: number, enabled: boolean) => Promise<void>
  setParametricEQFilters: (track: number, filters: Array<{type: string, frequency: number, gain: number, q: number}>) => Promise<void>
  setParametricEQEnabled: (track: number, enabled: boolean) => Promise<void>
  clearParametricEQ: (track: number) => Promise<void>
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => Promise<void>
  
  // Track Insert Effects Chain
  addTrackInsert: (track: number, effectType: string, position?: number) => Promise<void>
  removeTrackInsert: (track: number, insertId: number) => Promise<void>
  moveTrackInsert: (track: number, insertId: number, newPosition: number) => Promise<void>
  setTrackInsertEnabled: (track: number, insertId: number, enabled: boolean) => Promise<void>
  setTrackInsertCompressor: (track: number, insertId: number, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setTrackInsertGate: (track: number, insertId: number, threshold: number, range: number, attack: number, release: number) => Promise<void>
  setTrackInsertReverb: (track: number, insertId: number, roomSize: number, damping: number, wet: number, width: number, preDelay: number) => Promise<void>
  setTrackInsertDelay: (track: number, insertId: number, timeL: number, timeR: number, feedback: number, mix: number) => Promise<void>
  setTrackInsertExciter: (track: number, insertId: number, amount: number, frequency: number, mix: number) => Promise<void>
  setTrackInsertDeEsser: (track: number, insertId: number, threshold: number, frequency: number, range: number) => Promise<void>
  setTrackInsertChorus: (track: number, insertId: number, rate: number, depth: number, mix: number) => Promise<void>
  
  listDevices: () => Promise<RustAudioDevice[]>
  listAudioInputs: () => Promise<RustAudioDevice[]>
  onResponse: (callback: (response: any) => void) => void
  
  // Master Tap (Recording) - Rust generates path and saves WAV file automatically
  enableMasterTap: (settings: {
    format: 'wav' | 'mp3' | 'opus'
    sampleRate: number
    bitDepth: number
    bitrate: number
  }) => Promise<void>
  disableMasterTap: () => Promise<void>
  getRecordingFileInfo: (filePath: string) => Promise<{ name: string, size: string }>
  showRecordingInFolder: (filePath: string) => Promise<void>
  deleteRecordingFile: (filePath: string) => Promise<void>
  listRecordings: () => Promise<Array<{ id: string, name: string, path: string, size: string, created: string }>>
  
  // Track Source Selection
  setTrackSourceInput: (trackIndex: number, leftChannel: number, rightChannel: number, deviceName?: string | null) => Promise<void>
  setTrackSourceSignal: (trackIndex: number, waveform: string, frequency: number) => Promise<void>
  setSignalFrequency: (trackIndex: number, frequency: number) => Promise<void>
  setSignalWaveform: (trackIndex: number, waveform: string) => Promise<void>
  clearTrackSource: (trackIndex: number) => Promise<void>
  setTrackSourceFile: (trackIndex: number, filePath: string, artist?: string|null, title?: string|null, playlistId?: string|null, playlistName?: string|null, playlistIndex?: number|null) => Promise<void>
  saveTempAudioFile: (arrayBuffer: ArrayBuffer, fileName: string) => Promise<string>
  deleteTempFile: (filePath: string) => Promise<void>
  
  // File Playback Controls
  playFile: (trackIndex: number, fileId?: string) => Promise<void>
  pauseFile: (trackIndex: number) => Promise<void>
  stopFile: (trackIndex: number) => Promise<void>
  seekFile: (trackIndex: number, timeSeconds: number) => Promise<void>
  setFilePlaybackRate: (trackIndex: number, rate: number) => Promise<void>
  getWaveformData: (trackIndex: number, numPoints: number) => Promise<{ track: number; data: number[]; duration: number; sample_rate: number }>
  
  // Track Parameters
  setPan: (trackIndex: number, pan: number) => Promise<void>
  setTrackPad: (track: number, enabled: boolean) => Promise<void>
  setTrackHPF: (track: number, enabled: boolean) => Promise<void>
  setTrackPhaseInvert: (track: number, enabled: boolean) => Promise<void>
  setTrackPFL: (track: number, enabled: boolean) => Promise<void>
  
  // Master Section Controls
  setMasterGain: (gain: number) => Promise<void>
  setMasterGainLeft: (gain: number) => Promise<void>
  setMasterGainRight: (gain: number) => Promise<void>
  setMasterMute: (mute: boolean) => Promise<void>
  setMasterLinked: (linked: boolean) => Promise<void>
  setMasterParametricEQFilters: (filters: Array<{type: string, frequency: number, gain: number, q: number}>) => Promise<void>
  setMasterParametricEQEnabled: (enabled: boolean) => Promise<void>
  clearMasterParametricEQ: () => Promise<void>
  setMasterOutputChannels: (leftChannel: number, rightChannel: number) => Promise<void>
  setSelectedMasterOutput: (deviceId: string | null) => Promise<void>
  
  // EQ Presets
  getEQPresets: () => Promise<{ presets: Array<{ name: string; filters: Array<{ filter_type: string; frequency: number; gain: number; q: number }> }> }>
  applyEQPreset: (presetName: string) => Promise<void>
  
  // Master FX Controls
  setMasterCompressor: (enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setMasterLimiter: (enabled: boolean, ceiling: number, release: number) => Promise<void>
  setMasterDelay: (enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number) => Promise<void>
  setMasterReverb: (enabled: boolean, roomSize: number, damping: number, wet: number, width: number, preDelay: number) => Promise<void>
  addMasterFxEffect: (effectType: string) => Promise<void>
  removeMasterFxEffect: (effectType: string) => Promise<void>
  
  // Subgroup Controls
  addSubgroup: () => Promise<number>
  removeSubgroup: (subgroup: number) => Promise<void>
  setSubgroupGain: (subgroup: number, gain: number) => Promise<void>
  setSubgroupMute: (subgroup: number, mute: boolean) => Promise<void>
  setSubgroupOutputEnabled: (subgroup: number, enabled: boolean) => Promise<void>
  setSubgroupRouteToMaster: (subgroup: number, route: boolean) => Promise<void>
  setSubgroupOutputChannels: (subgroup: number, leftChannel: number, rightChannel: number) => Promise<void>
  setSelectedSubgroupOutput: (subgroup: number, deviceId: string | null) => Promise<void>
  setTrackRouteToSubgroup: (track: number, subgroup: number, route: boolean) => Promise<void>
  
  // Aux Bus Controls
  setTrackAuxSend: (track: number, aux: number, level: number, preFader: boolean, muted: boolean) => Promise<void>
  setAuxBusGain: (aux: number, gain: number) => Promise<void>
  setAuxBusMute: (aux: number, mute: boolean) => Promise<void>
  setAuxBusReverb: (aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number, preDelay: number) => Promise<void>
  setAuxBusDelay: (aux: number, enabled: boolean, time: number, feedback: number, mix: number) => Promise<void>
  setAuxBusRouteToMaster: (aux: number, route: boolean) => Promise<void>
  setAuxBusOutputEnabled: (aux: number, enabled: boolean) => Promise<void>
  setAuxBusOutputChannels: (aux: number, leftChannel: number, rightChannel: number) => Promise<void>
  setAuxBusRouteToSubgroup: (aux: number, subgroup: number, route: boolean) => Promise<void>
  setAuxBusSelectedOutput: (aux: number, deviceId: string | null) => Promise<void>
  setTrackSourceAuxReturn: (track: number, aux: number) => Promise<void>
  
  // Library Files
  saveLibraryFile: (arrayBuffer: ArrayBuffer, fileName: string, metadata?: any) => Promise<{ id: string, filePath: string }>
  listLibraryFiles: () => Promise<Array<any>>
  getLibraryFile: (fileId: string) => Promise<any>
  deleteLibraryFile: (fileId: string) => Promise<void>
  
  // Playlists
  savePlaylist: (playlist: any) => Promise<void>
  listPlaylists: () => Promise<Array<any>>
  getPlaylist: (playlistId: string) => Promise<any>
  deletePlaylist: (playlistId: string) => Promise<void>
  
  // Scenes (Rust engine snapshots - simplified API)
  saveScene: (name: string) => Promise<void>
  listScenes: () => Promise<Array<{ name: string; timestamp: number; version: number; pinned: boolean; track_count: number; track_ids: number[]; track_types: string[] }>>
  loadScene: (name: string) => Promise<void>
  deleteScene: (name: string) => Promise<void>
  renameScene: (oldName: string, newName: string) => Promise<void>
  pinScene: (name: string) => Promise<void>
  setActiveTracksInfo: (tracks: Array<{id: number, trackType: string}>) => Promise<void>
  
  // File dialog
  showOpenFileDialog: () => Promise<Array<{ name: string; path: string }> | null>
  readFileAsBuffer: (filePath: string) => Promise<{ name: string; buffer: ArrayBuffer }>
  
  // NDI Streaming
  startNdi: (streamName: string, source: string) => Promise<void>
  stopNdi: () => Promise<void>
  setNdiSource: (source: string) => Promise<void>
  setNdiName: (name: string) => Promise<void>
  setNdiVideoText: (text: string) => Promise<void>
  
  // Loudness Metering (EBU R128)
  getLoudness: () => Promise<void>
  resetLoudness: () => Promise<void>
  
  // Dynamic Range Metering
  getDynamicRange: () => Promise<void>
  resetDynamicRange: () => Promise<void>
  
  // Phase Correlation Metering (Master)
  getPhaseCorrelation: () => Promise<void>
  resetPhaseCorrelation: () => Promise<void>
  
  // Stereo Width Metering (Master)
  getStereoWidth: () => Promise<void>
  resetStereoWidth: () => Promise<void>
  
  // Headroom Metering (Master)
  getHeadroom: () => Promise<void>
  resetHeadroom: () => Promise<void>
  
  // License management
  saveLicense: (key: string, licenseType: string, expiresAt: string | null) => Promise<void>
  getLicense: () => Promise<{ key: string; license_type: string; expires_at: string | null; is_valid: boolean } | null>
  
  // Audio configuration management
  saveAudioConfig: (sampleRate: number, bufferSize: number) => Promise<void>
  getAudioConfig: () => Promise<{ sample_rate: number; buffer_size: number } | null>
}

// Electron API for window controls
interface ElectronAPI {
  // Platform detection
  getPlatform: () => Promise<string>
  getAppVersion: () => Promise<string>
  
  // License verification
  verifyLicense: (licenseKey: string) => Promise<{ valid: boolean, message?: string, type?: string }>
  
  // Network URL
  getLocalIp: () => Promise<{ ip: string, port: number, wsPort: number }>
  
  // Window controls
  minimizeWindow: () => void
  maximizeWindow: () => void
  unmaximizeWindow: () => void
  closeWindow: () => void
  isMaximized: () => Promise<boolean>
  
  // Window state listeners
  onMaximized: (callback: () => void) => void
  onUnmaximized: (callback: () => void) => void
  
  // Remote control state listener
  onRemoteControlState: (callback: (data: { active: boolean, clientsCount: number }) => void) => void
  
  // Disconnect remote clients
  disconnectRemoteClients: () => Promise<void>
  
  // Detached windows (pop-out components)
  openDetachedWindow: (componentType: string) => Promise<boolean>
  closeDetachedWindow: (componentType: string) => Promise<boolean>
  isDetachedWindowOpen: (componentType: string) => Promise<boolean>
  
  // Update state for detached windows
  updateAuxBusesState: (auxBuses: any[]) => Promise<void>
}

interface Window {
  audioEngine: AudioEngine
  electronAPI?: ElectronAPI
}