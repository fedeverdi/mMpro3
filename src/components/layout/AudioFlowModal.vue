<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/70" @click.self="close">
        <!-- Modal -->
        <div class="bg-gray-900 rounded-lg shadow-2xl max-w-7xl w-full max-h-[90vh] border border-gray-700 overflow-hidden">
          <!-- Header -->
          <div class="flex items-center justify-between p-4 border-b border-gray-700">
            <div class="flex items-center gap-3">
              <h2 class="text-xl font-bold text-white">🔀 System Architecture & Signal Flow</h2>
              <div class="px-2 py-1 bg-orange-600/20 border border-orange-500 rounded text-xs font-semibold text-orange-400">
                Rust DSP Engine
              </div>
            </div>
            <button @click="close" class="text-gray-400 hover:text-white transition-colors">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
          
          <!-- Content -->
          <div class="p-6 overflow-y-auto max-h-[calc(90vh-80px)] custom-scrollbar bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900">
            
            <!-- ============ SYSTEM OVERVIEW ============ -->
            <div class="mb-8">
              <h3 class="text-xl font-bold text-cyan-400 mb-6 flex items-center gap-2">
                <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z"/>
                </svg>
                System Architecture Overview
              </h3>
              
              <div class="flex flex-col gap-4">
                <!-- Top: ASIO Input -->
                <div class="flex justify-center">
                  <div class="bg-gradient-to-br from-green-600 to-emerald-700 border-2 border-green-400 rounded-xl p-4 shadow-2xl w-80">
                    <div class="flex items-center justify-center gap-2 mb-2">
                      <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 12a2 2 0 100-4 2 2 0 000 4z"/>
                        <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd"/>
                      </svg>
                      <h4 class="text-lg font-bold text-white">ASIO Audio Input</h4>
                    </div>
                    <p class="text-xs text-green-100 text-center">Hardware audio interface (via cpal library)</p>
                    <div class="mt-2 flex justify-center gap-2">
                      <div class="px-2 py-1 bg-white/20 rounded text-xs text-white font-mono">Input Channels 1-N</div>
                    </div>
                  </div>
