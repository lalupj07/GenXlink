# Testing GenXLink Installer

## Quick Test Checklist

### 1. Run the Installer
```powershell
.\dist\GenXLink-Setup-1.0.0.exe
```

### 2. Installation Steps
- [ ] Installer opens successfully
- [ ] License agreement displays
- [ ] Installation directory shown (default: C:\Program Files\GenXLink)
- [ ] Options displayed:
  - [ ] Create desktop icon
  - [ ] Create quick launch icon
  - [ ] Auto-start with Windows
- [ ] Installation completes without errors
- [ ] "Launch GenXLink" option at end

### 3. Test Installed Application
- [ ] Desktop icon created (if selected)
- [ ] Start menu entry exists
- [ ] Application launches from desktop icon
- [ ] Application launches from Start menu
- [ ] Main window displays correctly
- [ ] Settings panel opens
- [ ] Premium features panel opens
- [ ] Application closes properly

### 4. Test Auto-Start (if enabled)
- [ ] Registry entry created in:
  `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
- [ ] Application starts with Windows (test by restarting)

### 5. Test Uninstaller
- [ ] Open "Add or Remove Programs"
- [ ] Find "GenXLink" in the list
- [ ] Click "Uninstall"
- [ ] Uninstaller runs successfully
- [ ] All files removed from installation directory
- [ ] Desktop icon removed
- [ ] Start menu entry removed
- [ ] Registry entries cleaned up

## Expected Results

### Installation
- **Time:** ~30 seconds
- **Size:** ~5 MB installed
- **Location:** C:\Program Files\GenXLink\
- **Files:**
  - genxlink.exe
  - README.md
  - LICENSE
  - unins000.exe (uninstaller)
  - unins000.dat

### First Launch
- Window opens at 1200x800 pixels
- Dark theme applied
- Welcome screen shows:
  - "Welcome to GenXLink"
  - Feature list
  - Settings button
  - Premium Plans button

## Troubleshooting

### If installer doesn't run:
- Right-click → "Run as administrator"
- Check Windows SmartScreen (click "More info" → "Run anyway")

### If application doesn't launch:
- Check Windows Event Viewer for errors
- Verify all files were installed
- Try running from installation directory directly

### If uninstall fails:
- Run uninstaller as administrator
- Manually delete installation directory if needed
- Clean registry entries manually if needed

## Sign-Off

After testing, confirm:
- [ ] Installation works smoothly
- [ ] Application runs correctly
- [ ] Uninstallation is clean
- [ ] Ready for certification submission

**Tester:** _______________
**Date:** _______________
**Status:** ☐ PASS  ☐ FAIL
**Notes:** _______________________________________________
