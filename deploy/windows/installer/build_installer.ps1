#!/usr/bin/env pwsh

# GenXLink Windows Installer Build Script
# This script builds the complete Windows MSI installer for GenXLink

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release",
    
    [Parameter(Mandatory=$false)]
    [string]$Version = "0.2.0",
    
    [Parameter(Mandatory=$false)]
    [string]$OutputDir = ".\dist",
    
    [Parameter(Mandatory=$false)]
    [switch]$SkipTests = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$SignBinary = $false,
    
    [Parameter(Mandatory=$false)]
    [string]$CertificatePath = "",
    
    [Parameter(Mandatory=$false)]
    [SecureString]$CertificatePassword = ""
)

# Script configuration
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-SuccessMessage($message) {
    Write-ColorOutput Green "✅ $message"
}

function Write-ErrorMessage($message) {
    Write-ColorOutput Red "❌ $message"
}

function Write-WarningMessage($message) {
    Write-ColorOutput Yellow "⚠️  $message"
}

function Write-Info($message) {
    Write-ColorOutput Cyan "ℹ️  $message"
}

function Write-Step($message) {
    Write-ColorOutput Magenta "🔧 $message"
}

# Main script starts here
Write-Info "GenXLink Windows Installer Build Script v$Version"
Write-Info "=========================================="

# Check prerequisites
Write-Step "Checking prerequisites..."

# Check for Visual Studio Build Tools
try {
    $vsWhere = Get-Command "vswhere.exe" -ErrorAction Stop
    $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $vsPath) {
        throw "Visual Studio Build Tools not found"
    }
    Write-SuccessMessage "Visual Studio Build Tools found: $vsPath"
} catch {
    Write-Error "Visual Studio Build Tools not found. Please install Visual Studio 2022 with C++ build tools."
    exit 1
}

# Check for WiX Toolset
try {
    $wix = Get-Command "candle.exe" -ErrorAction Stop
    Write-SuccessMessage "WiX Toolset found: $($wix.Source)"
} catch {
    Write-Error "WiX Toolset not found. Please install WiX Toolset v3.11 or later."
    exit 1
}

# Check for Rust toolchain
try {
    $rustVersion = & rustc --version
    Write-SuccessMessage "Rust toolchain found: $rustVersion"
} catch {
    Write-Error "Rust toolchain not found. Please install Rust from https://rustup.rs/"
    exit 1
}

# Check for OpenSSL (for signing)
if ($SignBinary) {
    if (-not (Get-Command "openssl.exe" -ErrorAction SilentlyContinue)) {
        Write-Warning "OpenSSL not found. Binary signing will be skipped."
        $SignBinary = $false
    }
}

# Clean previous builds
Write-Step "Cleaning previous builds..."
if (Test-Path $OutputDir) {
    Remove-Item -Recurse -Force $OutputDir
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Set environment variables
$env:GENXLINK_VERSION = $Version
$env:GENXLINK_CONFIGURATION = $Configuration

# Build the application
Write-Step "Building GenXLink application..."

Set-Location "..\..\.."
try {
    # Build release version
    & cargo build --release --bin genxlink
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed"
    }
    Write-SuccessMessage "Application built successfully"
} catch {
    Write-Error "Failed to build application: $_"
    exit 1
}

# Run tests if not skipped
if (-not $SkipTests) {
    Write-Step "Running tests..."
    try {
        & cargo test --release
        if ($LASTEXITCODE -ne 0) {
            throw "Tests failed"
        }
        Write-SuccessMessage "All tests passed"
    } catch {
        Write-Warning "Tests failed: $_. Continuing with build..."
    }
}

# Copy binaries to installer directory
Write-Step "Preparing installer files..."

$installerDir = ".\deployment\windows\installer"
$buildDir = ".\target\release"
$stagingDir = Join-Path $installerDir "staging"

if (Test-Path $stagingDir) {
    Remove-Item -Recurse -Force $stagingDir
}
New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null

# Copy main executable
Copy-Item "$buildDir\genxlink.exe" "$stagingDir\" -Force
Write-SuccessMessage "Copied main executable"

# Copy DLL dependencies
$dependencies = @(
    "webrtc.dll",
    "crypto.dll", 
    "media.dll",
    "network.dll"
)

