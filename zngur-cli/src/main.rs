use std::path::PathBuf;

use clap::Parser;
use zngur::Zngur;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum Command {
    #[command(alias = "g")]
    Generate { path: PathBuf },
}

fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::Generate { path } => {
            let pp = path.parent().unwrap();
            Zngur::from_zng_file(&path)
                .with_cpp_file(&pp.join("generated.cpp"))
                .with_h_file(&pp.join("generated.h"))
                .with_rs_file(&pp.join("src/generated.rs"))
                .generate();
        }
    }
}
