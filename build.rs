#[macro_use]
extern crate clap;

use clap::{App, Arg, Shell};
use std::{env, fs::create_dir};

fn main() {
    create_dir("./completions").unwrap_or(());

    let yml = (load_yaml!("src/cli/clap_app.yml"));
    let mut app = App::from_yaml(dbg!(yml));

    app.gen_completions("peertube-viewer-rs", Shell::Bash, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Fish, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Zsh, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::PowerShell, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Elvish, "completions");
}
