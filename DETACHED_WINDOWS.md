# Sistema di Finestre Pop-Out (Detached Windows)

## Panoramica

Il sistema di finestre pop-out permette di "staccare" alcuni componenti del mixer (Master EQ, Spectrum, Aux Master, Master FX) in finestre separate di Electron. Queste finestre comunicano con il processo principale tramite WebSocket, sfruttando l'infrastruttura già esistente per i remote front.

## Architettura

### Differenza tra Remote Front e Detached Windows

- **Remote Front Clients**: 
  - Sono client browser esterni (web)
  - BLOCCANO il mixer principale di Electron quando attivi
  - Richiedono conferma prima di prendere il controllo
  - Gestiti dal sistema di "active remote clients"

- **Detached Window Clients**:
  - Sono finestre Electron separate dall'app principale
  - NON BLOCCANO il mixer principale
  - Ricevono aggiornamenti in tempo reale via WebSocket
  - Possono essere multipli contemporaneamente
  - Sono "read-only observers" degli stati del mixer

### Componenti del Sistema

1. **Server WebSocket** (`src/main.ts`)
   - Porta 3001
   - Distingue tra remote front clients e detached window clients
   - Broadcast degli aggiornamenti a tutti i client connessi

2. **Client WebSocket** (`src/lib/detachedWindowClient.ts`)
   - Si registra come "detached-window" invece di "remote-control"
   - Event-driven: usa listener per reagire agli update
   - Auto-reconnect in caso di disconnessione

3. **Entry Points HTML** (root directory)
   - `detached-master-eq.html`
   - `detached-spectrum.html`
   - `detached-aux-master.html`
   - `detached-master-fx.html`

4. **Script TypeScript** (`src/detached/`)
   - `master-eq.ts`
   - `spectrum.ts`
   - `aux-master.ts`
   - `master-fx.ts`

5. **Componenti Vue Wrapper** (`src/detached/components/`)
   - `DetachedMasterEQ.vue`
   - `DetachedSpectrum.vue`
   - `DetachedAuxMaster.vue`
   - `DetachedMasterFX.vue`

## Flusso di Comunicazione

```
┌─────────────────┐
│  Main Window    │
│  (Electron)     │
└────────┬────────┘
         │
         │ IPC: open-detached-window
         ▼
┌─────────────────┐
│   Main Process  │◄────┐
│   (main.ts)     │     │
└────────┬────────┘     │
         │              │
         │ Opens        │ Audio Engine
         ▼              │ Updates
┌─────────────────┐     │
│ Detached Window │     │
│   (Electron)    │     │
└────────┬────────┘     │
         │              │
         │ WebSocket    │
         │ Registration │
         ▼              │
┌─────────────────┐     │
│  WS Server      │◄────┘
│  (port 3001)    │
└─────────────────┘
```

## Utilizzo

### Per l'Utente

1. Clicca sull'icona "pop-out" (↗) nell'header di un componente nella RightSection
2. Si apre una finestra separata con il componente
3. La finestra riceve aggiornamenti in tempo reale dal mixer
4. Puoi chiudere la finestra in qualsiasi momento

### Per lo Sviluppatore

#### Aprire una finestra detached

```typescript
// Nel componente Vue
await window.electronAPI.openDetachedWindow('master-eq')
```

Tipi supportati:
- `'master-eq'`
- `'spectrum'`
- `'aux-master'`
- `'master-fx'`

#### Verificare se una finestra è aperta

```typescript
const isOpen = await window.electronAPI.isDetachedWindowOpen('master-eq')
```

#### Chiudere una finestra programmaticamente

```typescript
await window.electronAPI.closeDetachedWindow('master-eq')
```

## Registro Messaggi WebSocket

### Messaggi dal Client al Server

- `detached-window-register`: Registra la finestra come detached client
  ```json
  {
    "type": "detached-window-register",
    "componentType": "master-eq"
  }
  ```

### Messaggi dal Server al Client

- `detached-window-registered`: Conferma registrazione
- `parameters_changed`: Aggiornamento parametri
- `levels`: Aggiornamento livelli audio
- `started` / `stopped`: Stato audio engine

## File Modificati/Creati

### Modificati
- `src/main.ts`: Aggiunto tracking detached clients, funzioni gestione finestre
- `src/preload.ts`: Esposte nuove API IPC
- `src/env.d.ts`: Definizioni TypeScript nuove API
- `src/components/master/RightSection.vue`: Aggiunti pulsanti pop-out
- `vite.renderer.config.mjs`: Configurato multi-page build

### Creati
- `src/lib/detachedWindowClient.ts`: Client WebSocket per finestre detached
- `detached-master-eq.html`, `detached-spectrum.html`, etc.: Entry points HTML
- `src/detached/master-eq.ts`, etc.: Script entry per Vue apps
- `src/detached/components/DetachedMasterEQ.vue`, etc.: Wrapper components

## Note Tecniche

### WebSocket vs IPC

Perché WebSocket invece di IPC per le finestre Electron?

1. **Coerenza**: Usa la stessa infrastruttura dei remote front
2. **Flessibilità**: Possibile estendere a veri client web esterni in futuro
3. **Broadcasting**: Facile inviare update a tutti i client contemporaneamente
4. **Separazione**: Le finestre detached sono trattate come "observer" invece di controller diretti

### Performance

- Gli update sono throttled dal sistema esistente (~60fps per controlli continui)
- Le finestre detached ricevono solo gli update necessari
- Il broadcast WebSocket ha overhead minimo

### Sviluppo

In development mode (Vite dev server), tutte le pagine HTML sono servite automaticamente:
- `http://localhost:5173/` - Main window
- `http://localhost:5173/detached-master-eq.html` - Master EQ detached
- etc.

In production, i file sono nel bundle e caricati da file system.

## Possibili Estensioni Future

1. **Persistenza posizioni**: Salvare posizione/dimensione finestre
2. **Multi-monitor support**: Ottimizzazione per setup multi-monitor
3. **Enable/disable interactivity**: Toggle tra read-only e interactive mode
4. **Temi personalizzati**: Temi diversi per finestre detached
5. **Hotkeys**: Shortcut tastiera per aprire/chiudere finestre
