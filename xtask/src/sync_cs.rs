use anyhow::{Context, Result};
use glob::glob;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create_dir_all failed: {}", path.display()))
}

pub fn sync_generated_cs(source_dir: &Path, dest_dir: &Path) -> Result<()> {
    ensure_dir(source_dir)?; // allowed: if it doesn't exist yet, create it (empty)
    ensure_dir(dest_dir)?;

    let pattern = source_dir.join("*.cs");
    let pattern_str = pattern
        .to_str()
        .context("non-utf8 path in source pattern")?
        .to_string();

    let mut src_files: Vec<PathBuf> = Vec::new();
    let tmp = glob(&pattern_str).context("glob failed")?;
    for entry in tmp {
        let Ok(path) = entry else { continue };
        if path.is_file() {
            src_files.push(path);
        }
    }

    // Copy/update files
    let mut src_names = HashSet::<String>::new();
    for src in &src_files {
        let name = src
            .file_name()
            .and_then(|s| s.to_str())
            .context("non-utf8 filename")?
            .to_string();
        src_names.insert(name.clone());

        let dst = dest_dir.join(&name);
        copy_if_changed(src, &dst)?;
    }

    // Optional: remove stale dest files that no longer exist upstream
    // Since this is a "sync", it's usually what you want.
    remove_stale(dest_dir, &src_names)?;

    Ok(())
}

fn copy_if_changed(src: &Path, dst: &Path) -> Result<()> {
    let do_copy = match (fs::metadata(src), fs::metadata(dst)) {
        (Ok(ms), Ok(md)) => {
            // If sizes differ or src modified is newer, copy.
            let size_diff = ms.len() != md.len();
            let time_newer = ms
                .modified()
                .ok()
                .and_then(|ts| md.modified().ok().map(|td| ts > td))
                .unwrap_or(true);
            size_diff || time_newer
        }
        (Ok(_), Err(_)) => true,
        (Err(e), _) => {
            return Err(e).with_context(|| format!("metadata failed: {}", src.display()));
        }
    };

    if do_copy {
        fs::copy(src, dst)
            .with_context(|| format!("copy failed: {} -> {}", src.display(), dst.display()))?;
        eprintln!("synced: {} -> {}", src.display(), dst.display());
    }
    Ok(())
}

/// removes all g.cs files that are not needed anymore
fn remove_stale(dest_dir: &Path, src_names: &HashSet<String>) -> Result<()> {
    for entry in fs::read_dir(dest_dir)
        .with_context(|| format!("read_dir failed: {}", dest_dir.display()))?
    {
        let entry = entry.context("read_dir entry failed")?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        // only generated files!
        if path.extension().and_then(|e| e.to_str()) != Some("g.cs") {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .context("non-utf8 filename in dest")?
            .to_string();

        if !src_names.contains(&name) {
            fs::remove_file(&path).with_context(|| format!("remove failed: {}", path.display()))?;
            eprintln!("removed stale: {}", path.display());
        }
    }
    Ok(())
}
