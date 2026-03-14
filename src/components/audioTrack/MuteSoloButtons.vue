<template>
  <div class="flex flex-row gap-1 w-full">
    <button @click="$emit('toggle-mute')" 
      class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all"
      :class="isMuted ? 'bg-red-600 text-white animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
      MUTE
    </button>
    <button @click="$emit('toggle-solo')" 
      class="flex-1 py-1 text-[0.5rem] font-bold rounded transition-all"
      :class="isSolo ? 'bg-yellow-500 text-gray-900 animate-pulse' : 'bg-gray-700 hover:bg-gray-600 text-gray-300'">
      SOLO
    </button>
  </div>
</template>

<script setup lang="ts">
interface Props {
  isMuted: boolean
  isSolo: boolean
}

defineProps<Props>()

defineEmits<{
  'toggle-mute': []
  'toggle-solo': []
}>()
</script>
