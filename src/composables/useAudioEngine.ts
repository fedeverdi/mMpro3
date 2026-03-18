import { ref, onUnmounted, triggerRef, getCurrentInstance } from 'vue'
import { RemoteAudioEngine } from '~/lib/remoteAudioEngine'

export interface AudioDevice {
  id: string
  name: string
  input_channels: number
  output_channels: number
  default_sample_rate: number
  is_default: boolean
}

export interface AudioEngineState {
  isRunning: boolean
  devices: AudioDevice[]
  availableOutputDevices: AudioDevice[]
  selectedInputDevice: string | null
  selectedOutputDevice: string | null
  trackLevels: Map<number, {
    left: number
    right: number
    phaseCorrelation: number
    compressorInputDb: number
    compressorReductionDb: number
    gateInputDb: number
    gateAttenuationDb: number
    fileEnded: boolean
  }>
  trackWaveforms: Map<number, number[]>
  trackEQFilters: Map<number, any[]>
  trackParameters: Map<number, {
    gain: number
    volume: number
    mute: boolean
    solo: boolean
    pan: number
    routeToMaster: boolean
    routeToSubgroups: number[]
    padEnabled: boolean
    hpfEnabled: boolean
    phaseInverted: boolean
    compressor: {
      enabled: boolean
      thresholdDb: number
      ratio: number
      attackMs: number
      releaseMs: number
    }
    gate: {
      enabled: boolean
      thresholdDb: number
      rangeDb: number
      attackMs: number
      releaseMs: number
    }
    eqEnabled: boolean
    parametricEqEnabled: boolean
    eqLow: number
    eqLowMid: number
    eqHighMid: number
    eqHigh: number
    // File player state
    fileName: string
    fileArtist?: string
    fileTitle?: string
    isStereo: boolean
    isPlaying: boolean
    // Playlist state
    playlistId?: string
    playlistName?: string
    playlistCurrentIndex?: number
    // Aux sends
    auxSends: Array<{ level: number, preFader: boolean, muted: boolean }>
    // Insert effects
    inserts?: Array<any>
  }>
  subgroupLevels: Map<number, { 
    left: number; 
    right: number;
    gain: number;
    mute: boolean;
    routeToMaster: boolean;
    selectedOutput?: string | null;
  }>
  auxLevels: Map<number, {
    left: number;
    right: number;
    gain: number;
    mute: boolean;
    routeToMaster: boolean;
    routeToSubgroups: number[];
    outputEnabled: boolean;
    outputChannelSelectionLeft: number;
    outputChannelSelectionRight: number;
    selectedOutput?: string | null;
    reverb: {
      enabled: boolean;
      roomSize: number;
      damping: number;
      wet: number;
      width: number;
    };
    delay: {
      enabled: boolean;
      delayTimeLMs: number;
      delayTimeRMs: number;
      feedback: number;
      mix: number;
    };
  }>
  masterLevels: { 
    left: number; 
    right: number;
    gain: number;
    gainLeft: number;
    gainRight: number;
    mute: boolean;
    linked: boolean;
    selectedMasterOutput?: string | null;
  }
  masterEQFilters: any[]
  masterFxEffects: any[]
  fftData: { binsLeft: Float32Array; binsRight: Float32Array; sampleRate: number } | null
  trackFFTData: Record<number, { binsLeft: Float32Array; binsRight: Float32Array; sampleRate: number }> | null
  performanceStats: {
    bufferSize: number
    sampleRate: number
    latencyMs: number
    avgProcessMs: number
    cpuPercent: number
    minProcessMs: number
    maxProcessMs: number
  } | null
  recordingStats: {
    elapsedSeconds: number
    fileSizeBytes: number
    availableSpaceGb: number
  } | null
  loudnessData: {
    momentaryLufs: number
    shortTermLufs: number
    integratedLufs: number
    loudnessRangeLu: number
    truePeakDbtp: number
  } | null
  dynamicRangeData: {
    peakDbL: number
    peakDbR: number
    rmsDbL: number
    rmsDbR: number
    dynamicRangeL: number
    dynamicRangeR: number
    dynamicRangeStereo: number
  } | null
  phaseCorrelationData: {
    correlation: number
    monoCompatible: boolean
  } | null
  stereoWidthData: {
    widthPercent: number
    midRms: number
    sideRms: number
    balance: number
  } | null
  headroomData: {
    peakL: number
    peakR: number
    headroomL: number
    headroomR: number
    headroomStereo: number
  } | null
  lastConfigError: string | null
}

declare global {
  interface Window {
    audioEngine: AudioEngine
  }
}

const state = ref<AudioEngineState>({
  isRunning: false,
  devices: [],
  availableOutputDevices: [],
  selectedInputDevice: null,
  selectedOutputDevice: null,
  trackLevels: new Map(),
  trackWaveforms: new Map(),
  trackEQFilters: new Map(),
  trackParameters: new Map(),
  subgroupLevels: new Map(),
  auxLevels: new Map(),
  masterLevels: { left: -60, right: -60, gain: 1.0, gainLeft: 1.0, gainRight: 1.0, mute: false, linked: true, selectedMasterOutput: null },
  masterEQFilters: [],
  masterFxEffects: [],
  fftData: null,
  trackFFTData: null,
  performanceStats: null,
  recordingStats: null,
  loudnessData: null,
  dynamicRangeData: null,
  phaseCorrelationData: null,
  stereoWidthData: null,
  headroomData: null,
  lastConfigError: null
})

// NEW PARADIGM: Buffer for parameter updates (applied once per second)
const pendingParameterUpdates = {
  tracks: new Map<number, any>(),
  subgroups: new Map<number, any>(),
  auxes: new Map<number, any>(),
  master: null as any
}

// Remote control state (mutex - one device at a time)
const isRemoteControlActive = ref(false)

let isListening = false
let remoteEngineInitialized = false
let parameterUpdateTimer: number | null = null

