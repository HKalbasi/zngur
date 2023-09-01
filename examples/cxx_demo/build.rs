use std::{env, path::PathBuf};

use zngur::Zngur;

fn main() {
    build::rerun_if_changed("main.zng");
    build::rerun_if_changed("blobstore.cpp");

    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(crate_dir.join("generated.cpp"))
        .with_h_file(crate_dir.join("generated.h"))
        .with_rs_file(crate_dir.join("./src/generated.rs"))
        .generate();

    cc::Build::new()
        .cpp(true)
        .file("generated.cpp")
        .compile("zngur_generated");

    cc::Build::new()
        .cpp(true)
        .file("blobstore.cpp")
        .compile("blobstore");
}
