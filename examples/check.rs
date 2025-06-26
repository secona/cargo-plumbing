use cargo_plumbing::CargoResult;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {}

fn run(_args: &Args) -> CargoResult<()> {
    anyhow::bail!("not implemented!");
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {e}"),
    }
}
