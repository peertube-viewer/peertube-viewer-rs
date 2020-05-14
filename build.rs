extern crate clap;

use clap::{App, Shell, YamlLoader};
use std::{
    env,
    fs::{create_dir, read_to_string, write},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cli/clap_app.in.yml");
    create_dir("./completions").unwrap_or(());
    let yml_in = read_to_string("src/cli/clap_app.in.yml").unwrap();

    let yml_out = dbg!(yml_in.replace(
        "version:",
        &format!("version: {}", env!("CARGO_PKG_VERSION")),
    ));

    write("src/cli/clap_app.yml", &yml_out).unwrap();

    let temp = YamlLoader::load_from_str(&yml_out).unwrap();
    let mut app = App::from_yaml(&temp[0]);

    app.gen_completions("peertube-viewer-rs", Shell::Bash, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Fish, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Zsh, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::PowerShell, "completions");
    app.gen_completions("peertube-viewer-rs", Shell::Elvish, "completions");
}
