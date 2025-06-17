use anyhow::{Context, Result};
use xshell::{Shell, cmd};

fn check_crate(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo check").run()?;
    cmd!(sh, "cargo fmt --check")
        .run()
        .with_context(|| "Crate is not formatted. Run `cargo fmt`")?;
    Ok(())
}

fn check_examples(sh: &Shell, fix: bool) -> Result<()> {
    const CARGO_PROJECTS: &[&str] = &["cxx_demo", "tutorial_cpp"];
    sh.change_dir("examples");
    let examples = cmd!(sh, "ls").read()?;
    for example in examples.lines() {
        sh.change_dir(example);
        if CARGO_PROJECTS.contains(&example) {
            cmd!(sh, "cargo build")
                .run()
                .with_context(|| format!("Building example `{example}` failed"))?;
            let bash_cmd = format!("../../target/debug/example-{example} > actual_output.txt 2>&1");
            cmd!(sh, "bash -c {bash_cmd}")
                .run()
                .with_context(|| format!("Running example `{example}` failed"))?;
        } else {
            cmd!(sh, "make")
                .run()
                .with_context(|| format!("Building example `{example}` failed"))?;
            cmd!(sh, "bash -c './a.out > actual_output.txt 2>&1'")
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
        sh.change_dir("..");
    }
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
    }
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
