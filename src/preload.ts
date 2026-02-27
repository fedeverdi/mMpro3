// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge, ipcRenderer } from 'electron'

// Audio Engine API
contextBridge.exposeInMainWorld('audioEngine', {
  start: () => ipcRenderer.invoke('audio-engine:start'),
  stop: () => ipcRenderer.invoke('audio-engine:stop'),
  setGain: (track: number, gain: number) => ipcRenderer.invoke('audio-engine:set-gain', track, gain),
  setMute: (track: number, mute: boolean) => ipcRenderer.invoke('audio-engine:set-mute', track, mute),
  setEQ: (track: number, low: number, mid: number, high: number) => ipcRenderer.invoke('audio-engine:set-eq', track, low, mid, high),
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => 
    ipcRenderer.invoke('audio-engine:set-compressor', track, enabled, threshold, ratio, attack, release),
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => 
    ipcRenderer.invoke('audio-engine:set-gate', track, enabled, threshold, range, attack, release),
  listDevices: () => ipcRenderer.invoke('audio-engine:list-devices'),
  onResponse: (callback: (response: any) => void) => {
    ipcRenderer.on('audio-engine-response', (_, data) => callback(data))
  }
})