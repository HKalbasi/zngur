use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

/// A throwaway cargo project used to build a `//@ pass` test. The test's `.rs`
/// is copied to `src/main.rs` and its C++ file is compiled into the binary by
/// `build.rs`. Kept out of the workspace so no manifest is needed in `tests/`.
const RUNNER_CARGO_TOML: &str = r#"[package]
name = "zngur-e2e-runner"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]

[build-dependencies]
cc = "1.0"
"#;

const RUNNER_BUILD_RS: &str = r#"use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp = PathBuf::from(std::env::var("ZNGUR_E2E_CPP").unwrap());

    let mut build = cc::Build::new();
    build.cpp(true).std("c++17").include(&dir);
    // The test's own headers live next to its `.cpp`, so a `#cpp_additional_includes`
    // can pull them in even though it is emitted into the runner's `generated.h`.
    build.include(cpp.parent().unwrap());
    build.file(dir.join("generated.cpp")).file(&cpp).compile("zngur_e2e");

    println!("cargo:rerun-if-changed=generated.cpp");
    println!("cargo:rerun-if-changed={}", cpp.display());
    println!("cargo:rerun-if-env-changed=ZNGUR_E2E_CPP");
}
"#;

/// Repository root, resolved from this crate's manifest so that the paths inside
/// the diagnostics don't depend on the directory `cargo xtask` was invoked from.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives inside the repository")
        .to_owned()
}

/// The `//@ pass` or `//@ fail` directive of a test.
fn directive(zng_file: &Path) -> Result<&'static str> {
    let text = std::fs::read_to_string(zng_file)
        .with_context(|| format!("reading {}", zng_file.display()))?;
    for line in text.lines() {
        if let Some(directive) = line.trim().strip_prefix("//@") {
            return match directive.trim() {
                "pass" => Ok("pass"),
                "fail" => Ok("fail"),
                other => bail!("unknown directive `//@{other}` in {}", zng_file.display()),
            };
        }
    }
    bail!(
        "missing a `//@ pass` or `//@ fail` directive in {}",
        zng_file.display()
    )
}

