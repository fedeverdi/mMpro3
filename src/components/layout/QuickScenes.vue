<template>
  <div v-if="pinnedScenes.length > 0" class="flex items-center gap-1 border-l border-gray-600 pl-2">
    <div class="text-[0.6rem]  text-gray-400 flex items-center gap-1">
      <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 384 512">
        <path d="M32 32C32 14.3 46.3 0 64 0H320c17.7 0 32 14.3 32 32s-14.3 32-32 32H290.5l11.4 148.2c36.7 19.9 65.7 53.2 79.5 94.7l1 3c3.3 9.8 1.6 20.5-4.4 28.8s-15.7 13.3-26 13.3H32c-10.3 0-19.9-5-26-13.3s-7.7-19.1-4.4-28.8l1-3c13.8-41.5 42.8-74.8 79.5-94.7L93.5 64H64C46.3 64 32 49.7 32 32zM160 384h64v96c0 17.7-14.3 32-32 32s-32-14.3-32-32V384z"/>
      </svg>
      QUICK SCENES:
    </div>
    
    <button
      v-for="scene in pinnedScenes"
      :key="scene.id"
      @click="loadScene(scene)"
      :class="[
        'px-2 pt-[0.09rem] pb-[0.1rem] border rounded text-[0.6rem] transition-all flex items-center gap-1.5',
        currentSceneId === scene.id
          ? 'bg-emerald-500 text-white border-emerald-400 hover:bg-emerald-400'
          : 'bg-blue-600 hover:bg-blue-500 text-white border-blue-500 hover:border-blue-400'
      ]"
      :title="`${scene.name} (${scene.tracks.length} tracks)`"
    >
      <span class="max-w-[120px] truncate">{{ scene.name }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { useScenes } from '~/composables/useScenes'

const emit = defineEmits<{
  'loadScene': [scene: any]
}>()

const { scenes, currentSceneId, loadAllScenes } = useScenes()

const pinnedScenes = computed(() => {
  return scenes.value.filter(s => s.pinned).sort((a, b) => b.timestamp - a.timestamp)
})

// Load scenes when component mounts and watch for changes
onMounted(async () => {
  await loadAllScenes()
})

// Watch scenes array to reload when it changes
watch(scenes, async () => {
  // Scenes array is reactive, so this will trigger when scenes are updated
}, { deep: true })

function loadScene(scene: any) {
  emit('loadScene', scene)
}
</script>
