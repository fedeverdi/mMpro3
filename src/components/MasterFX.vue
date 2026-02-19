<template>
  <div class="master-fx bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-green-600/60 p-3 flex flex-col gap-2 h-full overflow-visible">
    <div class="text-left flex-shrink-0">
      <p class="text-sm font-bold text-green-200 tracking-wide uppercase">Master FX Chain</p>
      <p class="text-[10px] text-gray-400">Click + to add effects</p>
    </div>

    <div class="flex-1 h-full  grid grid-cols-4 gap-3 min-h-0 pr-1 custom-scrollbar overflow-y-none">
      <!-- Existing effects -->
      <div v-for="(effect, index) in effects" :key="effect.id" class="relative h-full" style="overflow: visible;">
        <!-- Remove button -->
        <button 
          @click="removeEffect(index)"
          class="absolute -top-3 pt-0.5 -right-3 w-7 h-7 bg-red-600 hover:bg-red-500 rounded-full text-white font-bold flex items-center justify-center transition-colors shadow-xl border-2 border-gray-900"
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
          :master-section="props.masterSection"
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
          :master-section="props.masterSection"
          @toggle="(enabled) => handleEffectToggle(index, enabled)"
          @update="(params) => handleEffectUpdate(index, params)"
        />
      </div>

      <!-- Add effect button -->
      <div 
      v-if="effects.length < 4"
        class="relative h-full border-2 border-dashed border-gray-600 rounded-lg p-4 hover:border-green-500 transition-colors cursor-pointer bg-gray-800/50 hover:bg-gray-800 flex items-center justify-center h-full"
        @click="showAddEffectMenu = !showAddEffectMenu"
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
          class="absolute bottom-24 left-1/2 transform -translate-x-1/2 bg-gray-800 border border-gray-600 rounded-lg shadow-xl overflow-hidden min-w-[200px]"
          style="z-index: 9999;"
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
            <span class="text-purple-400 font-bold">RE</span>
            <span>Reverb</span>
          </button>
          <button
            @click="addEffect('delay')"
            class="w-full px-4 py-3 text-left text-sm hover:bg-gray-700 transition-colors border-b border-gray-700 flex items-center gap-2"
          >
            <span class="text-green-400 font-bold">DL</span>
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
import { ref } from 'vue'
import CompressorEffect from './CompressorEffect.vue'
import ReverbEffect from './ReverbEffect.vue'
import DelayEffect from './DelayEffect.vue'
import LimiterEffect from './LimiterEffect.vue'

interface Props {
  masterSection?: any
}

const props = defineProps<Props>()

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
let nextEffectId = 1

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
  
  console.log(`[MasterFX] Added ${type} effect`)
}

// Remove effect from chain
async function removeEffect(index: number) {
  const effect = effects.value[index]
  
  // Disable effect first to disconnect it properly
  if (effect.enabled) {
    await handleEffectToggle(index, false)
  }
  
  effects.value.splice(index, 1)
  
  console.log(`[MasterFX] Removed ${effect.type} effect`)
}

// Handle effect toggle
async function handleEffectToggle(index: number, enabled: boolean) {
  const effect = effects.value[index]
  effect.enabled = enabled
  
  if (!props.masterSection) return
  
  try {
    switch (effect.type) {
      case 'compressor':
        await props.masterSection.toggleCompressor(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.compressorNode()
        }
        break
        
      case 'reverb':
        await props.masterSection.toggleReverb(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.reverbNode()
        }
        break
        
      case 'delay':
        await props.masterSection.toggleDelay(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.delayNode()
        }
        break
        
      case 'limiter':
        // Ensure threshold is in valid range
        effect.params.threshold = Math.max(-20, Math.min(3, effect.params.threshold))
        await props.masterSection.toggleLimiter(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.limiterNode()
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
  
  if (!props.masterSection || !effect.enabled) return
  
  switch (effect.type) {
    case 'compressor':
      props.masterSection.updateCompressor(params)
      break
      
    case 'reverb':
      props.masterSection.updateReverb(params)
      break
      
    case 'delay':
      props.masterSection.updateDelay(params)
      break
      
    case 'limiter':
      props.masterSection.updateLimiter(params)
      break
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
