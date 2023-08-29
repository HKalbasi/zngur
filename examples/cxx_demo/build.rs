fn main() {
    build::rerun_if_changed("generated.cpp");
    build::rerun_if_changed("blobstore.cpp");

    cc::Build::new()
        .cpp(true)
        .file("generated.cpp")
        .compile("zngur_generated");

    cc::Build::new()
        .cpp(true)
        .file("blobstore.cpp")
        .compile("blobstore");
}
