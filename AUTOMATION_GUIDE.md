# Sistema di Automazione - mMpro3

## ✨ Panoramica

Il sistema di automazione permette di registrare e riprodurre parametri che cambiano nel tempo, trasformando il mixer da statico a dinamico.

## 🎛️ Componenti Principali

### 1. **Timeline** (Barra di Trasporto)
Posizionata in fondo all'interfaccia, contiene:
- **Play/Pause/Stop**: Controlli di trasporto principali
- **Record**: Attiva/disattiva modalità registrazione
- **Time Display**: Mostra posizione corrente e durata totale
- **BPM**: Tempo del progetto (20-300 BPM)
- **Time Signature**: Indicazione tempo (es. 4/4)
- **Loop**: Attiva/disattiva riproduzione ciclica
- **Ruler**: Timeline cliccabile con playhead (linea teal)

### 2. **Automation Lanes**
Visualizzazione grafica dell'automazione per ogni parametro:
- **Automation Points**: Punti editabili sulla curva
- **Curve**: Linea che connette i punti (interpolazione)
- **Mode Selector**: OFF/READ/WRITE/TOUCH/LATCH

## 🎚️ Parametri Automatizzabili

### Implementati:
- ✅ **Volume** (fader principale di ogni traccia)
- ✅ **Pan** (bilanciamento stereo)
- ✅ **Aux Sends** (livelli di invio agli aux)

### Espandibili in futuro:
- 🔜 **Mute** (automazione on/off)
- 🔜 **Plugin Parameters** (EQ, Compressor, FX)
- 🔜 **Master Volume**

## 🎬 Modi di Automazione

### **OFF**
Automazione disabilitata. Il parametro è controllato manualmente.

### **READ** ⏯️
- Riproduce l'automazione registrata
- I controlli manuali sono temporaneamente ignorati
- Il parametro segue la curva di automazione

### **WRITE** ⏺️
- Sovrascrive completamente l'automazione esistente
- Ogni movimento viene registrato
- Perfetto per rifare completamente una automazione

### **TOUCH** ✋ (In Sviluppo)
- Registra solo quando tocchi il controllo
- Appena rilasci, torna alla curva esistente
- Utile per correzioni puntuali

### **LATCH** 🔒 (In Sviluppo)
- Come TOUCH ma mantiene l'ultimo valore dopo il rilascio
- Non torna alla curva precedente
- Utile per cambiamenti permanenti da un certo punto

## 🎯 Workflow Tipico

### Registrare Automazione Volume:

1. **Setup**:
   - Carica audio su una traccia
   - Imposta BPM e durata progetto (Timeline)

2. **Registrazione**:
   - Clicca **RECORD** (diventa rosso e pulsa)
   - Clicca **PLAY**
   - Muovi il fader del volume mentre l'audio suona
   - I punti di automazione vengono creati automaticamente
   - Clicca **STOP** quando finito

3. **Editing**:
   - Apri la Automation Lane della traccia (prossimamente)
   - **Double-click** sulla linea per **aggiungere** un punto
   - **Double-click** su un punto per **rimuoverlo**
   - **Drag** un punto per spostarlo
   - **Delete/Backspace** con punto selezionato lo elimina

4. **Playback**:
   - Imposta mode su **READ**
   - Clicca **PLAY**
   - L'automazione viene applicata automaticamente

## 📋 Gestione Curve

### Tipi di Interpolazione:

- **Linear** (default): Transizione lineare tra punti
- **Exponential**: Curva esponenziale (utile per fade out)
- **Logarithmic**: Curva logaritmica (utile per fade in)
- **Step**: Nessuna interpolazione, mantiene valore fino al punto successivo

### Editing Points:

- **Click** su punto: Seleziona (diventa arancione)
- **Drag** punto: Sposta tempo e valore
- **Double-click** spazio vuoto: Aggiungi punto
- **Double-click** punto: Rimuovi punto
- **Delete/Backspace**: Rimuovi punto selezionato

## 💾 Scene e Automazione

### Salvataggio:
Quando salvi una scena, l'automazione viene salvata automaticamente:
- Tutte le lanes
- Tutti i punti di automazione
- Transport settings (BPM, time signature)

### Caricamento:
Quando carichi una scena:
- L'automazione viene ripristinata completamente
- Se la scena non ha automazione, quella esistente viene cancellata
- Le lanes vengono ricreate con i parametri salvati

## ⚙️ Struttura Dati

### AutomationPoint
```typescript
{
  time: number      // in secondi
  value: number     // valore del parametro
  curve?: 'linear' | 'exponential' | 'logarithmic' | 'step'
}
```

### AutomationLane
```typescript
{
  trackId: number
  parameter: 'volume' | 'pan' | 'auxSend'
  auxId?: string        // Solo per aux sends
  points: AutomationPoint[]
  enabled: boolean      // READ mode attivo
  mode: 'off' | 'read' | 'write' | 'touch' | 'latch'
}
```

## 🚀 Prossimi Passi

### In Sviluppo:
1. **UI Integration nelle Tracce**
   - Bottone "A" su ogni traccia per mostrare/nascondere lanes
   - Visualizzazione lanes sotto ogni fader
   - Selezione parametro da automatizzare

2. **Touch & Latch Modes**
   - Implementazione logica touch/release
   - Smart recording con threshold

3. **Automation Thinning**
   - Riduzione automatica punti ridondanti
   - Ottimizzazione performance

4. **Curve Editor Avanzato**
   - Bezier curves
   - Curve presets
   - Multi-point editing

### Espansioni Future:
- 📊 **Visualizzazione waveform** nella timeline
- 🎵 **Snap to grid** (musical timing)
- 📍 **Markers e regioni**
- 🎹 **MIDI automation recording**
- 💫 **Automation effects** (LFO, step sequencer)
- 📤 **Export automation** come MIDI CC

## 🐛 Troubleshooting

### "L'automazione non viene applicata"
✅ Verifica che il mode sia su **READ**
✅ Controlla che ci siano punti di automazione (almeno 2)
✅ Assicurati che `enabled: true` sulla lane

### "Non riesco a registrare"
✅ Attiva **RECORD** prima di **PLAY**
✅ Verifica che il parametro supporti l'automazione
✅ Muovi il controllo mentre sta registrando

### "Il playhead non si muove"
✅ Controlla che il transport sia in **PLAY**
✅ Verifica che la durata del progetto sia > 0
✅ Controlla la console per errori JavaScript

## 📚 API Reference

### Composable: useAutomation()

```typescript
// Gestione Points
addPoint(trackId, parameter, time, value, auxId?, curve?)
removePoint(trackId, parameter, pointIndex, auxId?)
getValueAtTime(trackId, parameter, time, auxId?)

// Gestione Lanes
getAutomationLane(trackId, parameter, auxId?)
getOrCreateLane(trackId, parameter, auxId?)
setMode(trackId, parameter, mode, auxId?)
clearLane(trackId, parameter, auxId?)

// Transport
play()
pause()
stop()
seek(time)

// Recording
startRecording(trackId, parameter, auxId?)
stopRecording(trackId?, parameter?, auxId?)
isLaneRecording(trackId, parameter, auxId?)

// Import/Export
exportAutomation()  // Per scene
importAutomation(data)
clearAll()
```

---

**Versione**: 1.0.0  
**Data**: Febbraio 2026  
**Sistema**: mMpro3 Audio Mixer
