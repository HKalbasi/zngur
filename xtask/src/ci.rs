use anyhow::{Context, Result};
use xshell::{Shell, cmd};

fn check_crate(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo check").run()?;
    cmd!(sh, "cargo fmt --check")
        .run()
        .with_context(|| "Crate is not formatted. Run `cargo fmt`")?;
    Ok(())
}

fn check_examples(sh: &Shell) -> Result<()> {
    const CARGO_PROJECTS: &[&str] = &["cxx_demo", "osmium", "tutorial_cpp"];
    sh.change_dir("examples");
    let examples = cmd!(sh, "ls").read()?;
    for example in examples.lines() {
        sh.change_dir(example);
        if example == "osmium" {
            if cfg!(target_os = "macos") {
                sh.change_dir("..");
                continue;
            }
            if !sh.path_exists("map.osm") {
                cmd!(sh, "wget -O map.osm https://api.openstreetmap.org/api/0.6/map?bbox=36.58848,51.38459,36.63783,51.55314").run()?;
            }
        }
        if CARGO_PROJECTS.contains(&example) {
            cmd!(sh, "cargo build")
                .run()
                .with_context(|| format!("Building example `{example}` failed"))?;
            cmd!(sh, "cargo run")
                .run()
                .with_context(|| format!("Running example `{example}` failed"))?;
            cmd!(sh, "cargo fmt --check").run().with_context(|| {
                format!("Example `{example}` is not formatted. Run `cargo fmt`")
            })?;
        } else {
            cmd!(sh, "make")
                .run()
                .with_context(|| format!("Building example `{example}` failed"))?;
            cmd!(sh, "./a.out")
                .run()
                .with_context(|| format!("Running example `{example}` failed"))?;
            cmd!(sh, "cargo fmt --check").run().with_context(|| {
                format!("Example `{example}` is not formatted. Run `cargo fmt`")
            })?;
        }
        sh.change_dir("..");
    }
    Ok(())
}

pub fn main() -> Result<()> {
    let sh = &Shell::new()?;
    println!("Cargo version = {}", cmd!(sh, "cargo --version").read()?);
    let cxx = std::env::var("CXX")?;
    println!("CXX version = {}", cmd!(sh, "{cxx} --version").read()?);
    sh.set_var("RUSTFLAGS", "-D warnings");
    for dir in cmd!(sh, "ls").read()?.lines() {
        if sh.path_exists(format!("{dir}/Cargo.toml")) {
            sh.change_dir(dir);
            check_crate(sh).with_context(|| format!("Checking crate {dir} failed"))?;
            sh.change_dir("..")
        }
    }
    check_examples(sh).with_context(|| "Checking examples failed")?;
    Ok(())
}
