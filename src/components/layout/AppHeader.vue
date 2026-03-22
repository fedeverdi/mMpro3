<template>
  <header class="bg-black/20 backdrop-blur-sm border-b border-gray-700 px-4 py-2 relative z-[100]">
    <div class="flex items-center justify-between gap-4 flex-wrap relative">
      <div class="flex items-center gap-2">
        <img src="../../assets/logo_no_scritta.svg" alt="mMpro3" class="h-8" />

        <!-- Quick Scenes in Header -->
        <QuickScenes @load-scene="handleLoadScene" />
      </div>
      <div class="flex gap-2 items-center flex-wrap">
        <button @click="$emit('show-scenes')"
          class="px-3 py-1.5  hover:bg-green-500/10 rounded text-xs font-semibold text-gray-300 hover:text-green-400 transition-all flex items-center gap-1.5">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" class="h-3.5 w-3.5" fill="currentColor">
            <path
              d="M149.333 216v80c0 13.255-10.745 24-24 24H24c-13.255 0-24-10.745-24-24v-80c0-13.255 10.745-24 24-24h101.333c13.255 0 24 10.745 24 24zM0 376v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H24c-13.255 0-24 10.745-24 24zM125.333 32H24C10.745 32 0 42.745 0 56v80c0 13.255 10.745 24 24 24h101.333c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24zm80 448H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24zm-24-424v80c0 13.255 10.745 24 24 24H488c13.255 0 24-10.745 24-24V56c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24zm24 264H488c13.255 0 24-10.745 24-24v-80c0-13.255-10.745-24-24-24H205.333c-13.255 0-24 10.745-24 24v80c0 13.255 10.745 24 24 24z" />
          </svg>
          Scenes
        </button>

        <!-- File Manager Button -->
        <button @click="$emit('show-file-manager')"
          class="px-3 py-1.5 hover:bg-blue-500/10 rounded text-xs font-semibold text-gray-300 hover:text-blue-400 transition-all flex items-center gap-1.5">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          Library
        </button>

        <div class="w-px h-6 bg-gray-600"></div>

        <!-- Lock Button -->
        <button @click="$emit('lock-toggle')" :class="[
          'px-3 py-1.5 rounded text-xs font-semibold transition-all flex items-center gap-1.5',
          isLocked
            ? 'bg-blue-600/20 text-blue-400 hover:bg-blue-600/30'
            : 'hover:bg-yellow-500/10 text-gray-300 hover:text-yellow-400'
        ]">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path v-if="isLocked" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
          </svg>
          {{ isLocked ? 'Locked' : 'Lock' }}
        </button>

        <!-- Exit Remote Control Button (only in remote mode) -->
        <button v-if="isRemoteMode" @click="handleExitRemoteControl"
          class="px-3 py-1.5 bg-red-600/20 hover:bg-red-600/30 rounded text-xs font-semibold text-red-400 hover:text-red-300 transition-all flex items-center gap-1.5">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
          </svg>
          Exit
        </button>

        <!-- DJ Mode Button -->
        <button @click="$emit('toggle-dj-mode')"
          class="px-3 py-1.5 rounded text-xs font-semibold transition-all flex items-center gap-1.5"
          :class="isDjMode
            ? 'bg-purple-600/30 text-purple-300 border border-purple-600/50 shadow-[0_0_8px_rgba(147,51,234,0.25)]'
            : 'hover:bg-purple-500/10 text-gray-300 hover:text-purple-400 border border-transparent'">
          <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="1.5"/>
            <circle cx="12" cy="12" r="3" fill="currentColor"/>
            <circle cx="12" cy="12" r="6.5" fill="none" stroke="currentColor" stroke-width="0.8" stroke-dasharray="1.5 1.5"/>
          </svg>
          DJ Mode
        </button>

        <div class="w-px h-6 bg-gray-600"></div>

        <div class="relative -mt-[3px]">
          <button ref="addButtonRef" @click="handleAddButtonClick"
            class="mt-1 px-3 h-full py-1.5 hover:bg-emerald-500/10 rounded text-xs font-semibold text-gray-300 hover:text-emerald-400 transition-all flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add
          </button>
        </div>

        <!-- Dropdown Menu (Teleported to body) -->
        <Teleport to="body">
          <div v-if="showAddTrackMenu"
            class="fixed w-36 bg-gray-800 rounded shadow-2xl z-[9999]"
            :style="{ top: `${menuPosition.top}px`, left: `${menuPosition.left}px` }"
            @click.stop>
            <button @click="addTrackOfType('audio')"
              class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
              <div class="flex">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 256 512">
                  <path
                    d="M96 496V16c0-8.8-7.2-16-16-16H48c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16zm128 0V16c0-8.8-7.2-16-16-16h-32c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16z" />
                </svg>
              </div>
              Audio Track
            </button>
            <button @click="addTrackOfType('signal')"
              class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
              <div class="flex">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 640 512">
                  <path
                    d="M476 480H324a36 36 0 0 1-36-36V96h-96v156a36 36 0 0 1-36 36H16a16 16 0 0 1-16-16v-32a16 16 0 0 1 16-16h112V68a36 36 0 0 1 36-36h152a36 36 0 0 1 36 36v348h96V260a36 36 0 0 1 36-36h140a16 16 0 0 1 16 16v32a16 16 0 0 1-16 16H512v156a36 36 0 0 1-36 36z" />
                </svg>
              </div>
              Signal Track
            </button>
            <template v-if="buildLimits.maxSubgroups > 0">
              <div class="h-px bg-gray-600 my-1"></div>
              <button @click="handleAddSubgroup"
                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
                <div class="flex">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 512 512">
                    <path
                      d="M12.41 148.02l232.94 105.67c6.8 3.09 14.49 3.09 21.29 0l232.94-105.67c16.55-7.51 16.55-32.52 0-40.03L266.65 2.31a25.607 25.607 0 0 0-21.29 0L12.41 107.98c-16.55 7.51-16.55 32.53 0 40.04zm487.18 88.28l-58.09-26.33-161.64 73.27c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.51 209.97l-58.1 26.33c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 276.3c16.55-7.5 16.55-32.5 0-40zm0 127.8l-57.87-26.23-161.86 73.37c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.29 337.87 12.41 364.1c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 404.1c16.55-7.5 16.55-32.5 0-40z" />
                  </svg>
                </div>
                Subgroup
              </button>
            </template>
          </div>
        </Teleport>

        <div class="relative">
          <button ref="removeButtonRef" @click="handleRemoveButtonClick" :disabled="tracksCount <= 1 && subgroupsCount === 0"
            class="px-3 py-1.5 hover:bg-red-500/10 disabled:border-gray-700 disabled:bg-gray-800/50 disabled:cursor-not-allowed rounded text-xs font-semibold text-gray-300 hover:text-red-400 disabled:text-gray-600 transition-all flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
            </svg>
            Remove
          </button>
        </div>

        <!-- Fullscreen Button (Browser only) -->
        <template v-if="isBrowser">
          <div class="w-px h-6 bg-gray-600"></div>

          <button @click="toggleFullscreen"
            class="px-3 py-1.5 hover:bg-purple-500/10 rounded text-xs font-semibold text-gray-300 hover:text-purple-400 transition-all flex items-center gap-1.5">
            <svg v-if="!isFullscreen" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
            </svg>
            <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25" />
            </svg>
            {{ isFullscreen ? 'Exit' : 'Full' }}
          </button>
        </template>

        <!-- Remove Dropdown Menu (Teleported to body) -->
        <Teleport to="body">
          <div v-if="showRemoveMenu"
            class="fixed w-32 bg-gray-800 rounded shadow-2xl z-[9999]"
            :style="{ top: `${removeMenuPosition.top}px`, left: `${removeMenuPosition.left}px` }"
            @click.stop>
            <button v-if="tracksCount > 1" @click="handleRemoveTrack"
              class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
              <div class="flex">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 256 512">
                  <path
                    d="M96 496V16c0-8.8-7.2-16-16-16H48c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16zm128 0V16c0-8.8-7.2-16-16-16h-32c-8.8 0-16 7.2-16 16v480c0 8.8 7.2 16 16 16h32c8.8 0 16-7.2 16-16z" />
                </svg>
              </div>
              Track
            </button>
            <template v-if="buildLimits.maxSubgroups > 0 && subgroupsCount > 0">
              <div v-if="tracksCount > 1" class="h-px bg-gray-600 my-1"></div>
              <button @click="handleRemoveSubgroup"
                class="w-full px-3 py-2 text-left text-xs hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer">
                <div class="flex">
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="white" viewBox="0 0 512 512">
                    <path
                      d="M12.41 148.02l232.94 105.67c6.8 3.09 14.49 3.09 21.29 0l232.94-105.67c16.55-7.51 16.55-32.52 0-40.03L266.65 2.31a25.607 25.607 0 0 0-21.29 0L12.41 107.98c-16.55 7.51-16.55 32.53 0 40.04zm487.18 88.28l-58.09-26.33-161.64 73.27c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.51 209.97l-58.1 26.33c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 276.3c16.55-7.5 16.55-32.5 0-40zm0 127.8l-57.87-26.23-161.86 73.37c-7.56 3.43-15.59 5.17-23.86 5.17s-16.29-1.74-23.86-5.17L70.29 337.87 12.41 364.1c-16.55 7.5-16.55 32.5 0 40l232.94 105.59c6.8 3.08 14.49 3.08 21.29 0L499.59 404.1c16.55-7.5 16.55-32.5 0-40z" />
                  </svg>
                </div>
                Subgroup
              </button>
            </template>
          </div>
        </Teleport>

        <div class="text-xs text-gray-400">
          {{ tracksCount }}/{{ buildLimits.maxTracks }}
        </div>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, inject } from 'vue'
