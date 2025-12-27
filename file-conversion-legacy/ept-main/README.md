# EPT

A Tauri application built with vanilla HTML, CSS, and TypeScript.

## Prerequisites

Before installing, ensure you have the following installed on your system:

### Windows

1. **Node.js** (v18 or higher)
   - Download from [nodejs.org](https://nodejs.org/)
   - Or use a package manager like [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/): `winget install OpenJS.NodeJS`

2. **Rust** (latest stable version)
   - Download from [rustup.rs](https://rustup.rs/)
   - Or use PowerShell: `Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile rustup-init.exe; ./rustup-init.exe`
   - After installation, restart your terminal

3. **Microsoft Visual C++ Build Tools**
   - Required for compiling Rust on Windows
   - Download from [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
   - Or install via winget: `winget install Microsoft.VisualStudio.BuildTools`
   - Ensure you add the 'Desktop Development with C++' (defaults work)

4. **WebView2** (usually pre-installed on Windows 10/11)
   - If not installed, download from [Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)

### macOS

1. **Node.js** (v18 or higher)
   - Download from [nodejs.org](https://nodejs.org/)
   - Or use Homebrew: `brew install node`

2. **Rust** (latest stable version)
   - Download from [rustup.rs](https://rustup.rs/)
   - Or use Homebrew: `brew install rust`
   - After installation, restart your terminal

3. **Xcode Command Line Tools**
   - Run: `xcode-select --install`
   - This provides the necessary C compiler and build tools

## Installation

1. **Clone the repository** (if you haven't already):
   ```bash
   git clone <repository-url>
   cd ept
   ```

2. **Install Node.js dependencies**:
   ```bash
   npm install
   ```

3. **Verify Rust installation**:
   ```bash
   rustc --version
   cargo --version
   ```

4. **Install Tauri CLI** (if not already installed globally):
   ```bash
   npm install -g save-dev @tauri-apps/cli
   ```

## Development

### Running the Development Server

Start the development server with hot-reload:

```bash
npm run tauri:dev
```

This will:
- Start the Vite development server for the frontend
- Launch the Tauri application window
- Enable hot-reload for both frontend and backend changes

### Building for Production

#### Windows

Build the Windows executable:

```bash
npm run tauri build
```

The built application will be located in:
- `src-tauri/target/release/ept.exe` (executable)
- `src-tauri/target/release/bundle/msi/ept_0.1.0_x64_en-US.msi` (installer)

#### macOS

Build the macOS application:

```bash
npm run tauri build
```

The built application will be located in:
- `src-tauri/target/release/bundle/macos/ept.app` (application bundle)
- `src-tauri/target/release/bundle/dmg/ept_0.1.0_x64.dmg` (disk image)

**Note**: On macOS, you may need to sign the application for distribution. For development builds, you may need to allow the app to run in System Settings > Privacy & Security.

## Usage

### Development Mode

1. Run `npm run dev` to start the development server
2. The Tauri application window will open automatically
3. Make changes to your code - the app will reload automatically

### Production Build

1. Run `npm run tauri build` to create a production build
2. Locate the built application in the `src-tauri/target/release/bundle/` directory
3. Install or run the application as appropriate for your platform

### Troubleshooting

#### Windows

- **"link.exe not found"**: Install Microsoft Visual C++ Build Tools
- **"WebView2 not found"**: Install Microsoft Edge WebView2 runtime
- **Rust compilation errors**: Ensure you have the latest stable Rust toolchain: `rustup update stable`

#### macOS

- **"xcrun: error: invalid active developer path"**: Run `xcode-select --install`
- **App won't open after build**: Right-click the app and select "Open", or run: `xattr -cr /path/to/ept.app`
- **Code signing warnings**: For development, you can disable code signing in `tauri.conf.json` (not recommended for production)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
