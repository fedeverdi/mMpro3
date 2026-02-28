const { FusesPlugin } = require('@electron-forge/plugin-fuses');
const { FuseV1Options, FuseVersion } = require('@electron/fuses');
const fs = require('fs');
const path = require('path');

module.exports = {
  packagerConfig: {
    asar: true,
    appBundleId: 'com.federicoverdi.mMpro3',
    appCategoryType: 'public.app-category.music',
    extendInfo: {
      NSMicrophoneUsageDescription: 'This application needs access to the microphone to record and process audio.',
      NSCameraUsageDescription: 'This application does not use the camera.',
      LSMinimumSystemVersion: '10.15.0',
      'com.apple.security.device.audio-input': true
    },
    extraResource: [
      'audio-engine/target/release/mmpro3-engine'
    ],
    afterCopy: [
      (buildPath, electronVersion, platform, arch, callback) => {
        // Make the audio engine executable on Unix systems
        const binaryName = platform === 'win32' ? 'mmpro3-engine.exe' : 'mmpro3-engine';
        const enginePath = path.join(buildPath, '..', binaryName);
        
        if (fs.existsSync(enginePath) && platform !== 'win32') {
          fs.chmodSync(enginePath, 0o755);
          console.log('[Packager] Made audio engine executable:', enginePath);
        }
        
        callback();
      }
    ]
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {},
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-deb',
      config: {},
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {},
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
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};
