# GenXLink Linux RPM/DEB Package Specification
# This spec file builds the complete Linux packages for GenXLink

Name:           genxlink
Version:        0.2.0
Release:        1%{?dist}
Summary:        GenXLink Remote Desktop - Secure, high-performance remote desktop solution

License:        MIT
URL:            https://genxlink.com
Source0:        https://github.com/genxlink/genxlink/archive/v%{version}.tar.gz

# Build dependencies
BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  cmake
BuildRequires:  pkgconfig
BuildRequires:  openssl-devel
BuildRequires:  ffmpeg-devel
BuildRequires:  libvpx-devel
BuildRequires:  opus-devel
BuildRequires:  protobuf-devel
BuildRequires:  systemd-devel

# Runtime dependencies
Requires:       openssl >= 1.1.1
Requires:       ffmpeg >= 4.0
Requires:       libvpx >= 1.8
Requires:       opus >= 1.3
Requires:       systemd
Requires:       hicolor-icon-theme
Requires:       desktop-file-utils

# Distribution specific requirements
%if 0%{?fedora} || 0%{?rhel}
Requires:       glibc >= 2.28
%endif
%if 0%{?suse_version}
Requires:       glibc >= 2.31
%endif
%if 0%{?debian} || 0%{?ubuntu}
Requires:       libc6 >= 2.31
%endif

# Package information
Group:          Applications/Internet
Vendor:         GenXLink Innovations
Packager:       GenXLink Team <support@genxlink.com>

# Auto-provides and requires
AutoProv:       yes
AutoReqProv:    yes

# Description
%description
GenXLink is a secure, high-performance remote desktop solution that provides:
- Real-time screen sharing with hardware acceleration
- Low-latency audio streaming with noise cancellation
- Secure file transfer with resume support
- End-to-end encryption using industry-standard cryptography
- Cross-platform support (Windows, macOS, Linux)
- Enterprise-grade security and compliance features

Key features:
- Sub-100ms latency for real-time interaction
- Military-grade AES-256-GCM encryption
- Adaptive bandwidth management
- Multi-monitor support
- Session recording and playback
- Role-based access control

%package        devel
Summary:        Development files for GenXLink
Requires:       genxlink%{?_isa} = %{version}-%{release}
Group:          Development/Libraries

%description    devel
Development headers and libraries for GenXLink remote desktop.
This package includes C headers for integration with other applications
and development tools for plugin development.

%package        doc
Summary:        Documentation for GenXLink
BuildArch:      noarch
Group:          Documentation

%description    doc
Comprehensive documentation for GenXLink remote desktop including:
- User manual and getting started guide
- Administrator guide
- API documentation
- Security whitepaper
- Troubleshooting guide

%prep
%autosetup -n genxlink-%{version}

# Set up build environment
export CARGO_TARGET_DIR="%{_builddir}/target"
export GENXLINK_VERSION="%{version}"
export GENXLINK_BUILD="rpm"

%build
# Build the application
cargo build --release --bin genxlink

# Build additional components if they exist
if [ -f "src/service.rs" ]; then
    cargo build --release --bin genxlink-service
fi

if [ -f "src/cli.rs" ]; then
    cargo build --release --bin genxlink-cli
fi

# Generate man pages
if command -v help2man >/dev/null 2>&1; then
    help2man -n "GenXLink Remote Desktop" \
        --version-string="%{version}" \
        --no-info \
        "%{_builddir}/target/release/genxlink --help" > genxlink.1
fi

%install
# Create directory structure
%{__install} -d %{buildroot}%{_bindir}
%{__install} -d %{buildroot}%{_sysconfdir}/genxlink
%{__install} -d %{buildroot}%{_datadir}/genxlink
%{__install} -d %{buildroot}%{_libdir}/genxlink
%{__install} -d %{buildroot}%{_docdir}/genxlink
%{__install} -d %{buildroot}%{_mandir}/man1
%{__install} -d %{buildroot}%{_unitdir}
%{__install} -d %{buildroot}%{_datadir}/applications
%{__install} -d %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
%{__install} -d %{buildroot}%{_datadir}/polkit-1/actions
%{__install} -d %{buildroot}%{_localstatedir}/log/genxlink
%{__install} -d %{buildroot}%{_localstatedir}/lib/genxlink
%{__install} -d %{buildroot}%{_localstatedir}/cache/genxlink

# Install main executable
%{__install} -m 755 %{_builddir}/target/release/genxlink %{buildroot}%{_bindir}/

# Install additional binaries
if [ -f "%{_builddir}/target/release/genxlink-service" ]; then
    %{__install} -m 755 %{_builddir}/target/release/genxlink-service %{buildroot}%{_bindir}/
fi

if [ -f "%{_builddir}/target/release/genxlink-cli" ]; then
    %{__install} -m 755 %{_builddir}/target/release/genxlink-cli %{buildroot}%{_bindir}/
fi

# Install configuration files
%{__install} -m 644 config/default.toml %{buildroot}%{_sysconfdir}/genxlink/
%{__install} -m 644 config/logging.toml %{buildroot}%{_sysconfdir}/genxlink/
%{__install} -m 644 config/security.toml %{buildroot}%{_sysconfdir}/genxlink/

