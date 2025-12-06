# GenXLink Desktop Shortcut Creator
# Run this script to create a desktop shortcut for GenXLink

param(
    [string]$ExePath = ".\GenXLink.exe",
    [string]$IconPath = ".\genxlink.ico"
)

$WshShell = New-Object -ComObject WScript.Shell
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$ShortcutPath = Join-Path $DesktopPath "GenXLink.lnk"

# Get absolute paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ExeFullPath = if ([System.IO.Path]::IsPathRooted($ExePath)) { $ExePath } else { Join-Path $ScriptDir $ExePath }
$IconFullPath = if ([System.IO.Path]::IsPathRooted($IconPath)) { $IconPath } else { Join-Path $ScriptDir $IconPath }

# Check if exe exists
if (!(Test-Path $ExeFullPath)) {
    # Try dist folder
    $ExeFullPath = Join-Path (Split-Path $ScriptDir -Parent) "dist\windows-portable\GenXLink.exe"
    $IconFullPath = Join-Path (Split-Path $ScriptDir -Parent) "assets\icons\genxlink.ico"
}

if (!(Test-Path $ExeFullPath)) {
    Write-Host "Error: GenXLink.exe not found!" -ForegroundColor Red
    Write-Host "Please run this script from the GenXLink installation folder."
    exit 1
}

# Create shortcut
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $ExeFullPath
$Shortcut.WorkingDirectory = Split-Path $ExeFullPath -Parent
$Shortcut.Description = "GenXLink Remote Desktop"

if (Test-Path $IconFullPath) {
    $Shortcut.IconLocation = "$IconFullPath,0"
}

$Shortcut.Save()

Write-Host "Desktop shortcut created successfully!" -ForegroundColor Green
Write-Host "Location: $ShortcutPath"
