// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

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
