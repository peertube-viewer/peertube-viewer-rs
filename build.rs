#[macro_use]
extern crate clap;

use clap::{Arg, Shell, Values};
use std::env;

fn main() {
    let mut app = include!("src/cli/clap_app");

    app.gen_completions(
        "peertube-viewer-rs",
        Shell::Bash,
        env::var("OUT_DIR").unwrap(),
    );
    app.gen_completions(
        "peertube-viewer-rs",
        Shell::Fish,
        env::var("OUT_DIR").unwrap(),
    );
    app.gen_completions(
        "peertube-viewer-rs",
        Shell::Zsh,
        env::var("OUT_DIR").unwrap(),
    );
    app.gen_completions(
        "peertube-viewer-rs",
        Shell::PowerShell,
        env::var("OUT_DIR").unwrap(),
    );
    app.gen_completions(
        "peertube-viewer-rs",
        Shell::Elvish,
        env::var("OUT_DIR").unwrap(),
    );
}
