import { ref, computed, onMounted, onUnmounted } from 'vue'

/**
 * Composable per gestire l'URL di rete dell'applicazione
 */
export function useNetworkUrl() {
  const localIp = ref<string>('')
  const port = ref<number>(5173)
  const isLoading = ref(true)

  // URL completo per accedere all'app dalla rete
  const networkUrl = computed(() => {
    if (!localIp.value) return ''
    return `http://${localIp.value}:${port.value}`
  })

  // Funzione per ottenere l'IP locale
  const fetchLocalIp = async () => {
    try {
      isLoading.value = true

      // In ambiente Electron, usa l'IPC per ottenere l'IP dal processo principale
      if (window.electronAPI?.getLocalIp) {
        const result = await window.electronAPI.getLocalIp()
        if (result) {
          localIp.value = result.ip
          port.value = result.port || 5173
        }
      } else {
        // In ambiente browser, prova a usare l'hostname o localhost
        localIp.value = window.location.hostname || 'localhost'
        port.value = parseInt(window.location.port) || 5173
      }
    } catch (error) {
      console.error('[useNetworkUrl] Error fetching local IP:', error)
      localIp.value = 'localhost'
    } finally {
      isLoading.value = false
    }
  }

  // Funzione per copiare l'URL negli appunti
  const copyToClipboard = async (): Promise<boolean> => {
    if (!networkUrl.value) return false

    try {
      await navigator.clipboard.writeText(networkUrl.value)
      return true
    } catch (error) {
      console.error('[useNetworkUrl] Error copying to clipboard:', error)
      // Fallback per vecchi browser
      try {
        const textArea = document.createElement('textarea')
        textArea.value = networkUrl.value
        textArea.style.position = 'fixed'
        textArea.style.left = '-999999px'
        document.body.appendChild(textArea)
        textArea.select()
        const successful = document.execCommand('copy')
        document.body.removeChild(textArea)
        return successful
      } catch (fallbackError) {
        console.error('[useNetworkUrl] Fallback copy failed:', fallbackError)
        return false
      }
    }
  }

  onMounted(() => {
    fetchLocalIp()
  })

  return {
    localIp,
    port,
    networkUrl,
    isLoading,
    copyToClipboard,
    refresh: fetchLocalIp
  }
}
