; GenXLink NSIS Installer Script v1.0.0
!define APPNAME "GenXLink"
!define VERSION "1.0.0"
!define PUBLISHER "GenXis Innovations"

Name "${APPNAME} v${VERSION}"
OutFile "GenXLink-v${VERSION}-Windows-Setup.exe"
InstallDir "$PROGRAMFILES64\${APPNAME}"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    File "dist\api-server.exe"
    File "dist\signaling-server.exe"
    File "dist\start-genxlink.bat"
    CreateShortCut "$DESKTOP\${APPNAME}.lnk"
SectionEnd
