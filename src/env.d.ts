/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string
declare const MAIN_WINDOW_VITE_NAME: string

declare module 'tone'

// Audio Engine API
interface AudioEngine {
  start: () => Promise<void>
  stop: () => Promise<void>
  setGain: (track: number, gain: number) => Promise<void>
  setMute: (track: number, mute: boolean) => Promise<void>
  setEQ: (track: number, low: number, mid: number, high: number) => Promise<void>
  setCompressor: (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => Promise<void>
  setGate: (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => Promise<void>
  listDevices: () => Promise<void>
  onResponse: (callback: (response: any) => void) => void
}

interface Window {
  audioEngine: AudioEngine
}