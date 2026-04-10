use crate::format_book;
use anyhow::{Context, Result};
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

fn check_examples(sh: &Shell, fix: bool) -> Result<()> {
    const CARGO_PROJECTS: &[&str] = &["cxx_demo", "tutorial_cpp"];
    let examples_dir = sh.current_dir().join("examples");
    sh.change_dir("examples");
    let examples = sh.read_dir(".")?;
    for example in examples {
        let mut skip = false;
        let example = example
            .file_name()
            .unwrap_or_default()
            .to_str()
            .ok_or(anyhow::anyhow!("Non utf8 example name?"))?;
        let path = examples_dir.join(example);
        if !path.is_dir() {
            continue;
        }
        println!("Building and testing {example}");
        sh.change_dir(example);
        println!("Working in {}", sh.current_dir().display());
        if CARGO_PROJECTS.contains(&example) {
            #[cfg(not(target_os = "windows"))]
            {
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
                    "../../target/debug/example-{example} 2>&1 | sed -e s/thread.*\\(.*\\)/thread/g > actual_output.txt"
                );
                cmd!(sh, "bash -c {bash_cmd}")
                    .run()
                    .with_context(|| format!("Running example `{example}` failed"))?;
            }
            #[cfg(target_os = "windows")]
            {
                // Clean generated files for Cargo projects
                let _ = cmd!(
                    sh,
                    "cmd /c 'del /f /q generated.h generated.cpp src\\generated.rs actual_output.txt 2>nul'"
                )
                .run();
                cmd!(sh, "cmd /c 'cargo build'")
                    .run()
                    .with_context(|| format!("Building example `{example}` failed"))?;
                let batch_cmd =
                    format!("..\\..\\target\\debug\\example-{example} > actual_output.txt 2>&1");
                cmd!(sh, "cmd /c {batch_cmd}")
                    .run()
                    .with_context(|| format!("Running example `{example}` failed"))?;
                cmd!(sh, "pwsh -Command '(Get-Content actual_output.txt) -replace \"thread.*\\(.*\\)\", \"thread\" -replace \"\\\\\", \"/\"| Out-File actual_output.txt'")
                        .run()
                        .with_context(|| format!("Filtering example `{example}` thread output failed"))?;
            }
        } else {
            #[cfg(not(target_os = "windows"))]
            if sh.path_exists("Makefile") {
                // Clean generated files for Make projects
                cmd!(sh, "make clean")
                    .run()
                    .with_context(|| format!("Cleaning example `{example}` failed"))?;
                cmd!(sh, "make")
                    .run()
                    .with_context(|| format!("Building example `{example}` failed"))?;
                cmd!(
                    sh,
                    "bash -c './a.out 2>&1 | sed -e s/thread.*\\(.*\\)/thread/g > actual_output.txt'"
                )
                .run()
                .with_context(|| format!("Running example `{example}` failed"))?;
                if sh.path_exists("b.out") {
                    cmd!(
                        sh,
                        "bash -c './b.out 2>&1 | sed -r s/thread.*\\(.*\\)/thread/g >> actual_output.txt'"
                    )
                    .run()
                    .with_context(|| format!("Running example `{example}` b.out failed"))?;
                }
            } else {
                skip = true;
                println!("Skipping {example}, no Makefile for this example");
            }

            #[cfg(target_os = "windows")]
            if sh.path_exists("NMakefile") {
                // Clean generated files for NMake projects
                cmd!(sh, "nmake /f NMakefile clean")
                    .run()
                    .with_context(|| format!("Cleaning example `{example}` failed"))?;
                cmd!(sh, "nmake /f NMakefile")
                    .run()
                    .with_context(|| format!("Building example `{example}` failed"))?;
                if sh.path_exists("a.bat") {
                    cmd!(sh, "cmd /c '.\\a.bat > actual_output.txt 2>&1'")
                        .run()
                        .with_context(|| format!("Running example `{example}` failed"))?;
                } else {
                    cmd!(sh, "cmd /c '.\\a.exe > actual_output.txt 2>&1'")
                        .run()
                        .with_context(|| format!("Running example `{example}` failed"))?;
                }
                if sh.path_exists("b.exe") {
                    cmd!(sh, "cmd /c '.\\b.exe >> actual_output.txt 2>&1'")
                        .run()
                        .with_context(|| format!("Running example `{example}` b.exe failed"))?;
                }
                cmd!(sh, "pwsh -Command '(Get-Content actual_output.txt) -replace \"thread.*\\(.*\\)\", \"thread\" -replace \"\\\\\", \"/\"| Out-File actual_output.txt'")
                        .run()
                        .with_context(|| format!("Filtering example `{example}` thread output failed"))?;
            } else {
                skip = true;
                println!("Skipping {example}, no NMakefile for this example");
            }
        }
        if fix {
            sh.copy_file("./actual_output.txt", "./expected_output.txt")?;
        }

        if !skip {
            #[cfg(not(target_os = "windows"))]
            cmd!(sh, "diff actual_output.txt expected_output.txt")
                .run()
                .with_context(|| format!("Example `{example}` output differs from expected."))?;
            #[cfg(target_os = "windows")]
            cmd!(sh, "cmd /c 'fc actual_output.txt expected_output.txt'")
                .run()
                .with_context(|| format!("Example `{example}` output differs from expected."))?;

            cmd!(sh, "cargo fmt --check").run().with_context(|| {
                format!("Example `{example}` is not formatted. Run `cargo fmt`")
            })?;
        }

        sh.change_dir(&examples_dir);
    }
    Ok(())
}

