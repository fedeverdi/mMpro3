<template>
  <div class="master-fx bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-green-600/60 p-2 flex flex-col gap-2 ">
    <div class="text-left flex-shrink-0">
      <p class="text-[0.7rem] font-bold text-green-200 tracking-wide uppercase">Master FX Chain</p>
      <p class="text-[0.5rem] text-gray-400">Click + to add effects</p>
    </div>

    <div class="flex-1 h-full grid gap-2 min-h-0 pr-1 custom-scrollbar overflow-y-none" style="grid-template-columns: repeat(auto-fit, minmax(85px, 1fr));">
      <!-- Existing effects -->
      <div v-for="(effect, index) in effects" :key="effect.id" class="relative h-full" style="overflow: visible;">
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
          :initial-room-size="effect.params.roomSize"
          :initial-damping="effect.params.damping"
          :initial-wet="effect.params.wet"
          :initial-width="effect.params.width"
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
        class="relative h-full border-2 border-dashed border-gray-600 rounded-lg p-4 hover:border-green-500 transition-colors cursor-pointer bg-gray-800/50 hover:bg-gray-800 flex items-center justify-center"
        @click="toggleAddEffectMenu($event)"
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
import { useAudioEngine } from '../../composables/useAudioEngine'

interface Props {
  masterSection?: any       // For meter visualization in CompressorEffect and LimiterEffect
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'component', value: { getSnapshot: () => any, restoreSnapshot: (snapshot: any) => void, resetToDefaults: () => void }): void
}>()

// Audio Engine composable
const audioEngine = useAudioEngine()

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

