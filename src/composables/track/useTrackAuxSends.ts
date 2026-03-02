import { ref, type Ref } from 'vue'

/**
 * DEPRECATED: This file is no longer used.
 * Aux send routing is now handled by routing matrices or other components.
 * All Tone.js dependencies have been removed.
 */

export interface AuxSendsState {
  auxSendsData: Ref<Record<string, { level: number, preFader: boolean, muted: boolean }>>
}

export interface AuxSendsCallbacks {
  // Kept for backwards compatibility, but no longer used
  getTone?: () => any
  getAudioNodes?: () => any
  getAuxBuses?: () => any
  getIsMuted?: () => boolean
}

export function useTrackAuxSends(callbacks: AuxSendsCallbacks = {}) {
  const auxSendsData = ref<Record<string, { level: number, preFader: boolean, muted: boolean }>>({})
  
  return {
    auxSendsData,
    updateLocalAuxSend: () => console.warn('useTrackAuxSends is deprecated'),
    toggleLocalPrePost: () => console.warn('useTrackAuxSends is deprecated'),
    toggleLocalMute: () => console.warn('useTrackAuxSends is deprecated'),
    handleAuxSendsUpdate: () => console.warn('useTrackAuxSends is deprecated'),
    getAuxSendNodes: () => new Map(),
    cleanup: () => {}
  }
}
