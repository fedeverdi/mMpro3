<template>
  <div v-if="pinnedScenes.length > 0" class="flex items-center gap-1 border-l border-gray-600 pl-2">
    <div class="text-[0.6rem] text-gray-400 flex items-center gap-1">
      📌
      PINNED:
    </div>
    
    <button
      v-for="scene in pinnedScenes"
      :key="scene.name"
      @click="loadScene(scene.name)"
      :class="[
        'px-2 pt-[0.09rem] pb-[0.1rem] border rounded text-[0.6rem] transition-all flex items-center gap-1.5',
        currentSceneName === scene.name
          ? 'bg-emerald-500 text-white border-emerald-400 hover:bg-emerald-400'
          : 'bg-blue-600 hover:bg-blue-500 text-white border-blue-500 hover:border-blue-400'
      ]"
      :title="scene.name"
    >
      <span class="max-w-[120px] truncate">{{ scene.name }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useScenes } from '~/composables/useScenes'

const emit = defineEmits<{
  'loadScene': [sceneName: string]
}>()

const { scenes, currentSceneName, loadScene: loadSceneComposable, loadAllScenes } = useScenes()

// Show only pinned scenes
const pinnedScenes = computed(() => {
  return scenes.value.filter(s => s.pinned)
})

// Load scenes when component mounts
onMounted(async () => {
  await loadAllScenes()
})

async function loadScene(sceneName: string) {
  await loadSceneComposable(sceneName)
  emit('loadScene', sceneName)
}
</script>
