import { ref } from 'vue'

export interface TrackDragCallbacks {
  onDragStart?: () => void
}

export function useTrackDrag(callbacks: TrackDragCallbacks = {}) {
  // Ref per l'elemento traccia (usato per generare il ghost durante il drag)
  const trackElement = ref<HTMLElement | null>(null)

  /**
   * Gestisce l'inizio del drag con ghost personalizzato
   */
  function handleDragStart(event: DragEvent) {
    // Crea un ghost personalizzato della traccia
    if (trackElement.value && event.dataTransfer) {
      const clone = trackElement.value.cloneNode(true) as HTMLElement

      // Stile del ghost - forza le dimensioni esatte per evitare espansioni
      clone.style.position = 'absolute'
      clone.style.top = '-9999px'
      clone.style.left = '-9999px'
      clone.style.width = trackElement.value.offsetWidth + 'px'
      clone.style.height = trackElement.value.offsetHeight + 'px'
      clone.style.opacity = '0.5'
      clone.style.transform = 'rotate(3deg)'
      clone.style.boxShadow = '0 10px 30px rgba(0, 0, 0, 0.5)'
      clone.style.border = '2px solid rgba(59, 130, 246, 0.5)'
      clone.style.pointerEvents = 'none'
      clone.style.overflow = 'hidden'

      document.body.appendChild(clone)

      // Imposta il clone come drag image
      event.dataTransfer.setDragImage(clone, trackElement.value.offsetWidth / 2, 30)

      // Rimuovi il clone dopo un breve ritardo
      setTimeout(() => {
        document.body.removeChild(clone)
      }, 0)
    }

    // Notifica il parent che il drag è iniziato
    if (callbacks.onDragStart) {
      callbacks.onDragStart()
    }
  }

  return {
    trackElement,
    handleDragStart
  }
}
