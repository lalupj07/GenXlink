Name: genxlink
Version: 1.0.0
Release: 1
Summary: Remote Desktop Platform
License: MIT
URL: https://github.com/genxlink/genxlink

%description
GenXLink is a high-performance remote desktop platform
featuring WebRTC media streaming, end-to-end encryption,
and cross-platform support.

%prep
%build
cargo build --release

%install
mkdir -p %{buildroot}/usr/bin
cp target/release/api-server %{buildroot}/usr/bin/
cp target/release/signaling-server %{buildroot}/usr/bin/

%files
/usr/bin/api-server
/usr/bin/signaling-server
