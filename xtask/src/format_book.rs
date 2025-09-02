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

    println!("Checking for dprint installation...");

    // Try to get dprint version to check if it's installed
    let _dprint_version = match cmd!(sh, "dprint --version").read() {
        Ok(version) => {
            println!("Using dprint: {}", version.trim());
            version
        }
        Err(_) => {
            if check_only {
                eprintln!("Warning: dprint not found. Skipping book formatting check.");
                eprintln!("To enable book formatting checks, install dprint:");
                eprintln!("  curl -fsSL https://dprint.dev/install.sh | sh");
                eprintln!("  export PATH=\"$HOME/.dprint/bin:$PATH\"");
                return Ok(());
            } else {
                eprintln!("dprint not found. Please install it first:");
                eprintln!("  curl -fsSL https://dprint.dev/install.sh | sh");
                eprintln!("  export PATH=\"$HOME/.dprint/bin:$PATH\"");
                bail!("dprint is not installed");
            }
        }
    };

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
