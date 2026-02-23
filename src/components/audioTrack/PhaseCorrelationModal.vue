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
                            <div class="flex items-center gap-2">
                                <!-- View Toggle -->
                                <button
                                    @click="viewMode = viewMode === 'gauge' ? 'goniometer' : 'gauge'"
                                    class="px-3 py-1.5 rounded-lg bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white transition-colors text-xs font-semibold flex items-center gap-1.5"
                                    :title="viewMode === 'gauge' ? 'Switch to Goniometer' : 'Switch to Gauge'"
                                >
                                    <svg v-if="viewMode === 'gauge'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <circle cx="12" cy="12" r="10" stroke-width="2"/>
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6l4 2"/>
                                    </svg>
                                    <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                        <path d="M3 3h18v18H3V3zm2 2v14h14V5H5zm2 2h10v10H7V7z"/>
                                        <circle cx="12" cy="12" r="1.5"/>
                                    </svg>
                                    {{ viewMode === 'gauge' ? 'Goniometer' : 'Gauge' }}
                                </button>
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
                        </div>

                        <!-- Large Gauge Display -->
                        <div v-if="viewMode === 'gauge'" class="bg-gray-900/50 rounded-lg p-6 mb-4">
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

                        <!-- Goniometer Display -->
                        <div v-else class="bg-gray-900/50 rounded-lg p-6 mb-4">
                            <div class="relative w-full h-64 flex items-center justify-center">
                                <svg viewBox="0 0 200 200" class="w-full h-full">
                                    <!-- Background circle -->
                                    <circle cx="100" cy="100" r="90" fill="none" stroke="#374151" stroke-width="1" opacity="0.3" />
                                    <circle cx="100" cy="100" r="60" fill="none" stroke="#374151" stroke-width="1" opacity="0.2" />
                                    <circle cx="100" cy="100" r="30" fill="none" stroke="#374151" stroke-width="1" opacity="0.2" />
                                    
                                    <!-- Axes -->
                                    <line x1="10" y1="100" x2="190" y2="100" stroke="#4b5563" stroke-width="1.5" opacity="0.5" />
                                    <line x1="100" y1="10" x2="100" y2="190" stroke="#4b5563" stroke-width="1.5" opacity="0.5" />
                                    
                                    <!-- Phase zones (background) -->
                                    <!-- +1 zone (mono/center) - circle in middle -->
                                    <circle cx="100" cy="100" r="13" fill="#3b82f6" opacity="0.15" />
                                    
                                    <!-- +0.2 to +0.85 zone (good stereo) - diagonal wedge -->
                                    <path d="M 100 100 L 164 64 A 90 90 0 0 1 164 136 Z" fill="#22c55e" opacity="0.1" />
                                    <path d="M 100 100 L 36 64 A 90 90 0 0 0 36 136 Z" fill="#22c55e" opacity="0.1" />
                                    
                                    <!-- -0.2 to +0.2 zone (wide stereo) -->
                                    <path d="M 100 100 L 164 64 A 90 90 0 0 0 100 10 Z" fill="#eab308" opacity="0.1" />
                                    <path d="M 100 100 L 100 10 A 90 90 0 0 0 36 64 Z" fill="#eab308" opacity="0.1" />
                                    <path d="M 100 100 L 164 136 A 90 90 0 0 1 100 190 Z" fill="#eab308" opacity="0.1" />
                                    <path d="M 100 100 L 100 190 A 90 90 0 0 1 36 136 Z" fill="#eab308" opacity="0.1" />
                                    
                                    <!-- Reference lines for different phase correlations -->
                                    <!-- +1 (mono) - dot at center -->
                                    <circle cx="100" cy="100" r="2" fill="#3b82f6" opacity="0.5" />
                                    
                                    <!-- +0.707 (perfect stereo) - diagonal line -->
                                    <line x1="100" y1="100" x2="164" y2="64" stroke="#22c55e" stroke-width="1.5" stroke-dasharray="4,4" opacity="0.3" />
                                    
                                    <!-- -1 (out of phase) - opposite diagonal -->
                                    <line x1="100" y1="100" x2="164" y2="136" stroke="#ef4444" stroke-width="1.5" stroke-dasharray="4,4" opacity="0.3" />
                                    <line x1="100" y1="100" x2="36" y2="64" stroke="#ef4444" stroke-width="1.5" stroke-dasharray="4,4" opacity="0.3" />
                                    
                                    <!-- Real audio data points (Lissajous figure) -->
                                    <g v-for="(point, idx) in goniometerPoints" :key="idx">
                                        <circle 
                                            :cx="point.x" 
                                            :cy="point.y" 
                                            :r="point.size" 
                                            :fill="needleColor"
                                            :opacity="point.opacity"
                                        />
                                    </g>
                                    
                                    <!-- Labels -->
                                    <text x="195" y="105" class="text-[8px]" fill="#9ca3af" text-anchor="end">L</text>
                                    <text x="100" y="8" class="text-[8px]" fill="#9ca3af" text-anchor="middle">R</text>
                                    <text x="5" y="105" class="text-[8px]" fill="#9ca3af">-L</text>
                                    <text x="100" y="197" class="text-[8px]" fill="#9ca3af" text-anchor="middle">-R</text>
                                </svg>
                                
                                <!-- Phase indicator overlay -->
                                <div class="absolute inset-0 pointer-events-none flex items-start justify-center pt-2">
                                    <div class="text-[0.65rem] text-gray-400 bg-gray-900/80 px-2 py-1 rounded">
                                        {{ goniometerStatus }}
                                    </div>
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
                            <!-- <div v-if="props.correlation < -0.1" class="bg-red-900/30 border-2 border-red-600 rounded-lg p-3 animate-pulse">
                                <div class="flex items-center gap-2">
                                    <svg class="w-5 h-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                                    </svg>
                                    <div>
                                        <p class="text-red-300 font-bold text-xs">Phase Correlation Warning</p>
                                        <p class="text-red-400 text-[0.65rem] mt-0.5">Signal may have phase cancellation issues. Check mono compatibility.</p>
                                    </div>
                                </div>
                            </div> -->
                        </div>
                    </div>
                </Transition>
            </div>
        </Transition>
    </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'

