use clap::Args;
use std::collections::HashMap;

#[derive(Args)]
pub struct CfgFromRustc {
    /// Load rust cfg values using `rustc --print cfg`
    ///
    /// values loaded are in addition to any values provided with `--cfg` or `--feature`
    /// respects the `RUSTFLAGS` environment variable
    #[arg(long, group = "rustc")]
    pub load_cfg_from_rustc: bool,

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

pub fn cfg_from_rustc(
    args: CfgFromRustc,
    rust_features: &[String],
) -> HashMap<String, Vec<String>> {
    let mut cfg: HashMap<String, Vec<String>> = HashMap::new();
    let mut rustflags = parse_rustflags_env();
    if let Some(flags) = &args.rustc_flags {
        rustflags.extend(parse_rustflags(flags))
    }

    let mut cmd = if args.use_cargo_rustc {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("rustc");
        if let Some(package) = &args.cargo_package {
            cmd.args(["--package", package]);
        }
        if let Some(profile) = &args.cargo_profile {
            cmd.args(["--profile", profile]);
        }
        if !rust_features.is_empty() && !args.all_features {
            cmd.args(["--features", &rust_features.join(",")]);
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

    if let Some(target) = &args.rustc_target {
        cmd.args(["--target", target]);
    }

    if args.use_cargo_rustc {
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
