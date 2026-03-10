#!/bin/bash

# Build, sign and notarize the Electron app using official Electron tools

set -e

echo "🦀 Building Rust audio engine..."
cd audio-engine
cargo build --release
cd ..
echo "🌐 Building web interface..."
npm run build:web
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
