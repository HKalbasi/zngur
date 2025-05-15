use clap::Parser;

mod ci;

#[derive(Parser)]
enum Command {
    CI {
        #[arg(long)]
        fix: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::CI { fix } => ci::main(fix),
    }
}
