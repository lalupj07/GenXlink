#!/bin/bash

# GenXLink Linux Package Build Script
# This script builds RPM and DEB packages for GenXLink

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"
VERSION="${GENXLINK_VERSION:-0.2.0}"
BUILD_DIR="${PROJECT_ROOT}/target/release"
PACKAGE_DIR="${PROJECT_ROOT}/deployment/linux/packages"
DIST_DIR="${PROJECT_ROOT}/dist"
PACKAGE_NAME="genxlink"

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

# Detect distribution
detect_distribution() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif command -v lsb_release >/dev/null 2>&1; then
        lsb_release -si | tr '[:upper:]' '[:lower:]'
    else
        echo "unknown"
    fi
}

# Main script starts here
log_info "GenXLink Linux Package Build Script v$VERSION"
log_info "============================================="

# Check prerequisites
log_step "Checking prerequisites..."

# Check for build tools
for tool in cargo rpmbuild dpkg-deb fakeroot; do
    if ! command -v "$tool" &> /dev/null; then
        log_error "$tool not found"
        case "$tool" in
            cargo)
                log_info "Install Rust from https://rustup.rs/"
                ;;
            rpmbuild)
                log_info "Install with: sudo apt install rpm (Ubuntu/Debian) or sudo yum install rpm-build (RHEL/CentOS)"
                ;;
            dpkg-deb)
                log_info "Install with: sudo apt install dpkg-dev (Ubuntu/Debian)"
                ;;
            fakeroot)
                log_info "Install with: sudo apt install fakeroot (Ubuntu/Debian)"
                ;;
        esac
        exit 1
    fi
done
log_success "Build tools found"

# Check for development libraries
log_step "Checking development libraries..."

missing_libs=()
check_lib() {
    local lib=$1
    local package=$2
    
    if ! pkg-config --exists "$lib" 2>/dev/null; then
        missing_libs+=("$package")
    fi
}

check_lib "openssl" "libssl-dev (Ubuntu/Debian) or openssl-devel (RHEL/CentOS)"
check_lib "libavcodec" "libavcodec-dev (Ubuntu/Debian) or ffmpeg-devel (RHEL/CentOS)"
check_lib "vpx" "libvpx-dev (Ubuntu/Debian) or libvpx-devel (RHEL/CentOS)"
check_lib "opus" "libopus-dev (Ubuntu/Debian) or opus-devel (RHEL/CentOS)"

