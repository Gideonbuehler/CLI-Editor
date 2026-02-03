@echo off
echo Packaging axis for distribution...
mkdir dist 2>nul
copy "target\release\axis.exe" "dist\axis.exe"
copy "setup_scaling.ps1" "dist\setup_scaling.ps1"
echo.
echo Packaging complete!
echo.
echo To distribute:
echo 1. Copy the 'dist' folder to the target computer.
echo 2. Run 'axis.exe' to use the editor.
echo 3. (Optional) Right-click 'setup_scaling.ps1' and select "Run with PowerShell" 
echo    to enable Ctrl+,/Ctrl+. scaling support on that machine.
echo.
pause