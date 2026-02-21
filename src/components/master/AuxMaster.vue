<template>
  <div class="aux-master bg-gradient-to-b from-gray-900 to-black rounded-lg border-2 border-teal-600/60 p-2 flex flex-col gap-2">
    <div class="text-left flex-shrink-0">
      <p class="text-[0.7rem] font-bold text-teal-200 tracking-wide uppercase">Auxiliary Buses</p>
      <p class="text-[0.5rem] text-gray-400">Click + to add aux send</p>
    </div>

    <div v-if="auxBuses.length === 0" class="flex-1 flex items-center justify-center text-gray-500 text-xs">
      No aux buses yet. Click + to add one.
    </div>

    <div v-else class="flex-1 grid gap-1.5 min-h-0 overflow-y-auto overflow-x-auto custom-scrollbar" 
         style="grid-template-columns: repeat(6, 1fr); grid-auto-rows: max-content;">
      <div v-for="(aux, index) in auxBuses" :key="aux.id" class="relative flex flex-col gap-0.5 bg-gray-800/50 rounded-lg p-1.5 border border-teal-700/50">
        <!-- Remove button (hidden for now) -->
        <button 
          v-if="false"
          @click="removeAux(index)"
          class="absolute -top-2 -right-2 w-6 h-6 bg-gray-600/70 hover:bg-gray-500/80 rounded-full text-gray-300 hover:text-gray-100 font-bold flex items-center justify-center transition-colors shadow-lg border-2 border-gray-700"
          style="z-index: 50;"
          title="Remove aux"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
            <path fill-rule="evenodd" d="M6.225 4.811a.75.75 0 011.06 0L12 9.525l4.715-4.714a.75.75 0 111.06 1.06L13.06 10.586l4.715 4.715a.75.75 0 11-1.06 1.06L12 11.646l-4.715 4.715a.75.75 0 11-1.06-1.06l4.715-4.715-4.715-4.715a.75.75 0 010-1.06z" clip-rule="evenodd" />
          </svg>
        </button>

        <!-- Aux name -->
        <div class="text-center">
          <input 
            v-model="aux.name"
            @blur="updateAuxName(index, aux.name)"
            class="bg-transparent text-[10px] font-bold text-teal-300 text-center border-b border-transparent hover:border-teal-500 focus:border-teal-500 outline-none w-full"
            maxlength="10"
          />
        </div>

        <!-- Volume Knob -->
        <div class="flex items-center justify-center scale-[0.65] -my-2">
          <Knob
            :modelValue="aux.volume"
            @update:modelValue="(val) => updateAuxVolume(index, val)"
            :min="-60"
            :max="10"
            label="Vol"
            unit="dB"
            color="#14b8a6"
          />
        </div>

        <!-- Mute/Solo -->
        <!-- <div class="flex gap-0.5">
          <button
            @click="toggleAuxMute(index)"
            :class="[
              'flex-1 py-0.5 text-[0.45rem] font-bold rounded transition-colors',
              aux.muted ? 'bg-yellow-600 text-white' : 'bg-gray-700 text-gray-400'
            ]"
          >
            M
          </button>
          <button
            @click="toggleAuxSolo(index)"
            :class="[
              'flex-1 py-0.5 text-[0.45rem] font-bold rounded transition-colors',
              aux.soloed ? 'bg-green-600 text-white' : 'bg-gray-700 text-gray-400'
            ]"
          >
            S
          </button>
        </div> -->

        <!-- Output routing -->
        <button
          @click="showOutputModal(index)"
          class="py-0.5 text-[0.45rem] font-bold rounded bg-teal-600 hover:bg-teal-700 text-white transition-colors"
        >
          OUT
        </button>
      </div>
    </div>

    <!-- Add Aux button -->
    <button
      v-if="auxBuses.length < 6"
      @click="addAux"
      class="w-full py-2 border-2 border-dashed border-teal-600/50 rounded-lg hover:border-teal-500 hover:bg-teal-900/20 transition-colors text-teal-400 text-sm font-bold"
    >
      + ADD AUX
    </button>

    <!-- Output Modal -->
    <Teleport to="body">
      <div v-if="selectedAuxIndex !== null" class="fixed inset-0 bg-black/70 flex items-center justify-center z-[9999]"
        @mousedown.self="selectedAuxIndex = null">
        <div class="bg-gray-900 rounded-lg border-2 border-teal-600 p-6 max-w-md w-full mx-4 max-h-[80vh] overflow-y-auto" @click.stop>
          <h3 class="text-lg font-bold text-teal-300 mb-4">
            {{ auxBuses[selectedAuxIndex]?.name }} - Output Routing
          </h3>
          
          <div class="space-y-3">
            <!-- Route to Master -->
            <label class="flex items-center gap-3 p-3 bg-gray-800 rounded hover:bg-gray-750 cursor-pointer">
              <input 
                type="checkbox" 
                :checked="auxBuses[selectedAuxIndex]?.routeToMaster"
                @change="toggleAuxMasterRouting(selectedAuxIndex)"
                class="w-4 h-4"
              />
              <span class="text-sm text-gray-300">Route to Master</span>
            </label>

            <!-- Divider -->
            <div class="border-t border-gray-700 my-4"></div>

            <!-- Output Device Selection -->
            <div>
              <h4 class="text-sm font-bold text-gray-400 mb-2">Select Output Device</h4>
              
              <!-- No Output Option -->
              <button
                @click="selectOutputDevice(selectedAuxIndex, 'no-output')"
                :class="[
                  'w-full p-3 rounded border-2 transition-all mb-2 text-left flex items-center gap-3',
                  auxBuses[selectedAuxIndex]?.selectedOutputDevice === 'no-output'
                    ? 'bg-teal-900/30 border-teal-500'
                    : 'bg-gray-800 border-gray-700 hover:border-gray-600'
                ]"
              >
                <div class="text-2xl">🔇</div>
                <div class="flex-1">
                  <div class="text-sm font-bold text-gray-200">No Output</div>
                  <div class="text-xs text-gray-400">Disable audio output</div>
                </div>
                <div v-if="auxBuses[selectedAuxIndex]?.selectedOutputDevice === 'no-output'" class="text-teal-400">
                  <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                  </svg>
                </div>
              </button>

              <!-- Default Option -->
              <button
                @click="selectOutputDevice(selectedAuxIndex, null)"
                :class="[
                  'w-full p-3 rounded border-2 transition-all mb-2 text-left flex items-center gap-3',
                  !auxBuses[selectedAuxIndex]?.selectedOutputDevice || auxBuses[selectedAuxIndex]?.selectedOutputDevice === ''
                    ? 'bg-teal-900/30 border-teal-500'
                    : 'bg-gray-800 border-gray-700 hover:border-gray-600'
                ]"
              >
                <div class="text-2xl">🔊</div>
                <div class="flex-1">
                  <div class="text-sm font-bold text-gray-200">Default Output</div>
                  <div class="text-xs text-gray-400">System default audio output</div>
                </div>
                <div v-if="!auxBuses[selectedAuxIndex]?.selectedOutputDevice || auxBuses[selectedAuxIndex]?.selectedOutputDevice === ''" class="text-teal-400">
                  <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                  </svg>
                </div>
              </button>

              <!-- Audio Output Devices -->
              <div v-if="audioOutputDevices.length > 0" class="space-y-2">
                <button
                  v-for="device in audioOutputDevices"
                  :key="device.deviceId"
                  @click="selectOutputDevice(selectedAuxIndex, device.deviceId)"
                  :class="[
                    'w-full p-3 rounded border-2 transition-all text-left flex items-center gap-3',
                    auxBuses[selectedAuxIndex]?.selectedOutputDevice === device.deviceId
                      ? 'bg-teal-900/30 border-teal-500'
                      : 'bg-gray-800 border-gray-700 hover:border-gray-600'
                  ]"
                >
                  <div class="text-2xl">🎧</div>
                  <div class="flex-1 min-w-0">
                    <div class="text-sm font-bold text-gray-200 truncate">{{ device.label || 'Unknown Device' }}</div>
                    <div class="text-xs text-gray-400 truncate">{{ device.deviceId }}</div>
                  </div>
                  <div v-if="auxBuses[selectedAuxIndex]?.selectedOutputDevice === device.deviceId" class="text-teal-400 flex-shrink-0">
                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                    </svg>
                  </div>
                </button>
              </div>

              <div v-else class="text-center text-gray-500 text-sm py-4">
                No audio output devices found
              </div>
            </div>
          </div>

          <button
            @click="selectedAuxIndex = null"
            class="mt-6 w-full py-2 bg-teal-600 hover:bg-teal-700 text-white rounded font-bold"
          >
            Close
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, inject, toRaw, onMounted } from 'vue'
import Knob from '../core/Knob.vue'
import { useAudioDevices } from '~/composables/useAudioDevices'

