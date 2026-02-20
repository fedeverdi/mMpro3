<template>
  <div class="master-fx bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-green-600/60 p-3 flex flex-col gap-2 ">
    <div class="text-left flex-shrink-0">
      <p class="text-sm font-bold text-green-200 tracking-wide uppercase">Master FX Chain</p>
      <p class="text-[10px] text-gray-400">Click + to add effects</p>
    </div>

    <div class="flex-1 h-full  grid grid-cols-4 gap-3 min-h-0 pr-1 custom-scrollbar overflow-y-none">
      <!-- Existing effects -->
      <div v-for="(effect, index) in effects" :key="effect.id" class="relative h-full aspect-square" style="overflow: visible;">
        <!-- Remove button -->
        <button 
          @click="removeEffect(index)"
          class="absolute -top-3 pt-0.5 -right-3 w-7 h-7 bg-gray-600/70 hover:bg-gray-500/80 rounded-full text-gray-300 hover:text-gray-100 font-bold flex items-center justify-center transition-colors shadow-lg border-2 border-gray-700"
          style="z-index: 50;"
          title="Remove effect"
        >
          <!-- X in svg -->
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-3.5 h-3.5">
            <path fill-rule="evenodd" d="M6.225 4.811a.75.75 0 011.06 0L12 9.525l4.715-4.714a.75.75 0 111.06 1.06L13.06 10.586l4.715 4.715a.75.75 0 11-1.06 1.06L12 11.646l-4.715 4.715a.75.75 0 11-1.06-1.06l4.715-4.715-4.715-4.715a.75.75 0 010-1.06z" clip-rule="evenodd" />
          </svg>
        </button>

        <!-- Compressor Effect -->
        <CompressorEffect 
          v-if="effect.type === 'compressor'"
          :enabled="effect.enabled"
          :initial-threshold="effect.params.threshold"
          :initial-ratio="effect.params.ratio"
          :initial-attack="effect.params.attack"
          :initial-release="effect.params.release"
          :effect-node="effect.node"
          :master-section="masterSection"
          @toggle="(enabled) => handleEffectToggle(index, enabled)"
          @update="(params) => handleEffectUpdate(index, params)"
        />

        <!-- Reverb Effect -->
        <ReverbEffect 
          v-else-if="effect.type === 'reverb'"
          :enabled="effect.enabled"
          :initial-decay="effect.params.decay"
          :initial-pre-delay="effect.params.preDelay"
          :initial-wet="effect.params.wet"
          :effect-node="effect.node"
          @toggle="(enabled) => handleEffectToggle(index, enabled)"
          @update="(params) => handleEffectUpdate(index, params)"
        />

        <!-- Delay Effect -->
        <DelayEffect 
          v-else-if="effect.type === 'delay'"
          :enabled="effect.enabled"
          :initial-delay-time="effect.params.delayTime"
          :initial-feedback="effect.params.feedback"
          :initial-wet="effect.params.wet"
          :effect-node="effect.node"
          @toggle="(enabled) => handleEffectToggle(index, enabled)"
          @update="(params) => handleEffectUpdate(index, params)"
        />

        <!-- Limiter Effect -->
        <LimiterEffect 
          v-else-if="effect.type === 'limiter'"
          :enabled="effect.enabled"
          :initial-threshold="effect.params.threshold"
          :effect-node="effect.node"
          :master-section="masterSection"
          @toggle="(enabled) => handleEffectToggle(index, enabled)"
          @update="(params) => handleEffectUpdate(index, params)"
        />
      </div>

      <!-- Add effect button -->
      <div 
      v-if="effects.length < 4"
        ref="addEffectButton"
        class="relative h-full border-2 border-dashed border-gray-600 aspect-square rounded-lg p-4 hover:border-green-500 transition-colors cursor-pointer bg-gray-800/50 hover:bg-gray-800 flex items-center justify-center h-full"
        @click="toggleAddEffectMenu"
      >
        <div class="text-center">
          <div class="text-3xl text-gray-400 hover:text-green-400 transition-colors">+</div>
          <div class="text-xs text-gray-500 mt-1">Add Effect</div>
        </div>
      </div>
    </div>

    <!-- Add effect dropdown menu (outside scrollable area) -->
    <Teleport to="body">
      <div 
        v-if="showAddEffectMenu"
        class="fixed inset-0 z-[9998]"
        @click="showAddEffectMenu = false"
      >
        <div 
          class="absolute bg-gray-800 border border-gray-600 rounded-lg shadow-xl overflow-hidden min-w-[200px]"
          :style="{
            top: `${menuPosition.top}px`,
            left: `${menuPosition.left}px`,
            zIndex: 9999
          }"
          @click.stop
        >
          <button
            @click="addEffect('compressor')"
            class="w-full px-4 py-3 text-left text-sm hover:bg-gray-700 transition-colors border-b border-gray-700 flex items-center gap-2"
          >
            <span class="text-blue-400 font-bold">CO</span>
            <span>Compressor</span>
          </button>
          <button
            @click="addEffect('reverb')"
            class="w-full px-4 py-3 text-left text-sm hover:bg-gray-700 transition-colors border-b border-gray-700 flex items-center gap-2"
          >
            <span class="text-green-400 font-bold">RE</span>
            <span>Reverb</span>
          </button>
          <button
            @click="addEffect('delay')"
            class="w-full px-4 py-3 text-left text-sm hover:bg-gray-700 transition-colors border-b border-gray-700 flex items-center gap-2"
          >
            <span class="text-purple-400 font-bold">DL</span>
            <span>Delay</span>
          </button>
          <button
            @click="addEffect('limiter')"
            class="w-full px-4 py-3 text-left text-sm hover:bg-gray-700 transition-colors flex items-center gap-2"
          >
            <span class="text-red-400 font-bold">LI</span>
            <span>Limiter</span>
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, inject, toRaw } from 'vue'
import CompressorEffect from '../fx/CompressorEffect.vue'
import ReverbEffect from '../fx/ReverbEffect.vue'
import DelayEffect from '../fx/DelayEffect.vue'
import LimiterEffect from '../fx/LimiterEffect.vue'

