use std::{error::Error, fs, path::PathBuf, str::FromStr};

fn main() -> Result<(), Box<dyn Error>> {
    let package_name = env!("CARGO_PKG_NAME");
    let build_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;
    let csharp_dir = build_dir.join("target").join("csharp");
    let file_name = format!("{package_name}.g.cs");

    fs::create_dir_all(&csharp_dir)?;

    csbindgen::Builder::default()
        .csharp_namespace("CsBindGen")
        .input_extern_file("./src/lib.rs")
        .csharp_dll_name(package_name)
        .csharp_class_accessibility("internal")
        .generate_csharp_file(csharp_dir.join(file_name))
}
