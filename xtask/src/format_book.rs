use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

pub fn main(check_only: bool) -> Result<()> {
    let sh = Shell::new()?;

    // Check if dprint config exists
    if !sh.path_exists("dprint.json") {
        bail!("dprint.json config file not found. Please create it first.");
    }

    // Check if book source directory exists
    if !sh.path_exists("book/src") {
        bail!("Book source directory 'book/src' not found");
    }

    println!("Checking for dprint...");

    // Try to use dprint directly first
    let dprint_available = cmd!(sh, "dprint --version").read().is_ok();

    if !dprint_available {
        println!("dprint not found, installing via cargo...");
        cmd!(sh, "cargo install dprint --locked")
            .run()
            .with_context(|| "Failed to install dprint via cargo install")?;
        println!("✓ dprint installed successfully");
    } else {
        println!("✓ dprint is available");
    }

    // Check if plugins need to be initialized/updated
    println!("Initializing dprint plugins...");
    if let Err(e) = cmd!(sh, "dprint upgrade").run() {
        eprintln!("Warning: Failed to upgrade dprint plugins: {}", e);
        eprintln!("Continuing with existing plugins...");
    }

    if check_only {
        println!("Checking markdown formatting...");
        match cmd!(sh, "dprint check").run() {
            Ok(_) => {
                println!("✓ All markdown files are correctly formatted!");
                Ok(())
            }
            Err(_) => {
                eprintln!("✗ Some markdown files need formatting.");
                eprintln!("Run `cargo xtask format-book` to fix them.");
                bail!("Book formatting check failed");
            }
        }
    } else {
        println!("Formatting markdown files...");
        cmd!(sh, "dprint fmt")
            .run()
            .with_context(|| "Failed to format markdown files")?;
        println!("✓ Book formatting complete!");
        Ok(())
    }
}
