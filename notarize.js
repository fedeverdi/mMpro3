#!/usr/bin/env node

const { sign } = require('@electron/osx-sign');
const { notarize } = require('@electron/notarize');
const path = require('path');
require('dotenv').config();

const APP_PATH = path.join(__dirname, 'out/mMpro3-darwin-arm64/mMpro3.app');
const IDENTITY = 'Developer ID Application: Federico Verdi (JCVG5Y22QM)';

async function main() {
  console.log('✍️  Signing app with @electron/osx-sign...');
  
  try {
    await sign({
      app: APP_PATH,
      identity: IDENTITY,
      optionsForFile: () => {
        return {
          hardenedRuntime: true,
          entitlements: 'entitlements.mac.plist',
        };
      },
    });
    console.log('✅ Signing complete!');
    
    if (!process.env.APPLE_ID || !process.env.APPLE_ID_PASSWORD) {
      console.log('⏭️  Skipping notarization (no credentials in .env)');
      return;
    }
    
    console.log('\n☁️  Starting notarization...');
    
    await notarize({
      tool: 'notarytool',
      appPath: APP_PATH,
      appleId: process.env.APPLE_ID,
      appleIdPassword: process.env.APPLE_ID_PASSWORD,
      teamId: process.env.APPLE_TEAM_ID || 'JCVG5Y22QM',
    });
    
    console.log('✅ Notarization complete!');
    console.log('\n🎉 App is signed and notarized!');
    console.log(`\nApp ready at: ${APP_PATH}`);
    
  } catch (error) {
    console.error('❌ Error:', error.message);
    console.error(error);
    process.exit(1);
  }
}

main();