interface AuxBus {
  id: string
  name: string
  volume: number
  muted: boolean
  soloed: boolean
  routeToMaster: boolean
  selectedOutputDevice?: string | null
  node?: any
  outputStreamDest?: MediaStreamAudioDestinationNode | null
  outputAudioContext?: AudioContext | null
  outputSource?: MediaStreamAudioSourceNode | null
}

interface Props {
  auxBuses?: AuxBus[]
  masterChannel?: any
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'add-aux'): void
  (e: 'remove-aux', index: number): void
  (e: 'update-aux', index: number, aux: AuxBus): void
}>()

const ToneRef = inject<any>('Tone')
let Tone: any = null

const { audioOutputDevices, enumerateAudioOutputs } = useAudioDevices()

const selectedAuxIndex = ref<number | null>(null)
const auxBuses = ref<AuxBus[]>(props.auxBuses || [])

// Enumerate audio output devices on mount
onMounted(async () => {
  await enumerateAudioOutputs()
})

// Watch for prop changes
watch(() => props.auxBuses, (newVal) => {
  if (newVal) {
    auxBuses.value = newVal
  }
}, { deep: true })

// Add new aux
function addAux() {
  emit('add-aux')
}

// Remove aux
function removeAux(index: number) {
  emit('remove-aux', index)
}