pub fn main(fix: bool) -> Result<()> {
    let sh = &Shell::new()?;
    println!("Cargo version = {}", cmd!(sh, "cargo --version").read()?);

    #[cfg(not(target_os = "windows"))]
    {
        let cxx = sh.var("CXX")?;
        println!("CXX version = {}", cmd!(sh, "{cxx} --version").read()?);
    }
    #[cfg(target_os = "windows")]
    let msvc_ver: u32 = {
        let msvc = String::from_utf8(cmd!(sh, "cl.exe /help").output()?.stderr)?;
        let msvc = msvc
            .lines()
            .next()
            .and_then(|line| line.split("Version ").nth(1))
            .unwrap_or_default();

        let mut parts = msvc.split(".");
        let major = parts.next().unwrap_or("0");
        let minor = parts.next().unwrap_or("0");
        let msvc_ver: u32 = format!("{major}{minor}").parse()?;

        println!("MSVC version = {msvc} ({msvc_ver})",);
        msvc_ver
    };

    #[cfg(not(target_os = "windows"))]
    sh.set_var("RUSTFLAGS", "-D warnings");

    #[cfg(target_os = "windows")]
    {
        // Prevent link.exe error:1318
        let link_debug = if msvc_ver > 1944 {
            // sorry, msvc 2026 removed FASTLINK support so it can't prevent pdb
            // corruption. best we can do is disable debug symbols
            "/DEBUG:NONE"
        } else {
            "/DEBUG:FASTLINK"
        };
        sh.set_var(
            "RUSTFLAGS",
            format!("-D warnings -C link-arg={link_debug} -C link-arg=/INCREMENTAL:NO"),
        );
    }

    if fix {
        cmd!(sh, "cargo fmt --all").run()?;
        if let Err(e) = format_book::main(true /* fix */) {
            eprintln!("Warning: Failed to format book: {}", e);
        }
    }

    #[cfg(not(target_os = "windows"))]
    cmd!(sh, "cspell .")
        .run()
        .with_context(|| "Failed to check word spellings")?;
    #[cfg(target_os = "windows")]
    cmd!(sh, "cspell.cmd .")
        .run()
        .with_context(|| "Failed to check word spellings")?;

    // Check book formatting
    check_book_formatting().with_context(|| "Book formatting check failed")?;

    for dir in sh.read_dir(".")? {
        if sh.path_exists(dir.join("Cargo.toml")) {
            let _crate_dir = sh.push_dir(dir.file_name().unwrap_or_default());
            check_crate(sh).with_context(|| format!("Checking crate {dir:?} failed"))?;
        }
    }

    check_examples(sh, fix).with_context(|| "Checking examples failed")?;

    // it's nigh impossible to get this to run under MSVC
    #[cfg(not(target_os = "windows"))]
    cmd!(sh, "cargo test --all-features").run()?;

    Ok(())
}
