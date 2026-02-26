import { ref, toRaw, type Ref } from 'vue'

export interface RoutingState {
  routeToMaster: Ref<boolean>
  routedSubgroups: Ref<Set<number>>
}

export interface RoutingCallbacks {
  getVolumeMerge?: () => any
  getMasterChannel?: () => any
  getSubgroups?: () => Array<{ id: number, channel: any }> | undefined
  getTone?: () => any
  getTrackNumber?: () => number
}

export function useTrackRouting(callbacks: RoutingCallbacks = {}) {
  // State
  const routeToMaster = ref(true)
  const routedSubgroups = ref<Set<number>>(new Set()) // Set of subgroup IDs

  /**
   * Connect track output to master and/or subgroups
   */
  function connectToOutput() {
    const volumeMerge = callbacks.getVolumeMerge?.()
    const Tone = callbacks.getTone?.()
    
    if (!volumeMerge || !Tone) return false

    const masterChannel = callbacks.getMasterChannel?.()
    const subgroups = callbacks.getSubgroups?.()

    // Disconnect only from master and subgroups (preserve aux sends)
    if (masterChannel) {
      try {
        volumeMerge.disconnect(toRaw(masterChannel))
      } catch (e) {
        // Ignore if not connected
      }
    }

    // Disconnect from all subgroups
    subgroups?.forEach(subgroup => {
      if (subgroup?.channel) {
        try {
          volumeMerge.disconnect(toRaw(subgroup.channel))
        } catch (e) {
          // Ignore if not connected
        }
      }
    })

    // Connect to master if enabled
    if (routeToMaster.value && masterChannel) {
      volumeMerge.connect(toRaw(masterChannel))
    }

    // Connect to each enabled subgroup
    routedSubgroups.value.forEach(subgroupId => {
      const subgroup = subgroups?.find(s => s.id === subgroupId)
      if (subgroup?.channel) {
        volumeMerge.connect(toRaw(subgroup.channel))
      }
    })

    // Warn if no output is selected
    if (!routeToMaster.value && routedSubgroups.value.size === 0) {
      const trackNumber = callbacks.getTrackNumber?.() || '?'
      console.warn(`[Track ${trackNumber}] No output destination selected`)
    }
  }

  /**
   * Toggle routing to master
   */
  function toggleRouteToMaster() {
    routeToMaster.value = !routeToMaster.value
    connectToOutput()
  }

  /**
   * Toggle routing to a specific subgroup
   */
  function toggleRouteToSubgroup(subgroupId: number) {
    if (routedSubgroups.value.has(subgroupId)) {
      routedSubgroups.value.delete(subgroupId)
    } else {
      routedSubgroups.value.add(subgroupId)
    }
    routedSubgroups.value = new Set(routedSubgroups.value) // Trigger reactivity
    connectToOutput()
  }

  /**
   * Disconnect from a specific subgroup (called when subgroup is removed)
   */
  function disconnectFromSubgroup(subgroupId: number) {
    if (routedSubgroups.value.has(subgroupId)) {
      routedSubgroups.value.delete(subgroupId)
      routedSubgroups.value = new Set(routedSubgroups.value)
      connectToOutput()
    }
  }

  return {
    // State
    routeToMaster,
    routedSubgroups,

    // Functions
    connectToOutput,
    toggleRouteToMaster,
    toggleRouteToSubgroup,
    disconnectFromSubgroup
  }
}
