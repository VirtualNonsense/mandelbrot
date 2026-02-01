mod build_native;
mod paths;
mod process;
mod sync_cs;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use std::{fs, sync::mpsc, time::Duration};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project-specific build toolchain for Rust + MAUI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Watch rust_fractal/target/csharp/*.cs and sync into the MAUI destination.
    MonitorRustSource,

    /// Build Rust native libraries for a platform, stage them into destination,
    /// then optionally build .NET.
    BuildNative {
        /// Target platform
        platform: paths::Platform,

        /// Target CPU architecture
        arch: paths::Arch,

        /// Use release profile for Rust + dotnet
        #[arg(long)]
        release: bool,

        /// .NET version (defaults to 10)
        #[arg(long, value_enum)]
        dotnet_version: paths::DotNetVersion,

        /// Build the .NET MAUI solution after staging natives
        #[arg(long)]
        dotnet: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::MonitorRustSource => monitor_rust_source(),
        Cmd::BuildNative {
            platform,
            release,
            arch,
            dotnet_version,
            dotnet,
        } => build_native_cmd(platform, arch, dotnet_version, release, dotnet),
    }
}

fn monitor_rust_source() -> Result<()> {
    crate::paths::assert_rust_fractal_exists();

    let src_dir = crate::paths::rust_generated_csharp_dir();
    let dst_dir = crate::paths::destination_bindings_dir();

    // Allowed to create
    fs::create_dir_all(&src_dir)
        .with_context(|| format!("create_dir_all failed: {}", src_dir.display()))?;
    fs::create_dir_all(&dst_dir)
        .with_context(|| format!("create_dir_all failed: {}", dst_dir.display()))?;

    // Initial sync
    crate::sync_cs::sync_generated_cs(&src_dir, &dst_dir)?;

    eprintln!("Watching: {}", src_dir.display());
    eprintln!("Syncing to: {}", dst_dir.display());

    let (tx, rx) = mpsc::channel::<DebounceEventResult>();

    // IMPORTANT: new_debouncer expects (timeout, handler)
    let mut debouncer = new_debouncer(
        Duration::from_millis(250),
        move |res: DebounceEventResult| {
            // If the receiver is gone, we can’t do much—ignore send errors.
            let _ = tx.send(res);
        },
    )
    .context("failed to create debouncer")?;

    debouncer
        .watcher()
        .watch(&src_dir, RecursiveMode::NonRecursive)
        .with_context(|| format!("failed to watch {}", src_dir.display()))?;

    loop {
        match rx.recv().context("watch channel closed")? {
            Ok(_events) => {
                // Any change => resync folder
                if let Err(e) = crate::sync_cs::sync_generated_cs(&src_dir, &dst_dir) {
                    eprintln!("sync error: {e:#}");
                }
            }
            Err(error) => {
                // Debouncer aggregates errors; print them and continue.
                eprintln!("watch error: {error}");
            }
        }
    }
}

fn build_native_cmd(
    platform: paths::Platform,
    arch: paths::Arch,
    dotnet_version: paths::DotNetVersion,
    release: bool,
    dotnet: bool,
) -> Result<()> {
    // Ensure destination base exists (allowed)
    fs::create_dir_all(paths::destination_root()).with_context(|| {
        format!(
            "create_dir_all failed: {}",
            paths::destination_root().display()
        )
    })?;
    fs::create_dir_all(paths::destination_native_runtimes_dir()).with_context(|| {
        format!(
            "create_dir_all failed: {}",
            paths::destination_native_runtimes_dir().display()
        )
    })?;

    build_native::build_and_stage(platform, release)?;

    if dotnet {
        build_dotnet_solution(platform, arch, dotnet_version, release)?;
    }

    Ok(())
}

pub fn dotnet_tfm(version: paths::DotNetVersion, platform: paths::Platform) -> String {
    let base = version.as_str();

    match platform {
        paths::Platform::Android => format!("{base}-android"),
        paths::Platform::Ios => format!("{base}-ios"),
        paths::Platform::Osx => format!("{base}-maccatalyst"),
        paths::Platform::Windows => {
            // Keep the Windows min version explicit
            format!("{base}-windows10.0.19041.0")
        }
    }
}

fn dotnet_rid(platform: paths::Platform, arch: paths::Arch) -> &'static str {
    match (platform, arch) {
        (paths::Platform::Windows, paths::Arch::X64) => "win-x64",
        (paths::Platform::Windows, paths::Arch::Arm64) => "win-arm64",

        (paths::Platform::Android, paths::Arch::Arm64) => "android-arm64",
        (paths::Platform::Android, paths::Arch::X64) => "android-x64",

        // MacCatalyst RIDs:
        (paths::Platform::Osx, paths::Arch::Arm64) => "maccatalyst-arm64",
        (paths::Platform::Osx, paths::Arch::X64) => "maccatalyst-x64",

        // iOS RIDs (device):
        (paths::Platform::Ios, paths::Arch::Arm64) => "ios-arm64",

        _ => panic!("unsupported .NET RID for {:?} {:?}", platform, arch),
    }
}

fn build_dotnet_solution(
    platform: paths::Platform,
    arch: paths::Arch,
    version: paths::DotNetVersion,
    release: bool,
) -> Result<()> {
    let repo_root = paths::project_root();
    let config = if release { "Release" } else { "Debug" };

    let tfm = dotnet_tfm(version, platform);
    let rid = dotnet_rid(platform, arch);

    let mut cmd = process::cmd_in_dir("dotnet", &repo_root);
    cmd.arg("build")
        .arg("-c")
        .arg(config)
        .arg("-f")
        .arg(&tfm)
        .arg("-r")
        .arg(rid);

    process::run(cmd)?;
    Ok(())
}
