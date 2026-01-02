use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use zngur::Zngur;

use crate::cfg_extractor::{CfgFromRustc, cfg_from_rustc};

mod cfg_extractor;

#[derive(Clone)]
struct CfgKey {
    pub key: String,
    pub values: Vec<String>,
}

impl CfgKey {
    fn into_tuple(self) -> (String, Vec<String>) {
        (self.key, self.values)
    }
}

impl<'a> From<&'a str> for CfgKey {
    fn from(s: &'a str) -> Self {
        let (key, values_s) = s.split_once('=').unwrap_or((s, ""));
        let values: Vec<String> = values_s
            .split(',')
            .map(|s| {
                (if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                    &s[1..s.len() - 1]
                } else {
                    s
                })
                .to_owned()
            })
            .collect();
        CfgKey {
            key: key.to_owned(),
            values,
        }
    }
}

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

        /// Path of the dependency file (.d file) to generate
        ///
        /// The dependency file lists all .zng files that were processed.
        /// This can be used by build systems to detect when regeneration is needed.
        #[arg(long)]
        depfile: Option<PathBuf>,

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

        /// A rust config value of the form key(=value1(,value2 ...)) to use when
        /// generating the zngur spec.
        /// i.e.  -C target_os=linux -C target_feature=sse,sse2 -C debug_assertions
        ///
        /// see https://doc.rust-lang.org/reference/conditional-compilation.html
        /// for possible values
        ///
        /// combined with any values loaded from rustc (if enabled)
        ///
        /// Default is an empty configuration
        #[arg(long = "cfg", short = 'C')]
        rust_cfg: Vec<CfgKey>,

        /// A feature name to enable when generating the zngur spec
        ///
        /// combined with any values loaded from rustc (if enabled)
        ///
        /// Default is no features
        #[arg(long = "feature", short = 'F')]
        rust_features: Vec<String>,

        #[command(flatten)]
        load_rustc_cfg: CfgFromRustc,
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
            depfile,
            mangling_base,
            cpp_namespace,
            rust_cfg,
            rust_features,
            load_rustc_cfg,
        } => {
            let pp = path.parent().unwrap();
            let cpp_file = cpp_file.unwrap_or_else(|| pp.join("generated.cpp"));
            let h_file = h_file.unwrap_or_else(|| pp.join("generated.h"));
            let rs_file = rs_file.unwrap_or_else(|| pp.join("src/generated.rs"));
            let mut zng = Zngur::from_zng_file(&path)
                .with_cpp_file(cpp_file)
                .with_h_file(h_file)
                .with_rs_file(rs_file);
            let mut cfg: HashMap<String, Vec<String>> = HashMap::new();
            if load_rustc_cfg.load_cfg_from_rustc {
                cfg.extend(cfg_from_rustc(load_rustc_cfg, &rust_features));
            }
            if !rust_cfg.is_empty() {
                cfg.extend(rust_cfg.into_iter().map(CfgKey::into_tuple));
            }
            if !rust_features.is_empty() {
                cfg.insert("feature".to_owned(), rust_features);
            }
            if !cfg.is_empty() {
                zng = zng.with_rust_in_memory_cfg(cfg);
            }
            if let Some(depfile) = depfile {
                zng = zng.with_depfile(depfile);
            }
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
