// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge, ipcRenderer } from 'electron'

// Audio Engine API
contextBridge.exposeInMainWorld('audioEngine', {
  start: (inputDevice?: string, outputDevice?: string) => ipcRenderer.invoke('audio-engine:start', inputDevice, outputDevice),
  stop: () => ipcRenderer.invoke('audio-engine:stop'),
  
  // Track controls
  setGain: (track: number, gain: number) => ipcRenderer.invoke('audio-engine:set-gain', track, gain),
  setVolume: (track: number, volume: number) => ipcRenderer.invoke('audio-engine:set-volume', track, volume),
  setMute: (track: number, mute: boolean) => ipcRenderer.invoke('audio-engine:set-mute', track, mute),
  setPan: (track: number, pan: number) => ipcRenderer.invoke('audio-engine:set-pan', track, pan),
  setEQ: (track: number, low: number, low_mid: number, high_mid: number, high: number) => 
    ipcRenderer.invoke('audio-engine:set-eq', track, low, low_mid, high_mid, high),
  setEQEnabled: (track: number, enabled: boolean) => ipcRenderer.invoke('audio-engine:set-eq-enabled', track, enabled),
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => 
    ipcRenderer.invoke('audio-engine:set-compressor', track, enabled, threshold, ratio, attack, release),
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => 
    ipcRenderer.invoke('audio-engine:set-gate', track, enabled, threshold, range, attack, release),
  
  // Track source selection
  setTrackSourceInput: (track: number, leftChannel: number, rightChannel: number) => 
    ipcRenderer.invoke('audio-engine:set-track-source-input', track, leftChannel, rightChannel),
  setTrackSourceSignal: (track: number, waveform: string, frequency: number) => 
    ipcRenderer.invoke('audio-engine:set-track-source-signal', track, waveform, frequency),
  setTrackSourceFile: (track: number, filePath: string) => 
    ipcRenderer.invoke('audio-engine:set-track-source-file', track, filePath),
  saveTempAudioFile: (arrayBuffer: ArrayBuffer, fileName: string) => 
    ipcRenderer.invoke('audio-engine:save-temp-audio-file', arrayBuffer, fileName),
  
  // File playback controls
  playFile: (track: number) => ipcRenderer.invoke('audio-engine:play-file', track),
  pauseFile: (track: number) => ipcRenderer.invoke('audio-engine:pause-file', track),
  stopFile: (track: number) => ipcRenderer.invoke('audio-engine:stop-file', track),
  
  // Master controls
  setMasterGain: (gain: number) => ipcRenderer.invoke('audio-engine:set-master-gain', gain),
  setMasterMute: (mute: boolean) => ipcRenderer.invoke('audio-engine:set-master-mute', mute),
  setMasterOutputChannels: (leftChannel: number, rightChannel: number) => 
    ipcRenderer.invoke('audio-engine:set-master-output-channels', leftChannel, rightChannel),
  
  // Device management
  listDevices: () => ipcRenderer.invoke('audio-engine:list-devices'),
  
  // Response listener
  onResponse: (callback: (response: any) => void) => {
    ipcRenderer.on('audio-engine-response', (_, data) => callback(data))
  }
})