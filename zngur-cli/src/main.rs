use rustdoc_json::*;
use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use zngur::AutoZngur;

#[derive(Parser)]
#[command(version)]
enum Command {
    #[command(alias = "g")]
    Generate {
        /// Path to the zng file
        path: PathBuf,

        /// Path of the generated C++ file, if it is needed
        ///
        /// Default is {ZNG_FILE_PARENT}/generated.cpp
        #[arg(long)]
        cpp_file: Option<PathBuf>,

        /// Path of the generated header file
        ///
        /// Default is {ZNG_FILE_PARENT}/generated.h
        #[arg(long)]
        h_file: Option<PathBuf>,

        /// Path of the generated Rust file
        ///
        /// Default is {ZNG_FILE_PARENT}/src/generated.rs
        #[arg(long)]
        rs_file: Option<PathBuf>,

        /// A unique string which is included in zngur symbols to prevent duplicate
        /// symbols in linker
        ///
        /// Default is the value of cpp_namespace, so you don't need to set this manually
        /// if you change cpp_namespace as well
        #[arg(long)]
        mangling_base: Option<String>,

        /// The C++ namespace which zngur puts its things in it. You can change it
        /// to prevent violation of ODR when you have multiple independent zngur
        /// libraries
        ///
        /// Default is "rust"
        #[arg(long)]
        cpp_namespace: Option<String>,
    },
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Generate {
            path,
            cpp_file,
            h_file,
            rs_file,
            mangling_base,
            cpp_namespace,
        } => {
            let pp = path.parent().unwrap();
            let cpp_file = cpp_file.unwrap_or_else(|| pp.join("generated.cpp"));
            let h_file = h_file.unwrap_or_else(|| pp.join("generated.h"));
            let rs_file = rs_file.unwrap_or_else(|| pp.join("src/generated.rs"));
            let mut zng = Zngur::from_zng_file(&path)
                .with_cpp_file(cpp_file)
                .with_h_file(h_file)
                .with_rs_file(rs_file);
            if let Some(mangling_base) = mangling_base {
                zng = zng.with_mangling_base(&mangling_base);
            }
            if let Some(cpp_namespace) = cpp_namespace {
                zng = zng.with_cpp_namespace(&cpp_namespace);
            }
            zng.generate();
        }
    }
}
