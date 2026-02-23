<template>
  <div class="relative">
    <button @click.stop="showFadeOutMenu = !showFadeOutMenu"
      :disabled="!isPlaying || audioSourceType === 'input'"
      class="px-1.5 py-1 text-xs rounded transition-colors flex items-center justify-center"
      :class="isPlaying && audioSourceType === 'file' ? 'bg-slate-400 hover:bg-slate-300' : 'bg-gray-700 text-gray-500 cursor-not-allowed'"
      title="Fade Out Options">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="w-3 h-3">
        <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z" />
        <path opacity="0.5" d="M14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" />
      </svg>
    </button>
    
    <!-- Fade Out Curve Dropdown -->
    <div v-if="showFadeOutMenu && isPlaying && audioSourceType === 'file'"
      class="absolute top-full mt-1 left-0 bg-gray-800 border border-gray-600 rounded shadow-lg z-50 min-w-[140px]"
      @click.stop>
      <div class="text-[0.5rem] font-bold text-gray-400 px-2 py-1 border-b border-gray-700">FADE OUT</div>
      
      <!-- Duration Knob -->
      <div class="px-2 py-0 border-b border-gray-700">
        <div class="flex items-center justify-center scale-[0.7]">
          <Knob 
            :model-value="fadeDuration" 
            @update:model-value="fadeDuration = $event"
            :min="0.5" 
            :max="20" 
            :step="0.5"
            label="Duration"
            unit="s"
            color="#f97316"
          />
        </div>
      </div>
      
      <div class="text-[0.5rem] font-bold text-gray-400 px-2 py-1 border-b border-gray-700">CURVE TYPE</div>
      <button @click="handleFadeOut('linear')"
        class="w-full text-left px-2 py-1.5 text-[0.55rem] hover:bg-gray-700 transition-colors flex items-center gap-1.5">
        <svg width="10" height="10" viewBox="0 0 20 20" class="flex-shrink-0">
          <line x1="2" y1="18" x2="18" y2="2" stroke="currentColor" stroke-width="2" />
        </svg>
        <span>Linear</span>
      </button>
      <button @click="handleFadeOut('exponential')"
        class="w-full text-left px-2 py-1.5 text-[0.55rem] hover:bg-gray-700 transition-colors flex items-center gap-1.5">
        <svg width="10" height="10" viewBox="0 0 20 20" class="flex-shrink-0">
          <path d="M 2 18 Q 2 2, 18 2" fill="none" stroke="currentColor" stroke-width="2" />
        </svg>
        <span>Exponential</span>
      </button>
      <button @click="handleFadeOut('logarithmic')"
        class="w-full text-left px-2 py-1.5 text-[0.55rem] hover:bg-gray-700 transition-colors flex items-center gap-1.5">
        <svg width="10" height="10" viewBox="0 0 20 20" class="flex-shrink-0">
          <path d="M 2 18 Q 18 18, 18 2" fill="none" stroke="currentColor" stroke-width="2" />
        </svg>
        <span>Logarithmic</span>
      </button>
      <button @click="handleFadeOut('scurve')"
        class="w-full text-left px-2 py-1.5 text-[0.55rem] hover:bg-gray-700 transition-colors flex items-center gap-1.5">
        <svg width="10" height="10" viewBox="0 0 20 20" class="flex-shrink-0">
          <path d="M 2 18 Q 2 10, 10 10 Q 18 10, 18 2" fill="none" stroke="currentColor" stroke-width="2" />
        </svg>
        <span>S-Curve</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import Knob from '../core/Knob.vue'

interface Props {
  isPlaying: boolean
  audioSourceType: 'file' | 'input'
  volume: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:volume', value: number): void
  (e: 'fade-complete'): void
}>()

const showFadeOutMenu = ref(false)
const fadeDuration = ref(5) // Default 3 seconds

// Initialize
onMounted(() => {
  // Close fade-out menu when clicking outside
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// Close menu when playback stops
watch(() => props.isPlaying, (playing) => {
  if (!playing) {
    showFadeOutMenu.value = false
  }
})

function handleClickOutside() {
  if (showFadeOutMenu.value) {
    showFadeOutMenu.value = false
  }
}

function handleFadeOut(curveType: 'linear' | 'exponential' | 'logarithmic' | 'scurve') {
  showFadeOutMenu.value = false
  fadeOut(curveType)
}

// Fade out audio with different curve types
function fadeOut(curveType: 'linear' | 'exponential' | 'logarithmic' | 'scurve' = 'linear') {
  if (!props.isPlaying) return
  if (props.audioSourceType === 'input') return

  // Fade volume to -60dB over specified duration, then stop
  const fadeTime = fadeDuration.value // Use user-specified duration
  const targetVolume = -60 // dB
  const startVolume = props.volume
  const startTime = Date.now()
  
  // Calculate curve value based on progress (0 to 1)
  const getCurveValue = (progress: number): number => {
    switch (curveType) {
      case 'exponential':
        // Fast fade at start, slow at end
        return Math.pow(progress, 2.5)
      
      case 'logarithmic':
        // Slow fade at start, fast at end
        return 1 - Math.pow(1 - progress, 0.4)
      
      case 'scurve':
        // S-shaped curve (smooth acceleration and deceleration)
        return progress < 0.5
          ? 2 * Math.pow(progress, 2)
          : 1 - Math.pow(-2 * progress + 2, 2) / 2
      
      case 'linear':
      default:
        // Linear fade
        return progress
    }
  }
  
  // Animate the fader and emit volume changes
  const animateFader = () => {
    const elapsed = (Date.now() - startTime) / 1000 // seconds
    const progress = Math.min(elapsed / fadeTime, 1) // 0 to 1
    const curveProgress = getCurveValue(progress)
    
    // Apply curve to volume interpolation
    const newVolume = startVolume + (targetVolume - startVolume) * curveProgress
    emit('update:volume', newVolume)
    
    if (progress < 1) {
      requestAnimationFrame(animateFader)
    } else {
      // Fade complete - restore original volume and stop
      setTimeout(() => {
        emit('update:volume', startVolume)
        emit('fade-complete')
      }, 50)
    }
  }
  
  animateFader()
}
</script>
