use std::path::{Path, PathBuf};

use std::env;

fn main() {
    let embed_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("embed");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rustc-env=VSLANG=1033");
    println!("cargo:rerun-if-changed=embed");
    println!("cargo:rerun-if-changed=build.rs");

    let code_dir = out_path.join("quickjs");
    if code_dir.exists() {
        std::fs::remove_dir_all(&code_dir).unwrap();
    }
    copy_dir::copy_dir(embed_path.join("quickjs"), &code_dir)
        .expect("Could not copy quickjs directory");

    std::fs::copy(
        embed_path.join("extensions.c"),
        code_dir.join("extensions.c"),
    )
    .expect("Could not copy extensions.c");

    std::fs::copy(
        embed_path.join("extensions.h"),
        code_dir.join("extensions.h"),
    )
    .expect("Could not copy extensions.h");

    eprintln!("Applying patches...");
    apply_patches(&code_dir);

    eprintln!("Generating bindings...");
    do_bindgen();

    eprintln!("Compiling quickjs...");
    compile_lib(&code_dir);
}

fn apply_patches(code_dir: &PathBuf) {
    use std::fs;

    eprintln!("Applying patches...");
    let embed_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("embed");
    let patches_path = embed_path.join("patches");
    for patch in fs::read_dir(patches_path).expect("Could not open patches directory") {
        let patch = patch.expect("Could not open patch");
        eprintln!("Applying {:?}...", patch.file_name());
        let status = std::process::Command::new("patch")
            .current_dir(code_dir)
            .arg("-i")
            .arg(patch.path())
            .arg("--binary")
            .spawn()
            .expect("Could not apply patches")
            .wait()
            .expect("Could not apply patches");
        assert!(
            status.success(),
            "Patch command returned non-zero exit code"
        );
    }
}

fn compile_lib(code_dir: &Path) {
    cc::Build::new()
        .compiler("clang")
        .files(
            [
                // extensions.c has included quickjs.c
                "extensions.c",
                "cutils.c",
                "libbf.c",
                "libregexp.c",
                "libunicode.c",
            ]
            .iter()
            .map(|f| code_dir.join(f)),
        )
        // The below flags are used by the official Makefile.
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-Wchar-subscripts")
        .flag_if_supported("-Wno-array-bounds")
        .flag_if_supported("-Wno-format-truncation")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wundef")
        .flag_if_supported("-Wuninitialized")
        .flag_if_supported("-Wunused")
        .flag_if_supported("-Wwrite-strings")
        .flag_if_supported("-funsigned-char")
        // Below flags are added to supress warnings that appear on some
        // platforms.
        .flag_if_supported("-Wno-cast-function-type")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-shorten-64-to-32")
        .flag_if_supported("-Wno-implicit-int-conversion")
        .flag_if_supported("-Wno-enum-conversion")
        .compile("quickjs");
}

fn do_bindgen() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .header("embed/extensions.h")
        .allowlist_item("js_.+")
        .allowlist_item("JS.+")
        .clang_arg("-std=c11")
        .clang_arg(format!("-I{}", "embed/quickjs"))
        .default_enum_style(bindgen::EnumVariation::Consts {})
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
