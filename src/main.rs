#[macro_use]
extern crate clap;

mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = cli::Cli::init();
    cli.run()
}