// FX parameters (for scene snapshots)
const compressorParams = ref({
  threshold: -40,
  ratio: 2,
  attack: 0.01,
  release: 0.25
})
const reverbParams = ref({
  roomSize: 0.7,
  damping: 0.5,
  wet: 0.3,
  width: 1.0
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
const toggleAddEffectMenu = (event: MouseEvent) => {
  if (!showAddEffectMenu.value) {
    // Position menu above the mouse cursor
    const menuHeight = 192 // 4 buttons × ~48px each
    const menuWidth = 200
    
    menuPosition.value = {
      top: event.clientY - menuHeight - 8,
      left: event.clientX - menuWidth / 2 // Center horizontally on cursor
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
    roomSize: 0.7,
    damping: 0.5,
    wet: 0.3,
    width: 1.0
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
// Initialize component
onMounted(() => {
  // Emit component interface to parent
  emit('component', {
    getSnapshot,
    restoreSnapshot,
    resetToDefaults: () => {
      // Clear all effects
      effects.value = []
      
      // Reset parameter values to defaults
      compressorParams.value = { ...defaultParams.compressor }
      reverbParams.value = { ...defaultParams.reverb }
      delayParams.value = { ...defaultParams.delay }
      limiterParams.value = { ...defaultParams.limiter }
      
      // Disable all Rust engine effects
      audioEngine.setMasterCompressor(false, -24, 4, 30, 250)
      audioEngine.setMasterLimiter(false, -0.1, 100)
      audioEngine.setMasterDelay(false, 250, 250, 0.5, 0.3)
      audioEngine.setMasterReverb(false, 0.7, 0.5, 0.3, 1.0)
    }
  })
})

// Handle effect toggle
async function handleEffectToggle(index: number, enabled: boolean) {
  const effect = effects.value[index]
  effect.enabled = enabled
  
  try {
    switch (effect.type) {
      case 'compressor':
        await toggleCompressor(enabled, effect.params)
        effect.node = null
        break
        
      case 'reverb':
        await toggleReverb(enabled, effect.params)
        effect.node = null
        break
        
      case 'delay':
        await toggleDelay(enabled, effect.params)
        effect.node = null
        break
        
      case 'limiter':
        effect.params.threshold = Math.max(-50, Math.min(0, effect.params.threshold))
        await toggleLimiter(enabled, effect.params)
        effect.node = null
        break
    }
  } catch (error) {
    console.error(`[MasterFX] Error toggling ${effect.type}:`, error)
    effect.enabled = false
  }
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
  
  // Disable effect first
  if (effect.enabled) {
    await handleEffectToggle(index, false)
  }
  
  effects.value.splice(index, 1)
}

// Handle effect parameter update
function handleEffectUpdate(index: number, params: any) {
  const effect = effects.value[index]
  effect.params = { ...effect.params, ...params }
  
  if (!effect.enabled) return
  
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

// Compressor functions (Rust Engine)
async function toggleCompressor(enabled: boolean, params?: any) {
  const threshold = params?.threshold ?? compressorParams.value.threshold
  const ratio = params?.ratio ?? compressorParams.value.ratio
  const attack = params?.attack ?? compressorParams.value.attack
  const release = params?.release ?? compressorParams.value.release
  
  // Convert attack/release from seconds to milliseconds for Rust engine
  const attackMs = attack * 1000
  const releaseMs = release * 1000
  
  await audioEngine.setMasterCompressor(enabled, threshold, ratio, attackMs, releaseMs)
}

function updateCompressor(params: any) {
  // Save parameters to ref for snapshots
  if (params.threshold !== undefined) compressorParams.value.threshold = params.threshold
  if (params.ratio !== undefined) compressorParams.value.ratio = params.ratio
  if (params.attack !== undefined) compressorParams.value.attack = params.attack
  if (params.release !== undefined) compressorParams.value.release = params.release
  
  // Send to Rust engine (convert to ms)
  const attackMs = (params.attack ?? compressorParams.value.attack) * 1000
  const releaseMs = (params.release ?? compressorParams.value.release) * 1000
  
  audioEngine.setMasterCompressor(
    true, // Keep enabled during update
    params.threshold ?? compressorParams.value.threshold,
    params.ratio ?? compressorParams.value.ratio,
    attackMs,
    releaseMs
  )
}

// Reverb functions
// Reverb functions (Rust Engine)
async function toggleReverb(enabled: boolean, params?: any) {
  const roomSize = params?.roomSize ?? reverbParams.value.roomSize
  const damping = params?.damping ?? reverbParams.value.damping
  const wet = params?.wet ?? reverbParams.value.wet
  const width = params?.width ?? reverbParams.value.width
  
  await audioEngine.setMasterReverb(enabled, roomSize, damping, wet, width)
}

function updateReverb(params: any) {
  // Save parameters to ref for snapshots
  if (params.roomSize !== undefined) reverbParams.value.roomSize = params.roomSize
  if (params.damping !== undefined) reverbParams.value.damping = params.damping
  if (params.wet !== undefined) reverbParams.value.wet = params.wet
  if (params.width !== undefined) reverbParams.value.width = params.width
  
  // Send to Rust engine
  audioEngine.setMasterReverb(
    true, // Keep enabled during update
    params.roomSize ?? reverbParams.value.roomSize,
    params.damping ?? reverbParams.value.damping,
    params.wet ?? reverbParams.value.wet,
    params.width ?? reverbParams.value.width
  )
}

// Delay functions (Rust Engine)
async function toggleDelay(enabled: boolean, params?: any) {
  const delayTime = params?.delayTime ?? delayParams.value.delayTime
  const feedback = params?.feedback ?? delayParams.value.feedback
  const wet = params?.wet ?? delayParams.value.wet
  
  // Convert delay time from seconds to milliseconds for Rust engine
  const delayTimeMs = delayTime * 1000
  
  // Rust engine uses stereo delay, use same time for both channels
  await audioEngine.setMasterDelay(enabled, delayTimeMs, delayTimeMs, feedback, wet)
}

function updateDelay(params: any) {
  // Save parameters to ref for snapshots
  if (params.delayTime !== undefined) delayParams.value.delayTime = params.delayTime
  if (params.feedback !== undefined) delayParams.value.feedback = params.feedback
  if (params.wet !== undefined) delayParams.value.wet = params.wet
  
  // Clamp values to prevent clipping
  const clampedFeedback = Math.max(0, Math.min(0.7, params.feedback ?? delayParams.value.feedback))
  const clampedWet = Math.max(0, Math.min(0.5, params.wet ?? delayParams.value.wet))
  
  // Convert to ms and send to Rust engine
  const delayTimeMs = (params.delayTime ?? delayParams.value.delayTime) * 1000
  
  audioEngine.setMasterDelay(
    true, // Keep enabled during update
    delayTimeMs,
    delayTimeMs,
    clampedFeedback,
    clampedWet
  )
}

// Limiter functions (Rust Engine)
async function toggleLimiter(enabled: boolean, params?: any) {
  const threshold = params?.threshold ?? limiterParams.value.threshold
  
  // Rust engine limiter uses "ceiling" parameter (max output level)
  // Threshold in UI is the ceiling, release is fixed at 100ms in Rust
  await audioEngine.setMasterLimiter(enabled, threshold, 100)
}

function updateLimiter(params: any) {
  if (params.threshold !== undefined) {
    // Save parameter to ref for snapshots
    limiterParams.value.threshold = params.threshold
    
    // Clamp to valid range
    const threshold = Math.max(-50, Math.min(0, params.threshold))
    
    // Send to Rust engine
    audioEngine.setMasterLimiter(true, threshold, 100)
  }
}

// Scene Snapshot Support
// Scene Snapshot Support
function getSnapshot() {
  // Track enabled states from effects array
  const enabledEffects = new Set(effects.value.filter(e => e.enabled).map(e => e.type))
  
  return {
    compressorEnabled: enabledEffects.has('compressor'),
    compressorThreshold: compressorParams.value.threshold,
    compressorRatio: compressorParams.value.ratio,
    compressorAttack: compressorParams.value.attack,
    compressorRelease: compressorParams.value.release,
    reverbEnabled: enabledEffects.has('reverb'),
    reverbRoomSize: reverbParams.value.roomSize,
    reverbDamping: reverbParams.value.damping,
    reverbWet: reverbParams.value.wet,
    reverbWidth: reverbParams.value.width,
    delayEnabled: enabledEffects.has('delay'),
    delayTime: delayParams.value.delayTime,
    delayFeedback: delayParams.value.feedback,
    delayWet: delayParams.value.wet,
    limiterEnabled: enabledEffects.has('limiter'),
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

  // Restore Compressor (Rust Engine)
  if (snapshot.compressorEnabled !== undefined) {
    const params = {
      threshold: snapshot.compressorThreshold ?? compressorParams.value.threshold,
      ratio: snapshot.compressorRatio ?? compressorParams.value.ratio,
      attack: snapshot.compressorAttack ?? compressorParams.value.attack,
      release: snapshot.compressorRelease ?? compressorParams.value.release
    }
    compressorParams.value = params
    toggleCompressor(snapshot.compressorEnabled, params)
    
    const compEffect = effects.value.find(e => e.type === 'compressor')
    if (compEffect) {
      compEffect.enabled = snapshot.compressorEnabled
      compEffect.node = null
    }
  }

  // Restore Reverb (Rust Engine)
  if (snapshot.reverbEnabled !== undefined) {
    const params = {
      roomSize: snapshot.reverbRoomSize ?? reverbParams.value.roomSize,
      damping: snapshot.reverbDamping ?? reverbParams.value.damping,
      wet: snapshot.reverbWet ?? reverbParams.value.wet,
      width: snapshot.reverbWidth ?? reverbParams.value.width
    }
    reverbParams.value = params
    toggleReverb(snapshot.reverbEnabled, params)
    
    const reverbEffect = effects.value.find(e => e.type === 'reverb')
    if (reverbEffect) {
      reverbEffect.enabled = snapshot.reverbEnabled
      reverbEffect.node = null
    }
  }

  // Restore Delay (Rust Engine)
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
      delayEffect.node = null
    }
  }

  // Restore Limiter (Rust Engine)
  if (snapshot.limiterEnabled !== undefined) {
    const params = {
      threshold: snapshot.limiterThreshold ?? limiterParams.value.threshold
    }
    limiterParams.value = params
    toggleLimiter(snapshot.limiterEnabled, params)
    
    const limiterEffect = effects.value.find(e => e.type === 'limiter')
    if (limiterEffect) {
      limiterEffect.enabled = snapshot.limiterEnabled
      limiterEffect.node = null
    }
  }
}
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