# Install documentation
%{__install} -m 644 README.md %{buildroot}%{_docdir}/genxlink/
%{__install} -m 644 LICENSE %{buildroot}%{_docdir}/genxlink/
if [ -d "docs" ]; then
    cp -r docs/* %{buildroot}%{_docdir}/genxlink/
fi

# Install man pages
if [ -f "genxlink.1" ]; then
    %{__install} -m 644 genxlink.1 %{buildroot}%{_mandir}/man1/
fi

# Install desktop file
%{__install} -m 644 deployment/linux/genxlink.desktop %{buildroot}%{_datadir}/applications/

# Install icon
if [ -f "resources/icon.png" ]; then
    %{__install} -m 644 resources/icon.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/genxlink.png
fi

# Install systemd service
%{__install} -m 644 deployment/linux/genxlink.service %{buildroot}%{_unitdir}/

# Install polkit action
%{__install} -m 644 deployment/linux/com.genxlink.genxlink.policy %{buildroot}%{_datadir}/polkit-1/actions/

# Install shared resources
if [ -d "resources" ]; then
    cp -r resources/* %{buildroot}%{_datadir}/genxlink/
fi

%check
# Run tests
cargo test --release || :

# Test binary execution
%{_builddir}/target/release/genxlink --version
%{_builddir}/target/release/genxlink --help

%pre
# Create genxlink user and group
getent group genxlink >/dev/null || groupadd -r genxlink
getent passwd genxlink >/dev/null || useradd -r -g genxlink -d %{_localstatedir}/lib/genxlink -s /sbin/nologin genxlink

%post
# Update desktop database
update-desktop-database &> /dev/null || :
update-icon-caches &> /dev/null || :

# Enable and start systemd service
%systemd_post genxlink.service

# Set up log rotation
if [ -d %{_sysconfdir}/logrotate.d ]; then
    cat > %{_sysconfdir}/logrotate.d/genxlink << EOF
%{_localstatedir}/log/genxlink/*.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 644 genxlink genxlink
    postrotate
        systemctl reload genxlink.service >/dev/null 2>&1 || true
    endscript
}
EOF
fi

# Set proper permissions
chown -R genxlink:genxlink %{_localstatedir}/log/genxlink
chown -R genxlink:genxlink %{_localstatedir}/lib/genxlink
chown -R genxlink:genxlink %{_localstatedir}/cache/genxlink

%preun
# Stop and disable service before removal
%systemd_preun genxlink.service

%postun
# Update desktop database
update-desktop-database &> /dev/null || :
update-icon-caches &> /dev/null || :

%postun -n genxlink
# Clean up user data on removal (optional)
if [ $1 -eq 0 ]; then
    # Remove user and group
    userdel -r genxlink 2>/dev/null || :
    groupdel genxlink 2>/dev/null || :
fi

%files
# License
%license LICENSE

# Documentation
%doc README.md
%docdir %{_docdir}/genxlink

# Executables
%{_bindir}/genxlink
%{_bindir}/genxlink-service
%{_bindir}/genxlink-cli

# Configuration
%dir %{_sysconfdir}/genxlink
%config(noreplace) %{_sysconfdir}/genxlink/default.toml
%config(noreplace) %{_sysconfdir}/genxlink/logging.toml
%config(noreplace) %{_sysconfdir}/genxlink/security.toml

# Shared resources
%{_datadir}/genxlink

# Desktop integration
%{_datadir}/applications/genxlink.desktop
%{_datadir}/icons/hicolor/256x256/apps/genxlink.png

# System integration
%{_unitdir}/genxlink.service
%{_datadir}/polkit-1/actions/com.genxlink.genxlink.policy

# Runtime directories
%dir %attr(755, genxlink, genxlink) %{_localstatedir}/log/genxlink
%dir %attr(755, genxlink, genxlink) %{_localstatedir}/lib/genxlink
%dir %attr(755, genxlink, genxlink) %{_localstatedir}/cache/genxlink

%files devel
# Development files
%dir %{_includedir}/genxlink
%{_includedir}/genxlink/*.h
%{_libdir}/genxlink/*.so
%{_libdir}/pkgconfig/genxlink.pc

%files doc
# Documentation files
%docdir %{_docdir}/genxlink
%{_mandir}/man1/genxlink.1*

%changelog
* Thu Dec 12 2024 GenXLink Team <support@genxlink.com> - 0.2.0-1
- Initial release of GenXLink Remote Desktop
- Complete rewrite with Rust-based architecture
- End-to-end encryption with AES-256-GCM
- Hardware-accelerated screen sharing
- Low-latency audio streaming
- Secure file transfer with resume support
- Cross-platform support
- Enterprise-grade security features

* Mon Dec 09 2024 GenXLink Team <support@genxlink.com> - 0.1.0-1
- Development release
- Basic remote desktop functionality
- Screen sharing support
- Audio streaming
- File transfer
- Security implementation
