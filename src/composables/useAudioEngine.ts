import { ref, onUnmounted } from 'vue'

export interface AudioDevice {
  id: string
  name: string
  input_channels: number
  output_channels: number
}

export interface AudioEngineState {
  isRunning: boolean
  devices: AudioDevice[]
  selectedInputDevice: string | null
  selectedOutputDevice: string | null
  trackLevels: Map<number, { 
    left: number, 
    right: number,
    phaseCorrelation: number,
    compressorInputDb: number,
    compressorReductionDb: number,
    gateInputDb: number,
    gateAttenuationDb: number
  }>
  trackWaveforms: Map<number, number[]>
  subgroupLevels: Map<number, { left: number, right: number }>
  masterLevels: { left: number, right: number }
  fftData: { binsLeft: Float32Array, binsRight: Float32Array, sampleRate: number } | null
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
}

const state = ref<AudioEngineState>({
  isRunning: false,
  devices: [],
  selectedInputDevice: null,
  selectedOutputDevice: null,
  trackLevels: new Map(),
  trackWaveforms: new Map(),
  subgroupLevels: new Map(),
  masterLevels: { left: -60, right: -60 },
  fftData: null,
  performanceStats: null,
  recordingStats: null,
  loudnessData: null,
  dynamicRangeData: null
})

let isListening = false

