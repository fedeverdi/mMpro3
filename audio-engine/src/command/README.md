# Command Handlers

Questa cartella contiene i gestori dei comandi organizzati per funzionalità.
Ogni file implementa metodi sull'`AudioEngine` per un gruppo specifico di comandi.

## Struttura

- **stream.rs** - Gestione audio stream (Start, Stop, EnableMasterTap, DisableMasterTap)  
- **config.rs** - Configurazione e licenze (Save/Get License, Save/Get AudioConfig)
- **file_playback.rs** - Riproduzione file (Play, Pause, Stop, Seek, Waveform)
- **audio_input.rs** - Input audio (Open, Close, List inputs)
- **metering.rs** - Metering (Loudness, Dynamic Range, Phase, Stereo Width, Headroom)
- **ndi.rs** - Streaming NDI (Start, Stop, Set Source/Name/Text)

## Come aggiungere nuovi handler

1. Crea un nuovo file `categoria.rs`
2. Implementa i metodi con `impl AudioEngine { pub fn handle_categoria_command(...) }`
3. Aggiungi il modulo in `mod.rs`: `pub mod categoria;`
4. Aggiungi il dispatcher in `audio_engine.rs` handle_command()

## Note

- I metodi sono `pub` o `pub(crate)` perché implementano AudioEngine da un altro modulo
- Ogni handler riceve `Command` ed restituisce `Option<Response>`
- Il pattern è: match su un gruppo di comandi correlati, restituire None se non match
