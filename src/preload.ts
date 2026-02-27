// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge, ipcRenderer } from 'electron'

// Audio Engine API
contextBridge.exposeInMainWorld('audioEngine', {
  start: () => ipcRenderer.invoke('audio-engine:start'),
  stop: () => ipcRenderer.invoke('audio-engine:stop'),
  setGain: (track: number, gain: number) => ipcRenderer.invoke('audio-engine:set-gain', track, gain),
  setMute: (track: number, mute: boolean) => ipcRenderer.invoke('audio-engine:set-mute', track, mute),
  listDevices: () => ipcRenderer.invoke('audio-engine:list-devices'),
  onResponse: (callback: (response: any) => void) => {
    ipcRenderer.on('audio-engine-response', (_, data) => callback(data))
  }
})