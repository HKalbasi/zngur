fn main() {
    cc::Build::new()
        .cpp(true)
        .compiler("clang++")
        .file("generated.cpp")
        .compile("zngur_generated");
}
