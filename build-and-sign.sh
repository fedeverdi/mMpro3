#!/bin/bash

# Build and sign the Electron app with microphone permissions
# This script automates the process of building and signing the app with proper entitlements

set -e

echo "🔨 Building app..."
npm run package

echo "🧹 Removing quarantine attributes..."
xattr -cr out/mMpro3-darwin-arm64/mMpro3.app

echo "✍️  Signing app with entitlements..."
codesign --force --deep --sign - --entitlements entitlements.plist out/mMpro3-darwin-arm64/mMpro3.app

echo "✅ Build complete! App ready at: out/mMpro3-darwin-arm64/mMpro3.app"
echo ""
echo "To open the app, run:"
echo "  open out/mMpro3-darwin-arm64/mMpro3.app"