interface Props {
  masterEqDisplay?: any  // To get inputNode from MasterEQDisplay
  masterSection?: any    // For meter visualization in CompressorEffect and LimiterEffect
}

const props = defineProps<Props>()

// Inject Tone.js
const ToneRef = inject<any>('Tone')
let Tone: any = null

// Effect types
type EffectType = 'compressor' | 'reverb' | 'delay' | 'limiter'

interface Effect {
  id: number
  type: EffectType
  enabled: boolean
  params: any
  node?: any
}

// State
const effects = ref<Effect[]>([])
const showAddEffectMenu = ref(false)
const addEffectButton = ref<HTMLElement | null>(null)
const menuPosition = ref({ top: 0, left: 0 })
let nextEffectId = 1

// Audio nodes
let inputNode: any = null   // Input from MasterEQDisplay
let outputNode: any = null  // Output to MasterSection

// FX nodes
let compressorNode: any = null
let reverbNode: any = null
let delayNode: any = null
let limiterNode: any = null
const fxChainEnabled = new Set<string>()
let isRegeneratingReverb = false
let isUpdatingLimiter = false

// FX parameters (for scene snapshots)
const compressorParams = ref({
  threshold: -40,
  ratio: 2,
  attack: 0.01,
  release: 0.25
})
const reverbParams = ref({
  decay: 1.5,
  preDelay: 0.01,
  wet: 0.3
})
const delayParams = ref({
  delayTime: 0.25,
  feedback: 0.15,
  wet: 0.15
})
const limiterParams = ref({
  threshold: -1
})

// Toggle add effect menu with position calculation
const toggleAddEffectMenu = () => {
  if (!showAddEffectMenu.value && addEffectButton.value) {
    const rect = addEffectButton.value.getBoundingClientRect()
    // Position menu above and aligned to the right of the button
    const menuHeight = 192 // 4 buttons × ~48px each
    menuPosition.value = {
      top: rect.top - menuHeight - 8,
      left: rect.right - 170 // 200px is min-w-[200px] of the menu
    }
  }
  showAddEffectMenu.value = !showAddEffectMenu.value
}

