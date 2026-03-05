#!/usr/bin/env node

/**
 * Script to add licenses to Vercel Edge Config
 * Usage: node scripts/add-license-edge.js <license-key> <type> [expiresAt]
 * 
 * Example:
 *   node scripts/add-license-edge.js ABC-123-XYZ full
 *   node scripts/add-license-edge.js DEF-456-UVW medium 2027-12-31
 */

// Load environment variables from .env file
require('dotenv').config()

const https = require('https')

// Get arguments
const [,, licenseKey, type, expiresAt] = process.argv

// Validate arguments
if (!licenseKey || !type) {
  console.error('❌ Usage: node scripts/add-license-edge.js <license-key> <type> [expiresAt]')
  console.error('   Types: demo, medium, full')
  console.error('   Example: node scripts/add-license-edge.js ABC-123-XYZ full 2027-12-31')
  process.exit(1)
}

if (!['demo', 'medium', 'full'].includes(type)) {
  console.error('❌ Invalid type. Must be: demo, medium, or full')
  process.exit(1)
}

// Get environment variables
const edgeConfigUrl = process.env.EDGE_CONFIG
const vercelToken = process.env.VERCEL_API_TOKEN

if (!edgeConfigUrl) {
  console.error('❌ EDGE_CONFIG environment variable not set')
  console.error('   Add it to your .env file from Vercel Dashboard')
  process.exit(1)
}

if (!vercelToken) {
  console.error('❌ VERCEL_API_TOKEN environment variable not set')
  console.error('   Get it from: https://vercel.com/account/tokens')
  console.error('   Add to .env: VERCEL_API_TOKEN=your_token_here')
  process.exit(1)
}

// Extract Edge Config ID from URL
// Format: https://edge-config.vercel.com/ecfg_xxxxx?token=xxxxx
const edgeConfigIdMatch = edgeConfigUrl.match(/edge-config\.vercel\.com\/(ecfg_[a-zA-Z0-9]+)/)
if (!edgeConfigIdMatch) {
  console.error('❌ Invalid EDGE_CONFIG URL format')
  process.exit(1)
}
const edgeConfigId = edgeConfigIdMatch[1]

// Prepare license data
const createdAt = new Date().toISOString()
const licenseData = {
  type,
  active: true,
  createdAt
}

if (expiresAt) {
  try {
    const expiryDate = new Date(expiresAt)
    if (isNaN(expiryDate.getTime())) {
      throw new Error('Invalid date')
    }
    licenseData.expiresAt = expiryDate.toISOString()
  } catch (err) {
    console.error('❌ Invalid expiry date format. Use: YYYY-MM-DD')
    process.exit(1)
  }
}

// Prepare API request
const requestData = JSON.stringify({
  items: [
    {
      operation: 'upsert',
      key: `license_${licenseKey}`,
      value: licenseData
    }
  ]
})

const options = {
  hostname: 'api.vercel.com',
  port: 443,
  path: `/v1/edge-config/${edgeConfigId}/items`,
  method: 'PATCH',
  headers: {
    'Authorization': `Bearer ${vercelToken}`,
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(requestData)
  }
}

console.log(`🔑 Adding license: ${licenseKey}`)
console.log(`📦 Type: ${type.toUpperCase()}`)
if (licenseData.expiresAt) {
  console.log(`⏰ Expires: ${licenseData.expiresAt}`)
}
console.log(`🌐 Edge Config ID: ${edgeConfigId}`)
console.log('')

// Make API request
const req = https.request(options, (res) => {
  let data = ''

  res.on('data', (chunk) => {
    data += chunk
  })

  res.on('end', () => {
    if (res.statusCode === 200) {
      console.log('✅ License added successfully!')
      console.log('')
      console.log('📝 License details:')
      console.log(`   Key: license_${licenseKey}`)
      console.log(`   Type: ${type}`)
      console.log(`   Active: true`)
      console.log(`   Created: ${licenseData.createdAt}`)
      if (licenseData.expiresAt) {
        console.log(`   Expires: ${licenseData.expiresAt}`)
      }
      console.log('')
      console.log('⚡ Changes propagate globally within seconds')
    } else {
      console.error(`❌ Failed to add license (${res.statusCode})`)
      try {
        const errorData = JSON.parse(data)
        console.error('Error:', errorData.error?.message || errorData)
      } catch {
        console.error('Response:', data)
      }
      process.exit(1)
    }
  })
})

req.on('error', (error) => {
  console.error('❌ Request failed:', error.message)
  process.exit(1)
})

req.write(requestData)
req.end()
