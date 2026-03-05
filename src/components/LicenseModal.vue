<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="modal-overlay" @click.self="close">
        <div class="modal-container">
          <div class="modal-header">
            <h2 class="modal-title">License Activation</h2>
            <button class="btn-close" @click="close" aria-label="Close">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="modal-body">
            <!-- Current License Info -->
            <div v-if="currentLicense" class="license-info">
              <div class="license-badge" :class="`badge-${currentLicense.type}`">
                {{ currentLicense.type.toUpperCase() }}
              </div>
              <p class="license-key">
                <strong>Key:</strong> {{ currentLicense.key }}
              </p>
              <p v-if="currentLicense.expiresAt" class="license-expires">
                <strong>Expires:</strong> {{ new Date(currentLicense.expiresAt).toLocaleDateString() }}
              </p>
            </div>

            <!-- License Input Form -->
            <div class="license-form">
              <label for="license-key" class="form-label">Enter License Key</label>
              <input
                id="license-key"
                v-model="licenseKey"
                type="text"
                class="form-input"
                placeholder="XXXX-XXXX-XXXX-XXXX"
                :disabled="isLoading"
                @keyup.enter="handleVerify"
              />

              <!-- Error Message -->
              <p v-if="error" class="error-message">
                ⚠️ {{ error }}
              </p>

              <!-- Success Message -->
              <p v-if="showSuccess" class="success-message">
                ✅ License activated successfully! Reloading app...
              </p>

              <!-- License Tiers Info -->
              <div class="tiers-info">
                <h3>License Tiers:</h3>
                <div class="tier">
                  <strong>DEMO</strong> - 1 audio track, 1 signal track
                </div>
                <div class="tier">
                  <strong>MEDIUM</strong> - 4 audio tracks, 6 aux buses, subgroups
                </div>
                <div class="tier">
                  <strong>FULL</strong> - 24 audio tracks, full features
                </div>
              </div>
            </div>
          </div>

          <div class="modal-footer">
            <button
              v-if="currentLicense?.type !== 'demo'"
              class="btn btn-secondary"
              @click="handleClearLicense"
            >
              Clear License
            </button>
            <button
              class="btn btn-primary"
              :disabled="!licenseKey.trim() || isLoading"
              @click="handleVerify"
            >
              {{ isLoading ? 'Verifying...' : 'Activate License' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useLicense } from '@/composables/useLicense'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const { currentLicense, isLoading, error, verifyLicense, clearLicense } = useLicense()

const licenseKey = ref('')
const showSuccess = ref(false)

function close() {
  emit('update:modelValue', false)
}

async function handleVerify() {
  if (!licenseKey.value.trim()) return

  showSuccess.value = false
  const success = await verifyLicense(licenseKey.value.trim())

  if (success) {
    showSuccess.value = true
    licenseKey.value = ''
    // App will reload automatically in 500ms (see useLicense.ts)
  }
}

function handleClearLicense() {
  if (confirm('Are you sure you want to clear your license? The app will reload and revert to DEMO mode.')) {
    clearLicense()
    licenseKey.value = ''
    showSuccess.value = false
    // App will reload automatically in 300ms (see useLicense.ts)
  }
}

// Reset form when modal closes
watch(() => props.modelValue, (isOpen) => {
  if (!isOpen) {
    licenseKey.value = ''
    showSuccess.value = false
  }
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 20px;
}

.modal-container {
  background: linear-gradient(180deg, #2a2a2a 0%, #1f1f1f 100%);
  border-radius: 12px;
  border: 1px solid #3a3a3a;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  width: 100%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-header {
  padding: 24px;
  border-bottom: 1px solid #3a3a3a;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-title {
  font-size: 20px;
  font-weight: 600;
  color: #fff;
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.btn-close:hover {
  color: #fff;
  background: #3a3a3a;
}

.modal-body {
  padding: 24px;
}

.license-info {
  background: #252525;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
}

.license-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 12px;
}

.badge-demo {
  background: #666;
  color: #fff;
}

.badge-medium {
  background: #4a90e2;
  color: #fff;
}

.badge-full {
  background: #50c878;
  color: #fff;
}

.license-key,
.license-expires {
  margin: 8px 0;
  font-size: 14px;
  color: #ccc;
}

.license-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-label {
  font-size: 14px;
  font-weight: 500;
  color: #ccc;
  margin-bottom: 8px;
}

.form-input {
  width: 100%;
  padding: 12px;
  background: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
  font-family: 'Courier New', monospace;
  letter-spacing: 1px;
  transition: border-color 0.2s;
}

.form-input:focus {
  outline: none;
  border-color: #4a90e2;
}

.form-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error-message {
  color: #ff4444;
  font-size: 14px;
  margin: 0;
  padding: 12px;
  background: rgba(255, 68, 68, 0.1);
  border-radius: 6px;
  border: 1px solid rgba(255, 68, 68, 0.3);
}

.success-message {
  color: #50c878;
  font-size: 14px;
  margin: 0;
  padding: 12px;
  background: rgba(80, 200, 120, 0.1);
  border-radius: 6px;
  border: 1px solid rgba(80, 200, 120, 0.3);
}

.tiers-info {
  background: #252525;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  padding: 16px;
}

.tiers-info h3 {
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  margin: 0 0 12px 0;
}

.tier {
  font-size: 13px;
  color: #aaa;
  padding: 6px 0;
  border-bottom: 1px solid #333;
}

.tier:last-child {
  border-bottom: none;
}

.tier strong {
  color: #fff;
}

.modal-footer {
  padding: 24px;
  border-top: 1px solid #3a3a3a;
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn {
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: #4a90e2;
  color: #fff;
}

.btn-primary:hover:not(:disabled) {
  background: #357abd;
}

.btn-secondary {
  background: #3a3a3a;
  color: #fff;
}

.btn-secondary:hover {
  background: #4a4a4a;
}

/* Modal transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.3s;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.9);
}
</style>
