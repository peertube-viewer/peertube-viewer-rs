#[macro_use]
extern crate clap;

use clap::{Arg, Shell, Values};
use std::env;

fn main() {
    let mut app = include!("src/cli/clap_app");

    app.gen_completions("peertube-viewer-rs", Shell::Bash, ".");
    app.gen_completions("peertube-viewer-rs", Shell::Fish, ".");
    app.gen_completions("peertube-viewer-rs", Shell::Zsh, ".");
    app.gen_completions("peertube-viewer-rs", Shell::PowerShell, ".");
    app.gen_completions("peertube-viewer-rs", Shell::Elvish, ".");
}