foreach ($dep in $dependencies) {
    if (Test-Path "$buildDir\$dep") {
        Copy-Item "$buildDir\$dep" "$stagingDir\" -Force
        Write-SuccessMessage "Copied $dep"
    } else {
        Write-Warning "Dependency $dep not found"
    }
}

# Copy configuration files
$configDir = Join-Path $stagingDir "config"
New-Item -ItemType Directory -Path $configDir -Force | Out-Null

$configFiles = @(
    "config\default.toml",
    "config\logging.toml", 
    "config\security.toml"
)

foreach ($config in $configFiles) {
    if (Test-Path $config) {
        Copy-Item $config "$configDir\" -Force
        Write-SuccessMessage "Copied $config"
    }
}

# Copy resources
$resourcesDir = Join-Path $stagingDir "resources"
New-Item -ItemType Directory -Path $resourcesDir -Force | Out-Null

$resourceFiles = @(
    "resources\icon.ico",
    "resources\banner.bmp",
    "resources\dialog.bmp"
)

foreach ($resource in $resourceFiles) {
    if (Test-Path $resource) {
        Copy-Item $resource "$resourcesDir\" -Force
        Write-SuccessMessage "Copied $resource"
    }
}

# Copy documentation
$docsDir = Join-Path $stagingDir "docs"
New-Item -ItemType Directory -Path $docsDir -Force | Out-Null

if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" "$stagingDir\LICENSE.txt" -Force
}
if (Test-Path "README.md") {
    Copy-Item "README.md" "$stagingDir\README.txt" -Force
}

# Sign binaries if requested
if ($SignBinary -and $CertificatePath) {
    Write-Step "Signing binaries..."
    try {
        # Convert SecureString to plain text for signtool
        $passwordPlain = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($CertificatePassword))
        & signtool sign /f $CertificatePath /p $passwordPlain /t http://timestamp.digicert.com "$stagingDir\genxlink.exe"
        if ($LASTEXITCODE -eq 0) {
            Write-SuccessMessage "Binary signed successfully"
        } else {
            Write-Warning "Binary signing failed"
        }
    } catch {
        Write-Warning "Binary signing failed: $_"
    }
}

# Generate application manifest
Write-Step "Generating application manifest..."

$manifestContent = @"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="$Version.0"
                    processorArchitecture="amd64"
                    name="GenXLink.Innovations.GenXLink"
                    type="win32"
                    publicKeyToken="0000000000000000"/>
  <description>GenXLink Remote Desktop - Secure, high-performance remote desktop solution</description>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
      <!-- Windows 8.1 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}" />
      <!-- Windows 8 -->
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}" />
      <!-- Windows 7 -->
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}" />
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>
"@

$manifestContent | Out-File "$stagingDir\genxlink.exe.manifest" -Encoding UTF8
Write-SuccessMessage "Application manifest generated"

# Build WiX installer
Write-Step "Building WiX installer..."

Set-Location $installerDir

# Compile WiX source
try {
    & candle.exe genxlink.wxs -out genxlink.wixobj -ext WixUIExtension -ext WixUtilExtension
    if ($LASTEXITCODE -ne 0) {
        throw "WiX compilation failed"
    }
    Write-SuccessMessage "WiX source compiled"
} catch {
    Write-Error "WiX compilation failed: $_"
    exit 1
}

# Link WiX object
try {
    $msiPath = Join-Path (Resolve-Path $OutputDir).Path "GenXLink-$Version-x64.msi"
    & light.exe genxlink.wixobj -out $msiPath -ext WixUIExtension -ext WixUtilExtension -cultures:en-us
    if ($LASTEXITCODE -ne 0) {
        throw "WiX linking failed"
    }
    Write-SuccessMessage "WiX installer created: $msiPath"
} catch {
    Write-Error "WiX linking failed: $_"
    exit 1
}

# Create setup executable wrapper
Write-Step "Creating setup executable wrapper..."

$setupScript = @"
@echo off
echo GenXLink Remote Desktop Setup v$Version
echo =====================================
echo.
echo This will install GenXLink Remote Desktop on your computer.
echo.
pause

