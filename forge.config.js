const { FusesPlugin } = require('@electron-forge/plugin-fuses');
const { FuseV1Options, FuseVersion } = require('@electron/fuses');
const fs = require('fs');
const path = require('path');

module.exports = {
  packagerConfig: {
    asar: true,
    appBundleId: 'com.federicoverdi.mMpro3',
    appCategoryType: 'public.app-category.music',
    icon: './src/assets/macos/icon',
    darwinDarkModeSupport: true,
    osxSign: false,
    osxNotarize: false,
    extendInfo: {
      NSMicrophoneUsageDescription: 'This application needs access to the microphone to record and process audio.',
      NSCameraUsageDescription: 'This application does not use the camera.',
      LSMinimumSystemVersion: '10.15.0',
      'com.apple.security.device.audio-input': true,
      CFBundleDocumentTypes: []
    },
    extraResource: ['splash.html', 'logo.svg'],
    afterCopy: [
      (buildPath, electronVersion, platform, arch, callback) => {
        console.log('[Packager] Running afterCopy hook...');
        console.log('[Packager] Build path:', buildPath);
        
        // Copy the audio engine binary to the Resources folder
        const binaryName = platform === 'win32' ? 'mmpro3-engine.exe' : 'mmpro3-engine';
        const sourcePath = path.join(__dirname, 'audio-engine', 'target', 'release', binaryName);
        const destPath = path.join(buildPath, '..', binaryName);
        
        console.log('[Packager] Copying audio engine from:', sourcePath);
        console.log('[Packager] To:', destPath);
        
        if (fs.existsSync(sourcePath)) {
          try {
            fs.copyFileSync(sourcePath, destPath);
            
            // Make executable on Unix systems
            if (platform !== 'win32') {
              fs.chmodSync(destPath, 0o755);
            }
            
            console.log('[Packager] ✅ Audio engine copied and made executable');
          } catch (error) {
            console.error('[Packager] ❌ Failed to copy audio engine:', error);
            callback(error);
            return;
          }
        } else {
          console.error('[Packager] ❌ Audio engine not found at:', sourcePath);
          console.error('[Packager] Make sure to build the Rust engine first: cd audio-engine && cargo build --release');
          callback(new Error('Audio engine binary not found'));
          return;
        }
        
        // Copy NDI library on macOS
        if (platform === 'darwin') {
          const ndiLibSource = path.join(__dirname, 'native-libs', 'macos', 'libndi.dylib');
          const ndiLibDest = path.join(buildPath, '..', 'libndi.dylib');
          
          console.log('[Packager] Copying NDI library from:', ndiLibSource);
          console.log('[Packager] To:', ndiLibDest);
          
          if (fs.existsSync(ndiLibSource)) {
            try {
              fs.copyFileSync(ndiLibSource, ndiLibDest);
              console.log('[Packager] ✅ NDI library copied');
            } catch (error) {
              console.error('[Packager] ⚠ Failed to copy NDI library:', error);
              console.error('[Packager] App will fallback to system NDI installation');
            }
          } else {
            console.log('[Packager] ⚠ NDI library not found at:', ndiLibSource);
            console.log('[Packager] App will use system NDI installation if available');
          }
        }
        
        callback();
      }
    ]
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        setupIcon: './src/assets/windows/icon.ico',
        iconUrl: 'https://raw.githubusercontent.com/federicoverdi/MMpro3/main/src/assets/windows/icon.ico'
      },
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin', 'win32'],
    },
    {
      name: '@electron-forge/maker-deb',
      config: {
        options: {
          icon: './src/assets/linux/icons/512x512.png'
        }
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {
        options: {
          icon: './src/assets/linux/icons/512x512.png'
        }
      },
    },
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-vite',
      config: {
        // `build` can specify multiple entry builds, which can be Main process, Preload scripts, Worker process, etc.
        // If you are familiar with Vite configuration, it will look really familiar.
        build: [
          {
            // `entry` is just an alias for `build.lib.entry` in the corresponding file of `config`.
            entry: 'src/main.ts',
            config: 'vite.main.config.mjs',
            target: 'main',
          },
          {
            entry: 'src/preload.ts',
            config: 'vite.preload.config.mjs',
            target: 'preload',
          },
        ],
        renderer: [
          {
            name: 'main_window',
            config: 'vite.renderer.config.mjs',
          },
        ],
      },
    },
    // Fuses are used to enable/disable various Electron functionality
    // at package time, before code signing the application
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: false,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: false,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};
