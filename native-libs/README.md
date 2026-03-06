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

## How to Update

To update the NDI library on macOS:

1. Download and install the latest [NDI SDK for Apple](https://ndi.video/tools/)
2. Copy the library:
   ```bash
   cp "/Library/NDI SDK for Apple/lib/macOS/libndi.dylib" native-libs/macos/
   ```

## Platform Support

- ✅ macOS: Bundled
- ⏳ Windows: Coming soon
- ⏳ Linux: Coming soon
