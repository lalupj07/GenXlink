# GenXLink - MSIX Package & Microsoft Store Submission Guide

## 📦 What is MSIX?

MSIX is Microsoft's modern packaging format for Windows apps, required for Microsoft Store submission.

## 🚀 Quick Start - Create MSIX Package

### Step 1: Build the Application
```powershell
cargo build --release --bin genxlink
```

### Step 2: Create MSIX Package
```powershell
.\CREATE_MSIX.ps1
```

This will create: `dist\GenXLink.msix`

## 📋 Requirements for Microsoft Store

### 1. App Assets (PNG Images)

You MUST create these images and place them in `msix\package\Assets\`:

| File Name | Size | Purpose |
|-----------|------|---------|
| Square44x44Logo.png | 44x44 px | App list icon |
| Square71x71Logo.png | 71x71 px | Small tile |
| Square150x150Logo.png | 150x150 px | Medium tile |
| Square310x310Logo.png | 310x310 px | Large tile |
| Wide310x150Logo.png | 310x150 px | Wide tile |
| StoreLogo.png | 50x50 px | Store listing |
| SplashScreen.png | 620x300 px | Launch screen |

**Design Tips:**
- Use your app logo/icon
- Transparent background recommended
- Follow Microsoft Store design guidelines
- Use tools like Photoshop, GIMP, or Canva

### 2. Windows SDK

**Required for creating MSIX packages**

Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

The script will check if it's installed and guide you.

### 3. Code Signing Certificate

**Required for Microsoft Store submission**

Options:
1. **Microsoft Partner Center** - Get free signing (recommended)
2. **Purchase certificate** - From DigiCert, Sectigo, etc.
3. **Self-signed** - For testing only (not for Store)

## 🔧 Creating the MSIX Package

### Method 1: Using Our Script (Recommended)

```powershell
# 1. Build the app
cargo build --release --bin genxlink

# 2. Create MSIX
.\CREATE_MSIX.ps1

# Output: dist\GenXLink.msix
```

### Method 2: Manual Creation

```powershell
# 1. Create package structure
mkdir msix\package
mkdir msix\package\Assets

# 2. Copy files
copy target\release\genxlink.exe msix\package\
copy msix\AppxManifest.xml msix\package\

# 3. Add your PNG assets to msix\package\Assets\

# 4. Create MSIX
makeappx pack /d msix\package /p dist\GenXLink.msix /o
```

## 🔐 Signing the MSIX Package

### For Testing (Self-Signed)

```powershell
# Create self-signed certificate
$cert = New-SelfSignedCertificate -Type Custom -Subject "CN=GenXis Innovations" -KeyUsage DigitalSignature -FriendlyName "GenXLink Test" -CertStoreLocation "Cert:\CurrentUser\My"

# Export certificate
Export-PfxCertificate -Cert $cert -FilePath GenXLink-Test.pfx -Password (ConvertTo-SecureString -String "YourPassword" -Force -AsPlainText)

