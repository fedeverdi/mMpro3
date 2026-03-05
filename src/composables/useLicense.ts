import { ref, computed } from 'vue'
import type { BuildMode } from '@/config/buildLimits'

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

// Load license from localStorage on init
function loadStoredLicense() {
  try {
    const stored = localStorage.getItem(LICENSE_STORAGE_KEY)
    if (stored) {
      currentLicense.value = JSON.parse(stored)
    } else {
      // Default to demo if no license
      currentLicense.value = {
        key: 'DEMO',
        type: 'demo',
        isValid: true
      }
    }
  } catch (e) {
    console.error('Failed to load stored license:', e)
    currentLicense.value = {
      key: 'DEMO',
      type: 'demo',
      isValid: true
    }
  }
}

// Initialize on first use
let initialized = false
function ensureInitialized() {
  if (!initialized) {
    loadStoredLicense()
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
        const licenseInfo: LicenseInfo = {
          key: licenseKey,
          type: result.type,
          expiresAt: result.expiresAt,
          isValid: true
        }

        // Save to storage
        localStorage.setItem(LICENSE_STORAGE_KEY, JSON.stringify(licenseInfo))
        currentLicense.value = licenseInfo

        // Reload app to apply new build limits
        setTimeout(() => {
          window.location.reload()
        }, 500)

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
    localStorage.removeItem(LICENSE_STORAGE_KEY)
    currentLicense.value = {
      key: 'DEMO',
      type: 'demo',
      isValid: true
    }
    
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
