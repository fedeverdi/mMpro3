/**
 * Detached Window WebSocket Client
 * Used by pop-out component windows to receive real-time updates
 * Unlike RemoteAudioEngine, this client:
 * - Does NOT block the main Electron mixer
 * - Is read-only (receives updates but doesn't send commands)
 * - Can coexist with multiple other detached windows
 */

export class DetachedWindowClient {
  private ws: WebSocket | null = null
  private url: string
  private componentType: string
  private reconnectInterval: number = 2000
  private reconnectTimer: number | null = null
  private isConnecting: boolean = false
  private eventListeners: Map<string, Set<Function>> = new Map()

  constructor(componentType: string, host: string = 'localhost', port: number = 3001) {
    this.componentType = componentType
    this.url = `ws://${host}:${port}`
  }

  async connect(): Promise<void> {
    if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
      return
    }

    return new Promise((resolve, reject) => {
      this.isConnecting = true
      console.log(`[DetachedWindow:${this.componentType}] Connecting to ${this.url}...`)

      this.ws = new WebSocket(this.url)

      this.ws.onopen = () => {
        this.isConnecting = false
        console.log(`[DetachedWindow:${this.componentType}] Connected`)
        
        // Register as detached window client (not a remote front client)
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
          this.ws.send(JSON.stringify({
            type: 'detached-window-register',
            componentType: this.componentType
          }))
        }
        
        if (this.reconnectTimer) {
          clearTimeout(this.reconnectTimer)
          this.reconnectTimer = null
        }
        resolve()
      }

      this.ws.onmessage = (event) => {
        try {
          const response = JSON.parse(event.data)
          
          // Emit events for all registered listeners
          this.emit(response.type, response)
          
          // Also emit a generic 'message' event
          this.emit('message', response)
          
        } catch (error) {
          console.error(`[DetachedWindow:${this.componentType}] Error parsing message:`, error)
        }
      }

      this.ws.onerror = (error) => {
        console.error(`[DetachedWindow:${this.componentType}] WebSocket error:`, error)
        this.isConnecting = false
        reject(error)
      }

      this.ws.onclose = () => {
        console.log(`[DetachedWindow:${this.componentType}] Connection closed, will attempt to reconnect...`)
        this.isConnecting = false
        this.ws = null
        
        // Auto-reconnect after delay
        if (!this.reconnectTimer) {
          this.reconnectTimer = window.setTimeout(() => {
            this.reconnectTimer = null
            this.connect().catch(err => {
              console.error(`[DetachedWindow:${this.componentType}] Reconnection failed:`, err)
            })
          }, this.reconnectInterval)
        }
      }
    })
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    
    console.log(`[DetachedWindow:${this.componentType}] Disconnected`)
  }

  /**
   * Register event listener
   */
  on(eventType: string, callback: Function): void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set())
    }
    this.eventListeners.get(eventType)!.add(callback)
  }

  /**
   * Unregister event listener
   */
  off(eventType: string, callback: Function): void {
    const listeners = this.eventListeners.get(eventType)
    if (listeners) {
      listeners.delete(callback)
    }
  }

  /**
   * Emit event to all registered listeners
   */
  private emit(eventType: string, data: any): void {
    const listeners = this.eventListeners.get(eventType)
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data)
        } catch (error) {
          console.error(`[DetachedWindow:${this.componentType}] Error in event listener:`, error)
        }
      })
    }
  }

  /**
   * Send command to server (for interactive detached windows)
   */
  send(command: any): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket not connected'))
        return
      }

      try {
        this.ws.send(JSON.stringify(command))
        resolve()
      } catch (error) {
        reject(error)
      }
    })
  }

  /**
   * Send command and wait for specific response type
   */
  sendAndWaitForResponse(command: any, responseType: string, timeout: number = 5000): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket not connected'))
        return
      }

      const timeoutId = setTimeout(() => {
        this.off(responseType, responseHandler)
        reject(new Error(`Timeout waiting for response: ${responseType}`))
      }, timeout)

      const responseHandler = (data: any) => {
        clearTimeout(timeoutId)
        this.off(responseType, responseHandler)
        resolve(data)
      }

      this.on(responseType, responseHandler)

      try {
        this.ws.send(JSON.stringify(command))
      } catch (error) {
        clearTimeout(timeoutId)
        this.off(responseType, responseHandler)
        reject(error)
      }
    })
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN
  }
}
