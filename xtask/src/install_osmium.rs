use anyhow::{Context, Result};
use xshell::{cmd, Shell};

pub fn main() -> Result<()> {
    let sh = &Shell::new()?;
    let tmp = sh.create_temp_dir()?;
    sh.change_dir(tmp.path());
    cmd!(sh, "git clone https://github.com/osmcode/libosmium").run()?;
    cmd!(
        sh,
        "apt install cmake cmake-curses-gui make libexpat1-dev zlib1g-dev libbz2-dev libboost-dev libprotobuf-dev protobuf-compiler libosmpbf-dev libprotozero-dev libutfcpp-dev"
    ).run()?;
    sh.change_dir("libosmium");
    cmd!(sh, "mkdir build").run()?;
    sh.change_dir("build");
    cmd!(sh, "cmake ..").run()?;
    cmd!(sh, "make -j").run()?;
    cmd!(sh, "make install").run()?;
    Ok(())
}
