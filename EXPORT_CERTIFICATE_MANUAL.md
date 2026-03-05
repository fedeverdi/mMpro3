# Export Certificate - Metodo Manuale (più affidabile)

## 📋 Istruzioni Passo-Passo

### 1. Apri Accesso Portachiavi (Keychain Access)

- Apri **Spotlight** (Cmd+Space)
- Cerca "Accesso Portachiavi" o "Keychain Access"

### 2. Trova il certificato

- Nella barra laterale, seleziona **login** (o il keychain dove hai il certificato)
- Nella categoria, seleziona **I Miei Certificati** (My Certificates)
- Cerca: **Developer ID Application: Federico Verdi**

### 3. Esporta il certificato

- **Click destro** sul certificato
- Seleziona **Esporta "Developer ID Application: Federico Verdi..."**
- Nome file: `developer-id-cert.p12`
- Formato: **Personal Information Exchange (.p12)**
- Salva nella directory del progetto mMpro3

### 4. Proteggi con password

- Ti verrà chiesta una **password per il file .p12**
- Usa una password semplice ma sicura (es: `github-actions-2024`)
- **RICORDALA!** Ti servirà per GitHub

### 5. Converti in base64

```bash
base64 -i developer-id-cert.p12 -o certificate-base64.txt
```

### 6. Aggiungi su GitHub

**Repository → Settings → Secrets → New secret**

**Secret 1: `MACOS_CERTIFICATE`**
```bash
cat certificate-base64.txt
# Copia tutto l'output e incollalo come valore del secret
```

**Secret 2: `MACOS_CERTIFICATE_PASSWORD`**
- Valore: la password che hai usato (es: `github-actions-2024`)

### 7. Pulisci i file locali

```bash
rm developer-id-cert.p12 certificate-base64.txt
```

✅ Fatto!
