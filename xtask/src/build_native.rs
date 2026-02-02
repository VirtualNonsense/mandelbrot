use crate::paths::{self, Arch, Platform};
use crate::process::{self, cmd_in_dir, run};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TargetSpec {
    pub cargo_target: &'static str,
    pub arch: Arch,
}
pub fn targets_for(platform: Platform) -> &'static [TargetSpec] {
    match platform {
        Platform::Windows => &[TargetSpec {
            cargo_target: "x86_64-pc-windows-msvc",
            arch: Arch::X64,
        }],
        Platform::Android => &[
            TargetSpec {
                cargo_target: "aarch64-linux-android",
                arch: Arch::Arm64,
            },
            // Add more ABIs if you want them explicitly:
            // TargetSpec { cargo_target: "x86_64-linux-android", arch: Arch::X64 },
        ],
        Platform::Osx => &[
            TargetSpec {
                cargo_target: "aarch64-apple-darwin",
                arch: Arch::Arm64,
            },
            TargetSpec {
                cargo_target: "x86_64-apple-darwin",
                arch: Arch::X64,
            },
        ],
        Platform::Ios => &[
            // Minimal starting point (device):
            TargetSpec {
                cargo_target: "aarch64-apple-ios",
                arch: Arch::Arm64,
            },
        ],
    }
}

fn host_os_is_macos() -> bool {
    cfg!(target_os = "macos")
}
fn host_os_is_windows() -> bool {
    cfg!(target_os = "windows")
}

pub fn build_hash() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs();

    secs.to_string()
}
pub(crate) fn pack_nuget(release: bool, version: &str) -> Result<()> {
    paths::assert_rust_fractal_exists();

    let csproj = paths::bindings_root_project_file();
    if !csproj.is_file() {
        anyhow::bail!("NuGet csproj not found: {}", csproj.display());
    }

    // Ensure output feed dir exists (allowed)
    let out_dir = paths::bindings_nupkgs();

    let config = if release { "Release" } else { "Debug" };

    let mut cmd = process::cmd_in_dir("dotnet", &paths::bindings_root());
    cmd.arg("pack")
        .arg(csproj)
        .arg("-c")
        .arg(config)
        .arg("-o")
        .arg(out_dir);

    cmd.arg(format!("/p:PackageVersion={}", version)); // see note below

    process::run(cmd).context("dotnet pack failed")?;
    Ok(())
}
pub fn build_and_stage(platform: Platform, release: bool) -> Result<()> {
    paths::assert_rust_fractal_exists();

    if platform.requires_macos() && !host_os_is_macos() {
        return Err(anyhow!("{platform:?} builds require a macOS host"));
    }
    if platform.requires_windwos() && !host_os_is_windows() {
        return Err(anyhow!(
            "Windows builds are expected to run on a Windows host"
        ));
    }

    let rust_root = paths::rust_fractal_root();
    let profile_dir = if release { "release" } else { "debug" };

    for t in targets_for(platform) {
        cargo_build(&rust_root, t.cargo_target, release)?;

        let built_artifact =
            built_artifact_path(&rust_root, t.cargo_target, profile_dir, platform)?;
        let dest_file = paths::destination_native_lib_path(platform, t.arch);

        // Create destination directory
        fs::create_dir_all(
            dest_file
                .parent()
                .expect("destination native lib must have parent dir"),
        )
        .with_context(|| format!("create_dir_all failed: {}", dest_file.display()))?;

        fs::copy(&built_artifact, &dest_file).with_context(|| {
            format!(
                "copy failed: {} -> {}",
                built_artifact.display(),
                dest_file.display()
            )
        })?;

        eprintln!(
            "staged: {} -> {}",
            built_artifact.display(),
            dest_file.display()
        );
    }

    Ok(())
}

fn cargo_build(rust_root: &Path, cargo_target: &str, release: bool) -> Result<()> {
    let mut cmd = cmd_in_dir("cargo", rust_root);
    cmd.arg("build")
        .arg("-p")
        .arg("rust_fractal")
        .arg("--target")
        .arg(cargo_target);

    if release {
        cmd.arg("--release");
    }

    run(cmd).with_context(|| format!("cargo build failed for target {cargo_target}"))?;
    Ok(())
}

/// Deterministic expected artifact path based on naming rules.
/// This avoids “find a file that contains rust_fractal”.
fn built_artifact_path(
    rust_root: &Path,
    cargo_target: &str,
    profile_dir: &str,
    platform: Platform,
) -> Result<PathBuf> {
    let filename = paths::native_lib_filename(platform);

    // Standard cargo output dir:
    // rust_fractal/target/<triple>/<debug|release>/<libname>
    let p = rust_root
        .join("target")
        .join(cargo_target)
        .join(profile_dir)
        .join(filename);

    if !p.is_file() {
        return Err(anyhow!(
            "expected built artifact not found: {}",
            p.display()
        ));
    }

    Ok(p)
}
