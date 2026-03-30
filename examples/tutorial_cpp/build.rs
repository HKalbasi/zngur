#[cfg(not(target_os = "windows"))]
use std::env;

use zngur::Zngur;

fn main() {
    build::rerun_if_changed("main.zng");
    build::rerun_if_changed("impls.cpp");
    build::rerun_if_env_changed("CXX");

    #[cfg(not(target_os = "windows"))]
    let cxx = env::var("CXX").unwrap_or("c++".to_owned());

    let crate_dir = build::cargo_manifest_dir();
    let out_dir = build::out_dir();

    // Force rerun if generated files don't exist
    let generated_files = [
        out_dir.join("generated.cpp"),
        out_dir.join("generated.h"),
        out_dir.join("generated.rs"),
    ];
    for file in &generated_files {
        if !file.exists() {
            println!("cargo:rerun-if-changed=nonexistent_trigger_file");
            break;
        }
    }

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(out_dir.join("generated.cpp"))
        .with_h_file(out_dir.join("generated.h"))
        .with_rs_file(out_dir.join("generated.rs"))
        .with_zng_header_in_place()
        .generate();

    let my_build = &mut cc::Build::new();
    let my_build = my_build.cpp(true).std("c++20");

    #[cfg(not(target_os = "windows"))]
    my_build.compiler(&cxx);

    my_build.include(&crate_dir).include(&out_dir);

    let my_build = || my_build.clone();

    my_build()
        .file(out_dir.join("generated.cpp"))
        .compile("zngur_generated");
    my_build().file("impls.cpp").compile("impls");
}