// Apply buffered parameter updates (called once per second)
const applyPendingParameterUpdates = () => {
  // Apply track parameter updates
  pendingParameterUpdates.tracks.forEach((trackParams, trackId) => {
    const existing = state.value.trackParameters.get(trackId)
    const newParams = existing || {
      gain: 1.0,
      volume: 1.0,
      mute: false,
      solo: false,
      pan: 0.0,
      routeToMaster: true,
      routeToSubgroups: [],
      padEnabled: false,
      hpfEnabled: false,
      phaseInverted: false,
      compressor: {
        enabled: false,
        thresholdDb: -20,
        ratio: 4,
        attackMs: 5,
        releaseMs: 50
      },
      gate: {
        enabled: false,
        thresholdDb: -40,
        rangeDb: -80,
        attackMs: 1,
        releaseMs: 100
      },
      eqEnabled: true,
      parametricEqEnabled: true,
      eqLow: 0,
      eqLowMid: 0,
      eqHighMid: 0,
      eqHigh: 0,
      fileName: '',
      fileArtist: undefined,
      fileTitle: undefined,
      isStereo: false,
      isPlaying: false,
      playlistId: undefined,
      playlistName: undefined,
      playlistCurrentIndex: undefined,
      auxSends: [],
      inserts: []
    }
    
    // Update only provided fields
    if (trackParams.gain !== undefined && trackParams.gain !== null) newParams.gain = trackParams.gain
    if (trackParams.volume !== undefined && trackParams.volume !== null) newParams.volume = trackParams.volume
    if (trackParams.mute !== undefined && trackParams.mute !== null) newParams.mute = trackParams.mute
    if (trackParams.solo !== undefined && trackParams.solo !== null) newParams.solo = trackParams.solo
    if (trackParams.pan !== undefined && trackParams.pan !== null) newParams.pan = trackParams.pan
    if (trackParams.route_to_master !== undefined && trackParams.route_to_master !== null) newParams.routeToMaster = trackParams.route_to_master
    if (trackParams.route_to_subgroups !== undefined && trackParams.route_to_subgroups !== null) newParams.routeToSubgroups = trackParams.route_to_subgroups
    if (trackParams.pad_enabled !== undefined && trackParams.pad_enabled !== null) newParams.padEnabled = trackParams.pad_enabled
    if (trackParams.hpf_enabled !== undefined && trackParams.hpf_enabled !== null) newParams.hpfEnabled = trackParams.hpf_enabled
    if (trackParams.phase_inverted !== undefined && trackParams.phase_inverted !== null) newParams.phaseInverted = trackParams.phase_inverted
    
    // Compressor
    if (trackParams.compressor_enabled !== undefined && trackParams.compressor_enabled !== null) newParams.compressor.enabled = trackParams.compressor_enabled
    if (trackParams.compressor_threshold_db !== undefined && trackParams.compressor_threshold_db !== null) newParams.compressor.thresholdDb = trackParams.compressor_threshold_db
    if (trackParams.compressor_ratio !== undefined && trackParams.compressor_ratio !== null) newParams.compressor.ratio = trackParams.compressor_ratio
    if (trackParams.compressor_attack_ms !== undefined && trackParams.compressor_attack_ms !== null) newParams.compressor.attackMs = trackParams.compressor_attack_ms
    if (trackParams.compressor_release_ms !== undefined && trackParams.compressor_release_ms !== null) newParams.compressor.releaseMs = trackParams.compressor_release_ms
    
    // Gate
    if (trackParams.gate_enabled !== undefined && trackParams.gate_enabled !== null) newParams.gate.enabled = trackParams.gate_enabled
    if (trackParams.gate_threshold_db !== undefined && trackParams.gate_threshold_db !== null) newParams.gate.thresholdDb = trackParams.gate_threshold_db
    if (trackParams.gate_range_db !== undefined && trackParams.gate_range_db !== null) newParams.gate.rangeDb = trackParams.gate_range_db
    if (trackParams.gate_attack_ms !== undefined && trackParams.gate_attack_ms !== null) newParams.gate.attackMs = trackParams.gate_attack_ms
    if (trackParams.gate_release_ms !== undefined && trackParams.gate_release_ms !== null) newParams.gate.releaseMs = trackParams.gate_release_ms
    
    // EQ
    if (trackParams.eq_enabled !== undefined && trackParams.eq_enabled !== null) newParams.eqEnabled = trackParams.eq_enabled
    if (trackParams.parametric_eq_enabled !== undefined && trackParams.parametric_eq_enabled !== null) newParams.parametricEqEnabled = trackParams.parametric_eq_enabled
    if (trackParams.eq_low !== undefined && trackParams.eq_low !== null) newParams.eqLow = trackParams.eq_low
    if (trackParams.eq_low_mid !== undefined && trackParams.eq_low_mid !== null) newParams.eqLowMid = trackParams.eq_low_mid
    if (trackParams.eq_high_mid !== undefined && trackParams.eq_high_mid !== null) newParams.eqHighMid = trackParams.eq_high_mid
    if (trackParams.eq_high !== undefined && trackParams.eq_high !== null) newParams.eqHigh = trackParams.eq_high
    
    // File player
    if (trackParams.file_name !== undefined && trackParams.file_name !== null) newParams.fileName = trackParams.file_name
    if (trackParams.file_artist !== undefined) newParams.fileArtist = trackParams.file_artist
    if (trackParams.file_title !== undefined) newParams.fileTitle = trackParams.file_title
    if (trackParams.is_stereo !== undefined && trackParams.is_stereo !== null) newParams.isStereo = trackParams.is_stereo
    
    // Playlist
    if (trackParams.playlist_id !== undefined) newParams.playlistId = trackParams.playlist_id
    if (trackParams.playlist_name !== undefined) newParams.playlistName = trackParams.playlist_name
    if (trackParams.playlist_current_index !== undefined) newParams.playlistCurrentIndex = trackParams.playlist_current_index
    
    // Aux sends
    if (trackParams.aux_sends !== undefined && trackParams.aux_sends !== null) newParams.auxSends = trackParams.aux_sends
    
    // Insert effects
    if (trackParams.inserts !== undefined && trackParams.inserts !== null) {
      newParams.inserts = trackParams.inserts
    }
    
    // EQ filters
    if (trackParams.eq_filters !== undefined) {
      state.value.trackEQFilters.set(trackId, trackParams.eq_filters)
    }
    
    state.value.trackParameters.set(trackId, newParams)
  })
  
  // Apply subgroup parameter updates
  pendingParameterUpdates.subgroups.forEach((subgroupParams, subgroupId) => {
    const existing = state.value.subgroupLevels.get(subgroupId)
    if (existing) {
      if (subgroupParams.gain !== undefined && subgroupParams.gain !== null) existing.gain = subgroupParams.gain
      if (subgroupParams.mute !== undefined && subgroupParams.mute !== null) existing.mute = subgroupParams.mute
      if (subgroupParams.route_to_master !== undefined && subgroupParams.route_to_master !== null) existing.routeToMaster = subgroupParams.route_to_master
      if (subgroupParams.selected_output !== undefined) existing.selectedOutput = subgroupParams.selected_output
    }
  })
  
  // Apply aux parameter updates
  pendingParameterUpdates.auxes.forEach((auxParams, auxId) => {
    const existing = state.value.auxLevels.get(auxId)
    if (existing) {
      if (auxParams.gain !== undefined && auxParams.gain !== null) existing.gain = auxParams.gain
      if (auxParams.mute !== undefined && auxParams.mute !== null) existing.mute = auxParams.mute
      if (auxParams.route_to_master !== undefined && auxParams.route_to_master !== null) existing.routeToMaster = auxParams.route_to_master
      if (auxParams.route_to_subgroups !== undefined && auxParams.route_to_subgroups !== null) existing.routeToSubgroups = auxParams.route_to_subgroups
      if (auxParams.output_enabled !== undefined && auxParams.output_enabled !== null) existing.outputEnabled = auxParams.output_enabled
      if (auxParams.output_channel_selection_left !== undefined && auxParams.output_channel_selection_left !== null) existing.outputChannelSelectionLeft = auxParams.output_channel_selection_left
      if (auxParams.output_channel_selection_right !== undefined && auxParams.output_channel_selection_right !== null) existing.outputChannelSelectionRight = auxParams.output_channel_selection_right
      if (auxParams.selected_output !== undefined) existing.selectedOutput = auxParams.selected_output
      
      if (auxParams.reverb !== undefined && auxParams.reverb !== null) {
        if (auxParams.reverb.enabled !== undefined && auxParams.reverb.enabled !== null) existing.reverb.enabled = auxParams.reverb.enabled
        if (auxParams.reverb.room_size !== undefined && auxParams.reverb.room_size !== null) existing.reverb.roomSize = auxParams.reverb.room_size
        if (auxParams.reverb.damping !== undefined && auxParams.reverb.damping !== null) existing.reverb.damping = auxParams.reverb.damping
        if (auxParams.reverb.wet !== undefined && auxParams.reverb.wet !== null) existing.reverb.wet = auxParams.reverb.wet
        if (auxParams.reverb.width !== undefined && auxParams.reverb.width !== null) existing.reverb.width = auxParams.reverb.width
      }
      
      if (auxParams.delay !== undefined && auxParams.delay !== null) {
        if (auxParams.delay.enabled !== undefined && auxParams.delay.enabled !== null) existing.delay.enabled = auxParams.delay.enabled
        if (auxParams.delay.delay_time_l_ms !== undefined && auxParams.delay.delay_time_l_ms !== null) existing.delay.delayTimeLMs = auxParams.delay.delay_time_l_ms
        if (auxParams.delay.delay_time_r_ms !== undefined && auxParams.delay.delay_time_r_ms !== null) existing.delay.delayTimeRMs = auxParams.delay.delay_time_r_ms
        if (auxParams.delay.feedback !== undefined && auxParams.delay.feedback !== null) existing.delay.feedback = auxParams.delay.feedback
        if (auxParams.delay.mix !== undefined && auxParams.delay.mix !== null) existing.delay.mix = auxParams.delay.mix
      }
    }
  })
  
  // Apply master parameter updates
  if (pendingParameterUpdates.master) {
    const master = pendingParameterUpdates.master
    if (master.gain !== undefined && master.gain !== null) state.value.masterLevels.gain = master.gain
    if (master.gain_left !== undefined && master.gain_left !== null) state.value.masterLevels.gainLeft = master.gain_left
    if (master.gain_right !== undefined && master.gain_right !== null) state.value.masterLevels.gainRight = master.gain_right
    if (master.mute !== undefined && master.mute !== null) state.value.masterLevels.mute = master.mute
    if (master.linked !== undefined && master.linked !== null) state.value.masterLevels.linked = master.linked
    if (master.selected_output !== undefined) state.value.masterLevels.selectedMasterOutput = master.selected_output
    if (master.eq_filters !== undefined && master.eq_filters !== null) state.value.masterEQFilters = master.eq_filters
    if (master.fx_effects !== undefined && master.fx_effects !== null) {
      console.log('[useAudioEngine] Updating masterFxEffects:', master.fx_effects)
      state.value.masterFxEffects = master.fx_effects
    }
    if (master.available_output_devices !== undefined && master.available_output_devices !== null) state.value.availableOutputDevices = master.available_output_devices
  }
  
  // Clear buffers
  pendingParameterUpdates.tracks.clear()
  pendingParameterUpdates.subgroups.clear()
  pendingParameterUpdates.auxes.clear()
  pendingParameterUpdates.master = null
  
  // Trigger Vue reactivity
  triggerRef(state)
}

