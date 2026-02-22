import { ref, computed, watch } from 'vue'

// Automation point: time (seconds) and value
export interface AutomationPoint {
  time: number  // in seconds
  value: number // parameter value
  curve?: 'linear' | 'exponential' | 'logarithmic' | 'step'
}

// Automation lane for a specific parameter
export interface AutomationLane {
  trackId: number
  parameter: 'volume' | 'pan' | 'auxSend' | 'pluginParam'
  auxId?: string // For aux send automation
  pluginId?: string // For plugin parameter automation
  paramName?: string // For plugin parameter automation
  points: AutomationPoint[]
  enabled: boolean // Read automation when playing
  mode: 'off' | 'read' | 'write' | 'touch' | 'latch'
}

// Transport state
export interface TransportState {
  isPlaying: boolean
  currentTime: number // in seconds
  duration: number // total project duration
  bpm: number
  timeSignature: { numerator: number, denominator: number }
}

// Composable for automation management
export function useAutomation() {
  // Automation lanes for all tracks
  const automationLanes = ref<AutomationLane[]>([])
  
  // Transport state
  const transport = ref<TransportState>({
    isPlaying: false,
    currentTime: 0,
    duration: 300, // Default 5 minutes
    bpm: 120,
    timeSignature: { numerator: 4, denominator: 4 }
  })

  // Recording state
  const isRecording = ref(false)
  const recordingLanes = ref<Set<string>>(new Set()) // Set of lane IDs being recorded

  // Playhead position (0-1, normalized)
  const playheadPosition = computed(() => {
    if (transport.value.duration <= 0) return 0
    return transport.value.currentTime / transport.value.duration
  })

  // Get automation lane for a specific track and parameter
  function getAutomationLane(trackId: number, parameter: string, auxId?: string): AutomationLane | undefined {
    return automationLanes.value.find(lane => 
      lane.trackId === trackId && 
      lane.parameter === parameter &&
      (auxId ? lane.auxId === auxId : true)
    )
  }

  // Create or get automation lane
  function getOrCreateLane(trackId: number, parameter: 'volume' | 'pan' | 'auxSend', auxId?: string): AutomationLane {
    let lane = getAutomationLane(trackId, parameter, auxId)
    
    if (!lane) {
      lane = {
        trackId,
        parameter,
        auxId,
        points: [],
        enabled: true,
        mode: 'off'
      }
      automationLanes.value.push(lane)
    }
    
    return lane
  }

  // Add automation point
  function addPoint(trackId: number, parameter: string, time: number, value: number, auxId?: string, curve: AutomationPoint['curve'] = 'linear') {
    const lane = getOrCreateLane(trackId, parameter as any, auxId)
    
    // Check if point already exists at this time (within 10ms tolerance)
    const existingIndex = lane.points.findIndex(p => Math.abs(p.time - time) < 0.01)
    
    if (existingIndex >= 0) {
      // Update existing point
      lane.points[existingIndex].value = value
      lane.points[existingIndex].curve = curve
    } else {
      // Add new point
      lane.points.push({ time, value, curve })
      // Sort by time
      lane.points.sort((a, b) => a.time - b.time)
    }
  }

  // Remove automation point
  function removePoint(trackId: number, parameter: string, pointIndex: number, auxId?: string) {
    const lane = getAutomationLane(trackId, parameter, auxId)
    if (lane && pointIndex >= 0 && pointIndex < lane.points.length) {
      lane.points.splice(pointIndex, 1)
    }
  }

  // Update automation point
  function updatePoint(trackId: number, parameter: string, pointIndex: number, time: number, value: number, auxId?: string, curve: AutomationPoint['curve'] = 'linear') {
    const lane = getAutomationLane(trackId, parameter, auxId)
    if (lane && pointIndex >= 0 && pointIndex < lane.points.length) {
      lane.points[pointIndex].time = time
      lane.points[pointIndex].value = value
      lane.points[pointIndex].curve = curve
      // Re-sort by time
      lane.points.sort((a, b) => a.time - b.time)
    }
  }

  // Get interpolated value at current time
  function getValueAtTime(trackId: number, parameter: string, time: number, auxId?: string): number | null {
    const lane = getAutomationLane(trackId, parameter, auxId)
    
    if (!lane || !lane.enabled || lane.points.length === 0) {
      return null
    }

    // Find surrounding points
    let beforePoint: AutomationPoint | null = null
    let afterPoint: AutomationPoint | null = null

    for (let i = 0; i < lane.points.length; i++) {
      const point = lane.points[i]
      if (point.time <= time) {
        beforePoint = point
      } else {
        afterPoint = point
        break
      }
    }

    // If no points before, use first point value
    if (!beforePoint && afterPoint) {
      return afterPoint.value
    }

    // If no points after, use last point value (hold)
    if (beforePoint && !afterPoint) {
      return beforePoint.value
    }

    // Interpolate between two points
    if (beforePoint && afterPoint) {
      const t = (time - beforePoint.time) / (afterPoint.time - beforePoint.time)
      const curve = beforePoint.curve || 'linear'

      switch (curve) {
        case 'linear':
          return beforePoint.value + (afterPoint.value - beforePoint.value) * t
        
        case 'exponential':
          // Exponential curve (useful for fades)
          const expT = t * t
          return beforePoint.value + (afterPoint.value - beforePoint.value) * expT
        
        case 'logarithmic':
          // Logarithmic curve (inverse of exponential)
          const logT = 1 - Math.pow(1 - t, 2)
          return beforePoint.value + (afterPoint.value - beforePoint.value) * logT
        
        case 'step':
          // Step: hold value until next point
          return beforePoint.value
        
        default:
          return beforePoint.value + (afterPoint.value - beforePoint.value) * t
      }
    }

    return null
  }

  // Clear all points from a lane
  function clearLane(trackId: number, parameter: string, auxId?: string) {
    const lane = getAutomationLane(trackId, parameter, auxId)
    if (lane) {
      lane.points = []
    }
  }

  // Delete entire lane
  function deleteLane(trackId: number, parameter: string, auxId?: string) {
    const index = automationLanes.value.findIndex(lane =>
      lane.trackId === trackId &&
      lane.parameter === parameter &&
      (auxId ? lane.auxId === auxId : true)
    )
    if (index >= 0) {
      automationLanes.value.splice(index, 1)
    }
  }

  // Set automation mode for a lane
  function setMode(trackId: number, parameter: string, mode: AutomationLane['mode'], auxId?: string) {
    const lane = getOrCreateLane(trackId, parameter as any, auxId)
    lane.mode = mode
    lane.enabled = mode !== 'off'
  }

  // Transport controls
  function play() {
    transport.value.isPlaying = true
  }

  function pause() {
    transport.value.isPlaying = false
  }

  function stop() {
    transport.value.isPlaying = false
    transport.value.currentTime = 0
  }

  function seek(time: number) {
    transport.value.currentTime = Math.max(0, Math.min(time, transport.value.duration))
  }

  function seekNormalized(position: number) {
    seek(position * transport.value.duration)
  }

  // Start recording automation
  function startRecording(trackId: number, parameter: string, auxId?: string) {
    const lane = getOrCreateLane(trackId, parameter as any, auxId)
    const laneId = `${trackId}-${parameter}${auxId ? `-${auxId}` : ''}`
    
    // Clear existing points when starting recording (overwrite mode)
    if (lane.mode === 'write') {
      lane.points = []
    }
    
    recordingLanes.value.add(laneId)
    isRecording.value = true
  }

  // Stop recording automation
  function stopRecording(trackId?: number, parameter?: string, auxId?: string) {
    if (trackId !== undefined && parameter !== undefined) {
      const laneId = `${trackId}-${parameter}${auxId ? `-${auxId}` : ''}`
      recordingLanes.value.delete(laneId)
    } else {
      recordingLanes.value.clear()
    }
    
    if (recordingLanes.value.size === 0) {
      isRecording.value = false
    }
  }

  // Check if a lane is recording
  function isLaneRecording(trackId: number, parameter: string, auxId?: string): boolean {
    const laneId = `${trackId}-${parameter}${auxId ? `-${auxId}` : ''}`
    return recordingLanes.value.has(laneId)
  }

  // Thin automation points (remove redundant points on a straight line)
  function thinPoints(trackId: number, parameter: string, threshold = 0.01, auxId?: string) {
    const lane = getAutomationLane(trackId, parameter, auxId)
    if (!lane || lane.points.length < 3) return

    const thinned: AutomationPoint[] = [lane.points[0]]

    for (let i = 1; i < lane.points.length - 1; i++) {
      const prev = lane.points[i - 1]
      const curr = lane.points[i]
      const next = lane.points[i + 1]

      // Calculate expected value at curr.time using linear interpolation
      const t = (curr.time - prev.time) / (next.time - prev.time)
      const expected = prev.value + (next.value - prev.value) * t

      // If current point deviates from the line, keep it
      if (Math.abs(curr.value - expected) > threshold) {
        thinned.push(curr)
      }
    }

    thinned.push(lane.points[lane.points.length - 1])
    lane.points = thinned
  }

  // Export automation data for scene saving
  function exportAutomation() {
    return {
      lanes: automationLanes.value.map(lane => ({
        trackId: lane.trackId,
        parameter: lane.parameter,
        auxId: lane.auxId,
        points: [...lane.points],
        enabled: lane.enabled,
        mode: lane.mode
      })),
      transport: { ...transport.value }
    }
  }

  // Import automation data from scene
  function importAutomation(data: any) {
    if (data.lanes) {
      automationLanes.value = data.lanes
    }
    if (data.transport) {
      transport.value = { ...transport.value, ...data.transport }
    }
  }

  // Clear all automation
  function clearAll() {
    automationLanes.value = []
  }

  return {
    // State
    automationLanes,
    transport,
    isRecording,
    playheadPosition,
    
    // Lane management
    getAutomationLane,
    getOrCreateLane,
    deleteLane,
    clearLane,
    setMode,
    
    // Point management
    addPoint,
    removePoint,
    updatePoint,
    getValueAtTime,
    thinPoints,
    
    // Transport
    play,
    pause,
    stop,
    seek,
    seekNormalized,
    
    // Recording
    startRecording,
    stopRecording,
    isLaneRecording,
    
    // Import/Export
    exportAutomation,
    importAutomation,
    clearAll
  }
}
