# Building GenXLink Installer

## Prerequisites

### Option 1: NSIS Installer (Recommended - Easier)
1. Download NSIS from: https://nsis.sourceforge.io/Download
2. Install NSIS (default location: `C:\Program Files (x86)\NSIS`)
3. Add NSIS to PATH or use full path

### Option 2: WiX Toolset (MSI)
1. Download WiX from: https://wixtoolset.org/
2. Install WiX Toolset
3. Add WiX bin folder to PATH

## Building the Installer

### Using NSIS (Creates .exe installer)

```powershell
# Navigate to installer directory
cd installer

# Build with NSIS
makensis genxlink-installer.nsi

# Output: ../dist/GenXLink-v0.1.0-Setup-Windows-x64.exe
```

### Using WiX (Creates .msi installer)

```powershell
# Navigate to installer directory
cd installer

# Compile WiX source
candle genxlink.wxs

# Link to create MSI
light genxlink.wixobj -out ../dist/GenXLink-v0.1.0-Setup-Windows-x64.msi

# Output: ../dist/GenXLink-v0.1.0-Setup-Windows-x64.msi
```

## Quick Build Script

```powershell
# Build everything at once
.\build-installer.ps1
```

## What Gets Created

1. **Portable Package** (ZIP)
   - Location: `dist/GenXLink-v0.1.0-Portable-Windows-x64.zip`
   - Contents: genxlink.exe + documentation
   - No installation required

2. **NSIS Installer** (EXE)
   - Location: `dist/GenXLink-v0.1.0-Setup-Windows-x64.exe`
   - Installs to Program Files
   - Creates Start Menu shortcuts
   - Creates Desktop shortcut
   - Includes uninstaller

3. **WiX Installer** (MSI)
   - Location: `dist/GenXLink-v0.1.0-Setup-Windows-x64.msi`
   - Professional MSI package
   - Windows Installer format
   - Enterprise deployment ready

## Installer Features

- ✅ Installs to Program Files
- ✅ Creates Start Menu shortcuts
- ✅ Creates Desktop shortcut
- ✅ Adds to Add/Remove Programs
- ✅ Includes uninstaller
- ✅ License agreement display
- ✅ Custom install directory option

## Distribution

After building, you'll have:
- `GenXLink-v0.1.0-Portable-Windows-x64.zip` - For portable use
- `GenXLink-v0.1.0-Setup-Windows-x64.exe` - For standard installation

Upload these to:
- GitHub Releases
- Your website
- Distribution platforms

## Notes

- Portable version requires no admin rights
- Installer version requires admin rights
- Both versions are the same binary
- Portable stores settings in app folder
- Installed version uses AppData

## Copyright

Copyright (c) 2025 GenXis Innovations
Contact: genxisinnovation@outlook.com