msiexec /i "GenXLink-$Version-x64.msi" /l*v "install.log"

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Installation completed successfully!
    echo.
    pause
) else (
    echo.
    echo Installation failed. Check install.log for details.
    echo.
    pause
)
"@

$setupScript | Out-File "$OutputDir\setup.bat" -Encoding ASCII

# Create portable version
Write-Step "Creating portable version..."

$portableDir = Join-Path $OutputDir "GenXLink-Portable-$Version"
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null

# Copy portable files
Copy-Item "$stagingDir\*" "$portableDir\" -Recurse -Force

# Create portable launcher
$portableLauncher = @"
@echo off
cd /d "%~dp0"
echo Starting GenXLink Remote Desktop (Portable)...
genxlink.exe --portable
"@

$portableLauncher | Out-File "$portableDir\GenXLink-Portable.bat" -Encoding ASCII

# Create README for portable version
$portableReadme = @"
GenXLink Remote Desktop - Portable Version v$Version
====================================================

This is a portable version of GenXLink that doesn't require installation.

To run GenXLink:
1. Double-click GenXLink-Portable.bat
2. Or run genxlink.exe --portable from command line

Configuration and data will be stored in this directory.

For more information, visit https://genxlink.com
"@

$portableReadme | Out-File "$portableDir\README.txt" -Encoding ASCII

# Create ZIP archive of portable version
Write-Step "Creating portable ZIP archive..."

$zipPath = Join-Path $OutputDir "GenXLink-Portable-$Version.zip"
if (Get-Command "Compress-Archive" -ErrorAction SilentlyContinue) {
    Compress-Archive -Path "$portableDir\*" -DestinationPath $zipPath -Force
} else {
    # Fallback to PowerShell 5.1 method
    Add-Type -AssemblyName "System.IO.Compression.FileSystem"
    [System.IO.Compression.ZipFile]::CreateFromDirectory($portableDir, $zipPath)
}

Write-SuccessMessage "Portable ZIP created: $zipPath"

# Generate checksums
Write-Step "Generating checksums..."

$checksumFile = Join-Path $OutputDir "checksums.txt"
"GenXLink-$Version-x64.msi" | Out-File $checksumFile -Encoding UTF8
Get-FileHash "$msiPath" -Algorithm SHA256 | Select-Object -ExpandProperty Hash | Out-File $checksumFile -Append -Encoding UTF8

"" | Out-File $checksumFile -Append -Encoding UTF8
"GenXLink-Portable-$Version.zip" | Out-File $checksumFile -Append -Encoding UTF8
Get-FileHash $zipPath -Algorithm SHA256 | Select-Object -ExpandProperty Hash | Out-File $checksumFile -Append -Encoding UTF8

Write-SuccessMessage "Checksums generated: $checksumFile"

# Clean up temporary files
Write-Step "Cleaning up temporary files..."
Remove-Item -Recurse -Force $stagingDir -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force $portableDir -ErrorAction SilentlyContinue
Remove-Item "genxlink.wixobj" -ErrorAction SilentlyContinue

# Summary
Write-Info "Build completed successfully!"
Write-Info "================================"
Write-Info "Generated files:"
Write-Info "  • MSI Installer: GenXLink-$Version-x64.msi"
Write-Info "  • Portable ZIP: GenXLink-Portable-$Version.zip"
Write-Info "  • Setup Script: setup.bat"
Write-Info "  • Checksums: checksums.txt"
Write-Info ""
Write-Info "Output directory: $OutputDir"
Write-Info ""
Write-Info "Installation commands:"
Write-Info "  • Silent install: msiexec /i GenXLink-$Version-x64.msi /quiet"
Write-Info "  • Uninstall: msiexec /x GenXLink-$Version-x64.msi"

# Test the installer (optional)
Write-Step "Testing installer integrity..."
try {
    & msiexec /i "$msiPath" /qn /l+v "$OutputDir\test-install.log" | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-SuccessMessage "Installer test passed"
        # Clean up test installation
        & msiexec /x "$msiPath" /qn | Out-Null
    } else {
        Write-Warning "Installer test failed. Check test-install.log for details."
    }
} catch {
    Write-Warning "Installer test failed: $_"
}

Write-SuccessMessage "Windows installer build completed!"
