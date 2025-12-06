# GenXLink MSI Installer Script v1.0.0
param(
    [string]$InstallPath = "$env:ProgramFiles\GenXLink"
)

Write-Host "Installing GenXLink v1.0.0" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green

# Create installation directory
if (!(Test-Path $InstallPath)) {
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

# Copy files
$files = @(
    "api-server.exe",
    "signaling-server.exe", 
    "start-genxlink.bat",
    "README.md"
)

foreach ($file in $files) {
    $source = Join-Path $PSScriptRoot $file
    if (Test-Path $source) {
        Copy-Item $source $InstallPath -Force
        Write-Host "Installed $file"
    }
}

# Create desktop shortcut
$shell = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath("Desktop")
$shortcut = $shell.CreateShortcut((Join-Path $desktop "GenXLink.lnk"))
$shortcut.TargetPath = Join-Path $InstallPath "start-genxlink.bat"
$shortcut.WorkingDirectory = $InstallPath
$shortcut.Description = "Start GenXLink Remote Desktop Servers"
$shortcut.Save()

Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "Installation Path: $InstallPath"
Write-Host "Desktop shortcut created"
