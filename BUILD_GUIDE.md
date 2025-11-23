# 🔨 GenXLink Build & Installer Guide

Complete guide to building GenXLink Windows client and creating installers.

---

## 📋 **PREREQUISITES**

### **1. Rust Toolchain**
```powershell
# Install Rust (if not already installed)
# Download from: https://rustup.rs/
# Or run:
winget install Rustlang.Rustup
```

### **2. Visual Studio Build Tools** (Required for Rust)
```powershell
# Download from: https://visualstudio.microsoft.com/downloads/
# Install "Desktop development with C++" workload
```

### **3. Installer Tools** (Optional)

**For MSI Installer:**
- WiX Toolset v3.11+
- Download: https://wixtoolset.org/

**For EXE Installer:**
- Inno Setup 6.0+
- Download: https://jrsoftware.org/isinfo.php

---

## 🚀 **QUICK START: BUILD PORTABLE VERSION**

### **Step 1: Run Build Script**

```powershell
# Navigate to project directory
cd "C:\Users\lalup\OneDrive\Desktop\GenXis Innovations\GenXlink"

# Run build script
.\build_installer.ps1
```

This will:
- ✅ Check Rust installation
- ✅ Build release version
- ✅ Create portable .exe
- ✅ Generate README and version info
- ✅ Output to `dist/` folder

### **Step 2: Test Portable Version**

```powershell
# Run the portable version
.\dist\GenXLink-Portable.exe
```

---

## 📦 **BUILD TYPES**

### **1. Portable Version** ✅ **READY**

**What it is:**
- Single .exe file
- No installation required
- Run from any location
- Perfect for USB drives

**How to build:**
```powershell
.\build_installer.ps1
```

**Output:**
- `dist/GenXLink-Portable.exe`
- `dist/README.md`
- `dist/VERSION.txt`

---

### **2. MSI Installer** (Windows Installer)

**What it is:**
- Professional Windows installer
- Add/Remove Programs integration
- Start Menu shortcuts
- Desktop shortcut option
- Automatic updates support

**Prerequisites:**
```powershell
# Install WiX Toolset
winget install WiX.Toolset
```

**How to build:**

1. **Build the application first:**
```powershell
cargo build --release --package genxlink-client
```

2. **Build MSI:**
```powershell
cd installer
candle genxlink.wxs
light genxlink.wixobj -out ..\dist\GenXLink-Setup-0.1.0.msi
```

**Output:**
- `dist/GenXLink-Setup-0.1.0.msi`

---

### **3. EXE Installer** (Inno Setup)

**What it is:**
- Lightweight installer
- Custom wizard UI
- Compression
- Uninstaller included

**Prerequisites:**
```powershell
# Install Inno Setup
winget install JRSoftware.InnoSetup
```

**How to build:**

1. **Build the application first:**
```powershell
cargo build --release --package genxlink-client
```

2. **Build EXE:**
```powershell
# Open Inno Setup Compiler
# File -> Open -> installer/genxlink.iss
# Build -> Compile
```

Or via command line:
```powershell
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\genxlink.iss
```

**Output:**
- `dist/GenXLink-Setup-0.1.0.exe`

---

## 🛠️ **MANUAL BUILD STEPS**

### **Step 1: Clean Build**

```powershell
# Clean previous builds
cargo clean

# Or just remove target directory
Remove-Item -Recurse -Force target
```

### **Step 2: Build Release**

```powershell
# Build Windows client (Release mode)
cargo build --release --package genxlink-client

# This creates: target/release/genxlink-client.exe
```

### **Step 3: Test Build**

```powershell
# Run the built executable
.\target\release\genxlink-client.exe
```

### **Step 4: Create Portable**

```powershell
# Create dist folder
New-Item -ItemType Directory -Force -Path dist

# Copy executable
Copy-Item target\release\genxlink-client.exe dist\GenXLink-Portable.exe
```

---

