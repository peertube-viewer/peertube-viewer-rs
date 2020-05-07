#[macro_use]
extern crate clap;

mod cli;
mod error;

fn main() {
    let mut cli = cli::Cli::init();
    cli.run();
}
