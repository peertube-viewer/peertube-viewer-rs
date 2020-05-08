#[macro_use]
extern crate clap;

use clap::{Arg, Shell};
use std::{env, fs::create_dir};

fn main() {
    create_dir("./completions").unwrap_or(());

    let mut app = include!("src/cli/clap_app");

    app.gen_completions("peertube-viewer-rs", Shell::Bash, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Fish, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Zsh, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::PowerShell, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Elvish, "completions");
}