// Default parameters for each effect type
const defaultParams = {
  compressor: {
    threshold: -24,
    ratio: 4,
    attack: 0.003,
    release: 0.25
  },
  reverb: {
    decay: 1.5,
    preDelay: 0.01,
    wet: 0.3
  },
  delay: {
    delayTime: 0.25,
    feedback: 0.5,
    wet: 0.3
  },
  limiter: {
    threshold: -1
  }
}

// Initialize audio nodes
onMounted(async () => {
  // Get Tone.js
  if (ToneRef?.value) {
    Tone = ToneRef.value
  } else {
    // Wait for Tone to be available
    await new Promise<void>((resolve) => {
      const checkTone = setInterval(() => {
        if (ToneRef?.value) {
          Tone = ToneRef.value
          clearInterval(checkTone)
          resolve()
        }
      }, 100)
    })
  }

  if (!Tone) return

  // Create output node
  outputNode = new Tone.Gain(1)

  // Wait for MasterEQDisplay to be ready
  if (props.masterEqDisplay?.getOutputNode) {
    inputNode = props.masterEqDisplay.getOutputNode()
    // Initial connection: input -> output (no FX)
    if (inputNode) {
      inputNode.connect(outputNode)
    }
  }
})

// Watch for MasterEQDisplay to become available
watch(() => props.masterEqDisplay, (newVal) => {
  if (newVal && newVal.getOutputNode && Tone && outputNode) {
    // Try to get input node
    inputNode = newVal.getOutputNode()
    
    if (inputNode) {
      // Rebuild chain with new input
      rebuildFXChain()
    } else {
      // OutputNode might not be created yet, retry after a delay
      setTimeout(() => {
        if (newVal && newVal.getOutputNode) {
          inputNode = newVal.getOutputNode()
          if (inputNode) {
            rebuildFXChain()
          }
        }
      }, 150)
    }
  }
}, { immediate: true })

// Cleanup
onUnmounted(() => {
  if (outputNode) outputNode.dispose()
  if (compressorNode) compressorNode.dispose()
  if (reverbNode) reverbNode.dispose()
  if (delayNode) delayNode.dispose()
  if (limiterNode) limiterNode.dispose()
})

// Rebuild FX chain
function rebuildFXChain() {
  if (!inputNode || !outputNode || !Tone) return

  // Disconnect inputNode from everything
  try {
    inputNode.disconnect()
  } catch (e) {}

  // Disconnect all FX nodes
  if (compressorNode) {
    try { compressorNode.disconnect() } catch (e) {}
  }
  if (reverbNode) {
    try { reverbNode.disconnect() } catch (e) {}
  }
  if (delayNode) {
    try { delayNode.disconnect() } catch (e) {}
  }
  if (limiterNode) {
    try { limiterNode.disconnect() } catch (e) {}
  }

  // Build chain: input → [enabled FX] → output
  let currentNode = inputNode

  // If no FX enabled, direct connection
  if (fxChainEnabled.size === 0) {
    currentNode.connect(outputNode)
    return
  }

  // Chain order: Compressor → Reverb → Delay → Limiter
  if (fxChainEnabled.has('compressor') && compressorNode) {
    currentNode.connect(compressorNode)
    currentNode = compressorNode
  }

  if (fxChainEnabled.has('reverb') && reverbNode) {
    currentNode.connect(reverbNode)
    currentNode = reverbNode
  }

  if (fxChainEnabled.has('delay') && delayNode) {
    currentNode.connect(delayNode)
    currentNode = delayNode
  }

  if (fxChainEnabled.has('limiter') && limiterNode) {
    currentNode.connect(limiterNode)
    currentNode = limiterNode
  }

  // Final connection to output
  currentNode.connect(outputNode)
}

// Add effect to chain
function addEffect(type: EffectType) {
  const newEffect: Effect = {
    id: nextEffectId++,
    type,
    enabled: false,
    params: { ...defaultParams[type] },
    node: null
  }
  
  effects.value.push(newEffect)
  showAddEffectMenu.value = false
}

// Remove effect from chain
async function removeEffect(index: number) {
  const effect = effects.value[index]
  
  // Disable effect first to disconnect it properly
  if (effect.enabled) {
    await handleEffectToggle(index, false)
  }
  
  effects.value.splice(index, 1)
}

