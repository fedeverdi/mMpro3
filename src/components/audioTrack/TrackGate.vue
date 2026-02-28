<template>
  <!-- Gate Toggle -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between',
    enabled ? 'bg-purple-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <span>GT</span>
    <button @click.stop="showModal = true"
      class="p-0.5 rounded transition-colors"
      :class="enabled ? 'hover:bg-purple-700' : 'hover:bg-gray-500'">
      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
  </div>

  <!-- Gate Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-50"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-purple-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-xl font-bold text-purple-300">Track {{ trackNumber }} - Noise Gate</h3>
          <div class="flex items-center gap-3">
            <button @click="handleToggle" :class="[
              'px-4 py-2 rounded-lg font-bold text-sm transition-all',
              enabled ? 'bg-purple-600 text-white hover:bg-purple-500' : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            ]">
              {{ enabled ? 'ON' : 'OFF' }}
            </button>
            <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
          </div>
        </div>

        <!-- Gate Status Indicator -->
        <div class="mb-6 bg-black/50 rounded-lg p-4 border border-purple-600/30">
          <p class="text-xs text-purple-300 font-bold mb-2 text-center">GATE STATUS</p>
          <div class="flex items-center justify-center gap-4">
            <div class="flex flex-col items-center">
              <div class="text-xs text-gray-400 mb-1">Current Level</div>
              <div class="text-2xl font-mono text-white">{{ currentLevel.toFixed(1) }} dB</div>
            </div>
            <div class="flex flex-col items-center">
              <div class="text-xs text-gray-400 mb-1">Status</div>
              <div :class="[
                'text-2xl font-bold transition-all',
                isGateOpen ? 'text-green-400' : 'text-red-400'
              ]">
                {{ isGateOpen ? 'OPEN' : 'CLOSED' }}
              </div>
            </div>
          </div>
          <!-- Visual indicator bar -->
          <div class="mt-4 h-2 bg-gray-800 rounded-full overflow-hidden relative">
            <div :style="{ width: gateOpenPercentage + '%' }" 
              class="h-full bg-gradient-to-r from-purple-600 to-purple-400 transition-all duration-100">
            </div>
            <!-- Threshold marker -->
            <div :style="{ left: thresholdPosition + '%' }"
              class="absolute top-0 bottom-0 w-0.5 bg-yellow-400">
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <Knob v-model="threshold" :min="-80" :max="0" :step="0.5" label="Threshold" unit="dB"
            color="#a855f7" />
          <Knob v-model="range" :min="-60" :max="0" :step="1" label="Range" unit="dB" color="#a855f7" />
          <Knob v-model="attack" :min="0.001" :max="0.1" :step="0.001" label="Attack" unit="s" color="#a855f7" />
          <Knob v-model="release" :min="0.01" :max="2" :step="0.01" label="Release" unit="s" color="#a855f7" />
        </div>

        <div class="mt-4 text-xs text-gray-400 text-center">
          <p><strong>Threshold:</strong> Signal level required to open the gate</p>
          <p><strong>Range:</strong> Attenuation when closed (-30dB = natural, 0dB = off)</p>
          <p><strong>Attack:</strong> Opening speed (5ms vocals, 1ms drums)</p>
          <p><strong>Release:</strong> Closing speed (200-500ms = natural)</p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, watchEffect, inject, onUnmounted } from 'vue'
import Knob from '../core/Knob.vue'

interface Props {
  trackNumber: number
  enabled: boolean
  inputLevelDb?: number
  attenuationDb?: number
  gateNode?: any
  meter?: any
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'update-params', params: { threshold: number, attack: number, release: number, range: number }): void
}>()

const showModal = ref(false)

// Internal state for controls
const threshold = ref(-45) // Professional default for voice
const attack = ref(0.005) // 5ms - fast opening
const release = ref(0.3) // 300ms - natural release
const range = ref(-30) // Attenuation when closed (dB)

// Visual feedback
const currentLevel = ref(-80)
const isGateOpen = ref(false)

const gateOpenPercentage = ref(0)
const thresholdPosition = ref(50)

function handleToggle() {
  emit('toggle')
}

function updateGateNode() {
  if (!props.gateNode) return
  
  // Emit params for parent to update custom gate
  emit('update-params', {
    threshold: threshold.value,
    attack: attack.value,
    release: release.value,
    range: range.value
  })
}

let rafId: number | null = null

// Watch for parameter changes with throttling
watchEffect(() => {
  // Track dependencies
  threshold.value
  attack.value
  release.value
  range.value
  
  // Throttle updates with requestAnimationFrame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  
  rafId = requestAnimationFrame(() => {
    // Only update gate node if it exists and gate is enabled
    if (props.enabled && props.gateNode) {
      updateGateNode()
    }
    updateThresholdPosition()
    rafId = null
  })
})

onUnmounted(() => {
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})

function updateThresholdPosition() {
  // Map threshold from -80dB to 0dB → 0% to 100%
  thresholdPosition.value = ((threshold.value + 80) / 80) * 100
}

// Expose methods for snapshot save/restore
defineExpose({
  getParams: () => ({
    threshold: threshold.value,
    attack: attack.value,
    release: release.value,
    range: range.value
  }),
  setParams: (params: { threshold: number, attack: number, release: number, range: number }) => {
    threshold.value = params.threshold
    attack.value = params.attack
    release.value = params.release
    range.value = params.range
    // Update node if gate is enabled
    if (props.enabled && props.gateNode) {
      updateGateNode()
    }
  },
  resetToDefaults: () => {
    threshold.value = -45
    attack.value = 0.005
    release.value = 0.3
    range.value = -30
    // Update node if gate is enabled
    if (props.enabled && props.gateNode) {
      updateGateNode()
    }
  }
})

// Monitor gate status
let monitoringId: number | null = null

function startGateMonitoring() {
  if (monitoringId) return

  function updateGateStatus() {
    if (!showModal.value) {
      monitoringId = null
      return
    }

    // Use data from Rust engine if available
    if (props.inputLevelDb !== undefined) {
      currentLevel.value = props.inputLevelDb
      
      // Determine if gate is open (input is above threshold + attenuation is minimal)
      // When gate is truly open, attenuation should be close to 0dB
      isGateOpen.value = currentLevel.value > threshold.value && 
                         (props.attenuationDb !== undefined ? props.attenuationDb > -10 : false)
      
      // Calculate signal level percentage for visual feedback
      // Map from -80dB to 0dB → 0% to 100%
      gateOpenPercentage.value = Math.max(0, Math.min(100, ((currentLevel.value + 80) / 80) * 100))
    }

    monitoringId = requestAnimationFrame(updateGateStatus)
  }

  updateGateStatus()
}

function stopGateMonitoring() {
  if (monitoringId) {
    cancelAnimationFrame(monitoringId)
    monitoringId = null
  }
}

watch(showModal, (isOpen) => {
  if (isOpen) {
    updateThresholdPosition()
    startGateMonitoring()
  } else {
    stopGateMonitoring()
  }
})

onUnmounted(() => {
  stopGateMonitoring()
})
</script>
