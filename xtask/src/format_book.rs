use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

pub fn main(fix: bool) -> Result<()> {
    let sh = Shell::new()?;

    // Check if dprint config exists
    if !sh.path_exists("dprint.json") {
        bail!("dprint.json config file not found. Please create it first.");
    }

    // Check if book source directory exists
    if !sh.path_exists("book/src") {
        bail!("Book source directory 'book/src' not found");
    }

    // Check if plugins need to be initialized/updated
    println!("Initializing dprint plugins...");
    if let Err(e) = cmd!(sh, "dprint upgrade").run() {
        eprintln!("Warning: Failed to upgrade dprint plugins: {}", e);
        eprintln!("Continuing with existing plugins...");
    }

    if fix {
        println!("Formatting markdown files...");
        cmd!(sh, "dprint fmt")
            .run()
            .with_context(|| "Failed to format markdown files")?;
        println!("✓ Book formatting complete!");
        Ok(())
    } else {
        println!("Checking markdown formatting...");
        match cmd!(sh, "dprint check").run() {
            Ok(_) => {
                println!("✓ All markdown files are correctly formatted!");
                Ok(())
            }
            Err(_) => {
                eprintln!("✗ Some markdown files need formatting.");
                eprintln!("Run `cargo xtask format-book --fix` to fix them.");
                bail!("Book formatting check failed");
            }
        }
    }
}
