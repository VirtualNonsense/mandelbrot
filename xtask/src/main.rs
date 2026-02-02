mod build_native;
mod paths;
mod process;

use std::{fs, str::FromStr};

use crate::{build_native::build_hash, paths::*};
use anyhow::{self, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project-specific build toolchain for Rust + MAUI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Debug)]
struct Target {
    arch: paths::Arch,
    platform: paths::Platform,
}
impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // pick any separator you like: '-', ':', '/'
        let (platform_s, arch_s) = s
            .split_once('-')
            .ok_or_else(|| anyhow::anyhow!("invalid target '{s}', expected <platform>-<arch>"))?;

        let platform = Platform::from_str(platform_s, true)
            .map_err(|_| anyhow::anyhow!("bad platform: {platform_s}"))?;
        let arch =
            Arch::from_str(arch_s, true).map_err(|_| anyhow::anyhow!("bad arch: {arch_s}"))?;

        match (arch, platform) {
            (Arch::X64, Platform::Windows)
            | (Arch::X64, Platform::Osx)
            | (Arch::Arm64, Platform::Osx)
            | (Arch::Arm64, Platform::Android)
            | (Arch::Arm64, Platform::Ios) => {}
            (arch, os) => {
                return Err(anyhow::anyhow!(
                    "{}-{} is not a supported architecture",
                    arch.as_str(),
                    os.as_str()
                ));
            }
        }

        Ok(Target { platform, arch })
    }
}
impl Target {
    pub fn get_rust_architecture_representation(&self) -> String {
        let arch = match self.arch {
            Arch::X64 => "x86_64",
            Arch::Arm64 => "aarch64",
        };
        let platform = match self.platform {
            Platform::Windows => "pc-windows-msvc",
            Platform::Android => "linux-android",
            Platform::Ios => "apple-ios",
            Platform::Osx => "apple-darwin",
        };

        format!("{arch}-{platform}")
    }
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Build Rust native libraries for a platform, stage them into destination,
    /// then optionally build .NET.
    BuildNative {
        /// Targets
        #[arg(
            long, 
            required = true, 
            num_args=1..,
            help = "Build target in the form <platform>-<arch>",
            long_help = r#"
Build target in the form <platform>-<arch>.

Examples:
  --target windows-x64
  --target android-arm64
  --target ios-arm64
  --target osx-arm64

This option may be repeated to build multiple targets.
"#
        )]
        targets: Vec<Target>,

        /// Use release profile for Rust + dotnet
        #[arg(long)]
        release: bool,

        /// restores solution with the current build hash as Parameter
        #[arg(long)]
        reload_dotnet: bool,

        /// .NET version (defaults to 10)
        #[arg(long, value_enum,  default_value_t = paths::DotNetVersion::default())]
        dotnet_version: paths::DotNetVersion,

        /// Build the .NET MAUI solution after staging natives
        #[arg(long)]
        build_dotnet: bool,

    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::BuildNative {
            targets,
            release,
            reload_dotnet,
            dotnet_version,
            build_dotnet,
        } => build_native_cmd(targets, dotnet_version, release, reload_dotnet, build_dotnet
        ),
    }
}

fn build_native_cmd(
    targets: Vec<Target>,
    dotnet_version: paths::DotNetVersion,
    release: bool,
    reload_dotnet: bool,
    build_dotnet: bool,
) -> Result<()> {
    fs::create_dir_all(paths::bindings_runtime_dir()).with_context(|| {
        format!(
            "create_dir_all failed: {}",
            paths::bindings_runtime_dir().display()
        )
    })?;

    for target in &targets {
        build_native::build_and_stage(target, release)?;
    }
    let nuget_package_version = format!("1.0.{}", build_hash());
    build_native::pack_nuget(release, &nuget_package_version)?;
    if reload_dotnet{

        let mut restore = process::cmd_in_dir("dotnet", &paths::maui_root());
        restore
            .arg("restore")
            .arg(paths::maui_project_file())
            .arg(format!("-p:FractalNativeVersion={}", nuget_package_version));

        process::run(restore)?;
    } 

    if build_dotnet {
        for target in targets {
                build_dotnet_solution(
                    target.platform,
                    target.arch,
                    dotnet_version,
                    release,
                )?;
        }
    }

    Ok(())
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
    let rid = paths::dotnet_rid(platform, arch);

    let csproj = paths::maui_project_file();


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
