#!/bin/bash

# Script to completely reset mMpro3 app state
# This simulates a fresh install on a new device

set -e

echo "🧹 Resetting mMpro3 app state..."
echo ""

# 1. Chiudi l'app
echo "→ Closing app..."
pkill -9 mMpro3 2>/dev/null || true

# 2. Rimuovi l'app da Applications
echo "→ Removing app from /Applications..."
sudo rm -rf /Applications/mMpro3.app

# 3. Rimuovi tutte le preferenze e cache
echo "→ Removing preferences and cache..."
rm -rf ~/Library/Application\ Support/mMpro3
rm -rf ~/Library/Preferences/com.federicoverdi.mMpro3.*
rm -rf ~/Library/Caches/com.federicoverdi.mMpro3
rm -rf ~/Library/Saved\ Application\ State/com.federicoverdi.mMpro3.*
rm -rf ~/Library/HTTPStorages/com.federicoverdi.mMpro3
rm -rf ~/Library/WebKit/com.federicoverdi.mMpro3

# 4. Reset permessi (microfono, camera, ecc.)
echo "→ Resetting security permissions..."
tccutil reset Microphone com.federicoverdi.mMpro3 2>/dev/null || true
tccutil reset Camera com.federicoverdi.mMpro3 2>/dev/null || true

# 5. Pulisci Launch Services cache
echo "→ Clearing Launch Services cache..."
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -kill -r -domain local -domain system -domain user >/dev/null 2>&1

echo ""
echo "✅ Reset complete!"
echo ""
echo "Now reinstall the app:"
echo "  cp -R out/mMpro3-darwin-arm64/mMpro3.app /Applications/"
echo "  open /Applications/mMpro3.app"
echo ""
