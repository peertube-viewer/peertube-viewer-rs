#[macro_use]
extern crate clap;

mod cli;

fn main() {
    let mut cli = cli::Cli::init();
    cli.run();
}
