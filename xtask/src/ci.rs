use anyhow::{Context, Result, bail};
use regex::Regex;
use std::fs;
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
        let expected_path = format!("examples/{}/expected_output.txt", example);
        let actual_path = format!("examples/{}/actual_output.txt", example);
        compare_expected_with_actual(&expected_path, &actual_path)
            .with_context(|| format!("Example `{example}` output differs from expected."))?;
        cmd!(sh, "cargo fmt --check")
            .run()
            .with_context(|| format!("Example `{example}` is not formatted. Run `cargo fmt`"))?;
        sh.change_dir("..");
    }
    Ok(())
}

/// Compare `expected_path` to `actual_path` line-by-line.
/// Supports one extension in expected:
///  - Inline regex placeholders: `{{re:...}}` which are treated as raw regex
///    segments embedded within an otherwise literal line
fn compare_expected_with_actual(expected_path: &str, actual_path: &str) -> Result<()> {
    let expected = fs::read_to_string(expected_path)
        .with_context(|| format!("Failed to read {expected_path}"))?;
    let actual =
        fs::read_to_string(actual_path).with_context(|| format!("Failed to read {actual_path}"))?;

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    if expected_lines.len() != actual_lines.len() {
        bail!(
            "Line count differs: expected {} lines, actual {} lines",
            expected_lines.len(),
            actual_lines.len()
        );
    }

    for (idx, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
        let line_no = idx + 1;
        // Escape literals, splice in `{{re:...}}` as raw regex
        let mut pattern = String::new();
        let mut rest = *exp;
        while let Some(start) = rest.find("{{re:") {
            let (head, tail) = rest.split_at(start);
            pattern.push_str(&regex::escape(head));
            if let Some(end) = tail.find("}}") {
                let re_body = &tail[5..end]; // after '{{re:' up to before '}}'
                pattern.push_str(re_body);
                rest = &tail[end + 2..];
            } else {
                // No closing, treat literally
                pattern.push_str(&regex::escape(tail));
                rest = "";
            }
        }
        pattern.push_str(&regex::escape(rest));
        // Anchor the pattern to the full line
        let anchored = format!("^{}$", pattern);
        let re = Regex::new(&anchored)
            .with_context(|| format!("Invalid constructed regex on expected line {line_no}"))?;
        if !re.is_match(act) {
            bail!(
                "Line {line_no} differs.\n  expected: {exp}\n  actual:   {act}\n  pattern:  {anchored}"
            );
        }
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
