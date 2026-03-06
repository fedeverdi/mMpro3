# Audio Engine Assets

This folder contains embedded assets used by the audio engine.

## Fonts

### Roboto-Regular.ttf
- **License:** Apache License 2.0
- **Source:** [Google Fonts - Roboto](https://github.com/googlefonts/roboto)
- **Usage:** Used for rendering text on NDI video frames
- **Size:** ~503 KB

The font is embedded at compile-time using `include_bytes!()` to ensure cross-platform compatibility (macOS, Windows, Linux).

## License

Roboto is licensed under the Apache License, Version 2.0. You may obtain a copy of the License at:
http://www.apache.org/licenses/LICENSE-2.0
