#!/bin/bash

# Build, sign and notarize the Electron app using official Electron tools

set -e

echo "🦀 Building Rust audio engine..."
cd audio-engine
cargo build --release
cd ..


echo "�🔨 Building app (without signing)..."
npm run package

echo ""
echo "✍️  Signing and notarizing with Electron tools..."
node notarize.js

echo ""
echo "✅ Build complete!"
echo ""
echo "To test the app:"
echo "  open out/mMpro3-darwin-arm64/mMpro3.app"
echo ""

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
