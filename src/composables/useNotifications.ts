import { ref } from 'vue'

export interface Notification {
  id: number
  type: 'info' | 'success' | 'warning' | 'error' | 'confirm'
  message: string
  duration?: number
  onConfirm?: () => void
  onCancel?: () => void
}

// Global state for notifications
const notifications = ref<Notification[]>([])
let nextId = 1

// Core functions
const show = (
  message: string, 
  type: Notification['type'] = 'info',
  duration: number = 3000
) => {
  const notification: Notification = {
    id: nextId++,
    type,
    message,
    duration
  }
  
  notifications.value.push(notification)
  
  if (duration > 0) {
    setTimeout(() => {
      remove(notification.id)
    }, duration)
  }
}

const confirm = (message: string): Promise<boolean> => {
  return new Promise((resolve) => {
    const notification: Notification = {
      id: nextId++,
      type: 'confirm',
      message,
      onConfirm: () => {
        remove(notification.id)
        resolve(true)
      },
      onCancel: () => {
        remove(notification.id)
        resolve(false)
      }
    }
    
    notifications.value.push(notification)
  })
}

const remove = (id: number) => {
  const index = notifications.value.findIndex(n => n.id === id)
  if (index !== -1) {
    notifications.value.splice(index, 1)
  }
}

// Helper functions
const info = (msg: string, duration?: number) => show(msg, 'info', duration)
const success = (msg: string, duration?: number) => show(msg, 'success', duration)
const warning = (msg: string, duration?: number) => show(msg, 'warning', duration)
const error = (msg: string, duration?: number) => show(msg, 'error', duration)

// Export both as composable (for component use) and named exports (for anywhere)
export function useNotifications() {
  return {
    notifications,
    show,
    confirm,
    remove,
    info,
    success,
    warning,
    error
  }
}

// Named exports for direct use in composables
export { notifications, show, confirm, remove, info, success, warning, error }
