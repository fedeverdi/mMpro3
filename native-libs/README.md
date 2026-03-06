# Native Libraries

This folder contains native libraries that are bundled with the application so users don't need to install them separately.

## macOS

### libndi.dylib
NDI (Network Device Interface) library for video/audio streaming over network.

**Version:** NDI SDK 6.x for Apple  
**Size:** ~28 MB  
**License:** See libndi_licenses.txt (if available)  
**Source:** https://ndi.video/tools/

The application will automatically use the bundled library if available, otherwise it will fallback to system-installed NDI SDK.

## Windows

### Processing.NDI.Lib.x64.dll / Processing.NDI.Lib.x86.dll
NDI (Network Device Interface) library for video/audio streaming over network.

**Version:** NDI SDK 6.x  
**Size:** ~20-30 MB  
**License:** See NDI SDK license  
**Source:** https://ndi.video/tools/

The application will automatically detect and use the correct library based on the system architecture (x64 or x86).

## How to Update

### macOS

To update the NDI library on macOS:

1. Download and install the latest [NDI SDK for Apple](https://ndi.video/tools/)
2. Copy the library:
   ```bash
   cp "/Library/NDI SDK for Apple/lib/macOS/libndi.dylib" native-libs/macos/
   ```

### Windows

To update the NDI library on Windows:

1. Download and install the latest [NDI SDK](https://ndi.video/tools/)
2. Copy the libraries from the SDK installation folder (typically `C:\Program Files\NDI\NDI 6 SDK\Bin\x64\` or `Bin\x86\`):
   ```powershell
   # For 64-bit
   Copy-Item "C:\Program Files\NDI\NDI 6 SDK\Bin\x64\Processing.NDI.Lib.x64.dll" native-libs\windows\
   
   # For 32-bit (if needed)
   Copy-Item "C:\Program Files\NDI\NDI 6 SDK\Bin\x86\Processing.NDI.Lib.x86.dll" native-libs\windows\
   ```

## Platform Support

- ✅ macOS: Bundled
- ✅ Windows: Bundled (x64 and x86)
- ⏳ Linux: Coming soon
