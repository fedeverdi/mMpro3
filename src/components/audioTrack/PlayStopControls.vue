<template>
  <div class="flex gap-1">
    <button @click="$emit('play')" :disabled="disabled"
      class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all flex items-center justify-center gap-1 disabled:opacity-50 disabled:cursor-not-allowed"
      :class="isPlaying ? 'bg-green-600 text-white' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
      <span v-if="isPlaylistMode && isPlaying" class="flex items-center gap-0.5">
        <!-- Next Icon (Skip Forward) -->
        <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
        </svg>
        NEXT
      </span>
      <span v-else class="flex items-center gap-0.5">
        <!-- Play Icon -->
        <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z"/>
        </svg>
        PLAY
      </span>
    </button>
    <button @click="$emit('stop')" :disabled="disabled"
      class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all flex items-center justify-center gap-1 bg-gray-700 hover:bg-gray-600 text-gray-300 disabled:opacity-50 disabled:cursor-not-allowed">
      <!-- Stop Icon -->
      <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="currentColor">
        <rect x="6" y="6" width="12" height="12"/>
      </svg>
      STOP
    </button>
  </div>
</template>

<script setup lang="ts">
interface Props {
  disabled: boolean
  isPlaying: boolean
  isPlaylistMode: boolean
}

defineProps<Props>()

defineEmits<{
  play: []
  stop: []
}>()
</script>
