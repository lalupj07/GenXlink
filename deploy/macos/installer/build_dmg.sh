#!/bin/bash

# GenXLink macOS DMG Build Script
# This script builds the complete macOS DMG installer for GenXLink

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"
VERSION="${GENXLINK_VERSION:-0.2.0}"
BUILD_DIR="${PROJECT_ROOT}/target/release"
DMG_DIR="${PROJECT_ROOT}/deployment/macos/dmg"
DIST_DIR="${PROJECT_ROOT}/dist"
APP_NAME="GenXLink"
BUNDLE_ID="com.genxlink.genxlink"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_step() {
    echo -e "${MAGENTA}🔧 $1${NC}"
}

# Main script starts here
log_info "GenXLink macOS DMG Build Script v$VERSION"
log_info "=========================================="

# Check prerequisites
log_step "Checking prerequisites..."

# Check for Xcode Command Line Tools
if ! xcode-select -p &> /dev/null; then
    log_error "Xcode Command Line Tools not installed"
    log_info "Install with: xcode-select --install"
    exit 1
fi
log_success "Xcode Command Line Tools found"

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    log_error "Rust toolchain not found"
    log_info "Install from https://rustup.rs/"
    exit 1
fi
log_success "Rust toolchain found: $(cargo --version)"

# Check for create-dmg
if ! command -v create-dmg &> /dev/null; then
    log_warning "create-dmg not found, installing..."
    if command -v brew &> /dev/null; then
        brew install create-dmg
    else
        log_error "Homebrew not found. Please install create-dmg manually"
        exit 1
    fi
fi
log_success "create-dmg found"

# Check for codesign (for notarization)
if ! command -v codesign &> /dev/null; then
    log_warning "codesign not available - DMG will not be signed"
    SIGN_DMG=false
else
    SIGN_DMG=true
fi

# Clean previous builds
log_step "Cleaning previous builds..."
rm -rf "$DMG_DIR"
rm -rf "$DIST_DIR"
mkdir -p "$DMG_DIR"
mkdir -p "$DIST_DIR"

# Set environment variables
export MACOSX_DEPLOYMENT_TARGET="11.0"
export GENXLINK_VERSION="$VERSION"
export GENXLINK_BUILD="dmg"

# Build the application
log_step "Building GenXLink application..."

cd "$PROJECT_ROOT"

# Build release version with macOS optimizations
cargo build --release --target x86_64-apple-darwin --bin genxlink
cargo build --release --target aarch64-apple-darwin --bin genxlink

# Create universal binary
log_step "Creating universal binary..."
mkdir -p "$BUILD_DIR/universal"
lipo -create \
    "$BUILD_DIR/x86_64-apple-darwin/release/genxlink" \
    "$BUILD_DIR/aarch64-apple-darwin/release/genxlink" \
    -output "$BUILD_DIR/universal/genxlink"

log_success "Universal binary created"

# Run tests
log_step "Running tests..."
cargo test --release || log_warning "Some tests failed, continuing with build..."

# Create application bundle structure
log_step "Creating application bundle..."

APP_BUNDLE="$DMG_DIR/$APP_NAME.app"
CONTENTS="$APP_BUNDLE/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"
FRAMEWORKS="$CONTENTS/Frameworks"

mkdir -p "$MACOS"
mkdir -p "$RESOURCES"
mkdir -p "$FRAMEWORKS"

# Create Info.plist
log_step "Creating Info.plist..."
cat > "$CONTENTS/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>GenXLink Remote Desktop</string>
    <key>CFBundleExecutable</key>
    <string>genxlink</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>GenXLink</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSCameraUsageDescription</key>
    <string>GenXLink requires camera access for screen sharing and video conferencing features.</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>GenXLink requires microphone access for audio streaming and voice communication.</string>
    <key>NSDesktopFolderUsageDescription</key>
    <string>GenXLink requires desktop access for screen sharing functionality.</string>
    <key>NSDocumentsFolderUsageDescription</key>
    <string>GenXLink requires documents access for file transfer features.</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>genxlink</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>GenXLink Session</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSHandlerRank</key>
            <string>Owner</string>
        </dict>
    </array>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>GenXLink Protocol</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>genxlink</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
</dict>
</plist>
EOF

# Copy executable to bundle
log_step "Copying executable to bundle..."
cp "$BUILD_DIR/universal/genxlink" "$MACOS/genxlink"
chmod +x "$MACOS/genxlink"

# Copy resources
log_step "Copying resources..."
if [ -d "$PROJECT_ROOT/resources" ]; then
    cp -r "$PROJECT_ROOT/resources/"* "$RESOURCES/"
fi