interface Props {
    modelValue: boolean
    correlation: number
    trackNumber: number
    audioDataL: Float32Array
    audioDataR: Float32Array
    isStereo: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
    'update:modelValue': [value: boolean]
}>()

// View mode: 'gauge' or 'goniometer'
const viewMode = ref<'gauge' | 'goniometer'>('gauge')

// Animation frame counter to trigger reactivity
const frameCounter = ref(0)
let animationFrameId: number | null = null

const updateFrame = () => {
    frameCounter.value++
    animationFrameId = requestAnimationFrame(updateFrame)
}

onMounted(() => {
    updateFrame()
})

onUnmounted(() => {
    if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId)
    }
})

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

// Goniometer visualization - real audio data points
const goniometerPoints = computed(() => {
    // Trigger reactivity on every animation frame
    frameCounter.value
    
    if (!props.isStereo || !props.audioDataL || !props.audioDataR) {
        return []
    }
    
    const points: Array<{ x: number; y: number; opacity: number; size: number }> = []
    const dataLength = Math.min(props.audioDataL.length, props.audioDataR.length)
    
    // Sample points from audio data (take every nth sample for performance)
    const sampleStep = Math.max(1, Math.floor(dataLength / 400)) // Max 400 points for smooth visualization
    
    for (let i = 0; i < dataLength; i += sampleStep) {
        const l = props.audioDataL[i] || 0 // Left channel (-1 to +1)
        const r = props.audioDataR[i] || 0 // Right channel (-1 to +1)
        
        // Map audio values to screen coordinates
        // X axis = Left channel, Y axis = Right channel
        // Scale to fit in viewBox (center at 100,100, radius ~85)
        const x = 100 + l * 75 // Scale left channel to X
        const y = 100 - r * 75 // Scale right channel to Y (inverted because SVG Y grows down)
        
        // Calculate opacity based on how recent the sample is (fade older samples)
        // More recent samples = higher opacity (persistence effect)
        const age = (dataLength - i) / dataLength
        const opacity = Math.max(0.1, 1 - age * 0.9)
        
        // Size variation for depth effect - newer samples slightly larger
        const size = 0.5 + (1 - age) * 0.5 // Range: 0.5 to 1.0 px
        
        points.push({ x, y, opacity, size })
    }
    
    return points
})

const goniometerStatus = computed(() => {
    const c = clampedCorrelation.value
    if (Math.abs(c - 1) < 0.05) return 'Mono/Centered - Tight center dot'
    if (Math.abs(c - 0.707) < 0.1) return 'Perfect Stereo - ~45° diagonal line'
    if (c > 0.5) return 'Good Stereo - Diagonal pattern'
    if (c > 0) return 'Wide Stereo - Broad pattern'
    if (c > -0.5) return 'Very Wide - Phase spread'
    return 'Phase Issues - Inverted pattern'
})
</script>

