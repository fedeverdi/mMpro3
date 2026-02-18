# MMpro3 - Build Notes

## Microphone Access on macOS

L'applicazione richiede l'accesso al microfono per funzionare. Su macOS, questo richiede configurazioni specifiche:

### Configurazione Necessaria

1. **Bundle Identifier**: L'app deve avere un bundle ID univoco (`com.federicoverdi.mMpro3`)

2. **Info.plist**: Deve includere la descrizione per l'uso del microfono:
   ```xml
   <key>NSMicrophoneUsageDescription</key>
   <string>This application needs access to the microphone to record and process audio.</string>
   ```

3. **Entitlements**: L'app deve essere firmata con gli entitlements corretti per l'accesso ai dispositivi audio (`entitlements.plist`)

### Build dell'Applicazione

Per buildare l'app correttamente con tutti i permessi:

```bash
npm run build
```

Questo comando:
- Builda l'app con `electron-forge package`
- Rimuove gli attributi di quarantena di macOS
- Firma l'app con gli entitlements per il microfono (firma ad-hoc per sviluppo)

L'app sarà disponibile in: `out/mMpro3-darwin-arm64/mMpro3.app`

### Sviluppo

Durante lo sviluppo, usa:

```bash
npm run start
```

In modalità development, i permessi del microfono funzionano automaticamente.

### Prima Esecuzione

Al primo avvio, macOS chiederà all'utente il permesso di accedere al microfono. L'app apparirà in:
**Impostazioni di Sistema → Privacy e Sicurezza → Microfono**

### Reset Permessi (per test)

```bash
tccutil reset Microphone com.federicoverdi.mMpro3
```

## Files Chiave

- `forge.config.js` - Configurazione Electron Forge con bundle ID e Info.plist
- `entitlements.plist` - Entitlements per l'accesso al microfono
- `build-and-sign.sh` - Script automatico di build e firma
- `src/main.ts` - Configurazione permessi Electron per getUserMedia