export const useAudioEngine = () => {
  // Start listening immediately
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
          // Command acknowledged successfully (silent)
          break

        case 'error':
          console.error('[useAudioEngine] Engine error:', response.message)
          break

        case 'levels':
          // Update track levels and waveforms
          if (response.tracks) {
            response.tracks.forEach((trackLevel: any) => {
              state.value.trackLevels.set(trackLevel.track, {
                left: trackLevel.level_l,
                right: trackLevel.level_r,
                phaseCorrelation: trackLevel.phase_correlation || 0,
                compressorInputDb: trackLevel.compressor_input_db || -90,
                compressorReductionDb: trackLevel.compressor_reduction_db || 0,
                gateInputDb: trackLevel.gate_input_db || -90,
                gateAttenuationDb: trackLevel.gate_attenuation_db || 0
              })

              // Update waveform data if present
              if (trackLevel.waveform) {
                state.value.trackWaveforms.set(trackLevel.track, trackLevel.waveform)
              }
            })
          }
          // Update subgroup levels - Create new Map to trigger reactivity
          if (response.subgroups) {
            const newSubgroupLevels = new Map(state.value.subgroupLevels)
            response.subgroups.forEach((subgroupLevel: any) => {
              // Convert linear levels (0.0-1.0) to dB (-90 to 0)
              const leftDb = subgroupLevel.level_l > 0.0 
                ? 20 * Math.log10(subgroupLevel.level_l) 
                : -90
              const rightDb = subgroupLevel.level_r > 0.0 
                ? 20 * Math.log10(subgroupLevel.level_r) 
                : -90
              
              newSubgroupLevels.set(subgroupLevel.subgroup, {
                left: leftDb,
                right: rightDb
              })
            })
            state.value.subgroupLevels = newSubgroupLevels
          }
          // Update master levels - convert linear (0-1) to dB (-60 to 0)
          if (response.master_l !== undefined && response.master_r !== undefined) {
            state.value.masterLevels = {
              left: response.master_l > 0 ? 20 * Math.log10(response.master_l) : -60,
              right: response.master_r > 0 ? 20 * Math.log10(response.master_r) : -60
            }
          }
          break

        case 'fft':
          // Update FFT data for spectrum analyzer
          if (response.bins_left && response.bins_right && response.sample_rate) {
            state.value.fftData = {
              binsLeft: new Float32Array(response.bins_left),
              binsRight: new Float32Array(response.bins_right),
              sampleRate: response.sample_rate
            }
          }
          break

        case 'performance':
          // Update performance stats
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
          // Update recording stats from Rust
          state.value.recordingStats = {
            elapsedSeconds: response.elapsed_seconds,
            fileSizeBytes: response.file_size_bytes,
            availableSpaceGb: response.available_space_gb
          }
          break

        case 'loudness':
          // Update loudness measurements (EBU R128)
          state.value.loudnessData = {
            momentaryLufs: response.momentary_lufs,
            shortTermLufs: response.short_term_lufs,
            integratedLufs: response.integrated_lufs,
            loudnessRangeLu: response.loudness_range_lu,
            truePeakDbtp: response.true_peak_dbtp
          }
          break

        case 'dynamic_range':
          // Update dynamic range measurements
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

        default:
          console.log('[useAudioEngine] Unhandled response type:', response.type)
      }
    })

    isListening = true
  }

  // Start listening as soon as the composable is created
  if (window.audioEngine && !isListening) {
    startListening()
  }

  const loadDevices = async () => {
    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
      return
    }

    startListening()
    await window.audioEngine.listDevices()
  }

  const start = async (inputDevice?: string, outputDevice?: string) => {
    if (!window.audioEngine) {
      console.warn('[useAudioEngine] Audio engine API not available')
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
    // Increased delay to ensure streams are fully closed before restart
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

  const setTrackCompressor = (track: number, enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setCompressor(track, enabled, threshold, ratio, attack, release)
  }

  const setTrackGate = (track: number, enabled: boolean, threshold: number, range: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setGate(track, enabled, threshold, range, attack, release)
  }

  // Track source selection
  const setTrackSourceInput = (track: number, leftChannel: number, rightChannel: number, deviceName?: string | null) => {
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

  const setTrackSourceFile = (track: number, filePath: string) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setTrackSourceFile(track, filePath)
  }

  // File playback controls
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

  // Parametric EQ controls
  const setParametricEQFilters = (track: number, filters: Array<{ type: string, frequency: number, gain: number, q: number }>) => {
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

  // Master controls
  const setMasterGain = (gain: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterGain(gain)
  }

  const setMasterMute = (mute: boolean) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterMute(mute)
  }

  const setMasterOutputChannels = (leftChannel: number, rightChannel: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterOutputChannels(leftChannel, rightChannel)
  }

  // Master FX controls
  const setMasterCompressor = (enabled: boolean, threshold: number, ratio: number, attack: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterCompressor(enabled, threshold, ratio, attack, release)
  }

  const setMasterLimiter = (enabled: boolean, ceiling: number, release: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterLimiter(enabled, ceiling, release)
  }

  const setMasterDelay = (enabled: boolean, timeL: number, timeR: number, feedback: number, mix: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterDelay(enabled, timeL, timeR, feedback, mix)
  }

  const setMasterReverb = (enabled: boolean, roomSize: number, damping: number, wet: number, width: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setMasterReverb(enabled, roomSize, damping, wet, width)
  }

  // Subgroup methods
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

  // Aux bus methods
  const setTrackAuxSend = (track: number, aux: number, level: number, preFader: boolean, muted: boolean) => {
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

  const setAuxBusReverb = (aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number) => {
    if (!window.audioEngine || !state.value.isRunning) return

    window.audioEngine.setAuxBusReverb(aux, enabled, roomSize, damping, wet, width)
  }

  const setAuxBusDelay = (aux: number, enabled: boolean, time: number, feedback: number, mix: number) => {
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

  onUnmounted(() => {
    if (state.value.isRunning) {
      stop()
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
    setMasterMute,
    setMasterOutputChannels,
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
    // NDI Streaming
    startNdi: (streamName: string, source: string) => window.audioEngine.startNdi(streamName, source),
    stopNdi: () => window.audioEngine.stopNdi(),
    setNdiSource: (source: string) => window.audioEngine.setNdiSource(source),
    setNdiName: (name: string) => window.audioEngine.setNdiName(name),
    setNdiVideoText: (text: string) => window.audioEngine.setNdiVideoText(text),
    // Loudness metering
    getLoudness,
    resetLoudness,
    // Dynamic Range metering
    getDynamicRange,
    resetDynamicRange
  }
}
