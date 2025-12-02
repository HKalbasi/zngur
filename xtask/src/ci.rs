use crate::format_book;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;
use xshell::{Shell, cmd};

fn check_crate(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo check").run()?;
    cmd!(sh, "cargo fmt --check")
        .run()
        .with_context(|| "Crate is not formatted. Run `cargo fmt`")?;
    Ok(())
}

fn check_book_formatting() -> Result<()> {
    format_book::main(false /* don't fix */)
        .with_context(|| "Book markdown files are not formatted. Run `cargo xtask format-book`")
}

fn check_single_example(workspace_root: &PathBuf, example: &str, fix: bool) -> Result<()> {
    const CARGO_PROJECTS: &[&str] = &["cxx_demo", "tutorial_cpp"];

    let sh = Shell::new()?;
    sh.change_dir(workspace_root.join("examples").join(example));

    if CARGO_PROJECTS.contains(&example) {
        // Clean generated files for Cargo projects
        let _ = cmd!(
            sh,
            "rm -f generated.h generated.cpp src/generated.rs actual_output.txt"
        )
        .run();
        cmd!(sh, "cargo build")
            .run()
            .with_context(|| format!("Building example `{example}` failed"))?;
        let bash_cmd = format!(
            "../../target/debug/example-{example} 2>&1 | sed 's/thread .* panicked/thread panicked/g' > actual_output.txt"
        );
        cmd!(sh, "bash -c {bash_cmd}")
            .run()
            .with_context(|| format!("Running example `{example}` failed"))?;
    } else {
        // Clean generated files for Make projects
        cmd!(sh, "make clean")
            .run()
            .with_context(|| format!("Cleaning example `{example}` failed"))?;
        cmd!(sh, "make")
            .run()
            .with_context(|| format!("Building example `{example}` failed"))?;
        cmd!(
            sh,
            "bash -c './a.out 2>&1 | sed s/thread.*panicked/thread\\ panicked/g > actual_output.txt'"
        )
        .run()
        .with_context(|| format!("Running example `{example}` failed"))?;
    }
    if fix {
        sh.copy_file("./actual_output.txt", "./expected_output.txt")?;
    }
    cmd!(sh, "diff actual_output.txt expected_output.txt")
        .run()
        .with_context(|| format!("Example `{example}` output differs from expected."))?;
    cmd!(sh, "cargo fmt --check")
        .run()
        .with_context(|| format!("Example `{example}` is not formatted. Run `cargo fmt`"))?;

    Ok(())
}

fn check_examples(sh: &Shell, fix: bool) -> Result<()> {
    // Pre-build zngur-cli once - Makefiles will use the binary directly
    println!("Pre-building zngur-cli...");
    cmd!(sh, "cargo build --package zngur-cli")
        .run()
        .with_context(|| "Failed to build zngur-cli")?;

    let workspace_root = sh.current_dir();
    let zngur_bin = workspace_root.join("target/debug/zngur");

    // Set ZNGUR environment variable for all parallel Make processes
    // This allows Makefiles to run the binary directly instead of using cargo run
    sh.set_var("ZNGUR", &zngur_bin);

    sh.change_dir("examples");
    let examples: Vec<String> = cmd!(sh, "ls")
        .read()?
        .lines()
        .map(|s| s.to_string())
        .collect();
    sh.change_dir("..");

    // Build examples in parallel (limit to 4 at a time to avoid resource exhaustion)
    // Now each example runs the binary directly - no cargo run overhead!
    println!("Building {} examples in parallel...", examples.len());
    examples
        .par_iter()
        .with_max_len(4)
        .try_for_each(|example| {
            println!("Building example: {}", example);
            check_single_example(&workspace_root, example, fix)
        })?;

    Ok(())
}

pub fn main(fix: bool) -> Result<()> {
    let sh = &Shell::new()?;
    println!("Cargo version = {}", cmd!(sh, "cargo --version").read()?);
    let cxx = sh.var("CXX")?;
    println!("CXX version = {}", cmd!(sh, "{cxx} --version").read()?);
    sh.set_var("RUSTFLAGS", "-D warnings");
    if fix {
        cmd!(sh, "cargo fmt --all").run()?;
        if let Err(e) = format_book::main(true /* fix */) {
            eprintln!("Warning: Failed to format book: {}", e);
        }
    }
    // Check book formatting
    check_book_formatting().with_context(|| "Book formatting check failed")?;

    for dir in cmd!(sh, "ls").read()?.lines() {
        if sh.path_exists(format!("{dir}/Cargo.toml")) {
            sh.change_dir(dir);
            check_crate(sh).with_context(|| format!("Checking crate {dir} failed"))?;
            sh.change_dir("..")
        }
    }
    check_examples(sh, fix).with_context(|| "Checking examples failed")?;
    cmd!(sh, "cargo test --all-features").run()?;
    Ok(())
}