// Handle effect toggle
async function handleEffectToggle(index: number, enabled: boolean) {
  const effect = effects.value[index]
  effect.enabled = enabled
  
  if (!Tone) return
  
  try {
    switch (effect.type) {
      case 'compressor':
        await toggleCompressor(enabled, effect.params)
        if (enabled) {
          effect.node = compressorNode
        }
        break
        
      case 'reverb':
        await toggleReverb(enabled, effect.params)
        if (enabled) {
          effect.node = reverbNode
        }
        break
        
      case 'delay':
        await toggleDelay(enabled, effect.params)
        if (enabled) {
          effect.node = delayNode
        }
        break
        
      case 'limiter':
        // Ensure threshold is in valid range
        effect.params.threshold = Math.max(-50, Math.min(0, effect.params.threshold))
        await toggleLimiter(enabled, effect.params)
        if (enabled) {
          effect.node = limiterNode
        }
        break
    }
  } catch (error) {
    console.error(`[MasterFX] Error toggling ${effect.type}:`, error)
    effect.enabled = false
  }
}

// Handle effect parameter update
function handleEffectUpdate(index: number, params: any) {
  const effect = effects.value[index]
  effect.params = { ...effect.params, ...params }
  
  if (!Tone || !effect.enabled) return
  
  switch (effect.type) {
    case 'compressor':
      updateCompressor(params)
      break
      
    case 'reverb':
      updateReverb(params)
      break
      
    case 'delay':
      updateDelay(params)
      break
      
    case 'limiter':
      updateLimiter(params)
      break
  }
}

// Compressor functions
async function toggleCompressor(enabled: boolean, params?: any) {
  if (!Tone) return

  if (enabled) {
    if (!compressorNode) {
      compressorNode = new Tone.Compressor({
        threshold: params?.threshold ?? -40,
        ratio: params?.ratio ?? 2,
        attack: params?.attack ?? 0.01,
        release: params?.release ?? 0.25,
        knee: 30
      })
    }
    fxChainEnabled.add('compressor')
  } else {
    fxChainEnabled.delete('compressor')
  }

  rebuildFXChain()
}

function updateCompressor(params: any) {
  if (!compressorNode) return
  
  // Save parameters to ref for snapshots
  if (params.threshold !== undefined) compressorParams.value.threshold = params.threshold
  if (params.ratio !== undefined) compressorParams.value.ratio = params.ratio
  if (params.attack !== undefined) compressorParams.value.attack = params.attack
  if (params.release !== undefined) compressorParams.value.release = params.release
  
  // Use longer ramp times to avoid metallic artifacts
  if (params.threshold !== undefined) compressorNode.threshold.rampTo(params.threshold, 0.05)
  if (params.ratio !== undefined) compressorNode.ratio.rampTo(params.ratio, 0.05)
  if (params.attack !== undefined) compressorNode.attack.rampTo(params.attack, 0.05)
  if (params.release !== undefined) compressorNode.release.rampTo(params.release, 0.05)
  if (params.knee !== undefined && compressorNode.knee) compressorNode.knee.rampTo(params.knee, 0.05)
}

// Reverb functions
async function toggleReverb(enabled: boolean, params?: any) {
  if (!Tone) return

  if (enabled) {
    if (!reverbNode) {
      reverbNode = new Tone.Reverb({
        decay: params?.decay ?? 1.5,
        preDelay: params?.preDelay ?? 0.01
      })
      reverbNode.wet.value = params?.wet ?? 0.3
      await reverbNode.generate()
    }
    fxChainEnabled.add('reverb')
  } else {
    fxChainEnabled.delete('reverb')
  }

  rebuildFXChain()
}

