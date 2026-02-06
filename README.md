# Install on Windows
## Prerequisites
- [VS Build tools](https://visualstudio.microsoft.com/de/downloads/?q=build+tools#visual-studio-code)
  - C++
  - dotnet
  - Maui
- [rustup](https://rustup.rs)
  - targets
    - `rustup target add x86_64-pc-windows-msvc` (default)
    - `rustup target add aarch64-linux-android`
- [android ndk](https://developer.android.com/ndk/downloads?hl=en). the [dev env script](./scripts/android-dev-env.ps1) assumes it to be installed $env:LOCALAPPDATA\Android\Ndk\r27d
- VSStudio / VSCode / Rider
- android studio might be the quickest way to setup the android emulator + toolchain

# build
binaries will be located within ./mandelbrot/bin/release
- `cargo build-windows-release`
- `cargo build-android-release` (you may have to invoke the dev env using `. .\scripts\android-dev-env.ps1` first)
