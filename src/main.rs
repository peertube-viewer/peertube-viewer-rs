mod cli;
mod error;

fn main() {
    if let Ok(mut cli) = cli::Cli::init() {
        cli.run()
    };
}