function updateReverb(params: any) {
  if (!reverbNode) return
  
  // Save parameters to ref for snapshots
  if (params.decay !== undefined) reverbParams.value.decay = params.decay
  if (params.preDelay !== undefined) reverbParams.value.preDelay = params.preDelay
  if (params.wet !== undefined) reverbParams.value.wet = params.wet
  
  if (params.wet !== undefined && !isRegeneratingReverb) {
    reverbNode.wet.rampTo(params.wet, 0.1)
  }
  
  // Decay and preDelay require regeneration
  if (params.decay !== undefined || params.preDelay !== undefined) {
    if (isRegeneratingReverb) return
    
    isRegeneratingReverb = true
    const currentWet = reverbNode.wet.value
    const oldReverb = reverbNode
    
    const newReverb = new Tone.Reverb({
      decay: params.decay ?? oldReverb.decay,
      preDelay: params.preDelay ?? oldReverb.preDelay
    })
    newReverb.wet.value = 0
    
    if (oldReverb.wet.value > 0) {
      oldReverb.wet.rampTo(0, 0.05)
    }
    
    newReverb.generate().then(() => {
      setTimeout(() => {
        reverbNode = newReverb
        rebuildFXChain()
        
        if (oldReverb) {
          try {
            oldReverb.disconnect()
            oldReverb.dispose()
          } catch (e) {}
        }
        
        newReverb.wet.rampTo(params.wet ?? currentWet, 0.05)
        
        setTimeout(() => {
          isRegeneratingReverb = false
        }, 100)
      }, 60)
    })
  }
}

// Delay functions
async function toggleDelay(enabled: boolean, params?: any) {
  if (!Tone) return

  if (enabled) {
    if (!delayNode) {
      delayNode = new Tone.FeedbackDelay({
        delayTime: params?.delayTime ?? 0.25,
        feedback: params?.feedback ?? 0.15,
        wet: params?.wet ?? 0.15,
        maxDelay: 2
      })
    }
    fxChainEnabled.add('delay')
  } else {
    fxChainEnabled.delete('delay')
  }

  rebuildFXChain()
}

function updateDelay(params: any) {
  if (!delayNode) return
  
  // Save parameters to ref for snapshots
  if (params.delayTime !== undefined) delayParams.value.delayTime = params.delayTime
  if (params.feedback !== undefined) delayParams.value.feedback = params.feedback
  if (params.wet !== undefined) delayParams.value.wet = params.wet
  
  // Clamp values to prevent clipping/stretch
  const clampedFeedback = Math.max(0, Math.min(0.7, params.feedback ?? delayParams.value.feedback))
  const clampedWet = Math.max(0, Math.min(0.5, params.wet ?? delayParams.value.wet))
  
  if (params.delayTime !== undefined) {
    // Reduce feedback temporarily during time changes to prevent stretch
    const currentFeedback = delayNode.feedback.value
    if (currentFeedback > 0.1) {
      delayNode.feedback.rampTo(0.1, 0.02)
      setTimeout(() => {
        delayNode.delayTime.rampTo(params.delayTime, 0.05)
        setTimeout(() => {
          delayNode.feedback.rampTo(currentFeedback, 0.1)
        }, 60)
      }, 30)
    } else {
      delayNode.delayTime.rampTo(params.delayTime, 0.01)
    }
  }
  
  if (params.feedback !== undefined) {
    delayNode.feedback.rampTo(clampedFeedback, 0.01)
  }
  
  if (params.wet !== undefined) {
    delayNode.wet.rampTo(clampedWet, 0.01)
  }
}

// Limiter functions
async function toggleLimiter(enabled: boolean, params?: any) {
  if (!Tone) return

  if (enabled) {
    if (!limiterNode) {
      // Use Compressor with heavy ratio as limiter
      limiterNode = new Tone.Compressor({
        threshold: params?.threshold ?? -1,
        ratio: 20,
        attack: 0.003,
        release: 0.1,
        knee: 6
      })
    }
    fxChainEnabled.add('limiter')
  } else {
    fxChainEnabled.delete('limiter')
  }

  rebuildFXChain()
}

function updateLimiter(params: any) {
  if (!limiterNode || !Tone) return
  
  if (params.threshold !== undefined) {
    // Save parameter to ref for snapshots
    limiterParams.value.threshold = params.threshold
    
    // Direct rampTo on threshold parameter
    const threshold = Math.max(-50, Math.min(0, params.threshold))
    limiterNode.threshold.rampTo(threshold, 0.05)
  }
}

