<template>
  <!-- Gate Button -->
  <div @click="handleToggle" :class="[
    'w-full cursor-pointer py-1 px-2 text-[10px] font-bold rounded transition-all flex items-center justify-between gap-1',
    enabled ? 'bg-purple-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
  ]">
    <div class="flex items-center gap-1">
      <div class="drag-handle cursor-move text-gray-400 hover:text-gray-200" @click.stop>
        <svg class="w-2 h-2" fill="currentColor" viewBox="0 0 24 24">
          <path d="M9 3h2v2H9V3zm0 4h2v2H9V7zm0 4h2v2H9v-2zm0 4h2v2H9v-2zm0 4h2v2H9v-2zM13 3h2v2h-2V3zm0 4h2v2h-2V7zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2zm0 4h2v2h-2v-2z"/>
        </svg>
      </div>
      <span>GT</span>
    </div>
    <div class="flex items-center gap-0.5">
      <button :disabled="!enabled" @click.stop="showModal = true"
        class="p-0.5 rounded hover:bg-purple-700 disabled:opacity-50">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </button>
      <button @click.stop="$emit('remove')" class="p-0.5 rounded hover:bg-red-600 transition-all" title="Remove">
        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Gate Modal -->
  <Teleport to="body">
    <div v-if="showModal" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[1000]"
      @click="showModal = false">
      <div class="bg-gray-900 rounded-lg border-2 border-purple-600 p-6 max-w-2xl w-full mx-4" @click.stop>
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold text-purple-300">Track {{ trackNumber }} - Insert #{{ insertId }} - Noise Gate</h3>
          <button @click="showModal = false" class="text-gray-400 hover:text-white text-2xl">&times;</button>
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

        <div class="flex flex-wrap gap-4 justify-center">
          <Knob v-model="threshold" :min="-80" :max="0" :step="0.5" label="Threshold" unit="dB" color="#a855f7" />
          <Knob v-model="range" :min="-80" :max="0" :step="0.5" label="Range" unit="dB" color="#f59e0b" />
          <Knob v-model="attack" :min="1" :max="100" :step="1" label="Attack" unit="ms" color="#10b981" />
          <Knob v-model="release" :min="10" :max="1000" :step="10" label="Release" unit="ms" color="#06b6d4" />
        </div>

        <div class="mt-4 text-xs text-gray-400 text-center">
          <p><strong>Threshold:</strong> Signal level required to open the gate</p>
          <p><strong>Range:</strong> Attenuation when closed</p>
          <p><strong>Attack:</strong> Opening speed (1-5ms fast, 10-50ms slow)</p>
          <p><strong>Release:</strong> Closing speed (100-500ms natural)</p>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, inject, watch, onUnmounted } from 'vue'
import Knob from '../core/Knob.vue'

const props = defineProps<{
  trackNumber: number
  insertId: number
  enabled: boolean
  trackLevelL: number
  trackLevelR: number
  phaseCorrelation: number
  compressorInputDb?: number
}>()

const emit = defineEmits<{
  toggle: []
  remove: []
}>()

const audioEngine = inject('audioEngine') as any
const showModal = ref(false)

// Gate parameters
const threshold = ref(-40)
const range = ref(-80)
const attack = ref(1)
const release = ref(100)

// Visual feedback
const currentLevel = ref(-80)
const isGateOpen = ref(false)
const gateOpenPercentage = ref(0)
const thresholdPosition = ref(50)

function handleToggle() {
  emit('toggle')
}

// Watch for parameter changes and send to backend
watch([threshold, range, attack, release], () => {
  if (audioEngine?.state.value.isRunning && props.enabled) {
    audioEngine.setTrackInsertGate(
      props.trackNumber - 1,
      props.insertId,
      threshold.value,
      range.value,
      attack.value / 1000,
      release.value / 1000
    )
  }
  updateThresholdPosition()
})

function updateThresholdPosition() {
  thresholdPosition.value = ((threshold.value + 80) / 80) * 100
}

// Monitor gate status
let monitoringId: number | null = null

function startGateMonitoring() {
  if (monitoringId) return

  function updateGateStatus() {
    if (!showModal.value) {
      monitoringId = null
      return
    }

    const avgLevel = (props.trackLevelL + props.trackLevelR) / 2
    currentLevel.value = avgLevel
    isGateOpen.value = currentLevel.value > threshold.value
    gateOpenPercentage.value = Math.max(0, Math.min(100, ((currentLevel.value + 80) / 80) * 100))

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
