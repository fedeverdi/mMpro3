#!/bin/bash

# Build, sign and notarize the Electron app with microphone permissions
# This script automates the process of building, signing and notarizing the app

set -e

# Load environment variables from .env file
if [ -f .env ]; then
  export $(cat .env | grep -v '^#' | xargs)
else
  echo "⚠️  Warning: .env file not found. Notarization will be skipped."
  echo "   Create a .env file with APPLE_ID, APPLE_ID_PASSWORD, and APPLE_TEAM_ID"
fi

echo "🦀 Building Rust audio engine..."
cd audio-engine
cargo build --release
cd ..

echo "🔨 Building app..."
npm run package

APP_PATH="out/mMpro3-darwin-arm64/mMpro3.app"
ZIP_PATH="out/mMpro3-darwin-arm64/mMpro3.zip"

echo "🧹 Removing quarantine attributes..."
xattr -cr "$APP_PATH"

echo "✍️  Signing all binaries with Developer ID..."

# Sign the Rust audio engine first
echo "  → Signing mmpro3-engine..."
codesign --force --sign "Developer ID Application: Federico Verdi (JCVG5Y22QM)" \
  --options runtime \
  --timestamp \
  "$APP_PATH/Contents/Resources/mmpro3-engine"

# Sign all frameworks and libraries inside the app
echo "  → Signing Electron frameworks and libraries..."
find "$APP_PATH/Contents/Frameworks" -type f \( -name "*.dylib" -o -perm +111 \) | while read file; do
  echo "    • $(basename "$file")"
  codesign --force --sign "Developer ID Application: Federico Verdi (JCVG5Y22QM)" \
    --options runtime \
    --timestamp \
    "$file" 2>/dev/null || true
done

# Sign all frameworks
find "$APP_PATH/Contents/Frameworks" -type d -name "*.framework" | while read framework; do
  echo "    • $(basename "$framework")"
  codesign --force --sign "Developer ID Application: Federico Verdi (JCVG5Y22QM)" \
    --options runtime \
    --timestamp \
    "$framework" 2>/dev/null || true
done

# Sign the main app bundle with entitlements
echo "  → Signing main app bundle..."
codesign --force --sign "Developer ID Application: Federico Verdi (JCVG5Y22QM)" \
  --options runtime \
  --entitlements entitlements.mac.plist \
  --timestamp \
  "$APP_PATH"

echo "✅ Verifying signature..."
codesign --verify --verbose "$APP_PATH"
spctl --assess --verbose "$APP_PATH"

# Notarization
if [ -n "$APPLE_ID" ] && [ -n "$APPLE_ID_PASSWORD" ] && [ -n "$APPLE_TEAM_ID" ]; then
  echo ""
  echo "☁️  Starting notarization process..."
  
  # Create ZIP for notarization
  echo "📦 Creating ZIP archive..."
  ditto -c -k --keepParent "$APP_PATH" "$ZIP_PATH"
  
  # Submit for notarization
  echo "⬆️  Uploading to Apple for notarization..."
  xcrun notarytool submit "$ZIP_PATH" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait
  
  # Staple the notarization ticket
  echo "📎 Stapling notarization ticket..."
  xcrun stapler staple "$APP_PATH"
  
  # Verify notarization
  echo "✅ Verifying notarization..."
  xcrun stapler validate "$APP_PATH"
  spctl --assess -vv --type install "$APP_PATH"
  
  # Clean up ZIP
  rm "$ZIP_PATH"
  
  echo ""
  echo "🎉 App notarized successfully!"
else
  echo ""
  echo "⏭️  Skipping notarization (credentials not found in .env)"
fi

echo ""
echo "✅ Build complete! App ready at: $APP_PATH"
echo ""
echo "To open the app, run:"
echo "  open $APP_PATH"
