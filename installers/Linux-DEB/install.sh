#!/bin/bash
echo 'Installing GenXLink v1.0.0...'
mkdir -p /usr/local/bin
mkdir -p /usr/share/doc/genxlink
cp api-server.exe /usr/local/bin/
cp signaling-server.exe /usr/local/bin/
cp README.md /usr/share/doc/genxlink/
echo 'Installation complete!'
