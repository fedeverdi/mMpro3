import { ref, computed } from 'vue'
import type { BuildMode } from '@/config/buildLimits'
import { useAudioEngine } from './useAudioEngine'

export interface LicenseInfo {
  key: string
  type: BuildMode
  expiresAt?: string
  isValid: boolean
}

const LICENSE_STORAGE_KEY = 'mmpro3_license'

// Reactive state
const currentLicense = ref<LicenseInfo | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)

// Get audio engine instance once (only in Electron context)
let audioEngineInstance: ReturnType<typeof useAudioEngine> | null = null
const getAudioEngine = () => {
  if (!audioEngineInstance && (window as any).audioEngine) {
    audioEngineInstance = useAudioEngine()
  }
  return audioEngineInstance
}

// Load license from Rust engine on init
async function loadStoredLicense() {
  try {
    const isRemoteClient = !(window as any).electronAPI
    
    if (isRemoteClient) {
      // Remote client: wait for window.audioEngine to be ready (set by useAudioEngine)
      let retries = 0
      const maxRetries = 10
      
      while (!window.audioEngine && retries < maxRetries) {
        console.log('[useLicense] Waiting for audioEngine to be ready...')
        await new Promise(resolve => setTimeout(resolve, 200))
        retries++
      }
      
      if (window.audioEngine && typeof window.audioEngine.getLicense === 'function') {
        try {
          console.log('[useLicense] Remote client requesting license from Rust...')
          const license = await window.audioEngine.getLicense()
          console.log('[useLicense] Remote client received license:', license)
          
          if (license && license.key !== 'DEMO') {
            currentLicense.value = {
              key: license.key,
              type: license.license_type,
              expiresAt: license.expires_at,
              isValid: license.is_valid
            }
            // Save to localStorage AFTER Rust confirms
            localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(currentLicense.value))
            console.log('[useLicense] License loaded from Rust (remote):', currentLicense.value.type)
            return
          }
        } catch (err) {
          console.error('[useLicense] Failed to get license from Rust:', err)
        }
      } else {
        console.warn('[useLicense] audioEngine not available after waiting')
      }
    } else {
      // Electron: request from Rust via IPC using useAudioEngine
      const audioEngine = getAudioEngine()
      if (audioEngine) {
        try {
          const license = await audioEngine.getLicense()
          console.log('[useLicense] Received license from Rust:', license)
          
          if (license && license.key !== 'DEMO') {
            currentLicense.value = {
              key: license.key,
              type: license.license_type || license.type,
              expiresAt: license.expires_at,
              isValid: license.is_valid
            }
            
            // Save to localStorage AFTER Rust confirms
            localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(currentLicense.value))
            console.log('[useLicense] License loaded from Rust:', currentLicense.value.type)
            return
          }
        } catch (err) {
          console.error('[useLicense] Failed to get license from Rust:', err)
        }
      }
    }
    
    // Only set demo if Rust didn't return a valid license
    console.log('[useLicense] No valid license from Rust, using demo')
    currentLicense.value = {
      key: 'DEMO',
      type: 'demo',
      isValid: true
    }
    localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(currentLicense.value))
  } catch (e) {
    console.error('Failed to load license:', e)
    currentLicense.value = {
      key: 'DEMO',
      type: 'demo',
      isValid: true
    }
    localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(currentLicense.value))
  }
}

// Initialize on first use
let initialized = false
let initPromise: Promise<void> | null = null

