<template>
  <Transition name="modal">
    <div v-if="show" class="modal-overlay" @click.self="handleCancel">
      <div class="modal-container">
        <div class="modal-content">
          <!-- Icon -->
          <div class="modal-icon">
            <svg class="w-16 h-16 text-orange-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>

          <!-- Title -->
          <h3 class="modal-title">
            Client Remoto Già Attivo
          </h3>

          <!-- Message -->
          <p class="modal-message">
            {{ message }}
          </p>

          <!-- Actions -->
          <div class="modal-actions">
            <button @click="handleCancel" class="btn-cancel">
              Annulla
            </button>
            <button @click="handleConfirm" class="btn-confirm">
              Prendi il Controllo
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">

interface Props {
  show: boolean
  message?: string
}

const props = withDefaults(defineProps<Props>(), {
  message: 'C\'è già un client remoto attivo. Vuoi prendere il controllo?'
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const handleConfirm = () => {
  emit('confirm')
}

const handleCancel = () => {
  emit('cancel')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 1rem;
}

.modal-container {
  background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
  border-radius: 1rem;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  max-width: 28rem;
  width: 100%;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-content {
  padding: 2rem;
  text-align: center;
}

.modal-icon {
  display: flex;
  justify-content: center;
  margin-bottom: 1.5rem;
}

.modal-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 1rem;
}

.modal-message {
  font-size: 1rem;
  color: #a0a0a0;
  line-height: 1.5;
  margin-bottom: 2rem;
}

.modal-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.btn-cancel,
.btn-confirm {
  padding: 0.75rem 1.5rem;
  font-size: 1rem;
  font-weight: 600;
  border-radius: 0.5rem;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 120px;
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.btn-confirm {
  background: linear-gradient(135deg, #f97316 0%, #ea580c 100%);
  color: #ffffff;
  box-shadow: 0 4px 6px -1px rgba(249, 115, 22, 0.4);
}

.btn-confirm:hover {
  background: linear-gradient(135deg, #ea580c 0%, #c2410c 100%);
  box-shadow: 0 6px 8px -1px rgba(249, 115, 22, 0.6);
  transform: translateY(-1px);
}

.btn-confirm:active,
.btn-cancel:active {
  transform: translateY(0);
}

/* Modal transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.3s ease;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.9);
}
</style>
