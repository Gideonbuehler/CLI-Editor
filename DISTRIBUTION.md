# How to Distribute Axis

## Quick Start
1. Run the `package.bat` script in this folder.
2. Open the `dist` folder.
3. Take the `axis.exe` file and copy it to any computer you want.

## Dependencies
The application is self-contained. You do **not** need to install Rust or Cargo on the other computer.

## Features Note
The **Scaling (Ctrl+, / Ctrl+.)** feature we set up relies on **Windows Terminal** settings, not the application itself. 

If you want scaling to work on the other computer:
1. Use **Windows Terminal** on that computer.
2. Open Settings (Ctrl+,).
3. Add the keybindings for `adjustFontSize` as we did on your current machine.