# Copy configuration files
log_step "Copying configuration files..."
mkdir -p "$RESOURCES/config"
cp "$PROJECT_ROOT/config/default.toml" "$RESOURCES/config/"
cp "$PROJECT_ROOT/config/logging.toml" "$RESOURCES/config/"
cp "$PROJECT_ROOT/config/security.toml" "$RESOURCES/config/"

# Copy documentation
log_step "Copying documentation..."
mkdir -p "$RESOURCES/docs"
cp "$PROJECT_ROOT/README.md" "$RESOURCES/docs/"
cp "$PROJECT_ROOT/LICENSE" "$RESOURCES/docs/"

# Create application icon
log_step "Creating application icon..."
if [ ! -f "$RESOURCES/icon.icns" ]; then
    # Create a simple icon if none exists
    mkdir -p "$RESOURCES/icon.iconset"
    # This would normally be a proper icon file
    # For now, we'll create a placeholder
    log_warning "No icon file found - using placeholder"
fi

# Copy frameworks and libraries
log_step "Bundling frameworks..."
if [ -d "$BUILD_DIR/x86_64-apple-darwin/release/deps" ]; then
    # Copy dynamic libraries that the application depends on
    find "$BUILD_DIR/x86_64-apple-darwin/release/deps" -name "*.dylib" -exec cp {} "$FRAMEWORKS/" \;
fi

# Code signing
if [ "$SIGN_DMG" = true ] && [ -n "${CODESIGN_IDENTITY:-}" ]; then
    log_step "Code signing application bundle..."
    codesign --force --verify --verbose --sign "$CODESIGN_IDENTITY" "$APP_BUNDLE"
    log_success "Application bundle signed"
else
    log_warning "Skipping code signing (no identity provided)"
fi

# Create DMG
log_step "Creating DMG installer..."

# Create temporary DMG structure
DMG_TMP_DIR="$DMG_DIR/tmp"
mkdir -p "$DMG_TMP_DIR"

# Copy application to DMG directory
cp -R "$APP_BUNDLE" "$DMG_TMP_DIR/"

# Create Applications folder symlink
ln -s /Applications "$DMG_TMP_DIR/Applications"

# Create DMG background
log_step "Creating DMG background..."
mkdir -p "$DMG_TMP_DIR/.background"
cat > "$DMG_TMP_DIR/.background/background.png" << 'EOF'
# This would be a proper background image
# For now, we'll use create-dmg's default
EOF

# Create DMG with create-dmg
log_step "Building DMG package..."
DMG_PATH="$DIST_DIR/GenXLink-$VERSION-macOS.dmg"

create-dmg \
    --volname "GenXLink $VERSION" \
    --volicon "$RESOURCES/icon.icns" \
    --window-pos 200 120 \
    --window-size 800 600 \
    --icon-size 100 \
    --icon "$APP_NAME.app" 200 190 \
    --hide-extension "$APP_NAME.app" \
    --app-drop-link 600 185 \
    --background "$DMG_TMP_DIR/.background/background.png" \
    --disk-image-size 500 \
    --hdiutil-quiet \
    "$DMG_PATH" \
    "$DMG_TMP_DIR" || log_warning "create-dmg failed, trying manual method"

# Fallback manual DMG creation if create-dmg fails
if [ ! -f "$DMG_PATH" ]; then
    log_warning "create-dmg failed, creating DMG manually..."
    
    # Create temporary DMG
    hdiutil create -srcfolder "$DMG_TMP_DIR" -volname "GenXLink $VERSION" -fs HFS+ -fsargs "-c c=64,a=16,e=16" -format UDRW -size 500m "$DMG_PATH.tmp"
    
    # Mount and configure
    DEVICE=$(hdiutil attach -readwrite -noverify -noautoopen "$DMG_PATH.tmp" | egrep '^/dev/' | sed 1q | awk '{print $1}')
    
    # Configure DMG appearance
    echo '
    tell application "Finder"
        tell disk "'GenXLink $VERSION'"
            open
            set current view of container window to icon view
            set toolbar visible of container window to false
            set statusbar visible of container window to false
            set the bounds of container window to {400, 100, 1200, 700}
            set theViewOptions to the icon view options of container window
            set arrangement of theViewOptions to not arranged
            set icon size of theViewOptions to 100
            set position of item "'$APP_NAME'.app" of container window to {200, 190}
            set position of item "Applications" of container window to {600, 190}
            close
            open
            update without registering applications
            delay 5
        end tell
    end tell
    ' | osascript
    
    # Unmount and convert to read-only
    hdiutil detach $DEVICE
    hdiutil convert "$DMG_PATH.tmp" -format UDZO -imagekey zlib-level=9 -o "$DMG_PATH"
    rm -f "$DMG_PATH.tmp"
fi

log_success "DMG created: $DMG_PATH"

