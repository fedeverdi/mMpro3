/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string
declare const MAIN_WINDOW_VITE_NAME: string

declare module 'tone'
declare module 'electron-squirrel-startup'

// Audio Engine API
interface AudioEngine {
  start: () => Promise<void>
  stop: () => Promise<void>
  setGain: (track: number, gain: number) => Promise<void>
  setVolume: (track: number, volume: number) => Promise<void>
  setMute: (track: number, mute: boolean) => Promise<void>
  setEQ: (track: number, low: number, mid: number, high: number) => Promise<void>
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => Promise<void>
  listDevices: () => Promise<void>
  onResponse: (callback: (response: any) => void) => void
  
  // Track Source Selection
  setTrackSourceInput: (trackIndex: number, leftChannel: number, rightChannel: number) => Promise<void>
  setTrackSourceSignal: (trackIndex: number, waveform: string, frequency: number) => Promise<void>
  setTrackSourceFile: (trackIndex: number, filePath: string) => Promise<void>
  saveTempAudioFile: (arrayBuffer: ArrayBuffer, fileName: string) => Promise<string>
  
  // File Playback Controls
  playFile: (trackIndex: number) => Promise<void>
  pauseFile: (trackIndex: number) => Promise<void>
  stopFile: (trackIndex: number) => Promise<void>
  
  // Track Parameters
  setPan: (trackIndex: number, pan: number) => Promise<void>
  
  // Master Section Controls
  setMasterGain: (gain: number) => Promise<void>
  setMasterMute: (mute: boolean) => Promise<void>
  setMasterOutputChannels: (leftChannel: number, rightChannel: number) => Promise<void>
}

interface Window {
  audioEngine: AudioEngine
}