# Sign MSIX
SignTool sign /fd SHA256 /a /f GenXLink-Test.pfx /p YourPassword dist\GenXLink.msix
```

### For Microsoft Store

**You don't need to sign!** Microsoft Partner Center will sign it for you automatically.

## 📤 Microsoft Store Submission Process

### Step 1: Create Microsoft Partner Center Account

1. Go to: https://partner.microsoft.com/dashboard
2. Sign up for a developer account
3. Pay one-time fee: $19 (individual) or $99 (company)
4. Complete verification

### Step 2: Create New App Submission

1. Go to Partner Center Dashboard
2. Click "Apps and games"
3. Click "+ New product" → "App"
4. Reserve app name: "GenXLink Remote Desktop"

### Step 3: Fill App Information

#### Product Identity
- **App name:** GenXLink Remote Desktop
- **Category:** Productivity
- **Subcategory:** Remote Desktop

#### Pricing
- **Base price:** Free (or set your price)
- **Markets:** Select all or specific countries

#### Properties
- **Category:** Productivity Tools
- **Privacy policy URL:** (your website)
- **Support contact:** (your email)

#### Age Ratings
- **IARC:** Complete questionnaire
- Likely rating: PEGI 3 / ESRB Everyone

#### Packages
- **Upload:** `dist\GenXLink.msix`
- **Target device families:** Desktop
- **Minimum OS:** Windows 10 version 1809 (Build 17763)

#### Store Listings
- **Description:** 
  ```
  GenXLink Remote Desktop - Fast, Secure, Ultra-Low Latency Remote Access
  
  Connect to your PCs from anywhere with military-grade encryption and professional features.
  
  Features:
  • Ultra-low latency remote desktop
  • End-to-end encryption
  • Screen sharing with HD quality
  • File transfer between devices
  • Multi-monitor support
  • Session recording
  • Cross-platform compatibility
  
  Perfect for:
  - Remote work
  - IT support
  - System administration
  - Technical assistance
  ```

- **Screenshots:** Minimum 1, maximum 10 (1366x768 or larger)
- **App icon:** 300x300 px PNG
- **Promotional images:** Optional but recommended

### Step 4: Submit for Certification

1. Review all information
2. Click "Submit for certification"
3. Wait for review (usually 24-48 hours)

## 🧪 Testing Before Submission

### Install MSIX Locally

```powershell
# Install
Add-AppxPackage dist\GenXLink.msix

# Launch from Start Menu
# Or run: start shell:AppsFolder\GenXisInnovations.GenXLinkRemoteDesktop_[PublisherId]!GenXLink

# Uninstall
Remove-AppxPackage GenXisInnovations.GenXLinkRemoteDesktop
```

### Test Checklist

- [ ] App installs successfully
- [ ] App launches from Start Menu
- [ ] All features work correctly
- [ ] App closes properly
- [ ] No crashes or errors
- [ ] Uninstall works cleanly

## 📊 Current Status

### ✅ Completed
- [x] MSIX manifest created
- [x] Package structure defined
- [x] Build script ready
- [x] Submission guide written

### ⏳ Pending
- [ ] Create app icon assets (PNG images)
- [ ] Install Windows SDK (if not installed)
- [ ] Run CREATE_MSIX.ps1
- [ ] Test MSIX installation
- [ ] Create Partner Center account
- [ ] Submit to Microsoft Store

## 🎯 Quick Submission Checklist

1. **Build app:** `cargo build --release --bin genxlink`
2. **Create assets:** Design 7 PNG images (see sizes above)
3. **Create MSIX:** `.\CREATE_MSIX.ps1`
4. **Test locally:** `Add-AppxPackage dist\GenXLink.msix`
5. **Create Partner Center account**
6. **Upload MSIX** to Partner Center
7. **Fill app information** (description, screenshots, etc.)
8. **Submit for certification**
9. **Wait for approval** (24-48 hours)
10. **App goes live** on Microsoft Store!

## 💡 Tips for Faster Approval

1. **Clear description** - Explain what your app does
2. **Good screenshots** - Show actual app interface
3. **Privacy policy** - Required if you collect data
4. **Age rating** - Complete IARC questionnaire accurately
5. **Test thoroughly** - No crashes or major bugs
6. **Follow guidelines** - Read Microsoft Store Policies

## 🆘 Troubleshooting

### "Windows SDK not found"
- Download and install from Microsoft website
- Restart PowerShell after installation

### "MakeAppx failed"
- Check all required files are in msix\package\
- Verify AppxManifest.xml is valid
- Ensure assets folder has all PNG files

### "Installation failed"
- Enable Developer Mode in Windows Settings
- Check if app is already installed (uninstall first)
- Verify MSIX is signed (for non-Store testing)

### "Certification failed"
- Read rejection reason carefully
- Fix issues mentioned
- Resubmit

## 📞 Support

**Microsoft Partner Center Support:**
- https://partner.microsoft.com/support

**MSIX Documentation:**
- https://docs.microsoft.com/windows/msix/

**Store Policies:**
- https://docs.microsoft.com/windows/uwp/publish/store-policies

---

**Ready to submit!** Follow the steps above and your app will be on the Microsoft Store soon! 🚀
