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
  start: (inputDevice?: string, outputDevice?: string) => Promise<void>
  stop: () => Promise<void>
  setGain: (track: number, gain: number) => Promise<void>
  setVolume: (track: number, volume: number) => Promise<void>
  setMute: (track: number, mute: boolean) => Promise<void>
  setRouteToMaster: (track: number, route: boolean) => Promise<void>
  setEQ: (track: number, low: number, low_mid: number, high_mid: number, high: number) => Promise<void>
  setEQEnabled: (track: number, enabled: boolean) => Promise<void>
  setParametricEQFilters: (track: number, filters: Array<{type: string, frequency: number, gain: number, q: number}>) => Promise<void>
  setParametricEQEnabled: (track: number, enabled: boolean) => Promise<void>
  clearParametricEQ: (track: number) => Promise<void>
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => Promise<void>
  listDevices: () => Promise<RustAudioDevice[]>
  listAudioInputs: () => Promise<RustAudioDevice[]>
  onResponse: (callback: (response: any) => void) => void
  
  // Track Source Selection
  setTrackSourceInput: (trackIndex: number, leftChannel: number, rightChannel: number, deviceName?: string | null) => Promise<void>
  setTrackSourceSignal: (trackIndex: number, waveform: string, frequency: number) => Promise<void>
  setTrackSourceFile: (trackIndex: number, filePath: string) => Promise<void>
  saveTempAudioFile: (arrayBuffer: ArrayBuffer, fileName: string) => Promise<string>
  
  // File Playback Controls
  playFile: (trackIndex: number) => Promise<void>
  pauseFile: (trackIndex: number) => Promise<void>
  stopFile: (trackIndex: number) => Promise<void>
  
  // Track Parameters
  setPan: (trackIndex: number, pan: number) => Promise<void>
  setTrackPad: (track: number, enabled: boolean) => Promise<void>
  setTrackHPF: (track: number, enabled: boolean) => Promise<void>
  
  // Master Section Controls
  setMasterGain: (gain: number) => Promise<void>
  setMasterMute: (mute: boolean) => Promise<void>
  setMasterParametricEQFilters: (filters: Array<{type: string, frequency: number, gain: number, q: number}>) => Promise<void>
  setMasterParametricEQEnabled: (enabled: boolean) => Promise<void>
  clearMasterParametricEQ: () => Promise<void>
  setMasterOutputChannels: (leftChannel: number, rightChannel: number) => Promise<void>
  
  // Master FX Controls
  setMasterCompressor: (enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setMasterLimiter: (enabled: boolean, ceiling: number, release: number) => Promise<void>
  setMasterDelay: (enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number) => Promise<void>
  setMasterReverb: (enabled: boolean, roomSize: number, damping: number, wet: number, width: number) => Promise<void>
  
  // Subgroup Controls
  addSubgroup: () => Promise<number>
  removeSubgroup: (subgroup: number) => Promise<void>
  setSubgroupGain: (subgroup: number, gain: number) => Promise<void>
  setSubgroupMute: (subgroup: number, mute: boolean) => Promise<void>  setSubgroupOutputEnabled: (subgroup: number, enabled: boolean) => Promise<void>  setSubgroupRouteToMaster: (subgroup: number, route: boolean) => Promise<void>
  setSubgroupOutputChannels: (subgroup: number, leftChannel: number, rightChannel: number) => Promise<void>
  setTrackRouteToSubgroup: (track: number, subgroup: number, route: boolean) => Promise<void>
  
  // Aux Bus Controls
  setTrackAuxSend: (track: number, aux: number, level: number, preFader: boolean, muted: boolean) => Promise<void>
  setAuxBusGain: (aux: number, gain: number) => Promise<void>
  setAuxBusMute: (aux: number, mute: boolean) => Promise<void>
  setAuxBusReverb: (aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number) => Promise<void>
  setAuxBusDelay: (aux: number, enabled: boolean, time: number, feedback: number, mix: number) => Promise<void>
  setAuxBusRouteToMaster: (aux: number, route: boolean) => Promise<void>
  setAuxBusOutputEnabled: (aux: number, enabled: boolean) => Promise<void>
  setAuxBusOutputChannels: (aux: number, leftChannel: number, rightChannel: number) => Promise<void>
}

interface Window {
  audioEngine: AudioEngine
}