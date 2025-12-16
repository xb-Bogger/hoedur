use std::{
    env,
    fs::{self, File},
    path::PathBuf,
    process::Command,
};

use qemu_build::{build, Arch};

const BINDINGS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qemu-sys/src/bindings");
const PATCHES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qemu-sys/patches");

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("failed to get $OUT_DIR"));
    let qemu_dir = out_dir.join("qemu-7.1.0");
    let build_dir = out_dir.join("build");
    let bindings_dir = PathBuf::from(BINDINGS_DIR);

    // allow using a prebuilt libqemu via env to avoid network
    if let Ok(lib_path) = env::var("LIBQEMU_PATH") {
        let lib = PathBuf::from(&lib_path);
        if lib.is_file() {
            let parent = lib.parent().expect("lib path parent");
            println!("cargo:rustc-link-search=native={}", parent.display());
            // derive library name from filename (strip 'lib' prefix and '.so' suffix)
            let name = lib
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.strip_prefix("lib").unwrap_or(s))
                .expect("lib filename");
            println!("cargo:rustc-link-lib=dylib={}", name);
            return;
        }
    }

    // get QEMU
    if !qemu_dir.is_dir() {
        let qemu_tar = out_dir.join("qemu-7.1.0.tar.xz");

        // download QEMU
        let mut downloaded = false;
        // allow providing a pre-downloaded tarball via env
        if let Ok(tar_env) = env::var("QEMU_TARBALL") {
            let provided = PathBuf::from(tar_env);
            if provided.is_file() {
                fs::copy(&provided, &qemu_tar).expect("Failed to copy provided QEMU tarball");
                downloaded = true;
            }
        }
        if !downloaded {
            // try wget
            let wget = Command::new("wget")
                .arg("https://download.qemu.org/qemu-7.1.0.tar.xz")
                .arg("-O")
                .arg(&qemu_tar)
                .status();
            downloaded = wget.as_ref().map(|s| s.success()).unwrap_or(false);
        }
        if !downloaded {
            // try wget with no certificate check
            let wget_nc = Command::new("wget")
                .arg("--no-check-certificate")
                .arg("https://download.qemu.org/qemu-7.1.0.tar.xz")
                .arg("-O")
                .arg(&qemu_tar)
                .status();
            downloaded = wget_nc.as_ref().map(|s| s.success()).unwrap_or(false);
        }
        if !downloaded {
            // try curl
            let curl = Command::new("curl")
                .arg("-L")
                .arg("https://download.qemu.org/qemu-7.1.0.tar.xz")
                .arg("-o")
                .arg(&qemu_tar)
                .status();
            downloaded = curl.as_ref().map(|s| s.success()).unwrap_or(false);
        }
        assert!(downloaded, "QEMU download failed");

        // extract QEMU
        assert!(Command::new("tar")
            .current_dir(&out_dir)
            .arg("-xf")
            .arg(&qemu_tar)
            .status()
            .expect("QEMU extract failed")
            .success());

        // apply QEMU patches
        let patches_dir = PathBuf::from(PATCHES_DIR);
        let patches = [
            patches_dir.join("hoedur.patch"),
            patches_dir.join("5cb993ff131fca2abef3ce074a20258fd6fce557.patch"),
        ];
        for patch in patches {
            let file = File::open(patch).expect("Failed to open patch file");

            assert!(Command::new("patch")
                .current_dir(&qemu_dir)
                .stdin(file)
                .arg("-p1")
                .status()
                .expect("Apply QEMU patches failed")
                .success());
        }
    }

    // create build dir
    if !build_dir.is_dir() {
        fs::create_dir(&build_dir).expect("Failed to create build dir");
    }

    #[cfg(feature = "arm")]
    let arch = Arch::Arm;

    build(&qemu_dir, &build_dir, arch, &bindings_dir);
}
