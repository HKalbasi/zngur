use std::env;

use zngur::Zngur;

fn main() {
    build::rerun_if_changed("main.zng");
    build::rerun_if_changed("blobstore.cpp");
    build::rerun_if_env_changed("CXX");

    let cxx = env::var("CXX").unwrap_or("c++".to_owned());

    let crate_dir = build::cargo_manifest_dir();

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(crate_dir.join("generated.cpp"))
        .with_h_file(crate_dir.join("generated.h"))
        .with_rs_file(crate_dir.join("./src/generated.rs"))
        .with_output_dir(crate_dir.clone())
        .generate();

    let my_build = &mut cc::Build::new();
    let my_build = my_build
        .cpp(true)
        .compiler(&cxx)
        .include(&crate_dir)
        .std("c++17");
    let my_build = || my_build.clone();

    my_build().file("generated.cpp").compile("zngur_generated");
    my_build().file("blobstore.cpp").compile("blobstore");
}
