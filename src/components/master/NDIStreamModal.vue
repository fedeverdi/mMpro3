<template>
  <Transition name="modal">
    <div v-if="isOpen" class="fixed inset-0 z-[9999] flex items-center justify-center" @click.self="close">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/75"></div>

      <!-- Modal Panel -->
      <div class="relative w-full max-w-md transform overflow-hidden rounded-2xl bg-gradient-to-br from-gray-900 to-gray-800 p-6 shadow-2xl border border-gray-700">
        <!-- Header -->
        <div class="mb-6 flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center p-1.5">
            <img 
              src="@/assets/ndi_logo.png" 
              class="w-full h-full object-contain invert"
              alt="NDI" />
          </div>
          <h2 class="text-2xl font-bold text-white">NDI Stream Settings</h2>
        </div>

        <!-- Stream Name -->
        <div class="mb-6">
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Stream Name
          </label>
          <input
            v-model="localStreamName"
            type="text"
            class="w-full px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:border-purple-500 focus:ring-2 focus:ring-purple-500/20 transition-all"
            placeholder="My Audio Stream"
            @change="handleNameChange"
          />
          <p class="text-xs text-gray-500 mt-1">
            This name will be visible to NDI receivers on the network
          </p>
        </div>

        <!-- Video Frame Text -->
        <div class="mb-6">
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Video Frame Text
          </label>
          <input
            v-model="localVideoText"
            type="text"
            class="w-full px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all"
            placeholder="MMpro3"
            maxlength="20"
            @change="handleVideoTextChange"
          />
          <p class="text-xs text-gray-500 mt-1">
            Text displayed on the video frame (max 20 characters)
          </p>
        </div>

        <!-- Audio Source Selection -->
        <div class="mb-6">
          <label class="block text-sm font-medium text-gray-300 mb-2">
            Audio Source
          </label>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="source in availableSources"
              :key="source.value"
              @click="handleSourceChange(source.value)"
              :class="[
                'px-4 py-3 rounded-lg font-medium transition-all',
                localSource === source.value
                  ? 'bg-gradient-to-br from-purple-500 to-pink-600 text-white shadow-lg shadow-purple-500/50'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              ]"
            >
              {{ source.label }}
            </button>
          </div>
          <p class="text-xs text-gray-500 mt-2">
            Select which bus to stream via NDI
          </p>
        </div>

        <!-- Stream Status -->
        <div 
          v-if="isStreaming"
          class="mb-6 p-4 bg-green-500/10 border border-green-500/30 rounded-lg"
        >
          <div class="flex items-center gap-2 text-green-400">
            <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
            <span class="text-sm font-medium">Streaming Active</span>
          </div>
          <p class="text-xs text-green-300 mt-1">
            Broadcasting: {{ streamName }}
          </p>
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-3">
          <button
            @click="close"
            class="flex-1 px-4 py-2.5 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium"
          >
            Close
          </button>
          
          <button
            v-if="!isStreaming"
            @click="handleStartStream"
            class="flex-1 px-4 py-2.5 bg-gradient-to-r from-purple-500 to-pink-600 text-white rounded-lg hover:from-purple-600 hover:to-pink-700 transition-all font-medium shadow-lg shadow-purple-500/30"
          >
            Start Stream
          </button>
          
          <button
            v-else
            @click="handleStopStream"
            class="flex-1 px-4 py-2.5 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors font-medium"
          >
            Stop Stream
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useNDI, type NdiSource } from '@/composables/useNDI'

interface Subgroup {
  id: number
  name: string
  volume: number
  routeToMaster: boolean
}

interface Props {
  isOpen: boolean
  subgroups: Subgroup[]
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'close'): void
}>()

const { isStreaming, streamName, streamSource, videoText, startStreaming, stopStreaming, changeSource, setStreamName, setVideoText } = useNDI()

// Local state for inputs
const localStreamName = ref(streamName.value)
const localSource = ref(streamSource.value)
const localVideoText = ref(videoText.value)

// Watch for external changes
watch(streamName, (newName) => {
  localStreamName.value = newName
})

watch(streamSource, (newSource) => {
  localSource.value = newSource
})

watch(videoText, (newText) => {
  localVideoText.value = newText
})

// Generate available sources dynamically based on subgroups
const availableSources = computed(() => {
  const sources = [{ value: 'master' as NdiSource, label: 'Master' }]
  
  // Add all existing subgroups
  props.subgroups.forEach((subgroup) => {
    sources.push({
      value: `subgroup${subgroup.id}` as NdiSource,
      label: subgroup.name
    })
  })
  
  return sources
})

function close() {
  emit('close')
}

async function handleNameChange() {
  if (localStreamName.value.trim()) {
    await setStreamName(localStreamName.value.trim())
  }
}

async function handleVideoTextChange() {
  if (localVideoText.value.trim()) {
    await setVideoText(localVideoText.value.trim())
  }
}

async function handleSourceChange(source: NdiSource) {
  localSource.value = source
  await changeSource(source)
}

async function handleStartStream() {
  try {
    await startStreaming()
  } catch (error) {
    console.error('Failed to start stream:', error)
  }
}

async function handleStopStream() {
  try {
    await stopStreaming()
  } catch (error) {
    console.error('Failed to stop stream:', error)
  }
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active > div:last-child,
.modal-leave-active > div:last-child {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.modal-enter-from > div:last-child,
.modal-leave-to > div:last-child {
  opacity: 0;
  transform: scale(0.95);
}
</style>