// Update aux volume
function updateAuxVolume(index: number, volume: number) {
  const aux = { ...auxBuses.value[index], volume }
  if (aux.node) {
    aux.node.volume.value = volume
  }
  emit('update-aux', index, aux)
}

// Toggle mute
function toggleAuxMute(index: number) {
  const aux = { ...auxBuses.value[index], muted: !auxBuses.value[index].muted }
  if (aux.node) {
    aux.node.mute = aux.muted
  }
  emit('update-aux', index, aux)
}

// Toggle solo
function toggleAuxSolo(index: number) {
  const aux = { ...auxBuses.value[index], soloed: !auxBuses.value[index].soloed }
  emit('update-aux', index, aux)
}

// Update aux name
function updateAuxName(index: number, name: string) {
  const aux = { ...auxBuses.value[index], name }
  emit('update-aux', index, aux)
}

// Show output modal
function showOutputModal(index: number) {
  selectedAuxIndex.value = index
}

// Toggle master routing
function toggleAuxMasterRouting(index: number) {
  const aux = { ...auxBuses.value[index], routeToMaster: !auxBuses.value[index].routeToMaster }
  emit('update-aux', index, aux)
}

// Select output device
function selectOutputDevice(index: number, deviceId: string | null) {
  const aux = { ...auxBuses.value[index], selectedOutputDevice: deviceId }
  emit('update-aux', index, aux)
}

// Get device label
function getDeviceLabel(deviceId: string | null | undefined): string {
  if (!deviceId) return 'Default'
  if (deviceId === 'no-output') return 'No Output'
  const device = audioOutputDevices.value.find(d => d.deviceId === deviceId)
  return device?.label || 'Unknown Device'
}
</script>

<style scoped>
/* Vertical scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
  height: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(20, 184, 166, 0.5);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(20, 184, 166, 0.7);
}

/* Horizontal scrollbar - very minimal */
.custom-scrollbar::-webkit-scrollbar:horizontal {
  height: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:horizontal {
  background: rgba(20, 184, 166, 0.4);
  border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:horizontal:hover {
  background: rgba(20, 184, 166, 0.6);
}

/* Modal scrollbar */
.max-h-\[80vh\]::-webkit-scrollbar {
  width: 6px;
}

.max-h-\[80vh\]::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 3px;
}

.max-h-\[80vh\]::-webkit-scrollbar-thumb {
  background: rgba(20, 184, 166, 0.5);
  border-radius: 3px;
}

.max-h-\[80vh\]::-webkit-scrollbar-thumb:hover {
  background: rgba(20, 184, 166, 0.7);
}
</style>
