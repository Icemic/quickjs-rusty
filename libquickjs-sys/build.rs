use std::path::{Path, PathBuf};

use std::env;

fn main() {
    let embed_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("embed");

    println!("cargo:rustc-env=VSLANG=1033");
    println!("cargo:rerun-if-changed=embed");
    println!("cargo:rerun-if-changed=build.rs");

    eprintln!("Generating bindings...");
    do_bindgen();

    eprintln!("Compiling quickjs...");
    compile_lib(&embed_path);
}

fn compile_lib(code_dir: &Path) {
    let mut builder = cc::Build::new();

    // android ndk has its own clang compiler, so we can't use the default one
    if !is_cargo_ndk() {
        builder.compiler("clang");
    }

    // however, we respect the environment variable
    let xcompiler = env::var("TARGET_CC").or_else(|_| env::var("TARGET_CXX"));
    if let Ok(compiler) = xcompiler {
        builder.compiler(compiler);
    }

    builder.files(
        [
            // extensions.c has included quickjs.c
            "extensions.c",
            "./quickjs/cutils.c",
            "./quickjs/libregexp.c",
            "./quickjs/libunicode.c",
            "./quickjs/xsum.c",
        ]
        .iter()
        .map(|f| code_dir.join(f)),
    );

    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        builder.define("WIN32_LEAN_AND_MEAN", "_WIN32_WINNT=0x0601");
    }

    builder
        .define("_GNU_SOURCE", "1")
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

    let builder = bindgen::Builder::default()
        .header("embed/extensions.h")
        .allowlist_item("js_.+")
        .allowlist_item("JS.+")
        .clang_arg("-std=c11")
        .clang_arg(format!("-I{}", "embed/quickjs"));

    // detect if we are cross-compiling for android using cargo-ndk
    let builder = if is_cargo_ndk() {
        let target = env::var("TARGET").unwrap();
        let ndk_sysroot_path = env::var("CARGO_NDK_SYSROOT_PATH").unwrap();
        builder
            .clang_arg(format!("--sysroot={ndk_sysroot_path}"))
            .clang_arg(format!("--target={}", target))
    } else {
        builder
    };

    builder
        .default_enum_style(bindgen::EnumVariation::Consts {})
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn is_cargo_ndk() -> bool {
    // cargo-ndk sets this variable so we use it to detect if we are cross-compiling for android
    env::var("CARGO_NDK_ANDROID_PLATFORM").is_ok()
}
