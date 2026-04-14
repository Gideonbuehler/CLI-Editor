# Download and install Axis editor
$installDir = "$env:LOCALAPPDATA\axis"
$binUrl = "https://cloud.arcinger.com/s/fe76EZTrkgWEFHo/download?path=/axis.exe"

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Invoke-WebRequest -Uri $binUrl -OutFile "$installDir\axis.exe"

# Add to PATH
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$path;$installDir", "User")
}

Write-Host "Axis installed! Restart terminal and run 'axis'"