import QuickScenes from './QuickScenes.vue'

interface Props {
  isLocked: boolean
  isDjMode: boolean
  buildLimits: {
    maxTracks: number
    maxSubgroups: number
    allowSubgroupRouting: boolean
  }
  tracksCount: number
  subgroupsCount: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'show-scenes': []
  'show-file-manager': []
  'lock-toggle': []
  'toggle-dj-mode': []
  'add-track': [type: 'audio' | 'signal']
  'add-subgroup': []
  'remove-track': []
  'remove-subgroup': []
  'load-scene': [sceneName: string]
}>()

// Check if in remote mode
const isRemoteMode = ref(false)
const exitRemoteControl = inject<((notifyServer?: boolean) => void) | null>('exitRemoteControl', null)

onMounted(() => {
  isRemoteMode.value = !(window as any).electronAPI
})

const handleExitRemoteControl = () => {
  if (exitRemoteControl) {
    exitRemoteControl(true) // Notify server when user manually clicks Exit
  }
}

const showAddTrackMenu = ref(false)
const addButtonRef = ref<HTMLElement | null>(null)
const menuPosition = reactive({ top: 0, left: 0 })

const showRemoveMenu = ref(false)
const removeButtonRef = ref<HTMLElement | null>(null)
const removeMenuPosition = reactive({ top: 0, left: 0 })

