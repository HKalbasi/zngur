use clap::Parser;

mod ci;
mod install_osmium;

#[derive(Parser)]
enum Command {
    CI,
    InstallOsmium,
}

fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::CI => ci::main(),
        Command::InstallOsmium => install_osmium::main(),
    }
}
