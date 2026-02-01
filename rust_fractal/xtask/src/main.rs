use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    str::FromStr,
};

use clap::{ArgGroup, Parser, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum Profile {
    Debug,
    Release,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Target {
    Android,
    Windows,
}

#[derive(Parser, Debug)]
#[command(name = "xtask", version, about = "Repo automation tasks")]
#[command(group(
    ArgGroup::new("profile_flag")
        .args(["debug", "release"])
        .multiple(false)
))]
struct Args {
    /// Build & copy in debug mode
    #[arg(long)]
    debug: bool,

    /// Build & copy in release mode
    #[arg(long)]
    release: bool,

    /// Destination directory where the libaryar + generated C# interface will be copied to.
    /// This is required on purpose to avoid accidental copies into the repo.
    #[arg(long, value_name = "DIR")]
    dest_dir: PathBuf,

    /// If set, skip the cargo build step and only perform copying.
    #[arg(long)]
    no_build: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("xtask failed: {e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    let profile = match (args.debug, args.release) {
        (true, false) => Profile::Debug,
        (false, true) => Profile::Release,
        (false, false) => Profile::Debug, // default
        (true, true) => unreachable!("clap ArgGroup enforces exclusivity"),
    };

    let xtask_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;

    let rust_fractal_dir = canonicalize_dir(xtask_dir.parent().expect("Parent dir should exist"))?;

    let rust_fractal_manifest = rust_fractal_dir.join("Cargo.toml");
    if !rust_fractal_manifest.is_file() {
        return Err(anyhow::anyhow!(
            "rust_fractal Cargo.toml not found at: {}",
            rust_fractal_manifest.display()
        ));
    }

    let dest_dir =
        ensure_dir(&args.dest_dir).map_err(|e| anyhow::anyhow!("Invalid --dest-dir: {e}"))?;

    if !args.no_build {
        cargo_build(&rust_fractal_manifest, profile)?;
    }

    // NOTE: You stated your generated C# interface is here:
    // rust_fractal (manifest dir)/target/csharp
    // We'll also assume the library ends up under:
    // rust_fractal (manifest dir)/target/{debug|release}/<bin>
    let target_dir = rust_fractal_dir.join("target");

    let lib_src = binary_path(&target_dir, profile);
    println!("checking generated c# files in {lib_src:?}");
    if !lib_src.is_file() {
        return Err(anyhow::anyhow!(
            "Built library not found at: {}\n\
             If your build output goes to a workspace-level target dir, either:\n\
             - set CARGO_TARGET_DIR to rust_fractal/target, or\n\
             - adjust this xtask to point at the workspace target.\n",
            lib_src.display()
        ));
    }

    let lib_dest = dest_dir.join(
        lib_src
            .file_name()
            .unwrap_or_else(|| OsStr::new("rust_fractal")),
    );
    fs::copy(&lib_src, &lib_dest).map_err(|e| {
        anyhow::anyhow!(
            "Failed to copy library from {} to {}: {e}",
            lib_src.display(),
            lib_dest.display()
        )
    })?;

    let csharp_src = target_dir.join("csharp");
    if !csharp_src.is_dir() {
        return Err(anyhow::anyhow!(
            "C# interface directory not found at: {}\n\
             Expected: rust_fractal/target/csharp\n\
             If your generator writes elsewhere, adjust the path or generator config.",
            csharp_src.display()
        ));
    }

    let csharp_dest = dest_dir;
    copy_dir_recursive(&csharp_src, &csharp_dest).map_err(|e| {
        anyhow::anyhow!(
            "Failed to copy C# interface directory from {} to {}: {e}",
            csharp_src.display(),
            csharp_dest.display()
        )
    })?;

    println!("Copied:");
    println!("  lib   : {} -> {}", lib_src.display(), lib_dest.display());
    println!(
        "  csharp: {} -> {}",
        csharp_src.display(),
        csharp_dest.display()
    );

    Ok(())
}

fn cargo_build(manifest_path: &Path, profile: Profile) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--manifest-path").arg(manifest_path);

    match profile {
        Profile::Debug => {}
        Profile::Release => {
            cmd.arg("--release");
        }
    }

    // Helpful output for debugging CI / local environment.
    eprintln!("Running: {cmd:?}");

    let status = cmd.status().map_err(|e| {
        anyhow::anyhow!("Failed to spawn cargo. Is Rust installed and cargo on PATH? {e}")
    })?;

    ensure_success(status, "cargo build")?;
    Ok(())
}

fn ensure_success(status: ExitStatus, what: &str) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{what} failed with status: {status}"))
    }
}

fn binary_path(target_dir: &Path, profile: Profile) -> PathBuf {
    let prof = match profile {
        Profile::Debug => "debug",
        Profile::Release => "release",
    };

    let mut file = String::from("rust_fractal");
    #[cfg(windows)]
    file.push_str(".dll");

    #[cfg(target_os = "android")]
    file.push_str(".so");

    target_dir.join(prof).join(file)
}

fn canonicalize_dir(p: &Path) -> io::Result<PathBuf> {
    let c = fs::canonicalize(p)?;
    if !c.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Not a directory: {}", c.display()),
        ));
    }
    Ok(c)
}

fn ensure_dir(p: &Path) -> io::Result<PathBuf> {
    let p = if p.is_absolute() {
        p.to_path_buf()
    } else {
        env::current_dir()?.join(p)
    };
    fs::create_dir_all(&p)?;
    let c = fs::canonicalize(&p)?;
    if !c.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Not a directory: {}", c.display()),
        ));
    }
    Ok(c)
}

/// Recursive directory copy without extra dependencies.
/// Copies permissions best-effort; symlinks are treated as files (copied by content if possible).
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to)?;
        } else {
            // Skip special files (sockets, devices). If you need symlink support, say so.
            eprintln!("Skipping non-file/non-dir: {}", from.display());
        }
    }
    Ok(())
}
