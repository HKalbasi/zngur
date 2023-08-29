fn main() {
    cc::Build::new()
        .cpp(true)
        .file("generated.cpp")
        .compile("zngur_generated");
}
