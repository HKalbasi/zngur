//! This crate contains an API for using the Zngur code generator inside build scripts. For more information
//! about the Zngur itself, see [the documentation](https://hkalbasi.github.io/zngur).

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use zngur_generator::{ParsedZngFile, ZngurGenerator};

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
    output_dir: Option<PathBuf>,
    mangling_base: Option<String>,
    cpp_namespace: Option<String>,
    single_header: bool,
}

impl Zngur {
    pub fn from_zng_file(zng_file_path: impl AsRef<Path>) -> Self {
        Zngur {
            zng_file: zng_file_path.as_ref().to_owned(),
            h_file_path: None,
            cpp_file_path: None,
            rs_file_path: None,
            output_dir: None,
            mangling_base: None,
            cpp_namespace: None,
            single_header: false,
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

    pub fn with_output_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.output_dir = Some(path.as_ref().to_owned());
        self
    }

    pub fn with_single_header(mut self, single_header: bool) -> Self {
        self.single_header = single_header;
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

        let generated = file.render();

        if self.single_header {
            // Single header mode: merge utility headers into primary header
            let mut merged_header = String::new();

            // Add utility headers first
            for (_, content) in generated.utility_headers {
                merged_header.push_str(&content);
                merged_header.push('\n');
            }

            // Then add the primary header content, but skip the #include <zngur.h> line
            let primary_without_include = generated.primary_header
                .lines()
                .filter(|line| !line.contains("#include <zngur.h>"))
                .collect::<Vec<_>>()
                .join("\n");
            merged_header.push_str(&primary_without_include);

            File::create(&h_file_path)
                .unwrap()
                .write_all(merged_header.as_bytes())
                .unwrap();
        } else {
            // Split header mode: write utility headers to output directory
            let output_dir = self.output_dir.expect("No output directory provided. Use with_output_dir() to specify where utility headers should be generated.");

            // Create output directory for utility headers
            fs::create_dir_all(&output_dir).unwrap();

            // Write utility headers to output directory
            for (filename, content) in generated.utility_headers {
                let utility_path = output_dir.join(&filename);
                File::create(utility_path)
                    .unwrap()
                    .write_all(content.as_bytes())
                    .unwrap();
            }

            // Print the absolute path to the output directory
            let abs_output_dir = output_dir.canonicalize().unwrap();
            println!("Generated headers directory: {}", abs_output_dir.display());

            File::create(&h_file_path)
                .unwrap()
                .write_all(generated.primary_header.as_bytes())
                .unwrap();
        }

        File::create(rs_file_path)
            .unwrap()
            .write_all(generated.rust_file.as_bytes())
            .unwrap();

        if let Some(cpp) = generated.cpp_file {
            let cpp_file_path = self.cpp_file_path.expect("No cpp file path provided");
            File::create(cpp_file_path)
                .unwrap()
                .write_all(cpp.as_bytes())
                .unwrap();
        }
    }
}
