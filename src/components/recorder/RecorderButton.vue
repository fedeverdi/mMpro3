<template>
  <button @click="$emit('open')"
    class="flex-1 py-1 text-xs font-bold rounded transition-all flex items-center justify-center gap-1 bg-red-600 hover:bg-red-700 text-white"
    :class="{ 'animate-pulse': isRecording }">
    <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 24 24">
      <circle cx="12" cy="12" r="8" />
    </svg>
  </button>
</template>

<script setup lang="ts">
interface Props {
  isRecording?: boolean
}

defineProps<Props>()

defineEmits<{
  open: []
}>()
</script>
