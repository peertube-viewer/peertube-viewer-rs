#[macro_use]
extern crate clap;

mod cli;
mod error;

fn main() {
    cli::Cli::init().map(|mut cli| cli.run());
}
