import { ref, onUnmounted } from 'vue'
import { RemoteAudioEngine } from '~/lib/remoteAudioEngine'

export interface AudioDevice {
  id: string
  name: string
  input_channels: number
  output_channels: number
  default_sample_rate: number
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
    pan: number
    routeToMaster: boolean
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
  }>
  subgroupLevels: Map<number, { 
    left: number; 
    right: number;
    gain: number;
    mute: boolean;
    routeToMaster: boolean;
    selectedOutput?: string | null;
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
  fftData: { binsLeft: Float32Array; binsRight: Float32Array; sampleRate: number } | null
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
  masterLevels: { left: -60, right: -60, gain: 1.0, gainLeft: 1.0, gainRight: 1.0, mute: false, linked: true, selectedMasterOutput: null },
  masterEQFilters: [],
  fftData: null,
  performanceStats: null,
  recordingStats: null,
  loudnessData: null,
  dynamicRangeData: null,
  phaseCorrelationData: null,
  stereoWidthData: null,
  headroomData: null
})

let isListening = false
let remoteEngineInitialized = false

export const useAudioEngine = () => {
  const initializeRemoteEngine = async () => {
    if (remoteEngineInitialized || window.audioEngine) {
      return
    }

    console.log('[useAudioEngine] Detected browser mode - initializing remote audio engine')

    try {
      const host = window.location.hostname
      const remoteEngine = new RemoteAudioEngine(host, 3001)
      await remoteEngine.connect()

      ;(window as any).audioEngine = remoteEngine
      remoteEngineInitialized = true

      console.log('[useAudioEngine] Remote audio engine connected successfully')
    } catch (error) {
      console.error('[useAudioEngine] Failed to connect to remote audio engine:', error)
      throw new Error('Failed to connect to remote audio engine. Make sure the Electron app is running.')
    }
  }

  const startListening = () => {
    if (isListening || !window.audioEngine) return

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
          break

        case 'ok':
          break

        case 'error':
          console.error('[useAudioEngine] Engine error:', response.message)
          break

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
                state.value.trackParameters.set(trackLevel.track, {
                  gain: trackLevel.gain,
                  volume: trackLevel.volume,
                  mute: trackLevel.mute,
                  pan: trackLevel.pan,
                  routeToMaster: trackLevel.route_to_master,
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
                  isPlaying: trackLevel.is_playing ?? false
                })
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

  const setTrackSourceFile = (track: number, filePath: string, artist?: string|null, title?: string|null) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setTrackSourceFile(track, filePath, artist, title)
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
    width: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setMasterReverb(enabled, roomSize, damping, wet, width)
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
    width: number
  ) => {
    if (!window.audioEngine || !state.value.isRunning) return
    window.audioEngine.setAuxBusReverb(aux, enabled, roomSize, damping, wet, width)
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

  onUnmounted(() => {
    if (state.value.isRunning) {
      void stop()
    }
  })

  return {
    state,
    loadDevices,
    start,
    stop,
    restartWithDevices,
    setTrackGain,
    setTrackVolume,
    setTrackMute,
    setTrackRouteToMaster,
    setTrackEQ,
    setTrackEQEnabled,
    setParametricEQFilters,
    setParametricEQEnabled,
    clearParametricEQ,
    setTrackCompressor,
    setTrackGate,
    setTrackSourceInput,
    setTrackSourceSignal,
    setSignalFrequency,
    setSignalWaveform,
    clearTrackSource,
    setTrackSourceFile,
    playFile,
    pauseFile,
    stopFile,
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