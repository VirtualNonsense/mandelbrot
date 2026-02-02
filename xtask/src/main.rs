mod build_native;
mod paths;
mod process;

use std::fs;

use crate::{build_native::build_hash, paths::*};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project-specific build toolchain for Rust + MAUI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
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
        #[arg(long, value_enum,  default_value_t = paths::DotNetVersion::default())]
        dotnet_version: paths::DotNetVersion,

        /// Build the .NET MAUI solution after staging natives
        #[arg(long)]
        dotnet: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::BuildNative {
            platform,
            release,
            arch,
            dotnet_version,
            dotnet,
        } => build_native_cmd(platform, arch, dotnet_version, release, dotnet),
    }
}

fn build_native_cmd(
    platform: paths::Platform,
    arch: paths::Arch,
    dotnet_version: paths::DotNetVersion,
    release: bool,
    dotnet: bool,
) -> Result<()> {
    fs::create_dir_all(paths::bindings_runtime_dir()).with_context(|| {
        format!(
            "create_dir_all failed: {}",
            paths::bindings_runtime_dir().display()
        )
    })?;

    build_native::build_and_stage(platform, release)?;
    let nuget_package_version = format!("1.0.{}", build_hash());
    build_native::pack_nuget(release, &nuget_package_version)?;
    if dotnet {
        build_dotnet_solution(
            platform,
            arch,
            dotnet_version,
            release,
            &nuget_package_version,
        )?;
    }

    Ok(())
}

fn build_dotnet_solution(
    platform: paths::Platform,
    arch: paths::Arch,
    version: paths::DotNetVersion,
    release: bool,
    nuget_package_version: &str,
) -> Result<()> {
    let repo_root = paths::project_root();
    let config = if release { "Release" } else { "Debug" };

    let tfm = dotnet_tfm(version, platform);
    let rid = paths::dotnet_rid(platform, arch);

    let csproj = paths::maui_project_file();

    let mut restore = process::cmd_in_dir("dotnet", &repo_root);
    restore
        .arg("restore")
        .arg(format!("-p:FractalNativeVersion={}", nuget_package_version));

    process::run(restore)?;

    let mut cmd = process::cmd_in_dir("dotnet", &repo_root);
    cmd.arg("build")
        .arg(csproj)
        .arg("-c")
        .arg(config)
        .arg("-f")
        .arg(&tfm)
        .arg("-r")
        .arg(rid);

    if platform == paths::Platform::Windows {
        cmd.arg("/p:UseMonoRuntime=false");
    }
    process::run(cmd)?;
    Ok(())
}
