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
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Windows => "win",
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

/// c# nuget directory withing rust_fractal.
pub fn rust_fractal_nuget() -> PathBuf {
    rust_fractal_root().join("nuget")
}

/// c# packages location within rust_fractal
pub fn bindings_root() -> PathBuf {
    rust_fractal_nuget().join("RustFractals")
}
/// c# packages location within rust_fractal
pub fn bindings_root_project_file() -> PathBuf {
    bindings_root().join("RustFractals.csproj")
}
/// packed nuget artifacts
pub fn bindings_nupkgs() -> PathBuf {
    rust_fractal_nuget().join("nupkgs")
}

/// Path to the Rust library root (contains Cargo.toml)
pub fn rust_fractal_root() -> PathBuf {
    project_root().join("rust_fractal")
}

/// Where native binaries should be staged.
pub fn bindings_runtime_dir() -> PathBuf {
    bindings_root().join("runtimes")
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

pub fn dotnet_tfm(version: DotNetVersion, platform: Platform) -> String {
    let base = version.as_str();

    match platform {
        Platform::Android => format!("{base}-android"),
        Platform::Ios => format!("{base}-ios"),
        Platform::Osx => format!("{base}-maccatalyst"),
        Platform::Windows => {
            // Keep the Windows min version explicit
            format!("{base}-windows10.0.19041.0")
        }
    }
}

pub(crate) fn dotnet_rid(platform: Platform, arch: Arch) -> &'static str {
    match (platform, arch) {
        (Platform::Windows, Arch::X64) => "win-x64",
        (Platform::Windows, Arch::Arm64) => "win-arm64",

        (Platform::Android, Arch::Arm64) => "android-arm64",
        (Platform::Android, Arch::X64) => "android-x64",

        // MacCatalyst RIDs:
        (Platform::Osx, Arch::Arm64) => "maccatalyst-arm64",
        (Platform::Osx, Arch::X64) => "maccatalyst-x64",

        // iOS RIDs (device):
        (Platform::Ios, Arch::Arm64) => "ios-arm64",

        _ => panic!("unsupported .NET RID for {:?} {:?}", platform, arch),
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
    bindings_runtime_dir()
        .join(runtime_dir_name(platform, arch))
        .join("native")
        .join(native_lib_filename(platform))
}