if [ ${#missing_libs[@]} -gt 0 ]; then
    log_error "Missing development libraries:"
    for lib in "${missing_libs[@]}"; do
        echo "  - $lib"
    done
    exit 1
fi
log_success "Development libraries found"

# Clean previous builds
log_step "Cleaning previous builds..."
rm -rf "$PACKAGE_DIR"
rm -rf "$DIST_DIR"
mkdir -p "$PACKAGE_DIR"
mkdir -p "$DIST_DIR"

# Set environment variables
export GENXLINK_VERSION="$VERSION"
export GENXLINK_BUILD="package"

# Build the application
log_step "Building GenXLink application..."

cd "$PROJECT_ROOT"
cargo build --release --bin genxlink

# Build additional components
if [ -f "src/service.rs" ]; then
    cargo build --release --bin genxlink-service
fi

if [ -f "src/cli.rs" ]; then
    cargo build --release --bin genxlink-cli
fi

log_success "Application built"

# Run tests
log_step "Running tests..."
cargo test --release || log_warning "Some tests failed, continuing with build..."

# Get distribution information
DISTRO=$(detect_distribution)
log_info "Detected distribution: $DISTRO"

# Build DEB package (Ubuntu/Debian)
if command -v dpkg-deb &> /dev/null; then
    log_step "Building DEB package..."
    
    DEB_DIR="$PACKAGE_DIR/deb"
    mkdir -p "$DEB_DIR"
    
    # Create directory structure
    DEB_ROOT="$DEB_DIR/genxlink"
    mkdir -p "$DEB_ROOT/DEBIAN"
    mkdir -p "$DEB_ROOT/usr/bin"
    mkdir -p "$DEB_ROOT/etc/genxlink"
    mkdir -p "$DEB_ROOT/usr/share/genxlink"
    mkdir -p "$DEB_ROOT/usr/share/doc/genxlink"
    mkdir -p "$DEB_ROOT/usr/share/man/man1"
    mkdir -p "$DEB_ROOT/lib/systemd/system"
    mkdir -p "$DEB_ROOT/usr/share/applications"
    mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$DEB_ROOT/usr/share/polkit-1/actions"
    mkdir -p "$DEB_ROOT/var/log/genxlink"
    mkdir -p "$DEB_ROOT/var/lib/genxlink"
    mkdir -p "$DEB_ROOT/var/cache/genxlink"
    
    # Copy executables
    cp "$BUILD_DIR/genxlink" "$DEB_ROOT/usr/bin/"
    if [ -f "$BUILD_DIR/genxlink-service" ]; then
        cp "$BUILD_DIR/genxlink-service" "$DEB_ROOT/usr/bin/"
    fi
    if [ -f "$BUILD_DIR/genxlink-cli" ]; then
        cp "$BUILD_DIR/genxlink-cli" "$DEB_ROOT/usr/bin/"
    fi
    
    # Copy configuration files
    cp config/default.toml "$DEB_ROOT/etc/genxlink/"
    cp config/logging.toml "$DEB_ROOT/etc/genxlink/"
    cp config/security.toml "$DEB_ROOT/etc/genxlink/"
    
    # Copy documentation
    cp README.md "$DEB_ROOT/usr/share/doc/genxlink/"
    cp LICENSE "$DEB_ROOT/usr/share/doc/genxlink/"
    if [ -d "docs" ]; then
        cp -r docs/* "$DEB_ROOT/usr/share/doc/genxlink/"
    fi
    
    # Copy desktop file
    if [ -f "deployment/linux/genxlink.desktop" ]; then
        cp deployment/linux/genxlink.desktop "$DEB_ROOT/usr/share/applications/"
    fi
    
    # Copy icon
    if [ -f "resources/icon.png" ]; then
        cp resources/icon.png "$DEB_ROOT/usr/share/icons/hicolor/256x256/apps/genxlink.png"
    fi
    
    # Copy systemd service
    if [ -f "deployment/linux/genxlink.service" ]; then
        cp deployment/linux/genxlink.service "$DEB_ROOT/lib/systemd/system/"
    fi
    
    # Copy polkit action
    if [ -f "deployment/linux/com.genxlink.genxlink.policy" ]; then
        cp deployment/linux/com.genxlink.genxlink.policy "$DEB_ROOT/usr/share/polkit-1/actions/"
    fi
    
    # Copy resources
    if [ -d "resources" ]; then
        cp -r resources/* "$DEB_ROOT/usr/share/genxlink/"
    fi
    
    # Create DEB control file
    cat > "$DEB_ROOT/DEBIAN/control" << EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: net
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.31), libssl1.1 (>= 1.1.1), libavcodec58 (>= 4.0), libvpx6 (>= 1.8), libopus0 (>= 1.3), systemd
Maintainer: GenXLink Team <support@genxlink.com>
Description: GenXLink Remote Desktop - Secure, high-performance remote desktop solution
 GenXLink is a secure, high-performance remote desktop solution that provides:
  - Real-time screen sharing with hardware acceleration
  - Low-latency audio streaming with noise cancellation
  - Secure file transfer with resume support
  - End-to-end encryption using industry-standard cryptography
  - Cross-platform support (Windows, macOS, Linux)
  - Enterprise-grade security and compliance features
EOF

    # Create postinst script
    cat > "$DEB_ROOT/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# Update desktop database
update-desktop-database &> /dev/null || true
update-icon-caches &> /dev/null || true

# Create genxlink user
if ! getent group genxlink >/dev/null; then
    groupadd -r genxlink
fi
if ! getent passwd genxlink >/dev/null; then
    useradd -r -g genxlink -d /var/lib/genxlink -s /sbin/nologin genxlink
fi

# Set permissions
chown -R genxlink:genxlink /var/log/genxlink
chown -R genxlink:genxlink /var/lib/genxlink
chown -R genxlink:genxlink /var/cache/genxlink

# Enable systemd service
systemctl daemon-reload
systemctl enable genxlink.service || true

exit 0
EOF

    # Create prerm script
    cat > "$DEB_ROOT/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e

# Stop and disable service
if systemctl is-active --quiet genxlink.service; then
    systemctl stop genxlink.service
fi
systemctl disable genxlink.service || true

exit 0
EOF

    # Create postrm script
    cat > "$DEB_ROOT/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

# Remove user and group on purge
if [ "$1" = "purge" ]; then
    if getent passwd genxlink >/dev/null; then
        userdel -r genxlink
    fi
    if getent group genxlink >/dev/null; then
        groupdel genxlink
    fi
fi

exit 0
EOF

    # Make scripts executable
    chmod 755 "$DEB_ROOT/DEBIAN/postinst"
    chmod 755 "$DEB_ROOT/DEBIAN/prerm"
    chmod 755 "$DEB_ROOT/DEBIAN/postrm"

    # Calculate installed size
    INSTALLED_SIZE=$(du -s "$DEB_ROOT" | cut -f1)
    echo "Installed-Size: $INSTALLED_SIZE" >> "$DEB_ROOT/DEBIAN/control"

    # Build DEB package
    cd "$DEB_DIR"
    fakeroot dpkg-deb --build genxlink
    cd "$PROJECT_ROOT"

    # Copy to distribution directory
    cp "$DEB_DIR/genxlink.deb" "$DIST_DIR/genxlink_${VERSION}_amd64.deb"
    log_success "DEB package created: genxlink_${VERSION}_amd64.deb"
fi

# Build RPM package (RHEL/CentOS/Fedora)
if command -v rpmbuild &> /dev/null; then
    log_step "Building RPM package..."
    
    # Create rpmbuild directory structure
    mkdir -p "$HOME/rpmbuild/"{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    
    # Create source tarball
    tar -czf "$HOME/rpmbuild/SOURCES/genxlink-${VERSION}.tar.gz" \
        --exclude=target \
        --exclude=.git \
        --exclude=dist \
        --exclude=deployment \
        .
    
    # Copy spec file
    cp "$SCRIPT_DIR/genxlink.spec" "$HOME/rpmbuild/SPECS/"
    
    # Build RPM
    rpmbuild -ba "$HOME/rpmbuild/SPECS/genxlink.spec" \
        --define "_version $VERSION" \
        --define "_release 1"
    
    # Copy RPM to distribution directory
    if [ -f "$HOME/rpmbuild/RPMS/x86_64/genxlink-${VERSION}-1.x86_64.rpm" ]; then
        cp "$HOME/rpmbuild/RPMS/x86_64/genxlink-${VERSION}-1.x86_64.rpm" "$DIST_DIR/"
        log_success "RPM package created: genxlink-${VERSION}-1.x86_64.rpm"
    fi
    
    if [ -f "$HOME/rpmbuild/SRPMS/genxlink-${VERSION}-1.src.rpm" ]; then
        cp "$HOME/rpmbuild/SRPMS/genxlink-${VERSION}-1.src.rpm" "$DIST_DIR/"
        log_success "Source RPM created: genxlink-${VERSION}-1.src.rpm"
    fi
fi

# Create AppImage package
log_step "Creating AppImage package..."

APPIMAGE_DIR="$PACKAGE_DIR/appimage"
mkdir -p "$APPIMAGE_DIR"

# Create AppDir structure
APPDIR="$APPIMAGE_DIR/GenXLink.AppDir"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR"

# Copy application files
cp "$BUILD_DIR/genxlink" "$APPDIR/usr/bin/"
cp "$BUILD_DIR"/*.so "$APPDIR/usr/lib/" 2>/dev/null || true

# Copy desktop file
if [ -f "deployment/linux/genxlink.desktop" ]; then
    cp deployment/linux/genxlink.desktop "$APPDIR/usr/share/applications/"
fi

# Copy icon
if [ -f "resources/icon.png" ]; then
    cp resources/icon.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/genxlink.png"
    cp resources/icon.png "$APPDIR/genxlink.png"
fi

# Create AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/genxlink" "$@"
EOF
chmod +x "$APPDIR/AppRun"

# Download appimagetool if not present
APPIMAGETOOL="$APPIMAGE_DIR/appimagetool-x86_64.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    wget -O "$APPIMAGETOOL" \
        "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

# Create AppImage
"$APPIMAGETOOL" "$APPDIR" "$DIST_DIR/GenXLink-${VERSION}-x86_64.AppImage"
log_success "AppImage created: GenXLink-${VERSION}-x86_64.AppImage"

# Create portable tarball
log_step "Creating portable tarball..."

PORTABLE_DIR="$PACKAGE_DIR/portable"
mkdir -p "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64"

# Copy portable files
cp "$BUILD_DIR/genxlink" "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/"
cp "$BUILD_DIR"/*.so "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/" 2>/dev/null || true
cp -r config "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/"
cp README.md "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/"
cp LICENSE "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/"

# Create portable launcher
cat > "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/genxlink-portable.sh" << 'EOF'
#!/bin/bash
# GenXLink Portable Launcher

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

export LD_LIBRARY_PATH="$SCRIPT_DIR:$LD_LIBRARY_PATH"
export GENXLINK_CONFIG_DIR="$SCRIPT_DIR/config"
export GENXLINK_DATA_DIR="$SCRIPT_DIR/data"

echo "Starting GenXLink Remote Desktop (Portable)..."
./genxlink --portable --config "./config/default.toml"
EOF
chmod +x "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/genxlink-portable.sh"

# Create portable README
cat > "$PORTABLE_DIR/genxlink-$VERSION-linux-x86_64/README.txt" << EOF
GenXLink Remote Desktop - Portable Version v$VERSION
====================================================

This is a portable version of GenXLink that doesn't require installation.

To run GenXLink:
1. Double-click genxlink-portable.sh
2. Or run from terminal: ./genxlink-portable.sh

Configuration and data will be stored in this directory.

System Requirements:
- Linux x86_64
- glibc 2.31 or later
- OpenGL 3.3 or later (for hardware acceleration)

Dependencies:
The portable version includes most dependencies, but you may need:
- libssl.so.1.1
- libavcodec.so.58
- libvpx.so.6
- libopus.so.0

For more information, visit https://genxlink.com
EOF

# Create tarball
cd "$PACKAGE_DIR/portable"
tar -czf "$DIST_DIR/genxlink-$VERSION-linux-x86_64-portable.tar.gz" "genxlink-$VERSION-linux-x86_64"
cd "$PROJECT_ROOT"
log_success "Portable tarball created: genxlink-$VERSION-linux-x86_64-portable.tar.gz"

# Generate checksums
log_step "Generating checksums..."
cd "$DIST_DIR"
sha256sum *.deb *.rpm *.AppImage *.tar.gz > checksums.txt
log_success "Checksums generated"

# Create installation scripts
log_step "Creating installation scripts..."

# DEB installation script
cat > "$DIST_DIR/install-deb.sh" << 'EOF'
#!/bin/bash
# GenXLink DEB Package Installer

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEB_FILE=$(find "$SCRIPT_DIR" -name "genxlink_*_amd64.deb" | head -n1)

if [ -z "$DEB_FILE" ]; then
    echo "Error: DEB package not found"
    exit 1
fi

echo "Installing GenXLink from DEB package..."
echo "Package: $DEB_FILE"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges. Using sudo..."
    sudo dpkg -i "$DEB_FILE"
    sudo apt-get install -f  # Fix dependencies
else
    dpkg -i "$DEB_FILE"
    apt-get install -f  # Fix dependencies
fi

echo "GenXLink installation completed!"
echo "Start with: genxlink"
echo "Or enable service: sudo systemctl enable --now genxlink"
EOF

# RPM installation script
cat > "$DIST_DIR/install-rpm.sh" << 'EOF'
#!/bin/bash
# GenXLink RPM Package Installer

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RPM_FILE=$(find "$SCRIPT_DIR" -name "genxlink-*.x86_64.rpm" | head -n1)

if [ -z "$RPM_FILE" ]; then
    echo "Error: RPM package not found"
    exit 1
fi

echo "Installing GenXLink from RPM package..."
echo "Package: $RPM_FILE"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "This script requires root privileges. Using sudo..."
    sudo rpm -ivh "$RPM_FILE"
else
    rpm -ivh "$RPM_FILE"
fi

echo "GenXLink installation completed!"
echo "Start with: genxlink"
echo "Or enable service: sudo systemctl enable --now genxlink"
EOF

# AppImage installation script
cat > "$DIST_DIR/install-appimage.sh" << 'EOF'
#!/bin/bash
# GenXLink AppImage Installer

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APPIMAGE_FILE=$(find "$SCRIPT_DIR" -name "GenXLink-*.AppImage" | head -n1)

if [ -z "$APPIMAGE_FILE" ]; then
    echo "Error: AppImage not found"
    exit 1
fi

INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"

echo "Installing GenXLink AppImage..."
echo "AppImage: $APPIMAGE_FILE"
echo "Install directory: $INSTALL_DIR"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Copy AppImage
cp "$APPIMAGE_FILE" "$INSTALL_DIR/genxlink.AppImage"
chmod +x "$INSTALL_DIR/genxlink.AppImage"

# Create desktop entry
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/genxlink.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=GenXLink Remote Desktop
Comment=Secure, high-performance remote desktop solution
Exec=$INSTALL_DIR/genxlink.AppImage
Icon=genxlink
Terminal=false
Categories=Network;RemoteAccess;
EOF

# Create symlink for command line use
if [ -d "$HOME/.local/bin" ]; then
    ln -sf "$INSTALL_DIR/genxlink.AppImage" "$HOME/.local/bin/genxlink"
    echo "Command 'genxlink' is now available"
fi

echo "GenXLink AppImage installation completed!"
echo "Run with: $INSTALL_DIR/genxlink.AppImage"
echo "Or from applications menu"
EOF

chmod +x "$DIST_DIR/install-deb.sh"
chmod +x "$DIST_DIR/install-rpm.sh"
chmod +x "$DIST_DIR/install-appimage.sh"

# Clean up temporary files
log_step "Cleaning up temporary files..."
rm -rf "$PACKAGE_DIR"

# Summary
log_info "Build completed successfully!"
log_info "================================"
log_info "Generated packages:"

for file in "$DIST_DIR"/*.{deb,rpm,AppImage,tar.gz}; do
    if [ -f "$file" ]; then
        size=$(du -h "$file" | cut -f1)
        log_info "  • $(basename "$file") ($size)"
    fi
done

log_info ""
log_info "Installation scripts:"
log_info "  • install-deb.sh - For Ubuntu/Debian systems"
log_info "  • install-rpm.sh - For RHEL/CentOS/Fedora systems"
log_info "  • install-appimage.sh - Universal Linux installer"
log_info ""
log_info "Output directory: $DIST_DIR"

# Test packages
log_step "Testing packages..."

# Test DEB package
DEB_FILE=$(find "$DIST_DIR" -name "genxlink_*_amd64.deb" | head -n1)
if [ -n "$DEB_FILE" ] && command -v dpkg-deb &> /dev/null; then
    if dpkg-deb --info "$DEB_FILE" &> /dev/null; then
        log_success "DEB package test passed"
    else
        log_error "DEB package test failed"
    fi
fi

# Test RPM package
RPM_FILE=$(find "$DIST_DIR" -name "genxlink-*.x86_64.rpm" | head -n1)
if [ -n "$RPM_FILE" ] && command -v rpm &> /dev/null; then
    if rpm -qip "$RPM_FILE" &> /dev/null; then
        log_success "RPM package test passed"
    else
        log_error "RPM package test failed"
    fi
fi

# Test AppImage
APPIMAGE_FILE=$(find "$DIST_DIR" -name "GenXLink-*.AppImage" | head -n1)
if [ -n "$APPIMAGE_FILE" ]; then
    if "$APPIMAGE_FILE" --version &> /dev/null; then
        log_success "AppImage test passed"
    else
        log_warning "AppImage test failed (may need dependencies)"
    fi
fi

log_success "Linux package build completed!"
