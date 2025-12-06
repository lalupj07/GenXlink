# GenXLink Icon Assets

## Required Icon Files

Please save the GenXLink logo in the following formats:

1. **genxlink.ico** - Windows icon file (multi-resolution: 16x16, 32x32, 48x48, 256x256)
2. **genxlink-256.png** - 256x256 PNG for high-res displays
3. **genxlink-128.png** - 128x128 PNG
4. **genxlink-64.png** - 64x64 PNG
5. **genxlink-32.png** - 32x32 PNG
6. **genxlink-16.png** - 16x16 PNG

## How to Create ICO File

Use an online converter like:
- https://icoconvert.com/
- https://convertio.co/png-ico/

Or use ImageMagick:
```bash
magick convert logo.png -define icon:auto-resize=256,128,64,48,32,16 genxlink.ico
```

## Usage

- **genxlink.ico** - Used for Windows executable and installer
- **PNG files** - Used for MSIX package and in-app display
