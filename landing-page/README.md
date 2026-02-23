# MMpro3 Landing Page

Landing page moderna e professionale per MMpro3 Audio Mixer.

## 🚀 Caratteristiche

- ✅ Design moderno con gradients e animazioni
- ✅ Completamente responsive (mobile, tablet, desktop)
- ✅ Tailwind CSS via CDN (nessuna build richiesta)
- ✅ Smooth scroll navigation
- ✅ Sezioni: Hero, Features, Pricing, Download, Footer

## 📦 Utilizzo

Apri semplicemente `index.html` nel browser. Non è richiesta alcuna build o installazione.

```bash
# Opzione 1: Apri direttamente il file
open index.html

# Opzione 2: Usa un server locale
npx serve .

# Opzione 3: Python server
python3 -m http.server 8000
```

## 🎨 Personalizzazione

### Colori
I colori principali sono definiti nella configurazione Tailwind:
- `primary`: #10b981 (verde)
- `secondary`: #3b82f6 (blu)

Modifica questi valori nello script tag per cambiare il tema.

### Testi
Tutti i testi sono modificabili direttamente nell'HTML. Le sezioni principali sono:
- Hero Section (linea 53)
- Features (linea 140)
- Pricing (linea 246)
- Download (linea 402)

### Link Download
I pulsanti download sono attualmente placeholder. Aggiorna gli href con i link reali:
```html
<button onclick="window.location.href='LINK_DOWNLOAD_WINDOWS'">
```

## 📱 Sezioni

### Hero
- Introduzione al prodotto
- Call-to-action principale
- Statistiche chiave
- Animazione floating

### Features
6 funzionalità principali:
1. Tracce illimitate
2. Effetti premium
3. Automazione avanzata
4. Aux Bus & Routing
5. Registrazione diretta
6. Scene System

### Pricing
3 piani:
- **Demo**: Gratuita (8 tracce, effetti base)
- **Professional**: €49 (tutto illimitato)
- **Studio**: €299 (multi-licenza, sync cloud)

### Download
- Supporto Windows, macOS, Linux
- Requisiti di sistema
- Info versione demo

## 🛠️ Deploy

### Netlify
```bash
# Drag & drop della cartella landing-page su netlify.com
```

### Vercel
```bash
vercel --prod
```

### GitHub Pages
```bash
git subtree push --prefix landing-page origin gh-pages
```

## 📄 Licenza

Copyright 2026 MMpro3
