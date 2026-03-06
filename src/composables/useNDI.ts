import { ref, computed, inject } from 'vue'
import { useAudioEngine } from './useAudioEngine'

export type NdiSource = 'master' | 'subgroup1' | 'subgroup2' | 'subgroup3' | 'subgroup4'

interface NdiConfig {
  streamName: string
  source: NdiSource
}

const isStreaming = ref(false)
const streamName = ref('MMpro3 Audio')
const streamSource = ref<NdiSource>('master')
const videoText = ref('MMpro3')

export function useNDI() {
  const audioEngine = inject<any>('audioEngine', null)

  const config = computed<NdiConfig>(() => ({
    streamName: streamName.value,
    source: streamSource.value,
  }))

  /**
   * Start NDI streaming with current configuration
   */
  async function startStreaming() {
    if (!audioEngine) {
      console.error('[NDI] Audio engine not available')
      throw new Error('Audio engine not available')
    }
    
    try {
      await audioEngine.startNdi(streamName.value, streamSource.value)
      isStreaming.value = true
      console.log('[NDI] Stream started:', config.value)
    } catch (error) {
      console.error('[NDI] Failed to start streaming:', error)
      throw error
    }
  }

  /**
   * Stop NDI streaming
   */
  async function stopStreaming() {
    if (!audioEngine) {
      console.error('[NDI] Audio engine not available')
      return
    }
    
    try {
      await audioEngine.stopNdi()
      isStreaming.value = false
      console.log('[NDI] Stream stopped')
    } catch (error) {
      console.error('[NDI] Failed to stop streaming:', error)
      throw error
    }
  }

  /**
   * Change NDI stream source (master or subgroup)
   */
  async function changeSource(source: NdiSource) {
    if (!audioEngine) {
      console.error('[NDI] Audio engine not available')
      return
    }
    
    try {
      await audioEngine.setNdiSource(source)
      streamSource.value = source
      console.log('[NDI] Source changed to:', source)
    } catch (error) {
      console.error('[NDI] Failed to change source:', error)
      throw error
    }
  }

  /**
   * Update stream name (can be changed while streaming)
   */
  async function setStreamName(name: string) {
    if (!audioEngine) {
      console.error('[NDI] Audio engine not available')
      return
    }
    
    try {
      await audioEngine.setNdiName(name)
      streamName.value = name
      console.log('[NDI] Stream name updated:', name)
    } catch (error) {
      console.error('[NDI] Failed to update stream name:', error)
      throw error
    }
  }

  /**
   * Update video frame text
   */
  async function setVideoText(text: string) {
    if (!audioEngine) {
      console.error('[NDI] Audio engine not available')
      return
    }
    
    try {
      await audioEngine.setNdiVideoText(text)
      videoText.value = text
      console.log('[NDI] Video text updated:', text)
    } catch (error) {
      console.error('[NDI] Failed to update video text:', error)
      throw error
    }
  }

  /**
   * Toggle streaming on/off
   */
  async function toggleStreaming() {
    if (isStreaming.value) {
      await stopStreaming()
    } else {
      await startStreaming()
    }
  }

  return {
    // State
    isStreaming: computed(() => isStreaming.value),
    streamName: computed(() => streamName.value),
    streamSource: computed(() => streamSource.value),
    videoText: computed(() => videoText.value),
    config,

    // Actions
    startStreaming,
    stopStreaming,
    changeSource,
    setStreamName,
    setVideoText,
    toggleStreaming,
  }
}
