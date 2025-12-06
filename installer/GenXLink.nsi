; GenXLink Installer Script
; NSIS Modern UI Installer

!include "MUI2.nsh"
!include "FileFunc.nsh"

; General
Name "GenXLink"
OutFile "..\dist\GenXLink-v1.0.0-Setup.exe"
InstallDir "$PROGRAMFILES64\GenXLink"
InstallDirRegKey HKLM "Software\GenXLink" "InstallDir"
RequestExecutionLevel admin

; Version Info
VIProductVersion "1.0.0.0"
VIAddVersionKey "ProductName" "GenXLink"
VIAddVersionKey "CompanyName" "GenXis Innovations"
VIAddVersionKey "LegalCopyright" "Copyright (c) 2025 GenXis Innovations"
VIAddVersionKey "FileDescription" "GenXLink Remote Desktop"
VIAddVersionKey "FileVersion" "1.0.0"
VIAddVersionKey "ProductVersion" "1.0.0"

; Icon
!define MUI_ICON "..\assets\icons\genxlink.ico"
!define MUI_UNICON "..\assets\icons\genxlink.ico"

; UI Settings
!define MUI_ABORTWARNING
!define MUI_WELCOMEFINISHPAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Wizard\win.bmp"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE "English"

; Installer Section
Section "Install"
    SetOutPath "$INSTDIR"
    
    ; Copy main executable
    File "..\dist\windows-portable\GenXLink.exe"
    
    ; Copy icon
    File "..\assets\icons\genxlink.ico"
    
    ; Create Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\GenXLink"
    CreateShortcut "$SMPROGRAMS\GenXLink\GenXLink.lnk" "$INSTDIR\GenXLink.exe" "" "$INSTDIR\genxlink.ico" 0
    CreateShortcut "$SMPROGRAMS\GenXLink\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    
    ; Create Desktop shortcut
    CreateShortcut "$DESKTOP\GenXLink.lnk" "$INSTDIR\GenXLink.exe" "" "$INSTDIR\genxlink.ico" 0
    
    ; Write registry keys
    WriteRegStr HKLM "Software\GenXLink" "InstallDir" "$INSTDIR"
    WriteRegStr HKLM "Software\GenXLink" "Version" "1.0.0"
    
    ; Add to Add/Remove Programs
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "DisplayName" "GenXLink"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "DisplayIcon" "$INSTDIR\genxlink.ico"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "Publisher" "GenXis Innovations"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "DisplayVersion" "1.0.0"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "URLInfoAbout" "https://github.com/AeroSage/GenXLink"
    
    ; Get installed size
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink" "EstimatedSize" "$0"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

; Uninstaller Section
Section "Uninstall"
    ; Remove files
    Delete "$INSTDIR\GenXLink.exe"
    Delete "$INSTDIR\genxlink.ico"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    
    ; Remove shortcuts
    Delete "$DESKTOP\GenXLink.lnk"
    Delete "$SMPROGRAMS\GenXLink\GenXLink.lnk"
    Delete "$SMPROGRAMS\GenXLink\Uninstall.lnk"
    RMDir "$SMPROGRAMS\GenXLink"
    
    ; Remove registry keys
    DeleteRegKey HKLM "Software\GenXLink"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GenXLink"
SectionEnd
