use std::path::PathBuf;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Platform {
    Windows,
    Android,
    Ios,
    Osx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Arch {
    X64,
    Arm64,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DotNetVersion {
    #[value(name = "8")]
    Net8,
    #[default]
    #[value(name = "10")]
    Net10,
}

impl DotNetVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Net8 => "net8.0",
            Self::Net10 => "net10.0",
        }
    }
}
impl Platform {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "windows" => Ok(Self::Windows),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            "osx" | "macos" => Ok(Self::Osx),
            _ => anyhow::bail!("unknown platform: {s} (expected windows|android|ios|osx)"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::Android => "android",
            Platform::Ios => "ios",
            Platform::Osx => "osx",
        }
    }

    pub fn requires_macos(self) -> bool {
        matches!(self, Self::Ios | Self::Osx)
    }

    pub fn requires_windwos(self) -> bool {
        matches!(self, Self::Windows)
    }
}

impl Arch {
    #[allow(dead_code)]
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "x64" | "amd64" | "x86_64" => Ok(Self::X64),
            "arm64" | "aarch64" => Ok(Self::Arm64),
            _ => anyhow::bail!("unknown arch: {s} (expected x64|arm64)"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }
}
/// Root of the xtask crate (where xtask/Cargo.toml lives).
pub fn xtask_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Project root directory containing all subprojects.
/// Assumes xtask/ lives directly under the project root.
pub fn project_root() -> PathBuf {
    xtask_root()
        .parent()
        .expect("xtask_root() must have a parent directory")
        .to_path_buf()
}

pub fn maui_root() -> PathBuf {
    project_root().join("mandelbrot")
}
pub fn maui_project_file() -> PathBuf {
    let file = maui_root().join("mandelbrot.csproj");
    assert!(file.exists());
    assert!(file.is_file());
    file
}

/// Path to the Rust library root (contains Cargo.toml)
pub fn rust_fractal_root() -> PathBuf {
    project_root().join("rust_fractal")
}

/// Destination directory inside the MAUI solution tree.
/// Project-specific: this is your staging root.
pub fn destination_root() -> PathBuf {
    project_root().join("mandelbrot").join("CsBindGen")
}

/// Generated C# bindings destination.
/// In your layout, this equals destination_root().
pub fn destination_bindings_dir() -> PathBuf {
    destination_root()
}

/// Where native binaries should be staged.
pub fn destination_native_runtimes_dir() -> PathBuf {
    destination_bindings_dir().join("runtimes")
}

/// Where the Rust crate emits generated C# bindings.
pub fn rust_generated_csharp_dir() -> PathBuf {
    rust_fractal_root().join("target").join("csharp")
}

/// Validate that rust_fractal exists (we do NOT create this silently).
pub fn assert_rust_fractal_exists() {
    let cargo_toml = rust_fractal_root().join("Cargo.toml");
    assert!(
        cargo_toml.is_file(),
        "Expected rust_fractal Cargo.toml at: {}",
        cargo_toml.display()
    );
}

impl DotNetVersion {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s {
            "8" | "net8" | "net8.0" => Ok(Self::Net8),
            "10" | "net10" | "net10.0" => Ok(Self::Net10),
            _ => anyhow::bail!("unsupported .NET version: {s} (expected 8 or 10)"),
        }
    }
}

/// `{platform}-{arch}`
pub fn runtime_dir_name(platform: Platform, arch: Arch) -> String {
    format!("{}-{}", platform.as_str(), arch.as_str())
}

/// Native library stem (crate output name).
pub fn native_lib_stem() -> &'static str {
    "rust_fractal"
}

pub fn native_lib_prefix(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "",
        Platform::Android | Platform::Ios | Platform::Osx => "lib",
    }
}

pub fn native_lib_ext(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "dll",
        Platform::Android => "so",
        Platform::Osx | Platform::Ios => "dylib",
    }
}

/// `{prefix}{stem}.{ext}`
pub fn native_lib_filename(platform: Platform) -> String {
    format!(
        "{}{}.{}",
        native_lib_prefix(platform),
        native_lib_stem(),
        native_lib_ext(platform)
    )
}

/// Full staging path:
/// `destination_root/runtimes/{platform}-{arch}/{name}{ext}`
pub fn destination_native_lib_path(platform: Platform, arch: Arch) -> PathBuf {
    destination_native_runtimes_dir()
        .join(runtime_dir_name(platform, arch))
        .join(native_lib_filename(platform))
}
