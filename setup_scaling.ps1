$ErrorActionPreference = "Stop"
$settingsPath = "$env:LOCALAPPDATA\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"

Write-Host "Configuring Axis Editor Scaling Shortcuts..." -ForegroundColor Cyan

if (-not (Test-Path $settingsPath)) {
    Write-Host "Error: Windows Terminal settings file not found." -ForegroundColor Red
    Write-Host "Make sure Windows Terminal is installed and has been run at least once."
    exit 1
}

# Read content
try {
    $content = Get-Content $settingsPath -Raw
    # Simple comment stripping (standard user settings often have comments)
    $jsonContent = $content -replace "(?m)^\s*//.*$","" 
    $settings = $jsonContent | ConvertFrom-Json
} catch {
    Write-Host "Error parsing settings.json. The file format might be invalid or contain complex comments." -ForegroundColor Red
    Write-Host $_
    exit 1
}

$hasChanges = $false

# Ensure keybindings array exists
if ($null -eq $settings.keybindings) {
    $settings | Add-Member -MemberType NoteProperty -Name "keybindings" -Value @()
}

# Define the bindings
$scaleUp = @{ command = @{ action = "adjustFontSize"; delta = 1 }; keys = "ctrl+." }
$scaleDown = @{ command = @{ action = "adjustFontSize"; delta = -1 }; keys = "ctrl+," }

# Check for existing bindings to avoid duplicates
$existingKeys = $settings.keybindings | ForEach-Object { $_.keys }

if ($existingKeys -notcontains "ctrl+.") {
    $settings.keybindings += $scaleUp
    Write-Host "Added Ctrl+. (Scale Up)" -ForegroundColor Green
    $hasChanges = $true
} else {
    Write-Host "Ctrl+. is already configured." -ForegroundColor Yellow
}

if ($existingKeys -notcontains "ctrl+,") {
    $settings.keybindings += $scaleDown
    Write-Host "Added Ctrl+, (Scale Down)" -ForegroundColor Green
    $hasChanges = $true
} else {
    Write-Host "Ctrl+, is already configured." -ForegroundColor Yellow
}

if ($hasChanges) {
    try {
        $settings | ConvertTo-Json -Depth 100 | Set-Content $settingsPath -Encoding UTF8
        Write-Host "Windows Terminal settings updated successfully!" -ForegroundColor Green
        Write-Host "You may need to restart Windows Terminal for changes to take effect."
    } catch {
        Write-Host "Failed to save settings file." -ForegroundColor Red
        Write-Host $_
    }
} else {
    Write-Host "Configuration is already correct." -ForegroundColor Green
}

Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