</div>
                
                <!-- Arrow Down -->
                <div class="flex justify-center">
                  <svg class="w-8 h-8 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M19 14l-7 7m0 0l-7-7m7 7V3"/>
                  </svg>
                </div>
                
                <!-- Middle: Rust DSP Engine (BIG BOX) -->
                <div class="bg-gradient-to-br from-emerald-500 to-teal-600 border-4 border-emerald-400 rounded-2xl p-6 shadow-2xl">
                  <div class="flex items-center justify-center gap-3 mb-4">
                    <svg class="w-7 h-7 text-white" fill="currentColor" viewBox="0 0 20 20">
                      <path fill-rule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clip-rule="evenodd"/>
                    </svg>
                    <h4 class="text-2xl font-black text-white">RUST DSP ENGINE</h4>
                    <div class="px-2 py-1 bg-white/30 rounded-lg text-xs font-bold text-white">REALTIME AUDIO</div>
                  </div>
                  <p class="text-sm text-emerald-100 text-center mb-6">Dedicated audio processing engine running in separate Rust process</p>
                  
                  <!-- DSP Engine Internal Architecture -->
                  <div class="bg-black/30 rounded-xl p-5 border-2 border-emerald-300/50">
                    <div class="space-y-4">
                      
                      <!-- Router -->
                      <div class="bg-gray-800/80 rounded-lg p-3 border border-gray-600">
                        <div class="text-sm font-bold text-orange-300 mb-2 flex items-center gap-2">
                          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zM8 7a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zM14 4a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z"/>
                          </svg>
                          Router Core
                        </div>
                        <p class="text-xs text-gray-400">Central routing manager orchestrating all audio paths</p>
                      </div>
                      
                      <!-- Tracks Container -->
                      <div class="bg-purple-900/40 rounded-lg p-4 border-2 border-purple-400/50">
                        <div class="text-sm font-bold text-purple-300 mb-3 flex items-center gap-2">
                          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M2 6a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM2 12a2 2 0 012-2h12a2 2 0 012 2v2a2 2 0 01-2 2H4a2 2 0 01-2-2v-2z"/>
                          </svg>
                          Audio Tracks (1-N)
                        </div>
                        
                        <!-- Single Track Signal Chain (compact) -->
                        <div class="bg-black/40 rounded p-3 text-xs">
                          <div class="text-purple-200 font-semibold mb-2">Per-Track Signal Chain:</div>
                          <div class="flex flex-wrap items-center gap-2 text-[10px]">
                            <div class="px-2 py-1 bg-green-700 text-white rounded font-bold">Input</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-yellow-700 text-white rounded">PAD</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-gray-700 text-white rounded">Gain</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-blue-700 text-white rounded">HPF</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-purple-700 text-white rounded">Gate</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-indigo-700 text-white rounded">Compressor</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-blue-600 text-white rounded">EQ</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-cyan-700 text-white rounded">Pan</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-gray-700 text-white rounded">Volume</div>
                            <span class="text-gray-500">→</span>
                            <div class="px-2 py-1 bg-orange-600 text-white rounded font-bold">Routing</div>
                          </div>
                          <div class="mt-2 text-gray-400">Sources: Audio Input / Signal Generator / File Player / Aux Return</div>
                        </div>
                      </div>
                      
                      <!-- Aux Buses -->
                      <div class="bg-amber-900/40 rounded-lg p-4 border-2 border-amber-400/50">
                        <div class="text-sm font-bold text-amber-300 mb-2 flex items-center gap-2">
                          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                          </svg>
                          Aux Buses (FX Send/Return)
                        </div>
                        <p class="text-xs text-amber-200">Pre/Post-fader sends from tracks → FX processing → Return to mix</p>
                        <div class="mt-2 flex flex-wrap gap-2 text-[10px]">
                          <div class="px-2 py-1 bg-amber-700 text-white rounded">Aux 1-6</div>
                          <div class="px-2 py-1 bg-green-700 text-white rounded">Reverb</div>
                          <div class="px-2 py-1 bg-blue-700 text-white rounded">Delay</div>
                          <div class="px-2 py-1 bg-purple-700 text-white rounded">Custom FX</div>
                        </div>
                      </div>
                      
                      <!-- Subgroups -->
                      <div class="bg-teal-900/40 rounded-lg p-4 border-2 border-teal-400/50">
                        <div class="text-sm font-bold text-teal-300 mb-2 flex items-center gap-2">
                          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4z"/>
                          </svg>
                          Subgroups (Stem Buses)
                        </div>
                        <p class="text-xs text-teal-200">Group multiple tracks → Process as single bus → Route to master</p>
                        <div class="mt-2 text-[10px] text-gray-300">
                          Signal: Tracks → Subgroup → Volume → Master
                        </div>
                      </div>
                      
                      <!-- Master Bus -->
                      <div class="bg-red-900/40 rounded-lg p-4 border-2 border-red-400/50">
                        <div class="text-sm font-bold text-red-300 mb-2 flex items-center gap-2">
                          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-11a1 1 0 10-2 0v3.586L7.707 9.293a1 1 0 00-1.414 1.414l3 3a1 1 0 001.414 0l3-3a1 1 0 00-1.414-1.414L11 10.586V7z" clip-rule="evenodd"/>
                          </svg>
                          Master Bus
                        </div>
                        <p class="text-xs text-red-200 mb-2">Final mix bus combining all tracks/subgroups</p>
                        <div class="flex flex-wrap gap-2 text-[10px]">
                          <div class="px-2 py-1 bg-gray-700 text-white rounded">Master Volume</div>
                          <div class="px-2 py-1 bg-blue-600 text-white rounded">Master EQ</div>
                          <div class="px-2 py-1 bg-red-700 text-white rounded">Master Limiter</div>
                        </div>
                      </div>
                      
                    </div>
                  </div>
                  
                  <!-- DSP Statistics -->
                  <div class="mt-4 grid grid-cols-3 gap-3 text-xs">
                    <div class="bg-black/40 rounded p-2 text-center border border-emerald-400/30">
                      <div class="text-emerald-300 font-bold">Sample Rate</div>
                      <div class="text-white font-mono">44.1/48/96 kHz</div>
                    </div>
                    <div class="bg-black/40 rounded p-2 text-center border border-emerald-400/30">
                      <div class="text-emerald-300 font-bold">Bit Depth</div>
                      <div class="text-white font-mono">32-bit float</div>
                    </div>
                    <div class="bg-black/40 rounded p-2 text-center border border-emerald-400/30">
                      <div class="text-emerald-300 font-bold">Latency</div>
                      <div class="text-white font-mono">Ultra-low</div>
                    </div>
                  </div>
                </div>
                
                <!-- Arrow Down -->
                <div class="flex justify-center">
                  <svg class="w-8 h-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M19 14l-7 7m0 0l-7-7m7 7V3"/>
                  </svg>
                </div>
                
                <!-- Bottom: ASIO Output -->
                <div class="flex justify-center">
                  <div class="bg-gradient-to-br from-blue-600 to-indigo-700 border-2 border-blue-400 rounded-xl p-4 shadow-2xl w-80">
                    <div class="flex items-center justify-center gap-2 mb-2">
                      <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"/>
                      </svg>
                      <h4 class="text-lg font-bold text-white">ASIO Audio Output</h4>
                    </div>
                    <p class="text-xs text-blue-100 text-center">Hardware audio interface (via cpal library)</p>
                    <div class="mt-2 flex justify-center gap-2">
                      <div class="px-2 py-1 bg-white/20 rounded text-xs text-white font-mono">Output Channels 1-N</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <div class="border-t-2 border-gray-700 my-8"></div>
            
            <!-- ============ COMMUNICATION LAYER ============ -->
            <div class="mb-8">
              <h3 class="text-xl font-bold text-blue-400 mb-6 flex items-center gap-2">
                <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2.003 5.884L10 9.882l7.997-3.998A2 2 0 0016 4H4a2 2 0 00-1.997 1.884z"/>
                  <path d="M18 8.118l-8 4-8-4V14a2 2 0 002 2h12a2 2 0 002-2V8.118z"/>
                </svg>
                Inter-Process Communication (IPC)
              </h3>
              
              <div class="bg-gray-800/50 rounded-xl p-6 border-2 border-blue-500/50">
                <div class="flex items-center justify-between gap-8">
                  <!-- Vue/Electron UI -->
                  <div class="flex-1 bg-gradient-to-br from-cyan-600 to-blue-700 rounded-lg p-4 border-2 border-cyan-400">
                    <div class="text-center">
                      <div class="text-lg font-bold text-white mb-2">Vue.js / Electron UI</div>
                      <div class="text-xs text-cyan-100 mb-3">Frontend Interface</div>
                      <div class="space-y-1 text-[10px] text-left text-cyan-100">
                        <div>• User controls & parameters</div>
                        <div>• Visualization & metering</div>
                        <div>• File management</div>
                        <div>• Scene management</div>
                      </div>
                    </div>
                  </div>
                  
                  <!-- Communication Channel -->
                  <div class="flex flex-col items-center gap-2">
                    <svg class="w-8 h-8 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                    </svg>
                    <div class="bg-blue-900/50 rounded px-3 py-2 border border-blue-500">
                      <div class="text-xs font-bold text-blue-300 text-center">JSON Commands</div>
                      <div class="text-[9px] text-gray-400 text-center mt-1">stdin / stdout</div>
                    </div>
                    <svg class="w-8 h-8 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                    </svg>
                  </div>
                  
                  <!-- Rust Engine -->
                  <div class="flex-1 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-lg p-4 border-2 border-emerald-400">
                    <div class="text-center">
                      <div class="text-lg font-bold text-white mb-2">Rust Audio Engine</div>
                      <div class="text-xs text-emerald-100 mb-3">Backend DSP</div>
                      <div class="space-y-1 text-[10px] text-left text-emerald-100">
                        <div>• Audio input/output (ASIO)</div>
                        <div>• Realtime DSP processing</div>
                        <div>• Routing & mixing</div>
                        <div>• Effect processing</div>
                      </div>
                    </div>
                  </div>
                </div>
                
                <!-- Command Examples -->
                <div class="mt-6 bg-black/40 rounded-lg p-4 border border-gray-700">
                  <div class="text-sm font-bold text-blue-300 mb-3">Example Commands:</div>
                  <div class="space-y-2 text-xs font-mono">
                    <div class="flex items-start gap-2">
                      <div class="text-cyan-400">→</div>
                      <div class="flex-1 bg-gray-900 rounded p-2 text-cyan-300">
                        {"type": "set_gain", "track": 0, "gain": 0.8}
                      </div>
                      <div class="text-gray-500 text-[10px] self-center">Set track gain</div>
                    </div>
                    <div class="flex items-start gap-2">
                      <div class="text-cyan-400">→</div>
                      <div class="flex-1 bg-gray-900 rounded p-2 text-cyan-300">
                        {"type": "play_file", "track": 1, "file_path": "/path/to/file.wav"}
                      </div>
                      <div class="text-gray-500 text-[10px] self-center">Play audio file</div>
                    </div>
                    <div class="flex items-start gap-2">
                      <div class="text-orange-400">←</div>
                      <div class="flex-1 bg-gray-900 rounded p-2 text-orange-300">
                        {"type": "meter_update", "track": 0, "level_l": -12.5, "level_r": -14.2}
                      </div>
                      <div class="text-gray-500 text-[10px] self-center">Meter data</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <div class="border-t-2 border-gray-700 my-8"></div>
            
            <!-- ============ DETAILED TRACK SIGNAL CHAIN ============ -->
            <div class="mb-8">
              <h3 class="text-xl font-bold text-purple-400 mb-6 flex items-center gap-2">
                <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z"/>
                  <path fill-rule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clip-rule="evenodd"/>
                </svg>
                Detailed Audio Track Signal Chain
              </h3>
              
              <div class="bg-purple-900/20 rounded-xl p-6 border-2 border-purple-500/50">
                <div class="flex flex-wrap items-center gap-3 text-sm justify-center">
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-gradient-to-br from-green-600 to-green-700 text-white rounded-lg font-bold shadow-lg">Input</div>
                    <div class="text-xs text-gray-400 mt-1">Audio Source</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-yellow-700 text-white rounded-lg font-semibold border-2 border-dashed border-yellow-400">PAD</div>
                    <div class="text-xs text-yellow-400 mt-1">-24dB</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">Gain</div>
                    <div class="text-xs text-gray-400 mt-1">Preamp</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-blue-700 text-white rounded-lg font-semibold border-2 border-dashed border-blue-400">HPF</div>
                    <div class="text-xs text-blue-400 mt-1">80Hz</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-purple-700 text-white rounded-lg font-semibold border-2 border-dashed border-purple-400">Gate</div>
                    <div class="text-xs text-purple-400 mt-1">Noise gate</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-indigo-700 text-white rounded-lg font-semibold border-2 border-dashed border-indigo-400">Compressor</div>
                    <div class="text-xs text-indigo-400 mt-1">Dynamics</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-blue-600 text-white rounded-lg font-semibold shadow-lg">EQ</div>
                    <div class="text-xs text-gray-400 mt-1">3-band + Parametric</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-cyan-700 text-white rounded-lg font-semibold">Pan</div>
                    <div class="text-xs text-gray-400 mt-1">Stereo position</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center border-2 border-amber-500 rounded-lg p-3 bg-amber-900/20">
                    <div class="text-xs text-amber-300 font-bold mb-2">AUX SENDS</div>
                    <div class="flex flex-col gap-1 text-[10px]">
                      <div class="px-2 py-1 bg-amber-700 text-white rounded">Aux 1-6 Sends</div>
                      <div class="text-amber-300 text-center">Pre/Post Fader</div>
                    </div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center">
                    <div class="px-4 py-2 bg-gray-700 text-white rounded-lg font-semibold">Volume</div>
                    <div class="text-xs text-gray-400 mt-1">Fader</div>
                  </div>
                  
                  <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                  </svg>
                  
                  <div class="flex flex-col items-center border-2 border-orange-500 rounded-lg p-3 bg-orange-900/20">
                    <div class="text-xs text-orange-300 font-bold mb-2">ROUTING</div>
                    <div class="flex flex-col gap-1 text-[10px] text-center">
                      <div class="px-2 py-1 bg-red-700 text-white rounded">Master</div>
                      <div class="px-2 py-1 bg-teal-700 text-white rounded">Subgroups</div>
                    </div>
                  </div>
                </div>
                
                <div class="mt-6 p-4 bg-black/40 rounded-lg border border-purple-600/30 text-xs text-gray-300 space-y-2">
                  <p><strong class="text-purple-400">Processing Order:</strong> All processing happens in fixed order within Rust for optimal CPU efficiency and predictable results.</p>
                  <p><strong class="text-purple-400">Dashed Borders:</strong> Indicate optional/bypassable processors. Solid borders are always in the signal path.</p>
                  <p><strong class="text-purple-400">Aux Sends:</strong> Can tap signal at pre-fader (post-pan) or post-fader points for flexible FX routing.</p>
                </div>
              </div>
            </div>
            
            <!-- ============ LEGEND & NOTES ============ -->
            <div class="bg-gray-800/70 rounded-xl p-6 border-2 border-gray-600">
              <h4 class="text-lg font-bold text-gray-300 mb-4 flex items-center gap-2">
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
                </svg>
                System Notes & Key Features
              </h4>
              
              <div class="grid md:grid-cols-2 gap-4 text-sm text-gray-300">
                <div class="space-y-2">
                  <div class="flex items-start gap-2">
                    <div class="text-emerald-400 font-bold">•</div>
                    <div><strong>Separation of Concerns:</strong> Rust handles all realtime audio processing while Vue.js provides the UI. This ensures low-latency audio performance unaffected by UI rendering.</div>
                  </div>
                  <div class="flex items-start gap-2">
                    <div class="text-emerald-400 font-bold">•</div>
                    <div><strong>ASIO via cpal:</strong> Direct ASIO driver communication for ultra-low latency audio I/O on Windows, with CoreAudio on macOS and ALSA/JACK on Linux.</div>
                  </div>
                  <div class="flex items-start gap-2">
                    <div class="text-emerald-400 font-bold">•</div>
                    <div><strong>Zero-Copy Processing:</strong> Audio samples are processed in-place where possible to minimize memory allocation and CPU cache misses.</div>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <div class="flex items-start gap-2">
                    <div class="text-blue-400 font-bold">•</div>
                    <div><strong>JSON Command Protocol:</strong> Simple text-based protocol over stdin/stdout enables easy debugging and extensibility.</div>
                  </div>
                  <div class="flex items-start gap-2">
                    <div class="text-blue-400 font-bold">•</div>
                    <div><strong>Realtime Safe:</strong> All audio processing uses lock-free data structures and pre-allocated buffers to prevent dropouts.</div>
                  </div>
                  <div class="flex items-start gap-2">
                    <div class="text-blue-400 font-bold">•</div>
                    <div><strong>Rust Safety:</strong> Memory safety and thread safety guaranteed at compile time, eliminating entire classes of audio bugs.</div>
                  </div>
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
  background: rgba(31, 41, 55, 0.5);
  border-radius: 6px;
  margin: 4px 0;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #f97316 0%, #dc2626 100%);
  border-radius: 6px;
  border: 2px solid rgba(31, 41, 55, 0.5);
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #fb923c 0%, #ef4444 100%);
}

.custom-scrollbar::-webkit-scrollbar-thumb:active {
  background: linear-gradient(180deg, #fdba74 0%, #f87171 100%);
}

/* Firefox scrollbar */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: #f97316 rgba(31, 41, 55, 0.5);
}
</style>
