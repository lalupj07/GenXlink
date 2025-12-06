#!/bin/bash
# GenXLink DMG Creator for macOS v1.0.0

VERSION='1.0.0'
SOURCE='GenXLink'
DMG_NAME='GenXLink-v1.0.0-macOS'

echo 'Creating DMG package...'
hdiutil create -volname 'GenXLink v1.0.0' -srcfolder "$SOURCE" -ov -format UDZO "$DMG_NAME.dmg"
echo 'DMG created: $DMG_NAME.dmg'
