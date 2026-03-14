import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'
import { app } from 'electron'

const HTTP_PORT = 5173
let httpServer: http.Server | null = null

/**
 * Start HTTP server for serving web interface in production
 */
export const startHttpServer = (): void => {
  // Only start HTTP server in production (when app is packaged)
  if (!app.isPackaged) {
    console.log('[HTTP] Skipping HTTP server in development mode')
    return
  }

  try {
    // Determine the path to the web build
    const webBuildPath = path.join(process.resourcesPath, 'dist')
    
    if (!fs.existsSync(webBuildPath)) {
      console.error('[HTTP] Web build not found at:', webBuildPath)
      console.error('[HTTP] Make sure to run "npm run build:web" before packaging')
      return
    }

    httpServer = http.createServer((req, res) => {
      // Parse URL and handle routing
      const url = req.url || '/'
      let filePath = path.join(webBuildPath, url === '/' ? 'index.html' : url)
      
      // Security: prevent directory traversal
      if (!filePath.startsWith(webBuildPath)) {
        res.writeHead(403)
        res.end('Forbidden')
        return
      }
      
      // If file doesn't exist, serve index.html for SPA routing
      if (!fs.existsSync(filePath)) {
        filePath = path.join(webBuildPath, 'index.html')
      }
      
      // Determine content type
      const ext = path.extname(filePath)
      const contentTypes: Record<string, string> = {
        '.html': 'text/html',
        '.js': 'application/javascript',
        '.css': 'text/css',
        '.json': 'application/json',
        '.png': 'image/png',
        '.jpg': 'image/jpeg',
        '.svg': 'image/svg+xml',
        '.ico': 'image/x-icon',
        '.woff': 'font/woff',
        '.woff2': 'font/woff2',
      }
      const contentType = contentTypes[ext] || 'application/octet-stream'
      
      // Read and serve file
      fs.readFile(filePath, (err, data) => {
        if (err) {
          console.error('[HTTP] Error reading file:', filePath, err)
          res.writeHead(500)
          res.end('Internal Server Error')
          return
        }
        
        res.writeHead(200, { 
          'Content-Type': contentType,
          'Access-Control-Allow-Origin': '*' // Allow cross-origin requests
        })
        res.end(data)
      })
    })

    httpServer.listen(HTTP_PORT, '0.0.0.0', () => {
      console.log(`[HTTP] Server started on port ${HTTP_PORT}`)
      console.log(`[HTTP] Serving from: ${webBuildPath}`)
    })

    httpServer.on('error', (error: any) => {
      if (error.code === 'EADDRINUSE') {
        console.error(`[HTTP] Port ${HTTP_PORT} is already in use`)
      } else {
        console.error('[HTTP] Server error:', error)
      }
    })
  } catch (error) {
    console.error('[HTTP] Failed to start server:', error)
  }
}

/**
 * Stop HTTP server
 */
export const stopHttpServer = (): void => {
  if (httpServer) {
    httpServer.close()
    httpServer = null
    console.log('[HTTP] Server stopped')
  }
}

export { HTTP_PORT }
