<template>
  <div class="master-fx bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-green-600/60 p-3 flex flex-col gap-2 h-full" :class="showAddEffectMenu ? 'overflow-visible' : 'overflow-hidden'">
    <div class="text-left flex-shrink-0">
      <p class="text-sm font-bold text-green-200 tracking-wide uppercase">Master FX Chain</p>
      <p class="text-[10px] text-gray-400">Click + to add effects</p>
    </div>

    <div class="flex-1 flex flex-col gap-2 min-h-0 pr-1 custom-scrollbar" :class="showAddEffectMenu ? 'overflow-visible' : 'overflow-y-auto'">
      <!-- Existing effects -->
      <div v-for="(effect, index) in effects" :key="effect.id" class="relative">
        <!-- Remove button -->
        <button 
          @click="removeEffect(index)"
          class="absolute -top-1 -right-1 z-10 w-5 h-5 bg-red-600 hover:bg-red-500 rounded-full text-white text-xs flex items-center justify-center transition-colors"
          title="Remove effect"
        >
          ×
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
        class="relative border-2 border-dashed border-gray-600 rounded-lg p-4 hover:border-green-500 transition-colors cursor-pointer bg-gray-800/50 hover:bg-gray-800 flex items-center justify-center min-h-[80px]"
        @click="showAddEffectMenu = !showAddEffectMenu"
      >
        <div class="text-center">
          <div class="text-3xl text-gray-400 hover:text-green-400 transition-colors">+</div>
          <div class="text-xs text-gray-500 mt-1">Add Effect</div>
        </div>

        <!-- Add effect dropdown menu -->
        <div 
          v-if="showAddEffectMenu"
          class="absolute bottom-full left-0 right-0 mb-1 bg-gray-800 border border-gray-600 rounded-lg shadow-xl z-50 overflow-hidden"
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
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
          effect.node = props.masterSection.compressorNode?.value
        }
        break
        
      case 'reverb':
        await props.masterSection.toggleReverb(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.reverbNode?.value
        }
        break
        
      case 'delay':
        await props.masterSection.toggleDelay(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.delayNode?.value
        }
        break
        
      case 'limiter':
        // Ensure threshold is in valid range
        effect.params.threshold = Math.max(-20, Math.min(3, effect.params.threshold))
        await props.masterSection.toggleLimiter(enabled, effect.params)
        if (enabled) {
          effect.node = props.masterSection.limiterNode?.value
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

// Close menu when clicking outside
function handleClickOutside(event: MouseEvent) {
  showAddEffectMenu.value = false
}

// Watch for clicks to close menu
watch(showAddEffectMenu, (isOpen) => {
  if (isOpen) {
    setTimeout(() => {
      document.addEventListener('click', handleClickOutside)
    }, 0)
  } else {
    document.removeEventListener('click', handleClickOutside)
  }
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
