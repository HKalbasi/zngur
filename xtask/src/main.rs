use clap::Parser;

mod ci;
mod format_book;

#[derive(Parser)]
enum Command {
    CI {
        #[arg(long)]
        fix: bool,
    },
    FormatBook {
        #[arg(long)]
        check: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::CI { fix } => ci::main(fix),
        Command::FormatBook { check } => format_book::main(check),
    }
}