export const useAudioEngine = () => {
  const initializeRemoteEngine = async () => {
    if (remoteEngineInitialized || window.audioEngine) {
      return
    }

    try {
      const host = window.location.hostname
      const remoteEngine = new RemoteAudioEngine(host, 3001)
      await remoteEngine.connect()

      ;(window as any).audioEngine = remoteEngine
      remoteEngineInitialized = true

      // Notify other composables that audioEngine is ready
      window.dispatchEvent(new CustomEvent('audio-engine-ready'))
      console.log('[useAudioEngine] Remote audio engine ready')

    } catch (error) {
      console.error('[useAudioEngine] Failed to connect to remote audio engine:', error)
      throw new Error('Failed to connect to remote audio engine. Make sure the Electron app is running.')
    }
  }

  const startListening = () => {
    if (isListening || !window.audioEngine) return

    // Listen for remote control state changes (Electron only)
    if (typeof window !== 'undefined' && (window as any).electronAPI?.onRemoteControlState) {
      (window as any).electronAPI.onRemoteControlState((data: { active: boolean, clientsCount: number }) => {
        isRemoteControlActive.value = data.active
        console.log(`[useAudioEngine] Remote control: ${data.active ? 'ENABLED' : 'DISABLED'} (${data.clientsCount} clients)`)
      })
    }

    // Start parameter update timer (apply buffered updates every 200ms)
    if (!parameterUpdateTimer) {
      parameterUpdateTimer = window.setInterval(() => {
        applyPendingParameterUpdates()
      }, 200) // 5 Hz
    }

    window.audioEngine.onResponse((response: any) => {
      switch (response.type) {
        case 'devices':
          state.value.devices = response.devices
          break

        case 'started':
          state.value.isRunning = true
          break

        case 'stopped':
          state.value.isRunning = false
          // Reset performance stats quando si stoppa per mostrare i nuovi valori al riavvio
          state.value.performanceStats = null
          break

        case 'ok':
          break

        case 'error':
          console.error('[useAudioEngine] Engine error:', response.message)
          
          // Check if it's a configuration not supported error
          if (response.message.includes('not supported by the device')) {
            state.value.lastConfigError = response.message
          }
          break

        // NEW: Optimized meters stream (60fps) - Only real-time metering data
        case 'meters':
          if (response.tracks) {
            response.tracks.forEach((trackMeter: any) => {
              state.value.trackLevels.set(trackMeter.track, {
                left: trackMeter.level_l,
                right: trackMeter.level_r,
                phaseCorrelation: trackMeter.phase_correlation || 0,
                compressorInputDb: trackMeter.compressor_input_db || -90,
                compressorReductionDb: trackMeter.compressor_reduction_db || 0,
                gateInputDb: trackMeter.gate_input_db || -90,
                gateAttenuationDb: trackMeter.gate_attenuation_db || 0,
                fileEnded: trackMeter.file_ended || false
              })

              if (trackMeter.waveform) {
                state.value.trackWaveforms.set(trackMeter.track, trackMeter.waveform)
              }

              // Ensure track parameters exist with defaults if not already present
              let existingParams = state.value.trackParameters.get(trackMeter.track)
              if (!existingParams) {
                existingParams = {
                  gain: trackMeter.gain ?? 1.0,
                  volume: trackMeter.volume ?? 1.0,
                  mute: trackMeter.mute ?? false,
                  solo: trackMeter.solo ?? false,
                  pan: trackMeter.pan ?? 0.0,
                  routeToMaster: true,
                  routeToSubgroups: [],
                  padEnabled: false,
                  hpfEnabled: false,
                  phaseInverted: false,
                  compressor: {
                    enabled: false,
                    thresholdDb: -20,
                    ratio: 4,
                    attackMs: 5,
                    releaseMs: 50
                  },
                  gate: {
                    enabled: false,
                    thresholdDb: -40,
                    rangeDb: -80,
                    attackMs: 1,
                    releaseMs: 100
                  },
                  eqEnabled: true,
                  parametricEqEnabled: true,
                  eqLow: 0,
                  eqLowMid: 0,
                  eqHighMid: 0,
                  eqHigh: 0,
                  fileName: trackMeter.file_name ?? '',
                  fileArtist: trackMeter.file_artist,
                  fileTitle: trackMeter.file_title,
                  isStereo: trackMeter.is_stereo ?? false,
                  isPlaying: trackMeter.is_playing ?? false,
                  playlistId: trackMeter.playlist_id,
                  playlistName: trackMeter.playlist_name,
                  playlistCurrentIndex: trackMeter.playlist_current_index,
                  auxSends: []
                }
                state.value.trackParameters.set(trackMeter.track, existingParams)
              } else {
                // Update essential parameters from meters (for remote sync)
                if (trackMeter.gain !== undefined) existingParams.gain = trackMeter.gain
                if (trackMeter.volume !== undefined) existingParams.volume = trackMeter.volume
                if (trackMeter.mute !== undefined) existingParams.mute = trackMeter.mute
                if (trackMeter.solo !== undefined) existingParams.solo = trackMeter.solo
                if (trackMeter.pan !== undefined) existingParams.pan = trackMeter.pan
                if (trackMeter.is_stereo !== undefined) existingParams.isStereo = trackMeter.is_stereo
                if (trackMeter.file_name !== undefined) existingParams.fileName = trackMeter.file_name
                if (trackMeter.file_artist !== undefined) existingParams.fileArtist = trackMeter.file_artist
                if (trackMeter.file_title !== undefined) existingParams.fileTitle = trackMeter.file_title
                if (trackMeter.is_playing !== undefined) existingParams.isPlaying = trackMeter.is_playing
                if (trackMeter.playlist_id !== undefined) existingParams.playlistId = trackMeter.playlist_id
                if (trackMeter.playlist_name !== undefined) existingParams.playlistName = trackMeter.playlist_name
                if (trackMeter.playlist_current_index !== undefined) existingParams.playlistCurrentIndex = trackMeter.playlist_current_index
              }
            })
          }

          if (response.subgroups) {
            response.subgroups.forEach((subgroupMeter: any) => {
              const leftDb = subgroupMeter.level_l > 0.0
                ? 20 * Math.log10(subgroupMeter.level_l)
                : -90

              const rightDb = subgroupMeter.level_r > 0.0
                ? 20 * Math.log10(subgroupMeter.level_r)
                : -90

              const existing = state.value.subgroupLevels.get(subgroupMeter.subgroup)
              if (existing) {
                existing.left = leftDb
                existing.right = rightDb
              } else {
                state.value.subgroupLevels.set(subgroupMeter.subgroup, {
                  left: leftDb,
                  right: rightDb,
                  gain: 1.0,
                  mute: false,
                  routeToMaster: false,
                  selectedOutput: null
                })
              }
            })
          }

          if (response.auxes) {
            response.auxes.forEach((auxMeter: any) => {
              const leftDb = auxMeter.level_l > 0.0
                ? 20 * Math.log10(auxMeter.level_l)
                : -90

              const rightDb = auxMeter.level_r > 0.0
                ? 20 * Math.log10(auxMeter.level_r)
                : -90

              const existing = state.value.auxLevels.get(auxMeter.aux)
              if (existing) {
                existing.left = leftDb
                existing.right = rightDb
              } else {
                state.value.auxLevels.set(auxMeter.aux, {
                  left: leftDb,
                  right: rightDb,
                  gain: 1.0,
                  mute: false,
                  routeToMaster: false,
                  routeToSubgroups: [],
                  outputEnabled: false,
                  outputChannelSelectionLeft: 0,
                  outputChannelSelectionRight: 1,
                  selectedOutput: null,
                  reverb: {
                    enabled: false,
                    roomSize: 0.3,
                    damping: 0.5,
                    wet: 0.3,
                    width: 1.0,
                  },
                  delay: {
                    enabled: false,
                    delayTimeLMs: 250.0,
                    delayTimeRMs: 375.0,
                    feedback: 0.3,
                    mix: 0.3,
                  },
                })
              }
            })
          }

          if (response.master_l !== undefined && response.master_r !== undefined) {
            state.value.masterLevels.left = response.master_l > 0 ? 20 * Math.log10(response.master_l) : -60
            state.value.masterLevels.right = response.master_r > 0 ? 20 * Math.log10(response.master_r) : -60
          }

          if (response.loudness) {
            state.value.loudnessData = {
              momentaryLufs: response.loudness.momentary_lufs,
              shortTermLufs: response.loudness.short_term_lufs,
              integratedLufs: response.loudness.integrated_lufs,
              loudnessRangeLu: response.loudness.loudness_range_lu,
              truePeakDbtp: response.loudness.true_peak_dbtp
            }
          }

          if (response.dynamic_range) {
            state.value.dynamicRangeData = {
              peakDbL: response.dynamic_range.peak_db_l,
              peakDbR: response.dynamic_range.peak_db_r,
              rmsDbL: response.dynamic_range.rms_db_l,
              rmsDbR: response.dynamic_range.rms_db_r,
              dynamicRangeL: response.dynamic_range.dynamic_range_l,
              dynamicRangeR: response.dynamic_range.dynamic_range_r,
              dynamicRangeStereo: response.dynamic_range.dynamic_range_stereo
            }
          }

          if (response.phase_correlation) {
            state.value.phaseCorrelationData = {
              correlation: response.phase_correlation.correlation,
              monoCompatible: response.phase_correlation.mono_compatible
            }
          }

          if (response.stereo_width) {
            state.value.stereoWidthData = {
              widthPercent: response.stereo_width.width_percent,
              midRms: response.stereo_width.mid_rms,
              sideRms: response.stereo_width.side_rms,
              balance: response.stereo_width.balance
            }
          }

          if (response.headroom) {
            state.value.headroomData = {
              peakL: response.headroom.peak_l,
              peakR: response.headroom.peak_r,
              headroomL: response.headroom.headroom_l,
              headroomR: response.headroom.headroom_r,
              headroomStereo: response.headroom.headroom_stereo
            }
          }

          if (response.available_output_devices) {
            state.value.availableOutputDevices = response.available_output_devices
          }
          break

        // NEW PARADIGM: Parameters changed - Buffer updates (applied once per second)
        case 'parameters':
        case 'parameters_changed': // Alias used by detached windows
          if (response.tracks) {
            response.tracks.forEach((trackParams: any) => {
              // Merge with existing buffered updates for this track (if any)
              const existing = pendingParameterUpdates.tracks.get(trackParams.track) || {}
              pendingParameterUpdates.tracks.set(trackParams.track, { ...existing, ...trackParams })
            })
          }

          if (response.subgroups) {
            response.subgroups.forEach((subgroupParams: any) => {
              const existing = pendingParameterUpdates.subgroups.get(subgroupParams.subgroup) || {}
              pendingParameterUpdates.subgroups.set(subgroupParams.subgroup, { ...existing, ...subgroupParams })
            })
          }

          if (response.auxes) {
            response.auxes.forEach((auxParams: any) => {
              const existing = pendingParameterUpdates.auxes.get(auxParams.aux) || {}
              pendingParameterUpdates.auxes.set(auxParams.aux, { ...existing, ...auxParams })
            })
          }

          if (response.master) {
            pendingParameterUpdates.master = { ...pendingParameterUpdates.master, ...response.master }
          }
          break

        // LEGACY: Full state (kept for backwards compatibility during migration)
        case 'levels':
          if (response.tracks) {
            response.tracks.forEach((trackLevel: any) => {
              state.value.trackLevels.set(trackLevel.track, {
                left: trackLevel.level_l,
                right: trackLevel.level_r,
                phaseCorrelation: trackLevel.phase_correlation || 0,
                compressorInputDb: trackLevel.compressor_input_db || -90,
                compressorReductionDb: trackLevel.compressor_reduction_db || 0,
                gateInputDb: trackLevel.gate_input_db || -90,
                gateAttenuationDb: trackLevel.gate_attenuation_db || 0,
                fileEnded: trackLevel.file_ended || false
              })

              if (trackLevel.waveform) {
                state.value.trackWaveforms.set(trackLevel.track, trackLevel.waveform)
              }

              if (trackLevel.eq_filters) {
                state.value.trackEQFilters.set(trackLevel.track, trackLevel.eq_filters)
              }

              if (trackLevel.gain !== undefined) {
                // Check if routeToSubgroups changed
                const oldParams = state.value.trackParameters.get(trackLevel.track)
                const oldRoutes = oldParams?.routeToSubgroups || []
                const newRoutes = trackLevel.route_to_subgroups || []
                const routesChanged = JSON.stringify(oldRoutes) !== JSON.stringify(newRoutes)
                
                state.value.trackParameters.set(trackLevel.track, {
                  gain: trackLevel.gain,
                  volume: trackLevel.volume,
                  mute: trackLevel.mute,
                  solo: trackLevel.solo ?? false,
                  pan: trackLevel.pan,
                  routeToMaster: trackLevel.route_to_master,
                  routeToSubgroups: newRoutes,
                  padEnabled: trackLevel.pad_enabled,
                  hpfEnabled: trackLevel.hpf_enabled,
                  phaseInverted: trackLevel.phase_inverted,
                  compressor: {
                    enabled: trackLevel.compressor_enabled,
                    thresholdDb: trackLevel.compressor_threshold_db,
                    ratio: trackLevel.compressor_ratio,
                    attackMs: trackLevel.compressor_attack_ms,
                    releaseMs: trackLevel.compressor_release_ms
                  },
                  gate: {
                    enabled: trackLevel.gate_enabled,
                    thresholdDb: trackLevel.gate_threshold_db,
                    rangeDb: trackLevel.gate_range_db,
                    attackMs: trackLevel.gate_attack_ms,
                    releaseMs: trackLevel.gate_release_ms
                  },
                  eqEnabled: trackLevel.eq_enabled ?? true,
                  parametricEqEnabled: trackLevel.parametric_eq_enabled ?? true,
                  eqLow: trackLevel.eq_low ?? 0,
                  eqLowMid: trackLevel.eq_low_mid ?? 0,
                  eqHighMid: trackLevel.eq_high_mid ?? 0,
                  eqHigh: trackLevel.eq_high ?? 0,
                  // File player state
                  fileName: trackLevel.file_name || '',
                  fileArtist: trackLevel.file_artist,
                  fileTitle: trackLevel.file_title,
                  isStereo: trackLevel.is_stereo ?? false,
                  isPlaying: trackLevel.is_playing ?? false,
                  // Playlist state
                  playlistId: trackLevel.playlist_id,
                  playlistName: trackLevel.playlist_name,
                  playlistCurrentIndex: trackLevel.playlist_current_index,
                  // Aux sends
                  auxSends: trackLevel.aux_sends || []
                })
                
                // Trigger ref only if routeToSubgroups changed to update component watchers
                if (routesChanged) {
                  triggerRef(state)
                }
              }
            })
          }

          if (response.subgroups) {
            response.subgroups.forEach((subgroupLevel: any) => {
              const leftDb = subgroupLevel.level_l > 0.0
                ? 20 * Math.log10(subgroupLevel.level_l)
                : -90

              const rightDb = subgroupLevel.level_r > 0.0
                ? 20 * Math.log10(subgroupLevel.level_r)
                : -90

              state.value.subgroupLevels.set(subgroupLevel.subgroup, {
                left: leftDb,
                right: rightDb,
                gain: subgroupLevel.gain ?? 1.0,
                mute: subgroupLevel.mute ?? false,
                routeToMaster: subgroupLevel.route_to_master ?? false,
                selectedOutput: subgroupLevel.selected_output
              })
            })
          }

          if (response.auxes) {
            response.auxes.forEach((auxLevel: any) => {
              const leftDb = auxLevel.level_l > 0.0
                ? 20 * Math.log10(auxLevel.level_l)
                : -90

              const rightDb = auxLevel.level_r > 0.0
                ? 20 * Math.log10(auxLevel.level_r)
                : -90

              state.value.auxLevels.set(auxLevel.aux, {
                left: leftDb,
                right: rightDb,
                gain: auxLevel.gain ?? 1.0,
                mute: auxLevel.mute ?? false,
                routeToMaster: auxLevel.route_to_master ?? false,
                routeToSubgroups: auxLevel.route_to_subgroups ?? [],
                outputEnabled: auxLevel.output_enabled ?? false,
                outputChannelSelectionLeft: auxLevel.output_channel_selection_left ?? 0,
                outputChannelSelectionRight: auxLevel.output_channel_selection_right ?? 1,
                selectedOutput: auxLevel.selected_output ?? null,
                reverb: {
                  enabled: auxLevel.reverb?.enabled ?? false,
                  roomSize: auxLevel.reverb?.room_size ?? 0.3,
                  damping: auxLevel.reverb?.damping ?? 0.5,
                  wet: auxLevel.reverb?.wet ?? 0.3,
                  width: auxLevel.reverb?.width ?? 1.0,
                },
                delay: {
                  enabled: auxLevel.delay?.enabled ?? false,
                  delayTimeLMs: auxLevel.delay?.delay_time_l_ms ?? 250.0,
                  delayTimeRMs: auxLevel.delay?.delay_time_r_ms ?? 375.0,
                  feedback: auxLevel.delay?.feedback ?? 0.3,
                  mix: auxLevel.delay?.mix ?? 0.3,
                },
              })
            })
          }

          if (response.master_l !== undefined && response.master_r !== undefined) {
            state.value.masterLevels = {
              left: response.master_l > 0 ? 20 * Math.log10(response.master_l) : -60,
              right: response.master_r > 0 ? 20 * Math.log10(response.master_r) : -60,
              gain: response.master_gain ?? 1.0,
              gainLeft: response.master_gain_left ?? 1.0,
              gainRight: response.master_gain_right ?? 1.0,
              mute: response.master_mute ?? false,
              linked: response.master_linked ?? true,
              selectedMasterOutput: response.selected_master_output
            }
          }

          // Update available output devices list
          if (response.available_output_devices) {
            state.value.availableOutputDevices = response.available_output_devices
          }

          if (response.master_eq_filters) {
            state.value.masterEQFilters = response.master_eq_filters
          }

          if (response.loudness_data) {
            state.value.loudnessData = {
              momentaryLufs: response.loudness_data.momentary_lufs,
              shortTermLufs: response.loudness_data.short_term_lufs,
              integratedLufs: response.loudness_data.integrated_lufs,
              loudnessRangeLu: response.loudness_data.loudness_range_lu,
              truePeakDbtp: response.loudness_data.true_peak_dbtp
            }
          }

          if (response.dynamic_range_data) {
            state.value.dynamicRangeData = {
              peakDbL: response.dynamic_range_data.peak_db_l,
              peakDbR: response.dynamic_range_data.peak_db_r,
              rmsDbL: response.dynamic_range_data.rms_db_l,
              rmsDbR: response.dynamic_range_data.rms_db_r,
              dynamicRangeL: response.dynamic_range_data.dynamic_range_l,
              dynamicRangeR: response.dynamic_range_data.dynamic_range_r,
              dynamicRangeStereo: response.dynamic_range_data.dynamic_range_stereo
            }
          }

          if (response.phase_correlation_data) {
            state.value.phaseCorrelationData = {
              correlation: response.phase_correlation_data.correlation,
              monoCompatible: response.phase_correlation_data.mono_compatible
            }
          }

          if (response.stereo_width_data) {
            state.value.stereoWidthData = {
              widthPercent: response.stereo_width_data.width_percent,
              midRms: response.stereo_width_data.mid_rms,
              sideRms: response.stereo_width_data.side_rms,
              balance: response.stereo_width_data.balance
            }
          }

          if (response.headroom_peak_l !== undefined) {
            state.value.headroomData = {
              peakL: response.headroom_peak_l,
              peakR: response.headroom_peak_r,
              headroomL: response.headroom_l,
              headroomR: response.headroom_r,
              headroomStereo: response.headroom_stereo
            }
          }
          break

        case 'fft':
          if (response.bins_left && response.bins_right && response.sample_rate) {
            state.value.fftData = {
              binsLeft: new Float32Array(response.bins_left),
              binsRight: new Float32Array(response.bins_right),
              sampleRate: response.sample_rate
            }
          }
          break

        case 'track_fft':
          if (response.track !== undefined && response.bins_left && response.bins_right && response.sample_rate) {
            // Create a new object to ensure reactivity
            const currentTrackFFT = state.value.trackFFTData || {}
            state.value.trackFFTData = {
              ...currentTrackFFT,
              [response.track]: {
                binsLeft: new Float32Array(response.bins_left),
                binsRight: new Float32Array(response.bins_right),
                sampleRate: response.sample_rate
              }
            }
          }
          break

        case 'performance':
          state.value.performanceStats = {
            bufferSize: response.buffer_size,
            sampleRate: response.sample_rate,
            latencyMs: response.latency_ms,
            avgProcessMs: response.avg_process_ms,
            cpuPercent: response.cpu_percent,
            minProcessMs: response.min_process_ms,
            maxProcessMs: response.max_process_ms
          }
          break

        case 'audio_config':
          // Audio config response - can be ignored on frontend as it's just confirmation
          break

        case 'recording_stats':
          state.value.recordingStats = {
            elapsedSeconds: response.elapsed_seconds,
            fileSizeBytes: response.file_size_bytes,
            availableSpaceGb: response.available_space_gb
          }
          break

        case 'loudness':
          state.value.loudnessData = {
            momentaryLufs: response.momentary_lufs,
            shortTermLufs: response.short_term_lufs,
            integratedLufs: response.integrated_lufs,
            loudnessRangeLu: response.loudness_range_lu,
            truePeakDbtp: response.true_peak_dbtp
          }
          break

        case 'dynamic_range':
          state.value.dynamicRangeData = {
            peakDbL: response.peak_db_l,
            peakDbR: response.peak_db_r,
            rmsDbL: response.rms_db_l,
            rmsDbR: response.rms_db_r,
            dynamicRangeL: response.dynamic_range_l,
            dynamicRangeR: response.dynamic_range_r,
            dynamicRangeStereo: response.dynamic_range_stereo
          }
          break

        case 'phase_correlation':
          state.value.phaseCorrelationData = {
            correlation: response.correlation,
            monoCompatible: response.mono_compatible
          }
          break

        case 'stereo_width':
          state.value.stereoWidthData = {
            widthPercent: response.width_percent,
            midRms: response.mid_rms,
            sideRms: response.side_rms,
            balance: response.balance
          }
          break

        case 'headroom':
          state.value.headroomData = {
            peakL: response.peak_l,
            peakR: response.peak_r,
            headroomL: response.headroom_l,
            headroomR: response.headroom_r,
            headroomStereo: response.headroom_stereo
          }
          break

        case 'master_eq_filters_updated':
          break

        case 'track_eq_filters_updated':
          break

        case 'license':
          // Trigger event for useLicense to pick up (both Electron and remote)
          window.dispatchEvent(new CustomEvent('license-updated', { detail: response }))
          break

        case 'connected':
          break

        case 'license-update':
          break

        case 'subgroup_created':
          // Broadcast subgroup creation to the app
          window.dispatchEvent(new CustomEvent('subgroup-created', { 
            detail: { id: response.id } 
          }))
          break

        case 'audio_inputs':
          // Audio input devices list - handled by useAudioDevices composable
          break

        case 'aux_buses_state':
          // Frontend aux buses state sent to detached windows - handled by specific components
          break

        case 'waveform_data':
          // Waveform data response - handled by IPC promise resolution in Electron mode
          // or by specific component in remote mode
          break

        default:
          console.log('[useAudioEngine] Unhandled response type:', response.type)
      }
    })

    isListening = true
  }

  if (window.audioEngine && !isListening) {
    startListening()
  }

  const loadDevices = async () => {
    if (!window.audioEngine) {
      await initializeRemoteEngine()
    }

    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
      return
    }

    startListening()
    await window.audioEngine.listDevices()
  }

  const start = async (inputDevice?: string, outputDevice?: string) => {
    if (!window.audioEngine) {
      await initializeRemoteEngine()
    }

    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
      return
    }

    // Check if already running - don't restart to avoid interruptions
    if (state.value.isRunning) {
      console.log('[useAudioEngine] Audio engine already running, skipping start command')
      return
    }

    // Configuration is now loaded automatically by Rust engine:
    // - If no sample_rate/buffer_size passed, Rust loads from audio_config.json
    // - If file doesn't exist, uses defaults (Auto=0/null, 256 frames)
    // - If "Auto" (0), uses device native rate
    // - Otherwise forces specified rate

    startListening()
    await window.audioEngine.start(inputDevice, outputDevice)
  }

  const stop = async () => {
    if (!window.audioEngine) return
    await window.audioEngine.stop()
  }

  const restartWithDevices = async (inputDevice?: string, outputDevice?: string) => {
    if (!window.audioEngine) return

    await stop()
    await new Promise(resolve => setTimeout(resolve, 250))
    await start(inputDevice, outputDevice)
  }

  const setTrackGain = (track: number, gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setGain(track, gain)
  }

  const setTrackVolume = (track: number, volume: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setVolume(track, volume)
  }

  const setTrackMute = (track: number, mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMute(track, mute)
  }

  const setTrackSolo = (track: number, solo: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSolo(track, solo)
  }

  const setTrackRouteToMaster = (track: number, route: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setRouteToMaster(track, route)
  }

  const setTrackCompressor = (
    track: number,
    enabled: boolean,
    threshold: number,
    ratio: number,
    attack: number,
    release: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setCompressor(track, enabled, threshold, ratio, attack, release)
  }

  const setTrackGate = (
    track: number,
    enabled: boolean,
    threshold: number,
    range: number,
    attack: number,
    release: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setGate(track, enabled, threshold, range, attack, release)
  }

  // Track Insert Effects Chain methods
  const addTrackInsert = (track: number, effectType: string, position?: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.addTrackInsert(track, effectType, position)
  }

  const removeTrackInsert = (track: number, insertId: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.removeTrackInsert(track, insertId)
  }

  const moveTrackInsert = (track: number, insertId: number, newPosition: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.moveTrackInsert(track, insertId, newPosition)
  }

  const setTrackInsertEnabled = (track: number, insertId: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertEnabled(track, insertId, enabled)
  }

  const setTrackInsertCompressor = (
    track: number,
    insertId: number,
    threshold: number,
    ratio: number,
    attack: number,
    release: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertCompressor(track, insertId, threshold, ratio, attack, release)
  }

  const setTrackInsertGate = (
    track: number,
    insertId: number,
    threshold: number,
    range: number,
    attack: number,
    release: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertGate(track, insertId, threshold, range, attack, release)
  }

  const setTrackInsertReverb = (
    track: number,
    insertId: number,
    roomSize: number,
    damping: number,
    wet: number,
    width: number,
    preDelay: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertReverb(track, insertId, roomSize, damping, wet, width, preDelay)
  }

  const setTrackInsertDelay = (
    track: number,
    insertId: number,
    timeL: number,
    timeR: number,
    feedback: number,
    mix: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertDelay(track, insertId, timeL, timeR, feedback, mix)
  }

  const setTrackInsertExciter = (
    track: number,
    insertId: number,
    amount: number,
    frequency: number,
    mix: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertExciter(track, insertId, amount, frequency, mix)
  }

  const setTrackInsertDeEsser = (
    track: number,
    insertId: number,
    threshold: number,
    frequency: number,
    range: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertDeEsser(track, insertId, threshold, frequency, range)
  }

  const setTrackInsertChorus = (
    track: number,
    insertId: number,
    rate: number,
    depth: number,
    mix: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackInsertChorus(track, insertId, rate, depth, mix)
  }

  const setTrackSourceInput = (
    track: number,
    leftChannel: number,
    rightChannel: number,
    deviceName?: string | null
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackSourceInput(track, leftChannel, rightChannel, deviceName)
  }

  const setTrackSourceSignal = (track: number, waveform: string, frequency: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackSourceSignal(track, waveform, frequency)
  }

  const setSignalFrequency = (track: number, frequency: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSignalFrequency(track, frequency)
  }

  const setSignalWaveform = (track: number, waveform: string) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSignalWaveform(track, waveform)
  }

  const clearTrackSource = (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.clearTrackSource(track)
  }

  const setTrackSourceFile = (track: number, filePath: string, artist?: string|null, title?: string|null, playlistId?: string|null, playlistName?: string|null, playlistIndex?: number|null) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackSourceFile(track, filePath, artist, title, playlistId, playlistName, playlistIndex)
  }

  const playFile = (track: number, fileId?: string) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.playFile(track, fileId)
  }

  const pauseFile = (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.pauseFile(track)
  }

  const stopFile = (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.stopFile(track)
  }

  const seekFile = (track: number, timeSeconds: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.seekFile(track, timeSeconds)
  }

  const getWaveformData = async (track: number, numPoints: number) => {
    if (!window.audioEngine || !state.value.isRunning) return null
    return await window.audioEngine.getWaveformData(track, numPoints)
  }

  const getEQPresets = async () => {
    if (!window.audioEngine || !state.value.isRunning) return null
    return await window.audioEngine.getEQPresets()
  }

  const applyEQPreset = async (presetName: string) => {
    if (!window.audioEngine || !state.value.isRunning) return
    await window.audioEngine.applyEQPreset(presetName)
  }

  const setTrackPan = (track: number, pan: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setPan(track, pan)
  }

  const setTrackPad = (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackPad(track, enabled)
  }

  const setTrackHPF = (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackHPF(track, enabled)
  }

  const setTrackPhaseInvert = (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackPhaseInvert(track, enabled)
  }

  const setTrackEQ = (track: number, low: number, lowMid: number, highMid: number, high: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setEQ(track, low, lowMid, highMid, high)
  }

  const setTrackEQEnabled = (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setEQEnabled(track, enabled)
  }

  const setParametricEQFilters = (
    track: number,
    filters: Array<{ type: string; frequency: number; gain: number; q: number }>
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setParametricEQFilters(track, filters)
  }

  const setParametricEQEnabled = (track: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setParametricEQEnabled(track, enabled)
  }

  const clearParametricEQ = (track: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.clearParametricEQ(track)
  }

  const setMasterGain = (gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterGain(gain)
  }

  const setMasterGainLeft = (gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterGainLeft(gain)
  }

  const setMasterGainRight = (gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterGainRight(gain)
  }

  const setMasterMute = (mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterMute(mute)
  }

  const setMasterLinked = (linked: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    if (linked === undefined || linked === null) {
      console.error('[useAudioEngine] setMasterLinked called with invalid value:', linked)
      return
    }
    window.audioEngine.setMasterLinked(linked)
  }

  const setMasterOutputChannels = (leftChannel: number, rightChannel: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterOutputChannels(leftChannel, rightChannel)
  }

  const setSelectedMasterOutput = async (deviceId: string | null) => {
    if (!window.audioEngine) return
    await window.audioEngine.setSelectedMasterOutput(deviceId)
  }

  const setSelectedSubgroupOutput = async (subgroup: number, deviceId: string | null) => {
    if (!window.audioEngine) return
    await window.audioEngine.setSelectedSubgroupOutput(subgroup, deviceId)
  }

  const setMasterCompressor = (
    enabled: boolean,
    threshold: number,
    ratio: number,
    attack: number,
    release: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterCompressor(enabled, threshold, ratio, attack, release)
  }

  const setMasterLimiter = (enabled: boolean, ceiling: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterLimiter(enabled, ceiling, release)
  }

  const setMasterDelay = (
    enabled: boolean,
    timeL: number,
    timeR: number,
    feedback: number,
    mix: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterDelay(enabled, timeL, timeR, feedback, mix)
  }

  const setMasterReverb = (
    enabled: boolean,
    roomSize: number,
    damping: number,
    wet: number,
    width: number,
    preDelay: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterReverb(enabled, roomSize, damping, wet, width, preDelay)
  }

  const addMasterFxEffect = async (effectType: string): Promise<void> => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    try {
      await window.audioEngine.addMasterFxEffect(effectType)
      console.log(`[useAudioEngine] Added Master FX effect: ${effectType}`)
    } catch (error) {
      console.error(`[useAudioEngine] Failed to add Master FX effect ${effectType}:`, error)
      throw error
    }
  }

  const removeMasterFxEffect = async (effectType: string): Promise<void> => {
    if (!window.audioEngine || !state.value.isRunning) return
    
    try {
      await window.audioEngine.removeMasterFxEffect(effectType)
      console.log(`[useAudioEngine] Removed Master FX effect: ${effectType}`)
    } catch (error) {
      console.error(`[useAudioEngine] Failed to remove Master FX effect ${effectType}:`, error)
      throw error
    }
  }

  const addSubgroup = async (): Promise<number | null> => {
    if (!window.audioEngine) return null

    try {
      const id = await window.audioEngine.addSubgroup()
      return id
    } catch (error) {
      console.error('[useAudioEngine] Failed to add subgroup:', error)
      return null
    }
  }

  const removeSubgroup = (subgroup: number) => {
    if (!window.audioEngine) return
    window.audioEngine.removeSubgroup(subgroup)
  }

  const setSubgroupGain = (subgroup: number, gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSubgroupGain(subgroup, gain)
  }

  const setSubgroupMute = (subgroup: number, mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSubgroupMute(subgroup, mute)
  }

  const setSubgroupOutputEnabled = (subgroup: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSubgroupOutputEnabled(subgroup, enabled)
  }

  const setSubgroupRouteToMaster = (subgroup: number, route: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSubgroupRouteToMaster(subgroup, route)
  }

  const setSubgroupOutputChannels = (subgroup: number, leftChannel: number, rightChannel: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setSubgroupOutputChannels(subgroup, leftChannel, rightChannel)
  }

  const setTrackRouteToSubgroup = (track: number, subgroup: number, route: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackRouteToSubgroup(track, subgroup, route)
  }

  const setTrackAuxSend = (
    track: number,
    aux: number,
    level: number,
    preFader: boolean,
    muted: boolean
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackAuxSend(track, aux, level, preFader, muted)
  }

  const setAuxBusGain = (aux: number, gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusGain(aux, gain)
  }

  const setAuxBusMute = (aux: number, mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusMute(aux, mute)
  }

  const setAuxBusReverb = (
    aux: number,
    enabled: boolean,
    roomSize: number,
    damping: number,
    wet: number,
    width: number,
    preDelay: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusReverb(aux, enabled, roomSize, damping, wet, width, preDelay)
  }

  const setAuxBusDelay = (
    aux: number,
    enabled: boolean,
    time: number,
    feedback: number,
    mix: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusDelay(aux, enabled, time, feedback, mix)
  }

  const setAuxBusRouteToMaster = (aux: number, route: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusRouteToMaster(aux, route)
  }

  const setAuxBusOutputEnabled = (aux: number, enabled: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusOutputEnabled(aux, enabled)
  }

  const setAuxBusOutputChannels = (aux: number, leftChannel: number, rightChannel: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusOutputChannels(aux, leftChannel, rightChannel)
  }

  const setAuxBusRouteToSubgroup = (aux: number, subgroup: number, route: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusRouteToSubgroup(aux, subgroup, route)
  }

  const setAuxBusSelectedOutput = (aux: number, deviceId: string | null) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusSelectedOutput(aux, deviceId)
  }

  const setTrackSourceAuxReturn = (track: number, aux: number) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackSourceAuxReturn(track, aux)
  }

  const getInputDevices = () => {
    return state.value.devices.filter(d => d.input_channels > 0)
  }

  const getOutputDevices = () => {
    return state.value.devices.filter(d => d.output_channels > 0)
  }

  const getLoudness = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.getLoudness()
  }

  const resetLoudness = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.resetLoudness()
  }

  const getDynamicRange = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.getDynamicRange()
  }

  const resetDynamicRange = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.resetDynamicRange()
  }

  const getPhaseCorrelation = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.getPhaseCorrelation()
  }

  const resetPhaseCorrelation = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.resetPhaseCorrelation()
  }

  const getStereoWidth = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.getStereoWidth()
  }

  const resetStereoWidth = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.resetStereoWidth()
  }

  const getHeadroom = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.getHeadroom()
  }

  const resetHeadroom = () => {
    if (!window.audioEngine || !state.value.isRunning) return
    void window.audioEngine.resetHeadroom()
  }

  const saveLicense = async (key: string, licenseType: string, expiresAt: string | null) => {
    if (!window.audioEngine) return
    console.log('[useAudioEngine] Saving license with key:', key, 'type:', licenseType, 'expiresAt:', expiresAt)
    await window.audioEngine.saveLicense(key, licenseType, expiresAt)
  }

  const getLicense = async () => {
    if (!window.audioEngine) return null
    return await window.audioEngine.getLicense()
  }

  // Only register onUnmounted if called within a component context
  if (getCurrentInstance()) {
    onUnmounted(() => {
      // Clean up parameter update timer
      if (parameterUpdateTimer) {
        clearInterval(parameterUpdateTimer)
        parameterUpdateTimer = null
      }
      
      if (state.value.isRunning) {
        void stop()
      }
    })
  }

  return {
    state,
    isRemoteControlActive,
    loadDevices,
    start,
    stop,
    restartWithDevices,
    setTrackGain,
    setTrackVolume,
    setTrackMute,
    setTrackSolo,
    setTrackRouteToMaster,
    setTrackEQ,
    setTrackEQEnabled,
    setParametricEQFilters,
    setParametricEQEnabled,
    clearParametricEQ,
    setTrackCompressor,
    setTrackGate,
    addTrackInsert,
    removeTrackInsert,
    moveTrackInsert,
    setTrackInsertEnabled,
    setTrackInsertCompressor,
    setTrackInsertGate,
    setTrackInsertReverb,
    setTrackInsertDelay,
    setTrackInsertExciter,
    setTrackInsertDeEsser,
    setTrackInsertChorus,
    setTrackSourceInput,
    setTrackSourceSignal,
    setSignalFrequency,
    setSignalWaveform,
    clearTrackSource,
    setTrackSourceFile,
    playFile,
    pauseFile,
    stopFile,
    seekFile,
    getWaveformData,
    getEQPresets,
    applyEQPreset,
    setTrackPan,
    setTrackPad,
    setTrackHPF,
    setTrackPhaseInvert,
    setMasterGain,
    setMasterGainLeft,
    setMasterGainRight,
    setMasterMute,
    setMasterLinked,
    setMasterOutputChannels,
    setSelectedMasterOutput,
    setSelectedSubgroupOutput,
    setMasterCompressor,
    setMasterLimiter,
    setMasterDelay,
    setMasterReverb,
    addMasterFxEffect,
    removeMasterFxEffect,
    addSubgroup,
    removeSubgroup,
    setSubgroupGain,
    setSubgroupMute,
    setSubgroupOutputEnabled,
    setSubgroupRouteToMaster,
    setSubgroupOutputChannels,
    setTrackRouteToSubgroup,
    setTrackAuxSend,
    setAuxBusGain,
    setAuxBusMute,
    setAuxBusReverb,
    setAuxBusDelay,
    setAuxBusRouteToMaster,
    setAuxBusOutputEnabled,
    setAuxBusOutputChannels,
    setAuxBusRouteToSubgroup,
    setAuxBusSelectedOutput,
    setTrackSourceAuxReturn,
    getInputDevices,
    getOutputDevices,
    showOpenFileDialog: () => window.audioEngine.showOpenFileDialog(),
    readFileAsBuffer: (filePath: string) => window.audioEngine.readFileAsBuffer(filePath),
    startNdi: (streamName: string, source: string) => window.audioEngine.startNdi(streamName, source),
    stopNdi: () => window.audioEngine.stopNdi(),
    setNdiSource: (source: string) => window.audioEngine.setNdiSource(source),
    setNdiName: (name: string) => window.audioEngine.setNdiName(name),
    setNdiVideoText: (text: string) => window.audioEngine.setNdiVideoText(text),
    getLoudness,
    resetLoudness,
    getDynamicRange,
    resetDynamicRange,
    getPhaseCorrelation,
    resetPhaseCorrelation,
    getStereoWidth,
    resetStereoWidth,
    getHeadroom,
    resetHeadroom,
    saveLicense,
    getLicense
  }
}