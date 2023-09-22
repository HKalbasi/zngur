use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use zngur_generator::{ParsedZngFile, ZngurGenerator};

#[must_use]
pub struct Zngur {
    zng_file: PathBuf,
    h_file_path: Option<PathBuf>,
    cpp_file_path: Option<PathBuf>,
    rs_file_path: Option<PathBuf>,
}

impl Zngur {
    pub fn from_zng_file(zng_file_path: impl AsRef<Path>) -> Self {
        Zngur {
            zng_file: zng_file_path.as_ref().to_owned(),
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

    pub fn generate(self) {
        let path = self.zng_file;
        let file = std::fs::read_to_string(&path).unwrap();
        let file = ParsedZngFile::parse("main.zng", &file, |f| ZngurGenerator::build_from_zng(f));

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
