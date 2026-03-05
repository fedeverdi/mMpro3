# License System Setup Guide

## Overview
mMpro3 uses **Vercel Edge Config** for ultra-fast license validation. Edge Config provides:
- **FREE** up to 512KB storage
- **< 1ms** global latency from edge network
- Perfect for license key storage and validation

The verification happens in Electron's main process via IPC.

**Architecture:**
- **Renderer (Vue)**: UI for entering license key → calls IPC
- **Main Process (Node.js)**: Uses `@vercel/edge-config` SDK to verify against Edge Config
- **Storage**: Valid licenses saved in localStorage

## License Tiers

- **DEMO**: 1 audio track, 1 signal track (free, no license needed)
- **MEDIUM**: 4 audio tracks, 6 aux buses, subgroups
- **FULL**: 24 audio tracks, full features

## 1. Setup Vercel Edge Config

### Create Edge Config on Vercel

1. Go to [Vercel Dashboard](https://vercel.com/dashboard)
2. Navigate to **Storage** → **Create**
3. Select **Edge Config**
4. Choose a name (e.g., `mmpro3-licenses`)
5. Click **Create**

### Get Connection String

1. In the Edge Config page, click **Generate Token** or copy the connection string
2. It will look like: `https://edge-config.vercel.com/xxxxx?token=xxxxx`

3. Create a `.env` file in your project root (copy from `.env.example`)
4. Add the connection string:

```bash
# .env file
EDGE_CONFIG=https://edge-config.vercel.com/xxxxx?token=xxxxx
```

**Important**: Make sure `.env` is in `.gitignore` - never commit it!

## 2. Add License Keys to Edge Config

### Via Script (Raccomandato)

Usa lo script fornito per aggiungere licenze facilmente:

```bash
# 1. Assicurati di avere il VERCEL_API_TOKEN nel .env
#    Ottienilo da: https://vercel.com/account/tokens
#    Aggiungi al .env: VERCEL_API_TOKEN=your_token_here

# 2. Aggiungi licenze
# Licenza FULL senza scadenza
node scripts/add-license-edge.js ABC-123-XYZ full

# Licenza MEDIUM con scadenza
node scripts/add-license-edge.js DEF-456-UVW medium 2027-12-31

# Licenza DEMO
node scripts/add-license-edge.js GHI-789-RST demo
```

Lo script estrae automaticamente l'Edge Config ID dal tuo `EDGE_CONFIG` e usa l'API di Vercel per aggiungere la licenza.

### Via Vercel Dashboard

1. Go to your Edge Config in Vercel Dashboard
2. Click **Edit Items**
3. Add a new item with:
   - **Key**: `license_ABC-123-XYZ` (use format `license_` + your key)
   - **Value**: 
   ```json
   {
     "type": "full",
     "active": true,
     "createdAt": "2026-03-05T00:00:00.000Z",
     "expiresAt": "2027-12-31T23:59:59.999Z"
   }
   ```
4. Click **Save Changes**

## 3. License Data Structure

Each license key is stored with this structure:

```json
{
  "type": "demo" | "medium" | "full",
  "active": true,
  "createdAt": "2026-03-05T00:00:00.000Z",
  "expiresAt": "2027-12-31T23:59:59.999Z" // Optional
}
```

## 4. How It Works

**User Flow:**

1. User clicks license badge in footer (shows current tier)
2. Enters license key in modal
3. **Frontend** calls `electronAPI.verifyLicense(key)` via IPC
4. **Main Process** uses `@vercel/edge-config` SDK to query Edge Config
5. If valid, license is stored in localStorage
6. App applies new build limits immediately

**Code Flow:**

```
Vue Component (LicenseModal.vue)
    ↓
Composable (useLicense.ts) → electronAPI.verifyLicense()
    ↓
IPC Bridge (preload.ts)
    ↓
Main Process (main.ts) → import { get } from '@vercel/edge-config'
    ↓
Vercel Edge Config (Global Edge Network)
```

## 5. Testing

Test the license system by running the app in development:

```bash
# Make sure .env has EDGE_CONFIG connection string
npm start

# In the app:
# 1. Click license badge in footer
# 2. Enter a test license key (e.g., ABC-123-XYZ)
# 3. Verify it activates correctly
```

Check the console for verification logs if something fails.

## 6. Security Notes

- ✅ License keys validated from Edge Config (Electron main process)
- ✅ No Edge Config credentials exposed to renderer process
- ✅ IPC provides secure communication between renderer and main
- ✅ Cannot be bypassed client-side (main process controls validation)
- ✅ Edge Config uses read-only token in connection string (safe to bundle in app)
- ⚠️ **Never commit `.env` file** - add it to `.gitignore`
- ⚠️ License stored in localStorage (user can clear it, but will revert to DEMO)

## 7. Generating License Keys

Generate random license keys for distribution:

```bash
# Simple format: XXXX-XXXX-XXXX-XXXX
openssl rand -hex 8 | tr '[:lower:]' '[:upper:]' | sed 's/\(..\)\(..\)\(..\)\(..\)/\1\2-\3\4-/'
openssl rand -hex 8 | tr '[:lower:]' '[:upper:]' | sed 's/\(..\)\(..\)\(..\)\(..\)/\1\2-\3\4-/'
```

Or use UUID:
```bash
uuidgen | tr '[:lower:]' '[:upper:]'
```

## 9. Deactivating Licenses

To revoke a license:

```bash
# Set active to false in KV
curl -X POST "https://your-kv-url/set/license:ABC-123-XYZ" \
  -H "Authorization: Bearer $KV_REST_API_TOKEN" \
  -d '{"type": "full", "active": false}'
```

## 10. Troubleshooting

**License validation fails:**
- Check `.env` file has correct KV credentials
- Verify `@vercel/kv` package is installed
- Check Electron console (main process) for errors
- Ensure license key exists in KV database

**"License verification not available" error:**
- `electronAPI.verifyLicense` not found
- Check `preload.ts` exposes the method
- Check `main.ts` has the IPC handler

**License doesn't persist:**
- Check localStorage is enabled in browser
- Verify app has write permissions

**Build limits not updating:**
- License is cached in localStorage
- Clear license and re-activate
- Check `buildLimits.ts` is reading from localStorage correctly

**KV connection errors:**
- Verify KV_REST_API_URL and KV_REST_API_TOKEN are set
- Test KV connection with `scripts/add-license.js`
