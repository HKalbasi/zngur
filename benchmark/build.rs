use zngur::Zngur;

fn main() {
    build::rerun_if_changed("main.zng");
    build::rerun_if_changed("impls.cpp");
    build::rerun_if_env_changed("CXX");

    let cxx = std::env::var("CXX").unwrap_or("c++".to_owned());

    let lto_enabled = cxx.ends_with("clang++");

    if lto_enabled {
        build::rustc_env("RUSTC_FLAGS", "-C linker-plugin-lto -C linker=clang");
        build::rustc_link_arg("-fuse-ld=lld");
    }

    let crate_dir = build::cargo_manifest_dir();
    let out_dir = build::out_dir();

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(out_dir.join("generated.cpp"))
        .with_h_file(out_dir.join("generated.h"))
        .with_rs_file(out_dir.join("generated.rs"))
        .generate();

    let my_build = &mut cc::Build::new();
    let my_build = my_build
        .cpp(true)
        .compiler(cxx)
        .include(&crate_dir)
        .include(&out_dir);

    if lto_enabled {
        my_build.flag("-flto");
    }

    let my_build = || my_build.clone();

    my_build()
        .file(out_dir.join("generated.cpp"))
        .compile("zngur_generated");
    my_build().file("impls.cpp").compile("impls");
}
