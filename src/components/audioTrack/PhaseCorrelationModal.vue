<template>
    <Teleport to="body">
        <Transition 
            enter-active-class="transition-all duration-200 ease-out"
            enter-from-class="opacity-0"
            enter-to-class="opacity-100"
            leave-active-class="transition-all duration-150 ease-in"
            leave-from-class="opacity-100"
            leave-to-class="opacity-0"
        >
            <div v-if="modelValue" 
                @click="emit('update:modelValue', false)"
                class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[200]"
            >
                <Transition
                    enter-active-class="transition-all duration-200 ease-out"
                    enter-from-class="opacity-0 scale-90"
                    enter-to-class="opacity-100 scale-100"
                    leave-active-class="transition-all duration-150 ease-in"
                    leave-from-class="opacity-100 scale-100"
                    leave-to-class="opacity-0 scale-90"
                >
                    <div v-if="modelValue"
                        @click.stop
                        class="bg-gradient-to-br from-gray-800 to-gray-900 rounded-xl border-2 border-gray-700 shadow-2xl p-6 w-[500px] max-w-[90vw]"
                    >
                        <!-- Header -->
                        <div class="flex items-center justify-between mb-4">
                            <div>
                                <h3 class="text-lg font-bold text-white">Phase Correlation Meter</h3>
                                <p class="text-xs text-gray-400 mt-0.5">Track {{ trackNumber }} - Stereo Width Analysis</p>
                            </div>
                            <button
                                @click="emit('update:modelValue', false)"
                                class="w-8 h-8 rounded-full bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white transition-colors flex items-center justify-center"
                                title="Close"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>

                        <!-- Large Gauge Display -->
                        <div class="bg-gray-900/50 rounded-lg p-6 mb-4">
                            <div class="relative w-full h-40 flex items-center justify-center">
                                <svg viewBox="0 0 200 100" class="w-full h-full">
                                    <!-- Background arc with color zones -->
                                    <!-- Red zone: -1 to -0.2 -->
                                    <path 
                                        d="M 15 92 A 85 85 0 0 1 75 10" 
                                        fill="none" 
                                        stroke="#ef4444" 
                                        stroke-width="12"
                                        opacity="0.3"
                                    />
                                    
                                    <!-- Yellow zone: -0.2 to +0.2 -->
                                    <path 
                                        d="M 75 10 A 85 85 0 0 1 125 10" 
                                        fill="none" 
                                        stroke="#eab308" 
                                        stroke-width="12"
                                        opacity="0.3"
                                    />
                                    
                                    <!-- Green zone: +0.2 to +0.85 -->
                                    <path 
                                        d="M 125 10 A 85 85 0 0 1 182 73" 
                                        fill="none" 
                                        stroke="#22c55e" 
                                        stroke-width="12"
                                        opacity="0.3"
                                    />
                                    
                                    <!-- Blue zone: +0.85 to +1 -->
                                    <path 
                                        d="M 182 73 A 85 85 0 0 1 185 92" 
                                        fill="none" 
                                        stroke="#3b82f6" 
                                        stroke-width="12"
                                        opacity="0.3"
                                    />
                                    
                                    <!-- Tick marks -->
                                    <line x1="100" y1="92" x2="100" y2="82" stroke="#666" stroke-width="1.5" />
                                    <line x1="22" y1="92" x2="15" y2="92" stroke="#666" stroke-width="1.5" />
                                    <line x1="178" y1="92" x2="185" y2="92" stroke="#666" stroke-width="1.5" />
                                    <line x1="56" y1="36" x2="50" y2="30" stroke="#666" stroke-width="1.5" />
                                    <line x1="144" y1="36" x2="150" y2="30" stroke="#666" stroke-width="1.5" />
                                    
                                    <!-- Center point -->
                                    <circle cx="100" cy="92" r="4" :fill="needleColor" opacity="0.6" />
                                    
                                    <!-- Needle -->
                                    <line 
                                        :x1="100" 
                                        :y1="92" 
                                        :x2="needleX" 
                                        :y2="needleY" 
                                        :stroke="needleColor" 
                                        stroke-width="3" 
                                        stroke-linecap="round"
                                    />
                                    
                                    <!-- Needle tip -->
                                    <circle :cx="needleX" :cy="needleY" r="5" :fill="needleColor" />
                                </svg>
                                
                                <!-- Labels -->
                                <div class="absolute inset-0 pointer-events-none">
                                    <div class="absolute left-[8%] bottom-3 text-[0.65rem] text-red-400 font-bold">-1</div>
                                    <div class="absolute left-[20%] top-6 text-[0.65rem] text-yellow-400 font-bold">-0.2</div>
                                    <div class="absolute left-1/2 -translate-x-1/2 top-0 text-[0.65rem] text-yellow-400 font-bold">0</div>
                                    <div class="absolute right-[20%] top-6 text-[0.65rem] text-green-400 font-bold">+0.2</div>
                                    <div class="absolute right-[13%] top-[38%] text-[0.65rem] text-blue-400 font-bold">+0.85</div>
                                    <div class="absolute right-[8%] bottom-3 text-[0.65rem] text-blue-400 font-bold">+1</div>
                                </div>
                            </div>
                            
                            <!-- Value Display -->
                            <div class="text-center mt-4">
                                <div 
                                    class="inline-block text-3xl font-mono font-bold px-4 py-2 rounded-lg"
                                    :class="valueColorClass"
                                >
                                    {{ correlationDisplay }}
                                </div>
                            </div>
                        </div>

                        <!-- Info Section -->
                        <div class="space-y-3 text-sm">
                            <!-- Current Status -->
                            <div class="bg-gray-900/50 rounded-lg p-3">
                                <div class="flex items-center justify-between">
                                    <span class="text-gray-400">Status:</span>
                                    <span :class="statusColorClass" class="font-bold">{{ statusText }}</span>
                                </div>
                            </div>

                            <!-- Zone Explanations -->
                            <div class="grid grid-cols-2 gap-2">
                                <div class="bg-red-900/20 border border-red-700/50 rounded p-2">
                                    <div class="flex items-center gap-1 mb-1">
                                        <div class="w-2 h-2 rounded-full bg-red-500"></div>
                                        <span class="text-xs font-bold text-red-300">Phase Issues</span>
                                    </div>
                                    <p class="text-[0.65rem] text-gray-400">-1 to -0.2: Severe phase cancellation</p>
                                </div>
                                
                                <div class="bg-yellow-900/20 border border-yellow-700/50 rounded p-2">
                                    <div class="flex items-center gap-1 mb-1">
                                        <div class="w-2 h-2 rounded-full bg-yellow-500"></div>
                                        <span class="text-xs font-bold text-yellow-300">Wide Stereo</span>
                                    </div>
                                    <p class="text-[0.65rem] text-gray-400">-0.2 to +0.2: Very wide stereo</p>
                                </div>
                                
                                <div class="bg-green-900/20 border border-green-700/50 rounded p-2">
                                    <div class="flex items-center gap-1 mb-1">
                                        <div class="w-2 h-2 rounded-full bg-green-500"></div>
                                        <span class="text-xs font-bold text-green-300">Good Stereo</span>
                                    </div>
                                    <p class="text-[0.65rem] text-gray-400">+0.2 to +0.85: Normal stereo range</p>
                                </div>
                                
                                <div class="bg-blue-900/20 border border-blue-700/50 rounded p-2">
                                    <div class="flex items-center gap-1 mb-1">
                                        <div class="w-2 h-2 rounded-full bg-blue-500"></div>
                                        <span class="text-xs font-bold text-blue-300">Near Mono</span>
                                    </div>
                                    <p class="text-[0.65rem] text-gray-400">+0.85 to +1: Centered/Mono</p>
                                </div>
                            </div>

                            <!-- Warning -->
                            <div v-if="correlation < -0.1" class="bg-red-900/30 border-2 border-red-600 rounded-lg p-3 animate-pulse">
                                <div class="flex items-center gap-2">
                                    <svg class="w-5 h-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                                    </svg>
                                    <div>
                                        <p class="text-red-300 font-bold text-xs">Phase Correlation Warning</p>
                                        <p class="text-red-400 text-[0.65rem] mt-0.5">Signal may have phase cancellation issues. Check mono compatibility.</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Transition>
            </div>
        </Transition>
    </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
    modelValue: boolean
    correlation: number
    trackNumber: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
    'update:modelValue': [value: boolean]
}>()

