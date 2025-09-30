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
        fix: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::CI { fix } => ci::main(fix),
        Command::FormatBook { fix } => format_book::main(fix),
    }
}