/// Strip colors from the CLI output and drop the trailing panic report, which
/// contains a non-deterministic thread id.
fn normalize(stderr: &[u8]) -> String {
    let stderr = strip_ansi_escapes::strip(stderr);
    String::from_utf8_lossy(&stderr)
        .lines()
        .take_while(|line| !line.starts_with("thread 'main'"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

/// `zngur generate` must reject `<stem>.zng`. The diagnostics it prints,
/// including the source spans, are compared with the `<stem>.stderr` snapshot.
fn check_fail(sh: &Shell, fix: bool, stem: &str, test_dir: &Path) -> Result<()> {
    let out_dir = repo_root().join("target/e2e-fail");
    // Creating the output directory first makes an unexpectedly accepted file
    // fail the check below instead of failing to write its generated files.
    std::fs::create_dir_all(&out_dir)?;
    let rs_file = out_dir.join("generated.rs");
    let h_file = out_dir.join("generated.h");
    let cpp_file = out_dir.join("generated.cpp");

    // The CLI runs in the test's directory, so spans point at `<stem>.zng`.
    let output = {
        let _dir = sh.push_dir(test_dir);
        cmd!(
            sh,
            "cargo run -q -p zngur-cli -- generate {stem}.zng --rs-file {rs_file} --h-file {h_file} --cpp-file {cpp_file}"
        )
        .ignore_status()
        .output()?
    };
    if output.status.success() {
        bail!("`{stem}.zng` is marked `//@ fail` but the CLI accepted it");
    }

    check_stderr(fix, stem, test_dir, &normalize(&output.stderr), true)
}

/// Compare `actual` (normalized stderr) with the `<stem>.stderr` snapshot, or
/// write it when `fix` is set. When `required` is false a missing snapshot is
/// allowed only if there is no output; warnings without a snapshot are an error.
fn check_stderr(
    fix: bool,
    stem: &str,
    test_dir: &Path,
    actual: &str,
    required: bool,
) -> Result<()> {
    let expected_file = test_dir.join(format!("{stem}.stderr"));
    if fix {
        // Don't create an empty snapshot for a passing test that has none.
        if required || !actual.is_empty() || expected_file.exists() {
            std::fs::write(&expected_file, format!("{actual}\n"))
                .with_context(|| format!("writing {}", expected_file.display()))?;
        }
        return Ok(());
    }
    let expected = match std::fs::read_to_string(&expected_file) {
        Ok(expected) => expected,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && !required => {
            if actual.is_empty() {
                return Ok(());
            }
            bail!(
                "`{stem}.zng` produced stderr but has no `{}` snapshot. Run `cargo xtask test --fix` to create it.\n--- actual ---\n{actual}",
                expected_file.display()
            );
        }
        Err(err) => {
            return Err(err).with_context(|| format!("reading {}", expected_file.display()));
        }
    };
    if expected.trim_end() != actual {
        bail!(
            "`{stem}.zng` diagnostics differ from `{}`. Run `cargo xtask test --fix` to update the snapshot.\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
            expected_file.display()
        );
    }
    Ok(())
}

/// Generate the bridge for `<stem>.zng`, build `<stem>.rs` with `<stem>.cpp`
/// linked into it, and run it. The test must exit successfully. Generation
/// warnings (e.g. from deprecated items) are compared with `<stem>.stderr`;
/// the snapshot must exist whenever the generation emits anything.
fn check_pass(sh: &Shell, fix: bool, stem: &str, test_dir: &Path) -> Result<()> {
    let rs_file = test_dir.join(format!("{stem}.rs"));
    let cpp_file = test_dir.join(format!("{stem}.cpp"));
    if !rs_file.is_file() {
        bail!("`{stem}.zng` is marked `//@ pass` but `{stem}.rs` is missing");
    }
    if !cpp_file.is_file() {
        bail!("`{stem}.zng` is marked `//@ pass` but `{stem}.cpp` is missing");
    }

    let runner = repo_root().join("target/e2e");
    std::fs::create_dir_all(runner.join("src"))?;
    std::fs::write(runner.join("Cargo.toml"), RUNNER_CARGO_TOML)?;
    std::fs::write(runner.join("build.rs"), RUNNER_BUILD_RS)?;

    {
        let _dir = sh.push_dir(test_dir);
        let generate = cmd!(
            sh,
            "cargo run -q -p zngur-cli -- generate {stem}.zng -i --crate-name crate --rs-file {runner}/src/generated.rs --h-file {runner}/generated.h --cpp-file {runner}/generated.cpp"
        )
        .ignore_status()
        .output()?;
        if !generate.status.success() {
            bail!(
                "`{stem}.zng` is marked `//@ pass` but generating it failed:\n{}",
                String::from_utf8_lossy(&generate.stderr)
            );
        }
        check_stderr(fix, stem, test_dir, &normalize(&generate.stderr), false)?;
    }

    std::fs::copy(&rs_file, runner.join("src/main.rs"))
        .with_context(|| format!("copying {}", rs_file.display()))?;

    let output = {
        let _dir = sh.push_dir(&runner);
        cmd!(sh, "cargo run -q")
            .env("ZNGUR_E2E_CPP", &cpp_file)
            .ignore_status()
            .output()?
    };
    if !output.status.success() {
        bail!(
            "`{stem}.zng` is marked `//@ pass` but it failed to build or run:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

/// A single test, either a declarative `*.zng` test or a free-form Nushell test.
enum TestEntry {
    Zng(PathBuf),
    NuDir(PathBuf),
}

impl TestEntry {
    fn path(&self) -> &Path {
        match self {
            TestEntry::Zng(path) | TestEntry::NuDir(path) => path,
        }
    }
}

/// Collect tests under `dir`, recursively. A directory containing `main.nu` is a
/// free-form Nushell test and is not searched further. Files whose name starts
/// with `_` are fixtures imported by a `*.zng` test (via `merge`), not tests.
fn collect_tests(dir: &Path, out: &mut Vec<TestEntry>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with('_') {
            continue;
        }
        if path.is_dir() {
            if path.join("main.nu").is_file() {
                out.push(TestEntry::NuDir(path));
            } else {
                collect_tests(&path, out)?;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("zng") {
            out.push(TestEntry::Zng(path));
        }
    }
    Ok(())
}

/// Run every test under `tests/`, recursively. Directories group tests by
/// feature area; a `*.zng` gets its mode from the directive in the file, while a
/// directory with a `main.nu` is a free-form Nushell test.
pub fn main(fix: bool) -> Result<()> {
    let sh = &Shell::new()?;
    let tests_dir = repo_root().join("tests");

    let mut tests = Vec::new();
    collect_tests(&tests_dir, &mut tests)?;
    tests.sort_by(|a, b| a.path().cmp(b.path()));
    if tests.is_empty() {
        bail!("no tests found in {}", tests_dir.display());
    }

    let mut cli_built = false;
    for test in &tests {
        let name = test
            .path()
            .strip_prefix(&tests_dir)
            .unwrap_or(test.path())
            .display();
        println!("Checking {name}");

        match test {
            TestEntry::NuDir(dir) => {
                if !cli_built {
                    cmd!(sh, "cargo build -q -p zngur-cli").run()?;
                    cli_built = true;
                }
                run_nu_dir(dir, &tests_dir, fix)?;
            }
            TestEntry::Zng(zng_file) => {
                let stem = zng_file
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let test_dir = zng_file.parent().expect("a file has a parent directory");
                match directive(zng_file)? {
                    "fail" => check_fail(sh, fix, &stem, test_dir)?,
                    _ => check_pass(sh, fix, &stem, test_dir)?,
                }
            }
        }
    }

    Ok(())
}

/// Path of the `zngur` CLI built next to the running `xtask` (same cargo profile).
fn zngur_bin() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("resolving the xtask executable path")?;
    let dir = exe
        .parent()
        .context("the xtask executable has no parent directory")?;
    Ok(dir.join(format!("zngur{}", std::env::consts::EXE_SUFFIX)))
}

/// The `#@ run-status` (or `//@ run-status`) directive of a Nushell test.
fn nu_run_status(main_nu: &Path) -> Result<i32> {
    let text = std::fs::read_to_string(main_nu)
        .with_context(|| format!("reading {}", main_nu.display()))?;
    for line in text.lines() {
        let line = line.trim_start();
        let Some(body) = line.strip_prefix("#@").or_else(|| line.strip_prefix("//@")) else {
            continue;
        };
        if let Some((key, value)) = body.trim().split_once(':')
            && key.trim() == "run-status"
        {
            return value.trim().parse().with_context(|| {
                format!("`run-status` must be an integer in {}", main_nu.display())
            });
        }
    }
    Ok(0)
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).with_context(|| format!("creating {}", dst.display()))?;
    for entry in std::fs::read_dir(src).with_context(|| format!("reading {}", src.display()))? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), &to)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}

/// Run a free-form Nushell test. The directory's `main.nu` is executed (via
/// `nu`) inside a scratch copy of the directory, so it may create files freely;
/// its exit code must match the `#@ run-status` directive (default 0).
/// `tests/snapshot-utils.nu` is copied alongside it for snapshot assertions.
fn run_nu_dir(test_dir: &Path, tests_dir: &Path, fix: bool) -> Result<()> {
    let name = test_dir.strip_prefix(tests_dir).unwrap_or(test_dir);
    let scratch = repo_root().join("target/nu-tests").join(name);
    if scratch.exists() {
        std::fs::remove_dir_all(&scratch)
            .with_context(|| format!("cleaning {}", scratch.display()))?;
    }
    copy_dir_all(test_dir, &scratch)?;

    let snapshot_utils = tests_dir.join("snapshot-utils.nu");
    if snapshot_utils.is_file() {
        std::fs::copy(&snapshot_utils, scratch.join("snapshot-utils.nu"))
            .context("copying snapshot-utils.nu into the test scratch directory")?;
    }

    let main_nu = scratch.join("main.nu");
    let expected_status = nu_run_status(&main_nu)?;

    let bin = zngur_bin()?;
    let bin_dir = bin
        .parent()
        .context("the zngur binary has no parent directory")?;

    let mut cmd = std::process::Command::new("nu");
    cmd.arg(&main_nu)
        .current_dir(&scratch)
        .env("NO_COLOR", "1")
        .env("CARGO_TERM_COLOR", "never")
        .env("ZNGUR_BIN", &bin)
        .env("ZNGUR_ROOT", repo_root())
        .env("ZNGUR_TEST_DIR", &scratch)
        .env("ZNGUR_TEST_SOURCE_DIR", test_dir);

    let mut paths = vec![bin_dir.to_owned()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    if let Ok(joined) = std::env::join_paths(paths) {
        cmd.env("PATH", joined);
    }

    if fix {
        cmd.env("ZNGUR_UPDATE_SNAPSHOTS", "1");
    }

    let output = cmd
        .output()
        .with_context(|| format!("running Nushell test {}", main_nu.display()))?;
    let got_status = output.status.code().unwrap_or(-1);
    if got_status != expected_status {
        bail!(
            "Nushell test `{}` exit code mismatch: expected {expected_status}, got {got_status}\n--- stdout ---\n{}--- stderr ---\n{}",
            name.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(())
}
