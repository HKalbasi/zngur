//! This crate contains an API for using the Zngur code generator inside build scripts. For more information
//! about the Zngur itself, see [the documentation](https://hkalbasi.github.io/zngur).

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use zngur_generator::{LayoutPolicy, ParsedZngFile, ZngurGenerator, ZngurSpec};

#[must_use]
/// Builder for the Zngur generator.
///
/// Usage:
/// ```ignore
/// let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
/// let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
/// Zngur::from_zng_file(crate_dir.join("main.zng"))
///     .with_cpp_file(out_dir.join("generated.cpp"))
///     .with_h_file(out_dir.join("generated.h"))
///     .with_rs_file(out_dir.join("generated.rs"))
///     .generate();
/// ```
pub struct Zngur {
    zng_file: PathBuf,
    h_file_path: Option<PathBuf>,
    cpp_file_path: Option<PathBuf>,
    rs_file_path: Option<PathBuf>,
    mangling_base: Option<String>,
    cpp_namespace: Option<String>,
}

impl Zngur {
    pub fn from_zng_file(zng_file_path: impl AsRef<Path>) -> Self {
        Zngur {
            zng_file: zng_file_path.as_ref().to_owned(),
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
            mangling_base: None,
            cpp_namespace: None,
        }
    }

    pub fn with_h_file(mut self, path: impl AsRef<Path>) -> Self {
        self.h_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_cpp_file(mut self, path: impl AsRef<Path>) -> Self {
        self.cpp_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_rs_file(mut self, path: impl AsRef<Path>) -> Self {
        self.rs_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_mangling_base(mut self, mangling_base: &str) -> Self {
        self.mangling_base = Some(mangling_base.to_owned());
        self
    }

    pub fn with_cpp_namespace(mut self, cpp_namespace: &str) -> Self {
        self.cpp_namespace = Some(cpp_namespace.to_owned());
        self
    }

    pub fn generate(self) {
        let mut file = ZngurGenerator::build_from_zng(ParsedZngFile::parse(self.zng_file));

        let rs_file_path = self.rs_file_path.expect("No rs file path provided");
        let h_file_path = self.h_file_path.expect("No h file path provided");

        file.0.cpp_include_header_name = h_file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        file.0.cpp_namespace = "rust".to_owned();

        if let Some(cpp_namespace) = self.cpp_namespace {
            file.0.mangling_base = cpp_namespace.clone();
            file.0.cpp_namespace = cpp_namespace;
        }

        if let Some(mangling_base) = self.mangling_base {
            file.0.mangling_base = mangling_base;
        }

        let cpp_namespace = file.0.cpp_namespace.clone();

        let (rust, mut h, mut cpp) = file.render();

        // TODO: Don't hard code namespace as "::rust" and remove this replace
        h = h
            .replace("rust::", &format!("{cpp_namespace}::"))
            .replace("namespace rust", &format!("namespace {cpp_namespace}"));
        cpp = cpp.map(|cpp| cpp.replace("rust::", &format!("{cpp_namespace}::")));

        File::create(rs_file_path)
            .unwrap()
            .write_all(rust.as_bytes())
            .unwrap();
        File::create(h_file_path)
            .unwrap()
            .write_all(h.as_bytes())
            .unwrap();
        if let Some(cpp) = cpp {
            let cpp_file_path = self.cpp_file_path.expect("No cpp file path provided");
            File::create(cpp_file_path)
                .unwrap()
                .write_all(cpp.as_bytes())
                .unwrap();
        }
    }
}

pub struct AutoZngur {
    h_file_path: Option<PathBuf>,
    cpp_file_path: Option<PathBuf>,
    rs_file_path: Option<PathBuf>,
}

impl AutoZngur {
    pub fn new() -> Self {
        AutoZngur {
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
        }
    }

    pub fn with_h_file(mut self, path: impl AsRef<Path>) -> Self {
        self.h_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_cpp_file(mut self, path: impl AsRef<Path>) -> Self {
        self.cpp_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_rs_file(mut self, path: impl AsRef<Path>) -> Self {
        self.rs_file_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn generate(
        self,
        crate_map: HashMap<u32, rustdoc_types::Crate>,
        sizes: HashMap<String, LayoutPolicy>,
    ) {
        let spec = ZngurSpec::from_crate(crate_map, sizes);
        let file = ZngurGenerator::build_from_zng(spec);

        let (rust, h, cpp) = file.render();
        let rs_file_path = self.rs_file_path.expect("No rs file path provided");
        let h_file_path = self.h_file_path.expect("No h file path provided");
        File::create(rs_file_path)
            .unwrap()
            .write_all(rust.as_bytes())
            .unwrap();
        File::create(h_file_path)
            .unwrap()
            .write_all(h.as_bytes())
            .unwrap();
        if let Some(cpp) = cpp {
            let cpp_file_path = self.cpp_file_path.expect("No cpp file path provided");
            File::create(cpp_file_path)
                .unwrap()
                .write_all(cpp.as_bytes())
                .unwrap();
        }
    }
}

#[derive(Debug)]
pub struct SizeInfo {
    pub size: u32,
    pub alignment: u32,
}
