#!/bin/bash

# Script per esportare il certificato di firma da usare su GitHub Actions

set -e

echo "🔐 Esportazione certificato Developer ID Application"
echo ""

OUTPUT_FILE="developer-id-cert.p12"

# Lista i certificati disponibili
echo "Certificati disponibili:"
security find-identity -v -p codesigning
echo ""

# Prendi il primo certificato Developer ID Application
CERT_HASH=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | awk '{print $2}')

if [ -z "$CERT_HASH" ]; then
    echo "❌ Nessun certificato 'Developer ID Application' trovato!"
    exit 1
fi

echo "✅ Certificato trovato: $CERT_HASH"
echo ""
echo "📝 Inserisci una password per proteggere il file .p12"
echo "   Ricordati questa password, ti servirà per GitHub Actions!"
echo ""
read -s -p "Password (es: github-actions-2024): " CERT_PASSWORD
echo ""
read -s -p "Conferma password: " CERT_PASSWORD_CONFIRM
echo ""

if [ "$CERT_PASSWORD" != "$CERT_PASSWORD_CONFIRM" ]; then
    echo "❌ Le password non corrispondono!"
    exit 1
fi

echo ""
echo "📦 Esportando il certificato..."
echo "   (ti potrebbe essere chiesta la password del Keychain)"
echo ""

# Esporta usando l'hash del certificato
security find-identity -v -p codesigning | grep "$CERT_HASH" | awk -F'"' '{print $2}' | \
while read cert_name; do
    security export -k login.keychain-db \
        -t identities \
        -f pkcs12 \
        -o "$OUTPUT_FILE" \
        -P "$CERT_PASSWORD" \
        "$cert_name"
done

if [ ! -f "$OUTPUT_FILE" ]; then
    echo "❌ Errore nell'esportazione!"
    echo ""
    echo "Prova manualmente:"
    echo "1. Apri 'Accesso Portachiavi' (Keychain Access)"
    echo "2. Cerca 'Developer ID Application'"
    echo "3. Click destro → Esporta"
    echo "4. Salva come developer-id-cert.p12"
    exit 1
fi

echo "✅ Certificato esportato in: $OUTPUT_FILE"
echo ""
echo "🔐 Convertendo in base64..."

BASE64_CERT=$(base64 -i "$OUTPUT_FILE")

echo ""
echo "✅ Fatto!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 COPIA QUESTI VALORI NEI GITHUB SECRETS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Secret name: MACOS_CERTIFICATE"
echo "   Value: (salva il valore nel file certificate-base64.txt)"
echo "$BASE64_CERT" > certificate-base64.txt
echo "   ✅ Salvato in: certificate-base64.txt"
echo ""
echo "2. Secret name: MACOS_CERTIFICATE_PASSWORD"
echo "   Value: $CERT_PASSWORD"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "⚠️  ATTENZIONE:"
echo "   - NON committare questi file su Git!"
echo "   - Copia i valori su GitHub → Settings → Secrets"
echo "   - Poi elimina i file: rm $OUTPUT_FILE certificate-base64.txt"
echo ""
