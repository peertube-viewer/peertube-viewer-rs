// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

mod cli;
mod error;

use std::process::exit;

fn main() {
    match cli::Cli::init() {
        Ok(mut cli) => cli.run(),
        Err(error::Error::Readline(err)) => {
            eprintln!("Failed to open prompt: {err}");
            exit(1);
        }
        Err(err) => {
            eprintln!("Failed to initialize: {err}");
            exit(1);
        }
    }
}
