// Build configuration limits based on BUILD_MODE
export type BuildMode = 'demo' | 'medium' | 'full'

export interface BuildLimits {
  maxTracks: number
  maxAudioTracks: number
  maxSignalTracks: number
  maxAuxBuses: number
  maxSubgroups: number
  allowSubgroupRouting: boolean
  defaultAudioTracks: number
  defaultSignalTracks: number
}

const LIMITS: Record<BuildMode, BuildLimits> = {
  demo: {
    maxTracks: 2,
    maxAudioTracks: 1,
    maxSignalTracks: 1,
    maxAuxBuses: 1,
    maxSubgroups: 0,
    allowSubgroupRouting: false,
    defaultAudioTracks: 1,
    defaultSignalTracks: 1
  },
  medium: {
    maxTracks: 5,
    maxAudioTracks: 4,
    maxSignalTracks: 1,
    maxAuxBuses: 6,
    maxSubgroups: 4,
    allowSubgroupRouting: true,
    defaultAudioTracks: 4,
    defaultSignalTracks: 1
  },
  full: {
    maxTracks: 24,
    maxAudioTracks: 24,
    maxSignalTracks: 24,
    maxAuxBuses: 6,
    maxSubgroups: 4,
    allowSubgroupRouting: true,
    defaultAudioTracks: 6,
    defaultSignalTracks: 1
  }
}

// Get build mode from environment variable, default to 'full'
export function getBuildMode(): BuildMode {
  const mode = import.meta.env.VITE_BUILD_MODE as string
  if (mode === 'demo' || mode === 'medium' || mode === 'full') {
    return mode
  }
  return 'full' // default
}

// Get current build limits
export function getBuildLimits(): BuildLimits {
  const mode = getBuildMode()
  return LIMITS[mode]
}

// Check if a track type can be added
export function canAddTrack(
  currentTracks: Array<{ type: 'audio' | 'signal' }>,
  trackType: 'audio' | 'signal'
): boolean {
  const limits = getBuildLimits()
  
  // Check total tracks limit
  if (currentTracks.length >= limits.maxTracks) {
    return false
  }
  
  // Check type-specific limits
  const audioCount = currentTracks.filter(t => t.type === 'audio').length
  const signalCount = currentTracks.filter(t => t.type === 'signal').length
  
  if (trackType === 'audio' && audioCount >= limits.maxAudioTracks) {
    return false
  }
  
  if (trackType === 'signal' && signalCount >= limits.maxSignalTracks) {
    return false
  }
  
  return true
}

// Get track counts for display
export function getTrackCounts(tracks: Array<{ type: 'audio' | 'signal' }>): {
  audio: number
  signal: number
  total: number
} {
  return {
    audio: tracks.filter(t => t.type === 'audio').length,
    signal: tracks.filter(t => t.type === 'signal').length,
    total: tracks.length
  }
}
