use std::io;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_mangen::Man;
use oak::cli::Cli;

fn main() -> io::Result<()> {
    let out_dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "man".to_string()));
    std::fs::create_dir_all(&out_dir)?;

    let cmd = Cli::command();
    let man = Man::new(cmd);
    let out_path = out_dir.join("oak.1");

    let mut buffer: Vec<u8> = vec![];
    man.render(&mut buffer)?;
    std::fs::write(&out_path, &buffer)?;

    eprintln!("Generated {}", out_path.display());
    Ok(())
}