function ensureInitialized() {
  if (!initialized) {
    initialized = true
    
    // Start loading license from Rust (async)
    initPromise = loadStoredLicense()
    
    // For remote clients, listen to license updates from Rust engine
    const isRemoteClient = !(window as any).electronAPI
    if (isRemoteClient) {
      console.log('[useLicense] Setting up remote client license listener')
      
      // Wait for initial loading to complete before setting up the listener
      // This prevents reload loops during initial license fetch
      initPromise.finally(() => {
        setTimeout(() => {
          console.log('[useLicense] Initial license loaded, now listening for updates')
          
          window.addEventListener('license-updated', ((event: CustomEvent) => {
            const license = event.detail
            console.log('[useLicense] Remote license update received:', license.license_type)
            console.log('[useLicense] Current license:', currentLicense.value?.type)
            
            // Only reload if the license actually changed
            const hasChanged = 
              !currentLicense.value || 
              currentLicense.value.type !== license.license_type ||
              currentLicense.value.key !== license.key
            
            if (!hasChanged) {
              console.log('[useLicense] License unchanged, skipping reload')
              return
            }
            
            console.log('[useLicense] License changed from', currentLicense.value?.type, 'to', license.license_type)
            
            // Update the current license
            currentLicense.value = {
              key: license.key,
              type: license.license_type,
              expiresAt: license.expires_at,
              isValid: license.is_valid
            }
            
            // Save to localStorage for buildLimits.ts compatibility
            localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(currentLicense.value))
            
            // Reload page to apply new build limits
            setTimeout(() => {
              console.log('[useLicense] Reloading to apply new license...')
              window.location.reload()
            }, 500)
          }) as EventListener)
        }, 500) // Small delay to ensure everything is stable
      })
    }
    
    initialized = true
  }
}

export function useLicense() {
  ensureInitialized()

  const licenseType = computed(() => currentLicense.value?.type || 'demo')
  const isLicensed = computed(() => currentLicense.value?.isValid && currentLicense.value.type !== 'demo')

  async function verifyLicense(licenseKey: string): Promise<boolean> {
    isLoading.value = true
    error.value = null

    try {
      // Call IPC to verify license in main process
      const electronAPI = (window as any).electronAPI
      if (!electronAPI || !electronAPI.verifyLicense) {
        throw new Error('License verification not available')
      }

      const result = await electronAPI.verifyLicense(licenseKey)

      if (result.valid) {
        console.log('[useLicense] ===== LICENSE VERIFICATION SUCCESSFUL =====')
        console.log('[useLicense] License type:', result.type)
        console.log('[useLicense] Expires at:', result.expiresAt)
        
        // Save license to Rust engine (single source of truth)
        const audioEngine = getAudioEngine()
        if (audioEngine) {
          try {
            console.log('[useLicense] CALLING audioEngine.saveLicense with:', licenseKey, result.type, result.expiresAt || null)
            await audioEngine.saveLicense(
              licenseKey,
              result.type,
              result.expiresAt || null
            )
            console.log('[useLicense] License saved to Rust successfully')
          } catch (err) {
            console.error('[useLicense] Failed to save license to Rust:', err)
            throw err
          }
        } else {
          console.error('[useLicense] Audio engine not available!')
        }
        
        const licenseInfo: LicenseInfo = {
          key: licenseKey,
          type: result.type,
          expiresAt: result.expiresAt,
          isValid: true
        }

        currentLicense.value = licenseInfo
        
        // Save to localStorage for buildLimits.ts compatibility
        localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(licenseInfo))
        console.log('[useLicense] License saved to localStorage:', licenseInfo.type)

        // Reload app to apply new build limits (give Rust time to write file)
        setTimeout(() => {
          console.log('[useLicense] Reloading app to apply new license')
          window.location.reload()
        }, 1000)

        return true
      } else {
        error.value = result.message || 'Invalid license key'
        return false
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to verify license'
      console.error('License verification error:', e)
      return false
    } finally {
      isLoading.value = false
    }
  }

  function clearLicense() {
    // Clear license in Rust engine (single source of truth)
    const audioEngine = getAudioEngine()
    if (audioEngine) {
      audioEngine.saveLicense('DEMO', 'demo', null).then(() => {
        console.log('[useLicense] License cleared via Rust')
      }).catch((error: Error) => {
        console.error('[useLicense] Failed to clear license via Rust:', error)
      })
    }
    
    // Update local state
    currentLicense.value = {
      key: 'DEMO',
      type: 'demo',
      isValid: true
    }
    
    // Save to localStorage for buildLimits.ts compatibility
    localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(currentLicense.value))
    
    // Reload app to revert to demo mode
    setTimeout(() => {
      window.location.reload()
    }, 300)
  }

  return {
    currentLicense: computed(() => currentLicense.value),
    licenseType,
    isLicensed,
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    verifyLicense,
    clearLicense
  }
}
