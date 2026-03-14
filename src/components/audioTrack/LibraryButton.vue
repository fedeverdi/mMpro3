<template>
  <button @click="$emit('click')"
    class="w-full text-xs bg-blue-600 hover:bg-blue-500 text-white border border-blue-500 rounded px-2 py-1 transition-all flex items-center gap-1 overflow-hidden group">
    <!-- Library Icon -->
    <svg class="w-4 h-4 flex-shrink-0" viewBox="0 0 24 24" fill="currentColor">
      <path d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-1 9H9V9h10v2zm-4 4H9v-2h6v2zm4-8H9V5h10v2z"/>
    </svg>
    <div ref="containerRef" class="flex-1 min-w-0 overflow-hidden relative">
      <!-- Duplicate text for seamless marquee -->
      <div v-if="needsMarquee" class="flex animate-marquee-smooth group-hover:[animation-play-state:paused]">
        <span class="whitespace-nowrap pr-8">{{ displayText }}</span>
        <span class="whitespace-nowrap pr-8">{{ displayText }}</span>
      </div>
      <span v-else class="block truncate">{{ displayText }}</span>
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, nextTick, watch } from 'vue'

const props = defineProps<{
  fileName?: string | null
  defaultText?: string
}>()

defineEmits<{
  click: []
}>()

const displayText = computed(() => props.fileName || props.defaultText || 'Load from Library')
const needsMarquee = ref(false)
const containerRef = ref<HTMLElement | null>(null)

// Check if text needs marquee animation
async function checkTextWidth() {
  await nextTick()
  if (!containerRef.value) {
    needsMarquee.value = false
    return
  }
  
  // Create temporary span to measure text width
  const tempSpan = document.createElement('span')
  tempSpan.style.visibility = 'hidden'
  tempSpan.style.position = 'absolute'
  tempSpan.style.whiteSpace = 'nowrap'
  tempSpan.style.fontSize = '0.75rem' // text-xs
  tempSpan.textContent = displayText.value
  document.body.appendChild(tempSpan)
  
  const textWidth = tempSpan.offsetWidth
  document.body.removeChild(tempSpan)
  
  const containerWidth = containerRef.value.offsetWidth
  needsMarquee.value = textWidth > containerWidth
}

onMounted(() => {
  checkTextWidth()
})

watch(displayText, () => {
  checkTextWidth()
})
</script>

<style scoped>
@keyframes marquee-smooth {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(-50%);
  }
}

.animate-marquee-smooth {
  display: inline-flex;
  animation: marquee-smooth 15s linear infinite;
  will-change: transform;
}
</style>
