<template>
    <div class="phase-correlation-meter flex flex-col items-center gap-0.5">
        <!-- Label -->
        <div class="text-[0.4rem] text-gray-400 uppercase font-bold tracking-tight">Phase</div>
        
        <!-- Gauge Display -->
        <div class="relative w-16 h-8">
            <!-- Background arc with color zones -->
            <svg viewBox="0 0 60 30" class="w-full h-full">
                <!-- Red zone: -1 to -0.2 (severe phase issues) -->
                <path 
                    d="M 5 28 A 25 25 0 0 1 22.3 4.2" 
                    fill="none" 
                    stroke="#ef4444" 
                    stroke-width="4"
                    opacity="0.3"
                />
                
                <!-- Yellow zone: -0.2 to +0.2 (wide stereo, mono compatibility warning) -->
                <path 
                    d="M 22.3 4.2 A 25 25 0 0 1 37.7 4.2" 
                    fill="none" 
                    stroke="#eab308" 
                    stroke-width="4"
                    opacity="0.3"
                />
                
                <!-- Green zone: +0.2 to +0.85 (good stereo correlation - normal range) -->
                <path 
                    d="M 37.7 4.2 A 25 25 0 0 1 54.3 22.2" 
                    fill="none" 
                    stroke="#22c55e" 
                    stroke-width="4"
                    opacity="0.3"
                />
                
                <!-- Blue zone: +0.85 to +1 (mono or near-mono) -->
                <path 
                    d="M 54.3 22.2 A 25 25 0 0 1 55 28" 
                    fill="none" 
                    stroke="#3b82f6" 
                    stroke-width="4"
                    opacity="0.3"
                />
                
                <!-- Center point -->
                <circle cx="30" cy="28" r="1.5" :fill="needleColor" opacity="0.5" />
                
                <!-- Needle -->
                <line 
                    :x1="30" 
                    :y1="28" 
                    :x2="needleX" 
                    :y2="needleY" 
                    :stroke="needleColor" 
                    stroke-width="1.5" 
                    stroke-linecap="round"
                />
                
                <!-- Needle tip -->
                <circle :cx="needleX" :cy="needleY" r="2" :fill="needleColor" />
            </svg>
            
            <!-- Tick marks and labels -->
            <div class="absolute inset-0 pointer-events-none">
                <!-- -1 label -->
                <div class="absolute left-0 bottom-0 text-[0.35rem] text-red-400 font-bold">-1</div>
                
                <!-- 0 label -->
                <div class="absolute left-1/2 -translate-x-1/2 top-0 text-[0.35rem] text-yellow-400 font-bold">0</div>
                
                <!-- +1 label -->
                <div class="absolute right-0 bottom-0 text-[0.35rem] text-blue-400 font-bold">+1</div>
            </div>
        </div>
        
        <!-- Numeric Value Display -->
        <div 
            class="text-[0.45rem] font-mono font-bold px-1.5 py-0.5 rounded"
            :class="valueColorClass"
        >
            {{ correlationDisplay }}
        </div>
        
        <!-- Warning indicator -->
        <div 
            v-if="correlation < 0" 
            class="text-[0.35rem] text-red-400 font-bold animate-pulse"
        >
            ⚠ PHASE
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
    correlation: number  // Range: -1 to +1
}

const props = withDefaults(defineProps<Props>(), {
    correlation: 0
})

// Clamp correlation between -1 and +1
const clampedCorrelation = computed(() => {
    return Math.max(-1, Math.min(1, props.correlation))
})

// Convert correlation (-1 to +1) to angle (180° to 0°)
// -1 = 180° (left), 0 = 90° (top), +1 = 0° (right)
const needleAngle = computed(() => {
    const normalized = (clampedCorrelation.value + 1) / 2 // 0 to 1
    return 180 - (normalized * 180) // 180° to 0°
})

// Calculate needle endpoint coordinates
const needleX = computed(() => {
    const angleRad = (needleAngle.value * Math.PI) / 180
    return 30 + 22 * Math.cos(angleRad)
})

const needleY = computed(() => {
    const angleRad = (needleAngle.value * Math.PI) / 180
    return 28 - 22 * Math.sin(angleRad)
})

// Needle color based on correlation value
const needleColor = computed(() => {
    const c = clampedCorrelation.value
    if (c < -0.2) return '#ef4444' // Red: severe phase issues
    if (c < 0.2) return '#eab308'    // Yellow: wide stereo (potential mono compatibility issues)
    if (c < 0.85) return '#22c55e'  // Green: good stereo correlation (normal range)
    return '#3b82f6'               // Blue: mono or near-mono
})

// Value color class for numeric display
const valueColorClass = computed(() => {
    const c = clampedCorrelation.value
    if (c < -0.2) return 'bg-red-900/50 text-red-300'
    if (c < 0.2) return 'bg-yellow-900/50 text-yellow-300'
    if (c < 0.85) return 'bg-green-900/50 text-green-300'
    return 'bg-blue-900/50 text-blue-300'
})

// Display correlation with sign
const correlationDisplay = computed(() => {
    const c = clampedCorrelation.value
    return c >= 0 ? `+${c.toFixed(2)}` : c.toFixed(2)
})
</script>

<style scoped>
.phase-correlation-meter {
    user-select: none;
}
</style>
