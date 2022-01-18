extern crate clap;

use clap_complete::{
    generate_to,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};
use std::{
    env,
    fs::{create_dir, read_to_string, write},
};

include!("src/cli/clap_app.rs");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=peertube-viewer-rs.1.in");
    println!("cargo:rerun-if-changed=src/cli/clap_app.rs");

    let manpage = read_to_string("peertube-viewer-rs.1.in").unwrap();
    let manpage_out = manpage.replace("@version@", env!("CARGO_PKG_VERSION"));
    write("peertube-viewer-rs.1", manpage_out).unwrap();

    create_dir("./completions").unwrap_or(());
    let mut app = gen_app();

    generate_to(Bash, &mut app, env!("CARGO_PKG_NAME"), "./completions")
        .expect("Failed to generate completions");

    generate_to(Elvish, &mut app, env!("CARGO_PKG_NAME"), "./completions")
        .expect("Failed to generate completions");

    generate_to(Fish, &mut app, env!("CARGO_PKG_NAME"), "./completions")
        .expect("Failed to generate completions");

    generate_to(
        PowerShell,
        &mut app,
        env!("CARGO_PKG_NAME"),
        "./completions",
    )
    .expect("Failed to generate completions");

    generate_to(Zsh, &mut app, env!("CARGO_PKG_NAME"), "./completions")
        .expect("Failed to generate completions");
}
