fn main() {
    let dir = std::env::current_dir().unwrap();
    println!("cargo:rustc-link-arg={}/generated.o", dir.display());
    println!("cargo:rustc-link-arg={}/impls.o", dir.display());
    println!("cargo:rustc-link-lib=stdc++");
}
