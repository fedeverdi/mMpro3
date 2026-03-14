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
                class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/70"
            >
                <div @click.stop
                    class="bg-gray-900 rounded-lg shadow-2xl w-[500px] max-w-[90vw] border border-gray-700"
                >
                        <!-- Header -->
                        <div class="flex items-center justify-between px-4 py-2.5 border-b border-gray-700">
                            <h3 class="text-sm font-semibold text-white">Phase Correlation · Track {{ trackNumber }}</h3>
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

                        <!-- Horizontal Meter Display -->
                        <div v-if="viewMode === 'gauge'" class="bg-gray-900/50 rounded-lg p-8 m-6 mb-4">
                            <!-- Meter Container -->
                            <div class="relative w-full h-16 mb-6">
                                <!-- Background Meter Bar with Color Zones -->
                                <div class="absolute inset-0 rounded-full overflow-hidden flex">
                                    <!-- Red zone: -1 to -0.2 (40%) -->
                                    <div class="h-full bg-red-600/30" style="width: 40%"></div>
                                    <!-- Yellow zone: -0.2 to +0.2 (20%) -->
                                    <div class="h-full bg-yellow-500/30" style="width: 20%"></div>
                                    <!-- Green zone: +0.2 to +0.85 (32.5%) -->
                                    <div class="h-full bg-green-500/30" style="width: 32.5%"></div>
                                    <!-- Blue zone: +0.85 to +1 (7.5%) -->
                                    <div class="h-full bg-blue-500/30" style="width: 7.5%"></div>
                                </div>
                                
                                <!-- Tick marks and labels -->
                                <div class="absolute inset-0 pointer-events-none">
                                    <!-- -1 -->
                                    <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-gray-600"></div>
                                    <div class="absolute left-0 -bottom-6 text-xs text-red-400 font-bold">-1</div>
                                    
                                    <!-- -0.2 -->
                                    <div class="absolute left-[40%] top-0 bottom-0 w-0.5 bg-gray-600"></div>
                                    <div class="absolute left-[40%] -translate-x-1/2 -bottom-6 text-xs text-yellow-400 font-bold">-0.2</div>
                                    
                                    <!-- 0 -->
                                    <div class="absolute left-1/2 -translate-x-1/2 top-0 bottom-0 w-0.5 bg-gray-500"></div>
                                    <div class="absolute left-1/2 -translate-x-1/2 -bottom-6 text-xs text-gray-300 font-bold">0</div>
                                    
                                    <!-- +0.2 -->
                                    <div class="absolute left-[60%] top-0 bottom-0 w-0.5 bg-gray-600"></div>
                                    <div class="absolute left-[60%] -translate-x-1/2 -bottom-6 text-xs text-green-400 font-bold">+0.2</div>
                                    
                                    <!-- +0.85 -->
                                    <div class="absolute left-[92.5%] top-0 bottom-0 w-0.5 bg-gray-600"></div>
                                    <div class="absolute left-[92.5%] -translate-x-1/2 -bottom-6 text-xs text-blue-400 font-bold">+0.85</div>
                                    
                                    <!-- +1 -->
                                    <div class="absolute right-0 top-0 bottom-0 w-0.5 bg-gray-600"></div>
                                    <div class="absolute right-0 -bottom-6 text-xs text-blue-400 font-bold">+1</div>
                                </div>
                                
                                <!-- Moving Indicator -->
                                <div 
                                    class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 transition-all duration-100 ease-out"
                                    :style="{ left: indicatorPosition }"
                                >
                                    <!-- Needle/pointer -->
                                    <div class="relative">
                                        <div 
                                            class="w-1 h-20 rounded-full shadow-lg"
                                            :style="{ backgroundColor: needleColor }"
                                        ></div>
                                        <!-- Top circle -->
                                        <div 
                                            class="absolute -top-2 left-1/2 -translate-x-1/2 w-4 h-4 rounded-full shadow-lg"
                                            :style="{ backgroundColor: needleColor }"
                                        ></div>
                                        <!-- Bottom circle -->
                                        <div 
                                            class="absolute -bottom-2 left-1/2 -translate-x-1/2 w-4 h-4 rounded-full shadow-lg"
                                            :style="{ backgroundColor: needleColor }"
                                        ></div>
                                    </div>
                                </div>
                            </div>
                            
                            <!-- Value Display -->
                            <div class="text-center mt-10">
                                <div 
                                    class="inline-block text-3xl font-mono font-bold px-4 py-2 rounded-lg"
                                    :class="valueColorClass"
                                >
                                    {{ correlationDisplay }}
                                </div>
                            </div>
                        </div>

                        <!-- Goniometer Display -->
                        <div v-else class="bg-gray-900/50 rounded-lg p-8 m-6 mb-4">
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
                        <div class="space-y-3 text-sm px-6 pb-6">
                            <!-- Current Status -->
                            <div class="bg-gray-900/50 rounded-lg p-4">
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

// Smoothed correlation value for display
const smoothedCorrelation = ref(0)

// Animation frame counter to trigger reactivity (throttled for smoother updates)
const frameCounter = ref(0)
let animationFrameId: number | null = null
let frameSkipCounter = 0

const updateFrame = () => {
    // Update every 3 frames (~20Hz instead of 60Hz for smoother visualization)
    frameSkipCounter++
    if (frameSkipCounter >= 3) {
        frameCounter.value++
        frameSkipCounter = 0
        
        // Apply smooth interpolation to correlation value
        const smoothingFactor = 0.15 // Smooth transition
        smoothedCorrelation.value = smoothedCorrelation.value * (1 - smoothingFactor) + props.correlation * smoothingFactor
    }
    animationFrameId = requestAnimationFrame(updateFrame)
}

onMounted(() => {
    smoothedCorrelation.value = props.correlation
    updateFrame()
})

onUnmounted(() => {
    if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId)
    }
})

// Clamp correlation (use smoothed value)
const clampedCorrelation = computed(() => Math.max(-1, Math.min(1, smoothedCorrelation.value)))

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

// Horizontal meter indicator position (maps -1 to +1 → 0% to 100%)
const indicatorPosition = computed(() => {
    const c = clampedCorrelation.value
    const normalized = (c + 1) / 2 // Map -1..+1 to 0..1
    return `${normalized * 100}%`
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
    
    // Sample points from audio data (reduced for better dispersion)
    const sampleStep = Math.max(1, Math.floor(dataLength / 200)) // Max 200 points for better dispersion
    
    for (let i = 0; i < dataLength; i += sampleStep) {
        const l = props.audioDataL[i] || 0 // Left channel (-1 to +1)
        const r = props.audioDataR[i] || 0 // Right channel (-1 to +1)
        
        // Map audio values to screen coordinates
        // X axis = Left channel, Y axis = Right channel
        // Increased scale for better dispersion (85 instead of 75)
        const x = 100 + l * 85
        const y = 100 - r * 85 // (inverted because SVG Y grows down)
        
        // Calculate opacity based on how recent the sample is (fade older samples)
        // More recent samples = higher opacity (persistence effect)
        const age = (dataLength - i) / dataLength
        const opacity = Math.max(0.15, 1 - age * 0.85) // Increased minimum opacity
        
        // Size variation for depth effect - increased size for better visibility
        const size = 1.0 + (1 - age) * 1.5 // Range: 1.0 to 2.5 px (increased from 0.5-1.0)
        
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