## 📊 **BUILD SIZES**

**Expected sizes:**
- Debug build: ~50-80 MB
- Release build: ~5-10 MB
- Portable .exe: ~5-10 MB
- MSI installer: ~6-12 MB
- EXE installer: ~4-8 MB (compressed)

---

## 🔍 **TROUBLESHOOTING**

### **Problem: Cargo not found**

**Solution:**
```powershell
# Add Cargo to PATH
$env:Path += ";C:\Users\$env:USERNAME\.cargo\bin"

# Or permanently:
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Users\$env:USERNAME\.cargo\bin", "User")
```

### **Problem: Build fails with linker error**

**Solution:**
```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Install "Desktop development with C++" workload
```

### **Problem: WiX not found**

**Solution:**
```powershell
# Install WiX Toolset
winget install WiX.Toolset

# Add to PATH
$env:Path += ";C:\Program Files (x86)\WiX Toolset v3.11\bin"
```

### **Problem: Inno Setup not found**

**Solution:**
```powershell
# Install Inno Setup
winget install JRSoftware.InnoSetup
```

---

## 🎯 **RECOMMENDED WORKFLOW**

### **For Development:**
```powershell
# Quick debug build
cargo build --package genxlink-client

# Run immediately
cargo run --package genxlink-client
```

### **For Testing:**
```powershell
# Build release version
cargo build --release --package genxlink-client

# Test portable version
.\build_installer.ps1
.\dist\GenXLink-Portable.exe
```

### **For Distribution:**
```powershell
# 1. Build portable version
.\build_installer.ps1

# 2. Build MSI installer
cd installer
candle genxlink.wxs
light genxlink.wixobj -out ..\dist\GenXLink-Setup-0.1.0.msi

# 3. Build EXE installer
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\genxlink.iss
```

---

## 📦 **DISTRIBUTION CHECKLIST**

Before releasing:

- [ ] Test portable version
- [ ] Test MSI installer
- [ ] Test EXE installer
- [ ] Verify all shortcuts work
- [ ] Check Add/Remove Programs entry
- [ ] Test uninstaller
- [ ] Verify server connection
- [ ] Test on clean Windows install
- [ ] Check antivirus false positives
- [ ] Sign executables (optional)

---

## 🔐 **CODE SIGNING** (Optional)

For production releases, sign your executables:

```powershell
# Get a code signing certificate
# Use signtool.exe to sign

signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com dist\GenXLink-Portable.exe
```

---

## 📝 **VERSION MANAGEMENT**

Update version in these files:
- `Cargo.toml` (workspace version)
- `client/windows/Cargo.toml`
- `installer/genxlink.wxs` (Version attribute)
- `installer/genxlink.iss` (#define MyAppVersion)
- `build_installer.ps1` ($version variable)

---

## 🚀 **AUTOMATION**

### **GitHub Actions** (Future)

Create `.github/workflows/build.yml`:
```yaml
name: Build Windows Client

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --release --package genxlink-client
      - name: Create Portable
        run: .\build_installer.ps1
      - name: Upload Artifacts
        uses: actions/upload-artifact@v2
        with:
          name: GenXLink-Windows
          path: dist/
```

---

## 📞 **SUPPORT**

If you encounter issues:

1. Check Rust installation: `cargo --version`
2. Check build tools: `cl.exe` should be in PATH
3. Review error messages carefully
4. Check GitHub Issues: https://github.com/lalupj07/GenXlink/issues

---

## ✅ **QUICK REFERENCE**

```powershell
# Build everything
.\build_installer.ps1

# Manual build
cargo build --release --package genxlink-client

# Create MSI
cd installer && candle genxlink.wxs && light genxlink.wixobj -out ..\dist\GenXLink-Setup.msi

# Create EXE
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\genxlink.iss

# Test
.\dist\GenXLink-Portable.exe
```

---

**Last Updated:** November 23, 2025  
**Version:** 0.1.0