// Clamp correlation
const clampedCorrelation = computed(() => Math.max(-1, Math.min(1, props.correlation)))

// Needle angle calculation
const needleAngle = computed(() => {
    const normalized = (clampedCorrelation.value + 1) / 2
    return 180 - (normalized * 180)
})

// Needle position (scaled for larger gauge)
const needleX = computed(() => {
    const angleRad = (needleAngle.value * Math.PI) / 180
    return 100 + 75 * Math.cos(angleRad)
})

const needleY = computed(() => {
    const angleRad = (needleAngle.value * Math.PI) / 180
    return 92 - 75 * Math.sin(angleRad)
})

// Needle color
const needleColor = computed(() => {
    const c = clampedCorrelation.value
    if (c < -0.2) return '#ef4444'
    if (c < 0.2) return '#eab308'
    if (c < 0.85) return '#22c55e'
    return '#3b82f6'
})

// Value color class
const valueColorClass = computed(() => {
    const c = clampedCorrelation.value
    if (c < -0.2) return 'bg-red-900/50 text-red-300 border-2 border-red-500'
    if (c < 0.2) return 'bg-yellow-900/50 text-yellow-300 border-2 border-yellow-500'
    if (c < 0.85) return 'bg-green-900/50 text-green-300 border-2 border-green-500'
    return 'bg-blue-900/50 text-blue-300 border-2 border-blue-500'
})

// Status color
const statusColorClass = computed(() => {
    const c = clampedCorrelation.value
    if (c < -0.2) return 'text-red-400'
    if (c < 0.2) return 'text-yellow-400'
    if (c < 0.85) return 'text-green-400'
    return 'text-blue-400'
})

// Status text
const statusText = computed(() => {
    const c = clampedCorrelation.value
    if (c < -0.2) return '⚠ Phase Issues Detected'
    if (c < 0.2) return '⚠ Very Wide Stereo - Check Mono Compatibility'
    if (c < 0.85) return '✓ Good Stereo Correlation'
    return 'ⓘ Near Mono / Centered'
})

// Display value
const correlationDisplay = computed(() => {
    const c = clampedCorrelation.value
    return c >= 0 ? `+${c.toFixed(3)}` : c.toFixed(3)
})
</script>