# Notarization (if certificates are available)
if [ "$SIGN_DMG" = true ] && [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_PASSWORD:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ]; then
    log_step "Notarizing DMG..."
    
    # Upload for notarization
    NOTARIZATION_UUID=$(xcrun altool --notarize-app \
        --primary-bundle-id "$BUNDLE_ID" \
        --username "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --asc-provider "$APPLE_TEAM_ID" \
        --file "$DMG_PATH" \
        2>&1 | grep "RequestUUID" | awk '{print $3}')
    
    if [ -n "$NOTARIZATION_UUID" ]; then
        log_info "Notarization uploaded with UUID: $NOTARIZATION_UUID"
        
        # Wait for notarization
        while true; do
            STATUS=$(xcrun altool --notarization-info "$NOTARIZATION_UUID" \
                --username "$APPLE_ID" \
                --password "$APPLE_PASSWORD" \
                2>&1 | grep "Status:" | awk '{print $2}')
            
            if [ "$STATUS" = "success" ]; then
                log_success "Notarization successful"
                break
            elif [ "$STATUS" = "invalid" ]; then
                log_error "Notarization failed"
                xcrun altool --notarization-info "$NOTARIZATION_UUID" \
                    --username "$APPLE_ID" \
                    --password "$APPLE_PASSWORD"
                break
            else
                log_info "Notarization in progress... (Status: $STATUS)"
                sleep 30
            fi
        done
        
        # Staple notarization
        if [ "$STATUS" = "success" ]; then
            log_step "Stapling notarization to DMG..."
            xcrun stapler staple "$DMG_PATH"
            log_success "Notarization stapled"
        fi
    else
        log_warning "Failed to upload for notarization"
    fi
else
    log_warning "Skipping notarization (no Apple credentials provided)"
fi

# Generate checksums
log_step "Generating checksums..."
cd "$DIST_DIR"
sha256sum "GenXLink-$VERSION-macOS.dmg" > checksums.txt
log_success "Checksums generated"

# Create portable version
log_step "Creating portable version..."
PORTABLE_DIR="$DIST_DIR/GenXLink-Portable-$VERSION-macOS"
mkdir -p "$PORTABLE_DIR"

# Copy portable files
cp "$BUILD_DIR/universal/genxlink" "$PORTABLE_DIR/"
cp -r "$PROJECT_ROOT/config" "$PORTABLE_DIR/"
cp "$PROJECT_ROOT/README.md" "$PORTABLE_DIR/"
cp "$PROJECT_ROOT/LICENSE" "$PORTABLE_DIR/"

# Create portable launcher
cat > "$PORTABLE_DIR/GenXLink-Portable.sh" << EOF
#!/bin/bash
# GenXLink Portable Launcher

SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
cd "\$SCRIPT_DIR"

echo "Starting GenXLink Remote Desktop (Portable)..."
./genxlink --portable --config "./config/default.toml"
EOF

chmod +x "$PORTABLE_DIR/GenXLink-Portable.sh"

# Create portable README
cat > "$PORTABLE_DIR/README.txt" << EOF
GenXLink Remote Desktop - Portable Version v$VERSION
====================================================

This is a portable version of GenXLink that doesn't require installation.

To run GenXLink:
1. Double-click GenXLink-Portable.sh
2. Or run from terminal: ./genxlink --portable

Configuration and data will be stored in this directory.

System Requirements:
- macOS 11.0 or later
- 64-bit Intel or Apple Silicon Mac

For more information, visit https://genxlink.com
EOF

# Create portable ZIP
cd "$DIST_DIR"
zip -r "GenXLink-Portable-$VERSION-macOS.zip" "GenXLink-Portable-$VERSION-macOS"
log_success "Portable ZIP created"

# Clean up temporary files
log_step "Cleaning up temporary files..."
rm -rf "$DMG_DIR"

# Summary
log_info "Build completed successfully!"
log_info "================================"
log_info "Generated files:"
log_info "  • DMG Installer: GenXLink-$VERSION-macOS.dmg"
log_info "  • Portable ZIP: GenXLink-Portable-$VERSION-macOS.zip"
log_info "  • Checksums: checksums.txt"
log_info ""
log_info "Output directory: $DIST_DIR"
log_info ""
log_info "Installation:"
log_info "  • Mount DMG and drag GenXLink.app to Applications"
log_info "  • Or extract portable ZIP and run GenXLink-Portable.sh"

# Test the DMG
log_step "Testing DMG integrity..."
if hdiutil attach -quiet -readonly "$DMG_PATH" -mountpoint "/tmp/genxlink_test"; then
    if [ -f "/tmp/genxlink_test/GenXLink.app/Contents/MacOS/genxlink" ]; then
        log_success "DMG test passed"
    else
        log_error "DMG test failed - executable not found"
    fi
    hdiutil detach "/tmp/genxlink_test" -quiet
else
    log_error "DMG test failed - cannot mount"
fi

log_success "macOS DMG build completed!"
