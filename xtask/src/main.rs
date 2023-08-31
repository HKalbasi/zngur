use clap::Parser;

mod ci;

#[derive(Parser)]
enum Command {
    CI,
}

fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::CI => ci::main(),
    }
}
