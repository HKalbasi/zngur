#[cfg(not(target_os = "windows"))]
use std::env;

use zngur::Zngur;

fn main() {
    build::rerun_if_changed("main.zng");
    build::rerun_if_changed("impls.h");
    build::rerun_if_changed("impls.cpp");
    build::rerun_if_changed("task.h");
    build::rerun_if_changed("cpp_rust_inheritance.h");
    build::rerun_if_changed("src/");
    build::rerun_if_env_changed("CXX");

    #[cfg(not(target_os = "windows"))]
    let cxx = env::var("CXX").unwrap_or("c++".to_owned());

    let crate_dir = build::cargo_manifest_dir();
    dbg!(&crate_dir);

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(crate_dir.join("generated.cpp"))
        .with_h_file(crate_dir.join("generated.h"))
        .with_rs_file(crate_dir.join("./src/generated.rs"))
        .with_crate_name("crate")
        .with_zng_header(crate_dir.join("zngur.h"))
        .generate();

    let my_build = &mut cc::Build::new();
    let my_build = my_build.cpp(true).std("c++11").include(&crate_dir);

    #[cfg(not(target_os = "windows"))]
    my_build.compiler(&cxx);

    let my_build = || my_build.clone();

    my_build().file("impls.cpp").compile("impls");
    my_build().file("generated.cpp").compile("generated");
}
