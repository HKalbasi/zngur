#[cfg(not(target_os = "windows"))]
use std::env;

use zngur::{Zngur, ZngurHdr};

fn main() {
    build::rerun_if_changed("main.zng");
    build::rerun_if_changed("blobstore.cpp");
    build::rerun_if_changed("src/generated.rs");
    build::rerun_if_changed("generated.h");
    build::rerun_if_changed("generated.cpp");
    build::rerun_if_env_changed("CXX");

    #[cfg(not(target_os = "windows"))]
    let cxx = env::var("CXX").unwrap_or("c++".to_owned());

    let crate_dir = build::cargo_manifest_dir();

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(crate_dir.join("generated.cpp"))
        .with_h_file(crate_dir.join("generated.h"))
        .with_rs_file(crate_dir.join("./src/generated.rs"))
        .generate();

    ZngurHdr::new()
        .with_panic_to_exception()
        .with_zng_header("zngur.h")
        .generate();

    let my_build = &mut cc::Build::new();
    let my_build = my_build.cpp(true).std("c++17").include(".");

    #[cfg(not(target_os = "windows"))]
    my_build.compiler(&cxx);

    let my_build = || my_build.clone();

    my_build().file("generated.cpp").compile("zngur_generated");
    my_build().file("blobstore.cpp").compile("blobstore");
}
