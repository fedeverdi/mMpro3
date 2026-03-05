# GitHub Actions Setup per Build Automatiche

## Secrets da configurare su GitHub

Vai su: **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

### Required Secrets:

1. **VERCEL_BLOB_TOKEN**
   - Token per Vercel Blob Storage
   - Ottieni da: https://vercel.com/dashboard → Settings → Tokens
   - Oppure usa: `vercel env pull` per ottenere il token dal progetto

### Optional Secrets (per notarization):

2. **APPLE_ID**
   - Il tuo Apple ID email
   
3. **APPLE_ID_PASSWORD**
   - App-specific password per il tuo Apple ID
   - Genera da: https://appleid.apple.com/account/manage → App-Specific Passwords
   
4. **APPLE_TEAM_ID**
   - Il tuo Team ID Apple Developer
   - Trova qui: https://developer.apple.com/account → Membership

## Setup Vercel Blob Storage

1. Vai su https://vercel.com/dashboard
2. Vai al tuo progetto → Storage → Create Database → Blob
3. Crea un nuovo store (es: "mmpro3-builds")
4. Copia il token di read/write

## Come funziona il Workflow

Quando fai push su `dev`:

1. ✅ Compila Rust engine
2. ✅ Compila Electron app per macOS ARM64
3. ✅ Crea DMG
4. ✅ Carica ZIP e DMG su Vercel Blob
5. ✅ Commenta sul commit con i link di download

## Nome dei file

I file caricati hanno questo formato:
```
mMpro3-v{version}-build{buildNumber}-{timestamp}-arm64.{zip|dmg}
```

Esempio:
```
mMpro3-v1.1.0-build42-20260305_143022-arm64.zip
mMpro3-v1.1.0-build42-20260305_143022-arm64.dmg
```

## Trigger manuale

Puoi anche triggare la build manualmente:
1. Vai su **Actions** → **Build and Deploy to Vercel Blob (Dev)**
2. Click su **Run workflow** → seleziona branch `dev` → **Run workflow**

## Download dei file

I link di download saranno:
- `https://blob.vercel-storage.com/mMpro3-v{version}-build{build}-{timestamp}-arm64.zip`
- `https://blob.vercel-storage.com/mMpro3-v{version}-build{build}-{timestamp}-arm64.dmg`

I link saranno anche postati come commento nel commit.
