<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="close">
        <!-- Overlay -->
        <div class="absolute inset-0 bg-black/80 backdrop-blur-sm"></div>
        
        <!-- Modal -->
        <div class="relative bg-gradient-to-br from-gray-900 via-gray-800 to-black border-2 border-purple-600/50 rounded-xl shadow-2xl max-w-6xl w-full max-h-[90vh] overflow-hidden">
          <!-- Header -->
          <div class="bg-gradient-to-r from-purple-900/50 to-blue-900/50 px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <div>
              <h2 class="text-2xl font-bold text-white">Signal Flow</h2>
              <p class="text-sm text-gray-400 mt-1">Complete audio routing chain</p>
            </div>
            <button @click="close" class="text-gray-400 hover:text-white transition-colors">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
          
          <!-- Content -->
          <div class="p-6 overflow-y-auto max-h-[calc(90vh-80px)]">
            <!-- Audio Track Chain -->
            <div class="mb-8">
              <h3 class="text-lg font-bold text-purple-400 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M18 3a1 1 0 00-1.196-.98l-10 2A1 1 0 006 5v9.114A4.369 4.369 0 005 14c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V7.82l8-1.6v5.894A4.37 4.37 0 0015 12c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V3z"/>
                </svg>
                Audio Track Signal Chain
              </h3>
              
              <div class="flex flex-wrap items-center gap-3 text-sm">
                <!-- Input -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gradient-to-br from-green-600 to-green-700 text-white rounded-lg font-bold shadow-lg">
                    Input
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Audio Source</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Gain -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">
                    Gain
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Input Level</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- EQ3 -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-blue-700 text-white rounded-lg font-semibold">
                    EQ3
                  </div>
                  <div class="text-xs text-gray-500 mt-1">3-band EQ</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Meter/Waveform -->
                <div class="flex flex-col items-center border-2 border-yellow-500/50 rounded-lg p-3 bg-yellow-900/10">
                  <div class="text-xs text-yellow-400 font-bold mb-2">ANALYSIS</div>
                  <div class="flex flex-col gap-1">
                    <div class="px-3 py-1 bg-yellow-700/50 text-white rounded text-xs font-semibold">VU Meter</div>
                    <div class="px-3 py-1 bg-yellow-700/50 text-white rounded text-xs font-semibold">Waveform</div>
                  </div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Compressor -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-purple-700 text-white rounded-lg font-semibold border-2 border-purple-500">
                    Compressor
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Optional FX</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Reverb -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-purple-700 text-white rounded-lg font-semibold border-2 border-purple-500">
                    Reverb
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Optional FX</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Panner -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-cyan-700 text-white rounded-lg font-semibold">
                    Panner
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Stereo Balance/Pan</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Volume -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">
                    Volume
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Track Fader</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- To Master -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gradient-to-br from-orange-600 to-red-600 text-white rounded-lg font-bold shadow-lg">
                    Master
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Stereo Bus</div>
                </div>
              </div>
            </div>
            
            <div class="border-t border-gray-700 my-6"></div>
            
            <!-- Master Chain -->
            <div>
              <h3 class="text-lg font-bold text-orange-400 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"/>
                </svg>
                Master Bus Signal Chain
              </h3>
              
              <div class="flex flex-wrap items-center gap-3 text-sm">
                <!-- Tracks Input -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gradient-to-br from-gray-600 to-gray-700 text-white rounded-lg font-bold shadow-lg">
                    Tracks
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Stereo Mix</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Master Channel -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-orange-700 text-white rounded-lg font-semibold">
                    Master Channel
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Stereo In/Out</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Master FX Box -->
                <div class="flex flex-col items-center border-2 border-dashed border-orange-500/50 rounded-lg p-3 bg-orange-900/10">
                  <div class="text-xs text-orange-400 font-bold mb-2">MASTER FX</div>
                  <div class="flex flex-col gap-2">
                    <div class="px-3 py-1 bg-orange-700/50 text-white rounded text-xs font-semibold">Parametric EQ</div>
                    <div class="px-3 py-1 bg-orange-700/50 text-white rounded text-xs font-semibold">Compressor</div>
                    <div class="px-3 py-1 bg-orange-700/50 text-white rounded text-xs font-semibold">Reverb</div>
                    <div class="px-3 py-1 bg-orange-700/50 text-white rounded text-xs font-semibold">Limiter</div>
                  </div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- FX Chain Output -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">
                    FX Output
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Post-FX</div>
                </div>
                
                <!-- Split to 3 destinations -->
                <div class="flex flex-col items-center gap-3">
                  <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex gap-4">
                    <!-- Branch 1: Main Faders -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-blue-400 font-bold">MAIN OUTPUT</div>
                      <div class="flex flex-col gap-2 border-2 border-blue-500 rounded-lg p-2 bg-blue-900/10">
                        <div class="px-3 py-1 bg-gray-700 text-white rounded text-xs font-semibold">Fader Split</div>
                        <div class="flex gap-1">
                          <div class="px-2 py-1 bg-purple-600 text-white rounded text-xs">L Gain</div>
                          <div class="px-2 py-1 bg-blue-600 text-white rounded text-xs">R Gain</div>
                        </div>
                        <div class="px-3 py-1 bg-cyan-700 text-white rounded text-xs font-semibold">Merger</div>
                        <div class="px-3 py-1 bg-yellow-600 text-white rounded text-xs font-semibold">VU Meters</div>
                        <div class="px-3 py-1 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded text-xs font-bold">Speakers</div>
                      </div>
                    </div>
                    
                    <!-- Branch 2: Headphones -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-green-400 font-bold">HEADPHONES</div>
                      <div class="flex flex-col gap-2 border-2 border-green-500 rounded-lg p-2 bg-green-900/10">
                        <div class="px-3 py-1 bg-gray-700 text-white rounded text-xs font-semibold">HP Volume</div>
                        <div class="px-3 py-1 bg-gradient-to-r from-green-600 to-teal-600 text-white rounded text-xs font-bold">Headphones</div>
                      </div>
                    </div>
                    
                    <!-- Branch 3: Analysis -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-purple-400 font-bold">ANALYSIS</div>
                      <div class="flex flex-col gap-2 border-2 border-purple-500 rounded-lg p-2 bg-purple-900/10">
                        <div class="px-3 py-1 bg-gray-700 text-white rounded text-xs font-semibold">Analysis Tap</div>
                        <div class="px-3 py-1 bg-purple-600 text-white rounded text-xs font-semibold">Spectrum</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Legend -->
            <div class="mt-8 p-4 bg-gray-800/50 rounded-lg border border-gray-700">
              <h4 class="text-sm font-bold text-gray-400 mb-2">Legend</h4>
              <div class="grid grid-cols-2 gap-2 text-xs text-gray-500">
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 bg-gray-700 rounded"></div>
                  <span>Processing Node</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 bg-purple-600 rounded"></div>
                  <span>Left Channel</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 bg-blue-600 rounded"></div>
                  <span>Right Channel</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="w-3 h-3 border-2 border-dashed border-purple-500/50 rounded"></div>
                  <span>Optional/Conditional</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

interface Props {
  modelValue: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

function close() {
  emit('update:modelValue', false)
}

// Close on Escape key
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    close()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active > div:last-child,
.modal-leave-active > div:last-child {
  transition: transform 0.3s ease;
}

.modal-enter-from > div:last-child {
  transform: scale(0.9);
}

.modal-leave-to > div:last-child {
  transform: scale(0.9);
}
</style>
