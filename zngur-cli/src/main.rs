use std::{collections::HashMap, path::PathBuf};

use clap::{Args, Parser};
use zngur::Zngur;

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

#[derive(Args)]
struct CfgFromRustc {
    /// Load rust cfg values using `rustc --print cfg`
    ///
    /// values loaded are in addition to any values provided with `--cfg` or `--feature`
    /// respects the `RUSTFLAGS` environment variable
    #[arg(long, group = "rustc")]
    load_cfg_from_rustc: bool,

    /// Use `cargo rustc -- --print cfg` which allows automatically collecting profile flags and
    /// features
    ///
    /// WARNING: because `cargo rustc --print` is unstable this uses `cargo rustc -- --print` which
    /// may invoke side effects like downloading all crate dependencies
    ///
    /// (requires --load-cfg-from-rustc)
    #[arg(long, group = "cargo_rustc", requires = "rustc")]
    use_cargo_rustc: bool,

    /// flags to pass to rustc when loading cfg values
    ///
    /// (requires --load-cfg-from-rustc)
    #[arg(long, requires = "rustc")]
    rustc_flags: Option<String>,

    /// A target to provide to `rustc` when loading config values
    ///
    /// (requires --load-cfg-from-rustc)
    #[arg(long = "target", requires = "rustc")]
    rustc_target: Option<String>,

    /// cargo profile to use when loading cfg values
    ///
    /// (requires --use-cargo-rustc)
    #[arg(long = "profile", requires = "cargo_rustc")]
    cargo_profile: Option<String>,

    /// cargo package to use when loading cfg values
    ///
    /// (requires --use-cargo-rustc)
    #[arg(long = "package", requires = "cargo_rustc")]
    cargo_package: Option<String>,

    /// passes --no-default-features to cargo
    ///
    /// (requires --use-cargo-rustc)
    #[arg(long, requires = "cargo_rustc")]
    no_default_features: bool,

    /// passes --all_features to cargo
    ///
    /// (requires --use-cargo-rustc)
    #[arg(long, requires = "cargo_rustc")]
    all_features: bool,
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
                cfg.extend(cfg_from_rustc(RustcCfgArgs {
                    rustc_flags: load_rustc_cfg.rustc_flags.as_deref(),
                    target: load_rustc_cfg.rustc_target.as_deref(),
                    use_cargo: load_rustc_cfg.use_cargo_rustc,
                    profile: load_rustc_cfg.cargo_profile.as_deref(),
                    features: rust_features
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    package: load_rustc_cfg.cargo_package.as_deref(),
                    no_default_features: load_rustc_cfg.no_default_features,
                    all_features: load_rustc_cfg.all_features,
                }));
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

struct RustcCfgArgs<'a> {
    rustc_flags: Option<&'a str>,
    target: Option<&'a str>,
    use_cargo: bool,
    profile: Option<&'a str>,
    features: &'a [&'a str],
    package: Option<&'a str>,
    no_default_features: bool,
    all_features: bool,
}

fn cfg_from_rustc(args: RustcCfgArgs) -> HashMap<String, Vec<String>> {
    let mut cfg: HashMap<String, Vec<String>> = HashMap::new();
    let mut rustflags = parse_rustflags_env();
    if let Some(flags) = args.rustc_flags {
        rustflags.extend(parse_rustflags(flags))
    }

    let mut cmd = if args.use_cargo {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("rustc");
        if let Some(package) = args.package {
            cmd.args(["--package", package]);
        }
        if let Some(profile) = args.profile {
            cmd.args(["--profile", profile]);
        }
        if !args.features.is_empty() && !args.all_features {
            cmd.args(["--features", &args.features.join(",")]);
        }
        if args.all_features {
            cmd.arg("--all-features");
        }
        if args.no_default_features {
            cmd.arg("--no-default-features");
        }
        cmd
    } else {
        std::process::Command::new("rustc")
    };

    if let Some(target) = args.target {
        cmd.args(["--target", target]);
    }

    if args.use_cargo {
        cmd.arg("--");
    }

    cmd.args(rustflags);

    cmd.args(["--print", "cfg"]);

    let out = cmd.output().expect("failed to print cfg with rustc");

    if !out.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        std::process::exit(1);
    }

    let out = String::from_utf8(out.stdout).expect("failed to parse rustc output as utf8");

    let lines = out.split('\n').collect::<Vec<_>>();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let (key, value) = line.trim().split_once('=').unwrap_or((line, ""));
        let value = if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            &value[1..value.len() - 1]
        } else {
            value
        };
        let entry = cfg.entry(key.to_owned()).or_default();
        entry.push(value.to_owned())
    }

    cfg
}

fn parse_rustflags_env() -> Vec<String> {
    let flags_str = std::env::var("RUSTFLAGS").unwrap_or_default();
    parse_rustflags(&flags_str)
}

fn parse_rustflags(flags_str: &str) -> Vec<String> {
    let mut word: String = String::new();
    let mut flags: Vec<String> = Vec::new();

    #[derive(Copy, Clone)]
    enum State {
        Delem,
        Unquoted,
        Escape(&'static State),
        Single,
        Double,
    }
    use State::*;

    let mut state = Delem;

    let mut chars = flags_str.chars();

    loop {
        let Some(c) = chars.next() else {
            match state {
                Delem => break,
                Unquoted | Single | Double => {
                    flags.push(std::mem::take(&mut word));
                    break;
                }
                Escape(_) => {
                    word.push('\\');
                    flags.push(std::mem::take(&mut word));
                    break;
                }
            }
        };
        state = match state {
            Delem => match c {
                '\'' => Single,
                '"' => Double,
                '\\' => Escape(&Delem),
                '\t' | ' ' | '\n' => Delem,
                c => {
                    word.push(c);
                    Unquoted
                }
            },
            Unquoted => match c {
                '\'' => Single,
                '"' => Double,
                '\\' => Escape(&Unquoted),
                '\t' | ' ' | '\n' => {
                    flags.push(std::mem::take(&mut word));
                    Delem
                }
                c => {
                    word.push(c);
                    Unquoted
                }
            },
            Escape(next_state) => match c {
                c @ '"' | c @ '\\' if matches!(next_state, Double) => {
                    word.push(c);
                    Double
                }
                '\n' => *next_state,
                c => {
                    word.push('\\');
                    word.push(c);
                    *next_state
                }
            },
            Single => match c {
                '\'' => Unquoted,
                c => {
                    word.push(c);
                    Single
                }
            },
            Double => match c {
                '"' => Unquoted,
                '\\' => Escape(&Double),
                c => {
                    word.push(c);
                    Double
                }
            },
        }
    }

    flags
}
