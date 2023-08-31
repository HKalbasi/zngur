#!/usr/bin/env -S cargo +nightly -Zscript

//! ```cargo
//! [package]
//! edition = "2021"
//!
//! [dependencies]
//! xshell = "0.2.5"
//! anyhow = "1.0"
//! ```

use anyhow::{Context, Result};
use xshell::{cmd, Shell};

fn check_examples(sh: Shell) -> Result<()> {
    const CARGO_PROJECTS: &[&str] = &["cxx_demo"];
    sh.change_dir("examples");
    let examples = cmd!(sh, "ls").read()?;
    for example in examples.lines() {
        sh.change_dir(example);
        if CARGO_PROJECTS.contains(&example) {
            cmd!(sh, "cargo build")
                .run()
                .with_context(|| format!("Building example `{example}` failed"))?;
            cmd!(sh, "cargo run")
                .run()
                .with_context(|| format!("Running example `{example}` failed"))?;
        } else {
            cmd!(sh, "make")
                .run()
                .with_context(|| format!("Building example `{example}` failed"))?;
            cmd!(sh, "./a.out")
                .run()
                .with_context(|| format!("Running example `{example}` failed"))?;
        }
        sh.change_dir("..");
    }
    Ok(())
}

fn main() -> Result<()> {
    let sh = Shell::new()?;
    sh.set_var("RUSTFLAGS", "-D warnings");
    cmd!(sh, "cargo check").run()?;
    check_examples(sh).with_context(|| "Checking examples failed")?;
    Ok(())
}
