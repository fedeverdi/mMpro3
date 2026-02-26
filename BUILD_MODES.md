# Build Modes Configuration

This document explains the build modes system and how to use it.

## Overview

MMpro3 supports three build modes with different track and feature limits:

| Feature | Demo | Medium | Full |
|---------|------|--------|------|
| Audio Tracks | 1 | 4 | 24 |
| Signal Tracks | 1 | 1 | 24 |
| Total Tracks | 2 | 5 | 24 |
| Aux Buses | 1 | 6 | 6 |
| Subgroups | 0 | 4 | 4 |
| Subgroup Routing | ❌ | ✅ | ✅ |

## Usage

### Development

```bash
# Full mode (default)
npm run dev
npm run dev:browser
npm run dev:full

# Medium mode
npm run dev:medium

# Demo mode
npm run dev:demo
```

### Production Build

```bash
# Full mode (default)
npm run build
npm run build:full

# Medium mode
npm run build:medium

# Demo mode
npm run build:demo
```

## Implementation Details

### File Structure

- `src/config/buildLimits.ts` - Build limits configuration and helper functions
- `.env` - Default environment variables (full mode)
- `.env.demo` - Demo mode configuration
- `.env.medium` - Medium mode configuration
- `.env.production` - Production build configuration (full mode)

### Configuration

The build mode is controlled by the `VITE_BUILD_MODE` environment variable, which can be:
- `demo` - Limited demo version
- `medium` - Intermediate version
- `full` - Complete version (default)

### Key Functions

**`getBuildMode()`**
Returns the current build mode based on the environment variable.

**`getBuildLimits()`**
Returns the BuildLimits object for the current mode with:
- `maxTracks` - Maximum total tracks
- `maxAudioTracks` - Maximum audio tracks
- `maxSignalTracks` - Maximum signal tracks
- `maxAuxBuses` - Maximum aux buses
- `maxSubgroups` - Maximum subgroups
- `allowSubgroupRouting` - Whether subgroup routing is enabled
- `defaultAudioTracks` - Default audio tracks on startup
- `defaultSignalTracks` - Default signal tracks on startup

**`canAddTrack(currentTracks, trackType)`**
Checks if a track of the specified type can be added based on current limits.

**`getTrackCounts(tracks)`**
Returns the current count of audio, signal, and total tracks.

### UI Changes

The build limits system affects:
1. **Track Counter** - Shows current/max tracks (e.g., "7/24" or "2/2" in demo)
2. **Add Button** - Disabled when track limit is reached
3. **Add Menu** - Shows appropriate alerts when limits are hit
4. **Subgroup Button** - Hidden in demo mode, disabled when limit reached
5. **Routing Buttons** - Subgroup routing buttons hidden in demo mode
6. **Aux Add Button** - Disabled when aux limit is reached

### Track Initialization

On startup, tracks are automatically initialized based on the build mode:
- **Demo**: 1 audio + 1 signal
- **Medium**: 4 audio + 1 signal
- **Full**: 6 audio + 1 signal

## Testing

To test different build modes:

1. Start the dev server with the desired mode:
   ```bash
   npm run dev:demo
   ```

2. Verify the limits:
   - Try adding tracks beyond the limit
   - Check that subgroup routing buttons appear/disappear
   - Verify the track counter shows the correct maximum

3. Test the build process:
   ```bash
   npm run build:demo
   ```

## Notes

- The default mode (when no environment variable is set) is `full`
- Limits are enforced at both the UI level and in the add functions
- The system shows helpful alerts when limits are reached
- Build limits only affect the number of tracks/buses, not the audio processing capabilities
