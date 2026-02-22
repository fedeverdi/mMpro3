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
          <div class="p-6 overflow-y-auto max-h-[calc(90vh-80px)] custom-scrollbar">
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
                
                <!-- PAD -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-yellow-700 text-white rounded-lg font-semibold border-2 border-dashed border-yellow-400">
                    PAD
                  </div>
                  <div class="text-xs text-orange-400 mt-1">-26dB if enabled</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- HPF -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-blue-700 text-white rounded-lg font-semibold border-2 border-dashed border-blue-400">
                    HPF
                  </div>
                  <div class="text-xs text-orange-400 mt-1">80Hz if enabled</div>
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
                
                <!-- Gate with meter tap -->
                <div class="flex flex-col items-center gap-2">
                  <!-- Gate meter tap branching UP -->
                  <div class="flex flex-col items-center gap-1">
                    <div class="px-2 py-1 bg-purple-600 text-white rounded text-[10px] font-bold shadow-lg">
                      Gate Meter
                    </div>
                    <svg class="w-4 h-6 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M12 19V5"/>
                    </svg>
                    <div class="text-[9px] text-purple-500 font-bold">↓ TAP</div>
                  </div>
                  <!-- Gate main node -->
                  <div class="px-4 py-2 bg-purple-700 text-white rounded-lg font-semibold border-2 border-dashed border-purple-400">
                    Gate
                  </div>
                  <div class="text-xs text-orange-400">If Enabled</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- EQ3 with analysis taps -->
                <div class="flex flex-col items-center gap-2">
                  <!-- Analysis taps branching UP -->
                  <div class="flex gap-3 items-end">
                    <div class="flex flex-col items-center gap-1">
                      <div class="px-2 py-1 bg-yellow-600 text-white rounded text-[10px] font-bold shadow-lg">
                        VU L/R
                      </div>
                      <svg class="w-4 h-6 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 19V5"/>
                      </svg>
                      <div class="text-[9px] text-yellow-500 font-bold">↓ TAP</div>
                    </div>
                    <div class="flex flex-col items-center gap-1">
                      <div class="px-2 py-1 bg-cyan-600 text-white rounded text-[10px] font-bold shadow-lg">
                        Waveform
                      </div>
                      <svg class="w-4 h-6 text-cyan-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 19V5"/>
                      </svg>
                      <div class="text-[9px] text-cyan-500 font-bold">↓ TAP</div>
                    </div>
                  </div>
                  <!-- EQ3 main node -->
                  <div class="px-4 py-2 bg-blue-700 text-white rounded-lg font-semibold shadow-lg">
                    EQ3
                  </div>
                  <div class="text-xs text-gray-500">3-band EQ</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Compressor -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-indigo-700 text-white rounded-lg font-semibold border-2 border-dashed border-indigo-400">
                    Compressor
                  </div>
                  <div class="text-xs text-orange-400 mt-1">If Enabled</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Reverb Send/Return (Parallel) -->
                <div class="flex flex-col items-center border-2 border-green-500/50 rounded-lg p-3 bg-green-900/10 relative">
                  <div class="text-xs text-green-300 font-bold mb-2 uppercase">Parallel Reverb</div>
                  <div class="flex items-center gap-2">
                    <!-- Dry path -->
                    <div class="flex flex-col items-center gap-1">
                      <div class="text-[9px] text-gray-400 font-bold">DRY</div>
                      <svg class="w-4 h-12 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 5v14"/>
                      </svg>
                      <div class="text-[9px] text-gray-400">Direct</div>
                    </div>
                    
                    <!-- Wet path -->
                    <div class="flex flex-col items-center gap-1">
                      <div class="text-[9px] text-green-400 font-bold">WET</div>
                      <div class="px-2 py-1 bg-green-700 text-white rounded text-[10px] font-semibold">Send</div>
                      <svg class="w-3 h-2 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7"/>
                      </svg>
                      <div class="px-2 py-1 bg-green-600 text-white rounded text-[10px] font-semibold">Reverb</div>
                      <div class="text-[9px] text-green-400">Return</div>
                    </div>
                  </div>
                  <div class="text-[9px] text-green-400 mt-1.5">Send gain controls wet level</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Balance -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-cyan-700 text-white rounded-lg font-semibold">
                    Balance
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Split→L/R→Merge</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Volume -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">
                    Volume
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Split→L/R→Merge</div>
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
            
            <!-- Subgroups Chain -->
            <div v-if="props.subgroups && props.subgroups.length > 0" class="mb-8">
              <h3 class="text-lg font-bold text-teal-400 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                </svg>
                Subgroups Bus Signal Chain
              </h3>
              
              <div class="space-y-4">
                <div v-for="subgroup in props.subgroups.slice(0, 1)" :key="subgroup.id" class="flex flex-wrap items-center justify-between gap-3 text-sm bg-teal-900/10 p-3 rounded-lg border border-teal-700/30">
                  <!-- Subgroup Label -->
                  <div class="flex flex-col items-center w-full mb-2">
                    <div class="px-3 py-1 bg-gradient-to-br from-teal-600 to-teal-700 text-white rounded-lg font-bold shadow-lg">
                      {{ subgroup.name }}
                    </div>
                    <div class="text-xs text-gray-500 mt-1">Subgroup Bus</div>
                  </div>
                  
                  <!-- Tracks Input -->
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-gradient-to-br from-gray-600 to-gray-700 text-white rounded-lg font-bold shadow-lg">
                      Tracks
                    </div>
                    <div class="text-xs text-gray-500 mt-1">Routed tracks</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <!-- Subgroup Channel -->
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-teal-700 text-white rounded-lg font-semibold">
                      Channel
                    </div>
                    <div class="text-xs text-gray-500 mt-1">Tone.Channel</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <!-- Subgroup FX Chain -->
                  <div class="flex flex-col items-center border-2 border-dashed border-teal-400 rounded-lg p-3 bg-teal-900/10">
                    <div class="text-xs text-teal-300 font-bold mb-2">SUBGROUP FX</div>
                    <div class="flex flex-col gap-1.5">
                      <div class="flex items-center gap-1">
                        <div class="w-2 h-2 rounded-full bg-green-500"></div>
                        <div class="px-2 py-1 bg-teal-700/60 text-white rounded text-xs font-semibold">Compressor</div>
                      </div>
                      <div class="flex items-center gap-1">
                        <div class="w-2 h-2 rounded-full bg-green-500"></div>
                        <div class="px-2 py-1 bg-teal-700/60 text-white rounded text-xs font-semibold">Reverb</div>
                      </div>
                      <div class="flex items-center gap-1">
                        <div class="w-2 h-2 rounded-full bg-green-500"></div>
                        <div class="px-2 py-1 bg-teal-700/60 text-white rounded text-xs font-semibold">Delay</div>
                      </div>
                      <div class="flex items-center gap-1">
                        <div class="w-2 h-2 rounded-full bg-green-500"></div>
                        <div class="px-2 py-1 bg-teal-700/60 text-white rounded text-xs font-semibold">Limiter</div>
                      </div>
                    </div>
                    <div class="text-[9px] text-teal-400 mt-1">Green = if enabled</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <!-- Volume Fader -->
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">
                      Volume
                    </div>
                    <div class="text-xs text-gray-500 mt-1">Fader</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <!-- To Master/Output -->
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-gradient-to-br from-orange-600 to-red-600 text-white rounded-lg font-bold shadow-lg">
                      Master/Output
                    </div>
                    <div class="text-xs text-gray-500 mt-1">Routing</div>
                  </div>
                </div>
              </div>
            </div>
            
            <div v-if="props.subgroups && props.subgroups.length > 0" class="border-t border-gray-700 my-6"></div>
            
            <!-- Aux Buses Chain -->
            <div v-if="props.auxBuses && props.auxBuses.length > 0" class="mb-8">
              <h3 class="text-lg font-bold text-amber-400 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2 5a2 2 0 012-2h7a2 2 0 012 2v4a2 2 0 01-2 2H9l-3 3v-3H4a2 2 0 01-2-2V5z"></path>
                  <path d="M15 7v2a4 4 0 01-4 4H9.828l-1.766 1.767c.28.149.599.233.938.233h2l3 3v-3h2a2 2 0 002-2V9a2 2 0 00-2-2h-1z"></path>
                </svg>
                Aux Buses Signal Chain
              </h3>
              
              <div class="space-y-6">
                <!-- Aux Send from Audio Track -->
                <div class="bg-gray-800/30 rounded-lg p-4 border border-gray-700">
                  <div class="text-sm text-gray-400 italic mb-4 text-center">From Audio Track:</div>
                  
                  <div class="flex gap-8 items-start justify-center">
                    <!-- Pre-Fader Send -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-yellow-400 font-bold">PRE-FADER SEND</div>
                      <div class="px-3 py-2 bg-yellow-600/80 text-white rounded text-xs font-semibold border-2 border-yellow-500">
                        Tap @ Balance Merge
                      </div>
                      <div class="text-[9px] text-gray-400 text-center">Signal before track volume<br/>Unaffected by track fader</div>
                    </div>
                    
                    <!-- Post-Fader Send -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-amber-400 font-bold">POST-FADER SEND</div>
                      <div class="px-3 py-2 bg-amber-600/80 text-white rounded text-xs font-semibold border-2 border-amber-500">
                        Tap @ Volume Merge
                      </div>
                      <div class="text-[9px] text-gray-400 text-center">Signal after track volume<br/>Follows track fader level</div>
                    </div>
                  </div>
                  
                  <div class="text-xs text-gray-500 mt-3 text-center">↓</div>
                  <div class="flex justify-center mt-2">
                    <div class="px-3 py-1 bg-gray-700 text-white rounded text-xs">Aux Send Gain</div>
                  </div>
                </div>
                
                <!-- Aux Buses Loop -->
                <div class="space-y-3">
                  <div v-for="aux in props.auxBuses.slice(0, 1)" :key="aux.id" class="flex flex-wrap items-center justify-between gap-3 text-sm border-2 border-amber-500/50 rounded-lg p-3 bg-amber-900/10">
                    <!-- Aux Name -->
                    <div class="flex flex-col items-center">
                      <div class="px-3 py-2 bg-gradient-to-br from-yellow-600 to-amber-600 text-white rounded-lg font-bold shadow-lg">
                        {{ aux.name }}
                      </div>
                      <div class="text-xs text-gray-500 mt-1">Aux Bus</div>
                    </div>
                    
                    <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                    </svg>
                    
                    <!-- Tone Channel -->
                    <div class="flex flex-col items-center">
                      <div class="px-3 py-2 bg-gradient-to-r from-yellow-600 to-amber-600 text-white rounded text-xs font-bold">
                        Tone.Channel
                      </div>
                      <div class="text-[9px] text-gray-400 mt-1">Vol, Pan, M, S</div>
                    </div>
                    
                    <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                    </svg>
                    
                    <!-- Aux Volume -->
                    <div class="flex flex-col items-center">
                      <div class="px-3 py-1 bg-gray-700 text-white rounded text-xs">Aux Volume</div>
                      <div class="text-[9px] text-gray-400 mt-1">Gain control</div>
                    </div>
                    
                    <div class="text-xs text-gray-500">→ SPLIT</div>
                    
                    <!-- Route to Master -->
                    <div class="flex flex-col items-center border-2 border-dashed border-orange-400 rounded-lg p-2 bg-orange-900/10">
                      <div class="text-[10px] text-orange-400 font-bold mb-1">TO MASTER</div>
                      <div class="px-2 py-1 bg-gradient-to-r from-red-600 to-orange-600 text-white rounded text-[10px] font-semibold">
                        Optional
                      </div>
                      <div class="text-[8px] text-gray-400 mt-1">If enabled</div>
                    </div>
                    
                    <div class="text-xs text-gray-500">+</div>
                    
                    <!-- Independent Output -->
                    <div class="flex items-center gap-2 border-2 border-green-500 rounded-lg p-2 bg-green-900/10">
                      <div class="flex flex-col items-center gap-1">
                        <div class="text-[10px] text-green-400 font-bold">OUTPUT</div>
                        <div class="px-2 py-0.5 bg-gray-700 text-white rounded text-[9px]">MediaStream</div>
                      </div>
                      <div class="text-[10px] text-gray-500">→</div>
                      <div class="flex flex-col items-center gap-1">
                        <div class="px-2 py-0.5 bg-blue-600/80 text-white rounded text-[9px]">AudioContext</div>
                        <div class="text-[8px] text-gray-400">sinkId</div>
                      </div>
                      <div class="text-[10px] text-gray-500">→</div>
                      <div class="flex gap-1">
                        <div class="px-1.5 py-1 bg-gradient-to-r from-green-600 to-teal-600 text-white rounded text-[9px] font-bold">🔇</div>
                        <div class="px-1.5 py-1 bg-gradient-to-r from-green-600 to-teal-600 text-white rounded text-[9px] font-bold">🔊</div>
                        <div class="px-1.5 py-1 bg-gradient-to-r from-green-600 to-teal-600 text-white rounded text-[9px] font-bold">🎧</div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- Aux Buses Notes -->
                <div class="mt-4 p-4 bg-gray-800/30 rounded-lg border border-gray-700 text-xs text-gray-400 space-y-2">
                  <p><strong>Pre-Fader vs Post-Fader:</strong> Pre-fader sends tap the signal after panning but before the track volume control, ensuring consistent aux level regardless of the track fader position. Post-fader sends tap after the volume control, so the aux send level follows the track fader.</p>
                  <p><strong>Dual Routing:</strong> Each Aux Bus can simultaneously route to the Master Bus (for mixing back into the main output) AND to an independent output device (for separate monitoring or recording).</p>
                  <p><strong>Output Device Selection:</strong> Choose "No Output" to disable audio, "Default" for the system default output device, or select a specific audio device. Each aux can route to a different physical output.</p>
                  <p><strong>Audio Architecture:</strong> Uses MediaStreamDestination to create a separate audio stream, then connects it to a dedicated AudioContext with setSinkId() for device-specific routing, independent of the main Tone.js context.</p>
                </div>
              </div>
            </div>
            
            <div v-if="props.auxBuses && props.auxBuses.length > 0" class="border-t border-gray-700 my-6"></div>
            
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
                  <div class="text-xs text-gray-500 mt-1">All tracks mixed</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Master Channel -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-orange-700 text-white rounded-lg font-semibold">
                    Master Channel
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Tone.Channel</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Parametric EQ -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-blue-600 text-white rounded-lg font-semibold shadow-lg">
                    Parametric EQ
                  </div>
                  <div class="text-xs text-gray-500">5-band + outputNode</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Master FX Chain -->
                <div class="flex flex-col items-center border-2 border-dashed border-orange-400 rounded-lg p-3 bg-orange-900/10">
                  <div class="text-xs text-orange-300 font-bold mb-2">MASTER FX CHAIN</div>
                  <div class="flex flex-col gap-1.5">
                    <div class="flex items-center gap-1">
                      <div class="w-2 h-2 rounded-full bg-green-500"></div>
                      <div class="px-2 py-1 bg-orange-700/60 text-white rounded text-xs font-semibold">Compressor</div>
                    </div>
                    <div class="flex items-center gap-1">
                      <div class="w-2 h-2 rounded-full bg-green-500"></div>
                      <div class="px-2 py-1 bg-orange-700/60 text-white rounded text-xs font-semibold">Reverb</div>
                    </div>
                    <div class="flex items-center gap-1">
                      <div class="w-2 h-2 rounded-full bg-green-500"></div>
                      <div class="px-2 py-1 bg-orange-700/60 text-white rounded text-xs font-semibold">Delay</div>
                    </div>
                    <div class="flex items-center gap-1">
                      <div class="w-2 h-2 rounded-full bg-green-500"></div>
                      <div class="px-2 py-1 bg-orange-700/60 text-white rounded text-xs font-semibold">Limiter</div>
                    </div>
                  </div>
                  <div class="text-[9px] text-orange-400 mt-1">Green = if enabled</div>
                  <div class="text-[9px] text-gray-400 mt-1">inputNode → outputNode</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Output Node with Spectrum tap -->
                <div class="flex flex-col items-center gap-2">
                  <!-- Spectrum analyzer tap branching UP -->
                  <div class="flex flex-col items-center gap-1">
                    <div class="px-2 py-1 bg-purple-600 text-white rounded text-[10px] font-bold shadow-lg">
                      Spectrum
                    </div>
                    <svg class="w-4 h-6 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M12 19V5"/>
                    </svg>
                    <div class="text-[9px] text-purple-500 font-bold">↓ TAP (POST-FX)</div>
                  </div>
                  <!-- MasterFX Output Node -->
                  <div class="px-4 py-2 bg-orange-600 text-white rounded-lg font-semibold shadow-lg">
                    FX Output
                  </div>
                  <div class="text-xs text-gray-500">Tone.Gain</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Split Node -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-cyan-600 text-white rounded-lg font-semibold">
                    Split L/R
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Tone.Split</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- L/R Faders and Meters -->
                <div class="flex flex-col items-center border-2 border-blue-500 rounded-lg p-2 bg-blue-900/10">
                  <div class="text-xs text-blue-300 font-bold mb-1">MASTER FADERS</div>
                  <div class="flex gap-2">
                    <div class="flex flex-col gap-1">
                      <div class="px-2 py-1 bg-purple-600 text-white rounded text-xs font-semibold">L Gain</div>
                      <div class="px-2 py-1 bg-yellow-600/80 text-white rounded text-xs">L Meter</div>
                    </div>
                    <div class="flex flex-col gap-1">
                      <div class="px-2 py-1 bg-blue-600 text-white rounded text-xs font-semibold">R Gain</div>
                      <div class="px-2 py-1 bg-yellow-600/80 text-white rounded text-xs">R Meter</div>
                    </div>
                  </div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Merge -->
                <div class="flex flex-col items-center">
                  <div class="px-4 py-2 bg-cyan-600 text-white rounded-lg font-semibold">
                    Merge
                  </div>
                  <div class="text-xs text-gray-500 mt-1">Tone.Merge</div>
                </div>
                
                <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                </svg>
                
                <!-- Output Split -->
                <div class="flex flex-col gap-3">                  
                  <div class="flex gap-4">
                    <!-- Main Output -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-blue-400 font-bold">MAIN OUT</div>
                      <div class="flex flex-col gap-2 border-2 border-blue-500 rounded-lg p-2 bg-blue-900/10">
                        <div class="px-3 py-1 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded text-xs font-bold">
                          Speakers
                        </div>
                        <div class="text-[9px] text-gray-400">toDestination()</div>
                      </div>
                    </div>
                    
                    <!-- Headphones -->
                    <div class="flex flex-col items-center gap-2">
                      <div class="text-xs text-green-400 font-bold">HEADPHONES</div>
                      <div class="flex flex-col gap-1 border-2 border-green-500 rounded-lg p-2 bg-green-900/10">
                        <div class="px-2 py-1 bg-gray-700 text-white rounded text-xs font-semibold">HP Gain</div>
                        <div class="px-2 py-1 bg-yellow-600/80 text-white rounded text-xs">HP Meter</div>
                        <div class="px-2 py-1 bg-gray-700 text-white rounded text-xs">MediaStream</div>
                        <div class="px-2 py-1 bg-gradient-to-r from-green-600 to-teal-600 text-white rounded text-xs font-bold">
                          <div class="flex items-center gap-1 justify-center">
                            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                              <path d="M7 4a3 3 0 016 0v4a3 3 0 11-6 0V4zm4 10.93A7.001 7.001 0 0017 8a1 1 0 10-2 0A5 5 0 015 8a1 1 0 00-2 0 7.001 7.001 0 006 6.93V17H6a1 1 0 100 2h8a1 1 0 100-2h-3v-2.07z"/>
                            </svg>
                            <span>HP Out</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Legend -->
            <div class="mt-8 p-4 bg-gray-800/50 rounded-lg border border-gray-700">
              <h4 class="text-sm font-bold text-gray-400 mb-3">Legend</h4>
              <div class="grid grid-cols-2 gap-3 text-xs text-gray-500">
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
                  <div class="w-3 h-3 border-2 border-dashed border-orange-400 rounded"></div>
                  <span>Conditional Chain</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="w-2 h-2 rounded-full bg-green-500"></div>
                  <span>Active when enabled</span>
                </div>
                <div class="flex items-center gap-2">
                  <div class="text-xs font-bold text-yellow-500">↓ TAP</div>
                  <span>Analysis tap (non-blocking)</span>
                </div>
              </div>
              <div class="mt-3 pt-3 border-t border-gray-700 text-xs text-gray-400">
                <p class="mb-1"><strong>Note:</strong> Analysis taps (VU, Waveform, Spectrum) are non-blocking connections that monitor the signal without affecting the main audio flow.</p>
                <p class="mb-1"><strong>Master Architecture:</strong> MasterEQDisplay creates an outputNode that feeds into MasterFX. MasterFX processes effects and outputs to both SpectrumMeter (for visualization) and MasterSection (for audio output).</p>
                <p><strong>FX Chain Order:</strong> Compressor → Reverb → Delay → Limiter (only if enabled via MasterFX controls). Spectrum analyzes the signal POST-FX processing.</p>
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

interface Subgroup {
  id: number
  name: string
}

interface AuxBus {
  id: string
  name: string
}

interface Props {
  modelValue: boolean
  subgroups?: Subgroup[]
  auxBuses?: AuxBus[]
}

const props = withDefaults(defineProps<Props>(), {
  subgroups: () => [],
  auxBuses: () => []
})

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

/* Custom scrollbar styling */
.custom-scrollbar::-webkit-scrollbar {
  width: 12px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(31, 41, 55, 0.5); /* gray-800 with opacity */
  border-radius: 6px;
  margin: 4px 0;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #9333ea 0%, #3b82f6 100%); /* purple-600 to blue-600 */
  border-radius: 6px;
  border: 2px solid rgba(31, 41, 55, 0.5);
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #a855f7 0%, #60a5fa 100%); /* purple-500 to blue-400 */
}

.custom-scrollbar::-webkit-scrollbar-thumb:active {
  background: linear-gradient(180deg, #c084fc 0%, #93c5fd 100%); /* purple-400 to blue-300 */
}

/* Firefox scrollbar */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: #9333ea rgba(31, 41, 55, 0.5);
}
</style>