const isBrowser = ref(false)
const isFullscreen = ref(false)

const handleAddButtonClick = () => {
  showAddTrackMenu.value = !showAddTrackMenu.value
  
  if (showAddTrackMenu.value && addButtonRef.value) {
    const rect = addButtonRef.value.getBoundingClientRect()
    menuPosition.top = rect.bottom + 4
    menuPosition.left = rect.left
  }
}

const addTrackOfType = (type: 'audio' | 'signal') => {
  emit('add-track', type)
  showAddTrackMenu.value = false
}

const handleAddSubgroup = () => {
  emit('add-subgroup')
  showAddTrackMenu.value = false
}

const handleRemoveButtonClick = () => {
  showRemoveMenu.value = !showRemoveMenu.value
  
  if (showRemoveMenu.value && removeButtonRef.value) {
    const rect = removeButtonRef.value.getBoundingClientRect()
    removeMenuPosition.top = rect.bottom + 4
    removeMenuPosition.left = rect.left
  }
}

const handleRemoveTrack = () => {
  emit('remove-track')
  showRemoveMenu.value = false
}

const handleRemoveSubgroup = () => {
  emit('remove-subgroup')
  showRemoveMenu.value = false
}

const handleLoadScene = (sceneName: string) => {
  emit('load-scene', sceneName)
}

const toggleFullscreen = async () => {
  if (!document.fullscreenElement) {
    try {
      await document.documentElement.requestFullscreen()
    } catch (err) {
      console.error('Error entering fullscreen:', err)
    }
  } else {
    try {
      await document.exitFullscreen()
    } catch (err) {
      console.error('Error exiting fullscreen:', err)
    }
  }
}

// Detect browser mode and setup fullscreen listener
onMounted(() => {
  isBrowser.value = !(window as any).electronAPI
  
  if (isBrowser.value) {
    const handleFullscreenChange = () => {
      isFullscreen.value = !!document.fullscreenElement
    }
    
    document.addEventListener('fullscreenchange', handleFullscreenChange)
    
    onUnmounted(() => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange)
    })
  }
})

// Close menus when clicking outside
if (typeof document !== 'undefined') {
  document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement
    
    // Handle Add menu
    if (showAddTrackMenu.value) {
      if (!addButtonRef.value?.contains(target) && !target.closest('.fixed.w-36.bg-gray-800')) {
        showAddTrackMenu.value = false
      }
    }
    
    // Handle Remove menu
    if (showRemoveMenu.value) {
      if (!removeButtonRef.value?.contains(target) && !target.closest('.fixed.w-40.bg-gray-800')) {
        showRemoveMenu.value = false
      }
    }
  })
}
</script>
