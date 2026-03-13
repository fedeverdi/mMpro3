/**
 * Detached Window Audio Engine Proxy
 * Provides a minimal audio engine interface for detached windows
 * Sends commands via WebSocket instead of direct IPC
 */

import { ref } from 'vue'
import type { DetachedWindowClient } from './detachedWindowClient'

export class DetachedAudioEngineProxy {
  // Minimal state object for compatibility (read-only in detached windows)
  public state = ref({
    isRunning: true, // Assume engine is running in detached windows
    availableOutputDevices: []
  })

  constructor(private wsClient: DetachedWindowClient) {}

  // Master EQ methods
  async setMasterParametricEQFilters(filters: Array<{type: string, frequency: number, gain: number, q: number}>): Promise<void> {
    return this.wsClient.send({ 
      type: 'set_master_parametric_eq_filters', 
      filters 
    })
  }

  async setMasterParametricEQEnabled(enabled: boolean): Promise<void> {
    return this.wsClient.send({ 
      type: 'set_master_parametric_eq_enabled', 
      enabled 
    })
  }

  async clearMasterParametricEQ(): Promise<void> {
    return this.wsClient.send({ 
      type: 'clear_master_parametric_eq' 
    })
  }

  // Master volume/mute
  async setMasterGain(gain: number): Promise<void> {
    return this.wsClient.send({ 
      type: 'set_master_gain', 
      gain 
    })
  }

  async setMasterMute(mute: boolean): Promise<void> {
    return this.wsClient.send({ 
      type: 'set_master_mute', 
      mute 
    })
  }

  // Aux methods
  async setAuxBusGain(aux: number, gain: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_gain',
      aux,
      gain
    })
  }

  async setAuxBusMute(aux: number, mute: boolean): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_mute',
      aux,
      mute
    })
  }

  async setAuxBusReverb(aux: number, enabled: boolean, roomSize: number, damping: number, wet: number, width: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_reverb',
      aux,
      enabled,
      room_size: roomSize,
      damping,
      wet,
      width
    })
  }

  async setAuxBusDelay(aux: number, enabled: boolean, time: number, feedback: number, mix: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_delay',
      aux,
      enabled,
      time,
      feedback,
      mix
    })
  }

  async setAuxBusRouteToMaster(aux: number, route: boolean): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_route_to_master',
      aux,
      route
    })
  }

  async setAuxBusRouteToSubgroup(aux: number, subgroup: number, route: boolean): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_route_to_subgroup',
      aux,
      subgroup,
      route
    })
  }

  async setAuxBusSelectedOutput(aux: number, deviceId: string | null): Promise<void> {
    return this.wsClient.send({
      type: 'set_selected_aux_bus_output',
      aux,
      device_id: deviceId
    })
  }

  async setAuxBusOutputChannels(aux: number, leftChannel: number, rightChannel: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_aux_bus_output_channels',
      aux,
      left_channel: leftChannel,
      right_channel: rightChannel
    })
  }

  // Master FX methods
  async setMasterCompressor(enabled: boolean, threshold: number, ratio: number, attack: number, release: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_master_compressor',
      enabled,
      threshold,
      ratio,
      attack,
      release
    })
  }

  async setMasterLimiter(enabled: boolean, threshold: number, release: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_master_limiter',
      enabled,
      threshold,
      release
    })
  }

  async setMasterDelay(enabled: boolean, delayTimeL: number, delayTimeR: number, feedback: number, mix: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_master_delay',
      enabled,
      time_l: delayTimeL,
      time_r: delayTimeR,
      feedback,
      mix
    })
  }

  async setMasterReverb(enabled: boolean, roomSize: number, damping: number, wet: number, width: number): Promise<void> {
    return this.wsClient.send({
      type: 'set_master_reverb',
      enabled,
      room_size: roomSize,
      damping,
      wet,
      width
    })
  }

  async addMasterFxEffect(effectType: string): Promise<void> {
    return this.wsClient.send({
      type: 'add_master_fx_effect',
      effect_type: effectType
    })
  }

  async removeMasterFxEffect(effectType: string): Promise<void> {
    return this.wsClient.send({
      type: 'remove_master_fx_effect',
      effect_type: effectType
    })
  }
}
