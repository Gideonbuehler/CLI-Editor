# Building and Deploying Axis Editor

## Cross-Compiling from Windows to Linux

### Option 1: Using WSL (Recommended)

1. Open WSL and navigate to your project:
   ```bash
   cd /mnt/c/Users/gideo/Downloads/Text\ Editor/blackarchEditor
   ```

2. Install Rust in WSL if not already installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. Build the release binary:
   ```bash
   cargo build --release
   ```

4. The Linux binary will be at:
   ```
   target/release/axis
   ```

### Option 2: Cross-Compile from Windows (More Complex)

1. Install the Linux target:
   ```powershell
   rustup target add x86_64-unknown-linux-gnu
   ```

2. Install a cross-linker. You need either:
   - **MinGW-w64**: `choco install mingw`
   - Or use **cargo-cross** with Docker

3. Using cargo-cross (requires Docker):
   ```powershell
   cargo install cross
   cross build --release --target x86_64-unknown-linux-gnu
   ```

---

## Nextcloud Setup

### Folder Structure on Nextcloud

Create a shared folder with these files:

```
axis-releases/
├── install-axis.sh          # The install script
├── axis.exe                  # Windows binary
├── axis-linux-x86_64         # Linux x86_64 binary
├── axis-linux-aarch64        # Linux ARM64 binary (optional)
├── axis-macos-x86_64         # macOS Intel binary (optional)
└── axis-macos-arm64          # macOS Apple Silicon (optional)
```

### Step-by-Step Nextcloud Setup

1. **Build your binaries:**
   - Windows: `cargo build --release` → `target/release/axis.exe`
   - Linux (in WSL): `cargo build --release` → `target/release/axis`

2. **Rename Linux binary:**
   ```bash
   cp target/release/axis axis-linux-x86_64
   ```

3. **Upload to Nextcloud:**
   - Create a folder called `axis-releases`
   - Upload `install-axis.sh`
   - Upload `axis.exe`
   - Upload `axis-linux-x86_64`

4. **Share the folder:**
   - Right-click the folder → Share
   - Create a public link
   - Note the share ID (e.g., `fe76EZTrkgWEFHo`)

5. **Update the install script:**
   Edit `install-axis.sh` and update `NEXTCLOUD_BASE` with your share URL.

### File Download URLs

Nextcloud public share URLs follow this pattern:
```
https://cloud.arcinger.com/s/{SHARE_ID}/download?path=/{FILENAME}
```

For example:
- Script: `https://cloud.arcinger.com/s/fe76EZTrkgWEFHo/download?path=/install-axis.sh`
- Linux binary: `https://cloud.arcinger.com/s/fe76EZTrkgWEFHo/download?path=/axis-linux-x86_64`

---

## Installation Commands

### Linux/macOS/WSL
```bash
curl -fsSL "https://cloud.arcinger.com/s/fe76EZTrkgWEFHo/download?path=/install-axis.sh" | bash
```

### Windows (PowerShell)
```powershell
$url = "https://cloud.arcinger.com/s/fe76EZTrkgWEFHo/download?path=/axis.exe"
$dest = "$env:USERPROFILE\bin\axis.exe"
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\bin"
Invoke-WebRequest -Uri $url -OutFile $dest
Write-Host "Installed to $dest"
```

---

## Quick Reference

| Platform | Build Command | Output Path |
|----------|---------------|-------------|
| Windows | `cargo build --release` | `target/release/axis.exe` |
| Linux (native/WSL) | `cargo build --release` | `target/release/axis` |
| Linux (cross) | `cross build --release --target x86_64-unknown-linux-gnu` | `target/x86_64-unknown-linux-gnu/release/axis` |

---

## Troubleshooting

### "curl: command not found"
Install curl: `sudo apt install curl` (Debian/Ubuntu) or `sudo dnf install curl` (Fedora)

### Binary won't run: "Permission denied"
Make it executable: `chmod +x axis`

### Binary won't run: "cannot execute binary file"
You downloaded the wrong architecture. Check with `file axis` and `uname -m`.

### Nextcloud returns HTML instead of file
Make sure your URL includes `download?path=/filename` not just the share link.
