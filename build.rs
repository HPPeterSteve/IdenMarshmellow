use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        // Ignoring icon for now on Windows since we don't have it copied over
        res.set("FileDescription", "IdenMarshmellow - Vault Security System");
        res.set("ProductName", "IdenMarshmellow");
        res.set("CompanyName", "Pedrão Projects");
        res.set("LegalCopyright", "© 2026 Pedrão");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.compile().unwrap();
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let c_sources = [
        "c_src/vault_crypto.c",
        "c_src/vault_catalog.c",
        "c_src/vault_monitor.c",
        "c_src/vault_sandbox.c",
        "c_src/vault_engine.c",
        "c_src/vault_ffi.c",
    ];

    let mut object_files: Vec<String> = Vec::new();

    for src in &c_sources {
        let base = std::path::Path::new(src)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        let obj_path = format!("{}/{}.o", out_dir, base);

        let mut gcc_args = vec![
            "-Os".to_string(),
            "-fdata-sections".to_string(),
            "-ffunction-sections".to_string(),
            "-Wall".to_string(),
            "-Wextra".to_string(),
            "-DVAULT_FFI_BUILD".to_string(),
            "-fPIC".to_string(),
            "-c".to_string(),
            "-I".to_string(),
            "c_src".to_string(),
            src.to_string(),
            "-o".to_string(),
            obj_path.clone(),
        ];

        if cfg!(target_os = "linux") {
            gcc_args.push("-pthread".to_string());
        }

        let status = Command::new("gcc")
            .args(&gcc_args)
            .status()
            .unwrap_or_else(|e| panic!("Failed to compile {}: {}", src, e));

        assert!(status.success(), "Compilation of {} failed", src);
        object_files.push(obj_path);
    }

    let lib_path = format!("{}/libvault_security.a", out_dir);
    let mut ar_args = vec!["rcs".to_string(), lib_path];
    ar_args.extend(object_files);

    let status = Command::new("ar")
        .args(&ar_args)
        .status()
        .expect("Failed to create libvault_security.a");

    assert!(status.success(), "Static library creation failed");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=vault_security");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=seccomp");
        println!("cargo:rustc-link-lib=cap");
    }

    println!("cargo:rerun-if-changed=c_src/vault_core.h");
    for src in &c_sources {
        println!("cargo:rerun-if-changed={}", src);
    }
}