// Scene Snapshot Support
function getSnapshot() {
  return {
    compressorEnabled: fxChainEnabled.has('compressor'),
    compressorThreshold: compressorParams.value.threshold,
    compressorRatio: compressorParams.value.ratio,
    compressorAttack: compressorParams.value.attack,
    compressorRelease: compressorParams.value.release,
    reverbEnabled: fxChainEnabled.has('reverb'),
    reverbDecay: reverbParams.value.decay,
    reverbPreDelay: reverbParams.value.preDelay,
    reverbWet: reverbParams.value.wet,
    delayEnabled: fxChainEnabled.has('delay'),
    delayTime: delayParams.value.delayTime,
    delayFeedback: delayParams.value.feedback,
    delayWet: delayParams.value.wet,
    limiterEnabled: fxChainEnabled.has('limiter'),
    limiterThreshold: limiterParams.value.threshold,
    // Also save effects array for UI state
    effects: effects.value.map(e => ({
      type: e.type,
      enabled: e.enabled,
      params: { ...e.params }
    }))
  }
}

function restoreSnapshot(snapshot: any) {
  if (!snapshot) return

  // Clear existing effects
  effects.value = []
  nextEffectId = 1

  // Restore effects UI state
  if (snapshot.effects && Array.isArray(snapshot.effects)) {
    snapshot.effects.forEach((effectSnapshot: any) => {
      const effect: Effect = {
        id: nextEffectId++,
        type: effectSnapshot.type,
        enabled: false, // Will be enabled by toggle
        params: { ...effectSnapshot.params },
        node: null
      }
      effects.value.push(effect)
    })
  }

  // Restore Compressor
  if (snapshot.compressorEnabled !== undefined) {
    const params = {
      threshold: snapshot.compressorThreshold ?? compressorParams.value.threshold,
      ratio: snapshot.compressorRatio ?? compressorParams.value.ratio,
      attack: snapshot.compressorAttack ?? compressorParams.value.attack,
      release: snapshot.compressorRelease ?? compressorParams.value.release
    }
    compressorParams.value = params
    toggleCompressor(snapshot.compressorEnabled, params)
    
    // Update effect in UI
    const compEffect = effects.value.find(e => e.type === 'compressor')
    if (compEffect) {
      compEffect.enabled = snapshot.compressorEnabled
      compEffect.node = compressorNode
    }
  }

  // Restore Reverb
  if (snapshot.reverbEnabled !== undefined) {
    const params = {
      decay: snapshot.reverbDecay ?? reverbParams.value.decay,
      preDelay: snapshot.reverbPreDelay ?? reverbParams.value.preDelay,
      wet: snapshot.reverbWet ?? reverbParams.value.wet
    }
    reverbParams.value = params
    toggleReverb(snapshot.reverbEnabled, params)
    
    const reverbEffect = effects.value.find(e => e.type === 'reverb')
    if (reverbEffect) {
      reverbEffect.enabled = snapshot.reverbEnabled
      reverbEffect.node = reverbNode
    }
  }

  // Restore Delay
  if (snapshot.delayEnabled !== undefined) {
    const params = {
      delayTime: snapshot.delayTime ?? delayParams.value.delayTime,
      feedback: snapshot.delayFeedback ?? delayParams.value.feedback,
      wet: snapshot.delayWet ?? delayParams.value.wet
    }
    delayParams.value = params
    toggleDelay(snapshot.delayEnabled, params)
    
    const delayEffect = effects.value.find(e => e.type === 'delay')
    if (delayEffect) {
      delayEffect.enabled = snapshot.delayEnabled
      delayEffect.node = delayNode
    }
  }

  // Restore Limiter
  if (snapshot.limiterEnabled !== undefined) {
    const params = {
      threshold: snapshot.limiterThreshold ?? limiterParams.value.threshold
    }
    limiterParams.value = params
    toggleLimiter(snapshot.limiterEnabled, params)
    
    const limiterEffect = effects.value.find(e => e.type === 'limiter')
    if (limiterEffect) {
      limiterEffect.enabled = snapshot.limiterEnabled
      limiterEffect.node = limiterNode
    }
  }
}

// Expose methods and output node
defineExpose({
  getOutputNode: () => outputNode,
  getSnapshot,
  restoreSnapshot,
  // Keep these for backward compatibility with effect components
  compressorNode: () => compressorNode,
  reverbNode: () => reverbNode,
  delayNode: () => delayNode,
  limiterNode: () => limiterNode
})
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(75, 85, 99, 0.6);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(75, 85, 99, 0.8);
}
</style>
