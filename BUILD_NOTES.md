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

## Audio Signal Flow

### Audio Track Chain

```
Audio Source (File/Mic)
  ↓
Gain Node (Input Level)
  ↓
EQ3 (3-band EQ: Low, Mid, High)
  ├─→ VU Meters (TAP - Left/Right analysis)
  └─→ Waveform Display (TAP - Visual representation)
  ↓
Compressor (If enabled via track controls)
  ↓
Reverb (Per-track reverb, Wet=0 when off)
  ↓
Balance Control (Split → L/R Gain → Merge)
  ↓
Volume Fader (Split → L/R Gain → Merge)
  ↓
Master Channel (Stereo bus - all tracks mixed)
```

### Master Bus Chain

```
Master Channel (Tone.Channel - receives all tracks)
  ↓
MasterEQDisplay Component
  • 5-band Parametric EQ
  • Creates outputNode (Tone.Gain)
  • masterChannel → outputNode connection
  ↓
MasterFX Component (Input/Output Architecture)
  • inputNode = MasterEQDisplay.outputNode
  • FX Chain (if enabled):
    - Compressor
    - Reverb  
    - Delay
    - Limiter
  • outputNode = Tone.Gain
  • Signal flow: inputNode → [FX Chain] → outputNode
  ↓
MasterFX outputNode connects to TWO destinations:
  ├─→ SpectrumMeter (TAP - POST-FX visualization)
  └─→ MasterSection (Audio output processing)
       ↓
     Split L/R (Tone.Split)
       ├─→ Left Channel
       │    ├─→ L Gain (Master fader)
       │    └─→ L Meter (VU meter)
       └─→ Right Channel
            ├─→ R Gain (Master fader)
            └─→ R Meter (VU meter)
       ↓
     Merge (Tone.Merge - recombines L/R)
       ├─→ Main Output
       │    └─→ toDestination() → Speakers
       └─→ Headphones Output
            ├─→ HP Gain
            ├─→ HP Meter  
            ├─→ MediaStreamDestination
            └─→ Headphone Device
```

### Key Architecture Notes

1. **Analysis Taps**: VU meters, waveform displays, and spectrum analyzer use non-blocking tap connections that don't affect the main audio signal flow.

2. **MasterFX Input/Output Pattern**: 
   - MasterFX is a self-contained processing unit
   - Receives input from MasterEQDisplay.outputNode
   - Processes through FX chain (if enabled)
   - Outputs to multiple destinations (Spectrum + MasterSection)
   - Important: MasterSection does NOT disconnect FX output (allows multiple connections)

3. **Spectrum Post-FX**: The spectrum analyzer connects to MasterFX.outputNode, showing the frequency content AFTER effects processing.

4. **Stereo Preservation**: All processing maintains stereo signal integrity using Split/Merge patterns for L/R channel operations.

5. **Effects Chain Order**:
   - Track: EQ3 → Compressor → Reverb → Balance → Volume
   - Master: Parametric EQ → Compressor → Reverb → Delay → Limiter

## Files Chiave

- `forge.config.js` - Configurazione Electron Forge con bundle ID e Info.plist
- `entitlements.plist` - Entitlements per l'accesso al microfono
- `build-and-sign.sh` - Script automatico di build e firma
- `src/main.ts` - Configurazione permessi Electron per getUserMedia
- `src/components/MasterEQDisplay.vue` - Parametric EQ with outputNode creation
- `src/components/MasterFX.vue` - Master effects processor (input/output architecture)
- `src/components/MasterSection.vue` - Final master fader and output routing
- `src/components/SpectrumMeter.vue` - Frequency spectrum analyzer (post-FX)
