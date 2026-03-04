#!/bin/bash

# Create, sign and notarize DMG for distribution

set -e

# Load environment variables
if [ -f .env ]; then
  export $(cat .env | grep -v '^#' | xargs)
fi

APP_NAME="mMpro3"
APP_PATH="out/mMpro3-darwin-arm64/mMpro3.app"
DMG_PATH="out/mMpro3-darwin-arm64/${APP_NAME}.dmg"
IDENTITY="Developer ID Application: Federico Verdi (JCVG5Y22QM)"

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
  echo "❌ Error: App not found at $APP_PATH"
  echo "   Run 'npm run pack:full' first"
  exit 1
fi

echo "📦 Creating DMG..."
rm -f "$DMG_PATH"

create-dmg \
  --volname "${APP_NAME}" \
  --volicon "src/assets/macos/icon.icns" \
  --window-pos 200 120 \
  --window-size 600 300 \
  --icon-size 130 \
  --icon "${APP_NAME}.app" 175 120 \
  --hide-extension "${APP_NAME}.app" \
  --app-drop-link 425 120 \
  --no-internet-enable \
  "$DMG_PATH" \
  "$APP_PATH"

echo "✍️  Signing DMG..."
codesign --sign "$IDENTITY" --timestamp "$DMG_PATH"

echo "✅ DMG created and signed!"

# Notarize DMG if credentials are available
if [ -n "$APPLE_ID" ] && [ -n "$APPLE_ID_PASSWORD" ] && [ -n "$APPLE_TEAM_ID" ]; then
  echo ""
  echo "☁️  Notarizing DMG..."
  
  xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait
  
  echo "📎 Stapling notarization ticket to DMG..."
  xcrun stapler staple "$DMG_PATH"
  
  echo "✅ DMG notarized!"
else
  echo ""
  echo "⏭️  Skipping DMG notarization (no credentials in .env)"
fi

echo ""
echo "🎉 DMG ready at: $DMG_PATH"
echo ""
echo "To test: open $DMG_PATH"
