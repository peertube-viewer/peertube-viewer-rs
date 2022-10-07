// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use clap::{Arg, ArgAction, Command};

pub fn gen_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author("Sosthène Guédon <dev@peertube-viewer.com>")
    .about("PeerTube CLI client")
    .args(
        &[
            Arg::new("use-raw-urls")
                .long("use-raw-urls")
                .action(ArgAction::SetTrue)
                .help("The raw url to the video will be passed to the player instead of the url to web interface to watch it. It may be necessary for players without native support for peertube such as vlc. Some players (ex : mpv) may be able to show the video title in their interface if this option isn't used"),
            Arg::new("print-default-config")
                .long("print-default-config")
                .conflicts_with("print-full-config")
                .action(ArgAction::SetTrue)
                .help("Print the default confing to stdout and exit"),
            Arg::new("print-full-config")
                .long("print-full-example-config")
                .action(ArgAction::SetTrue)
                .help("Print an example of all possible config options and exit"),
            Arg::new("select-quality")
                .short('s')
                .long("select-quality")
                .action(ArgAction::SetTrue)
                .help("When playing a video with this option, the user will be prompted to chose the video quality\n Note: this implies --use-raw-urls"),
            Arg::new("local")
                .long("local")
                .action(ArgAction::SetTrue)
                .help("Only browse video hosted on the instance you are connected to"),
            Arg::new("torrent")
                .long("use-torrent")
                .action(ArgAction::SetTrue)
                .conflicts_with("use-raw-urls")
                .help("Will download the video via the torrent downloader instead of playing it"),
            Arg::new("trending")
                .short('t')
                .long("trending")
                .conflicts_with("chandle")
                .action(ArgAction::SetTrue)
                .help("Will start browsing trending videos"),
            Arg::new("channels")
                .long("channels")
                .conflicts_with("trending")
                .action(ArgAction::SetTrue)
                .help("Will start searching video channels"),
            Arg::new("chandle")
                .long("chandle")
                .num_args(1)
                .conflicts_with("channels")
                .help("Start browsing the videos of a channel with its handle (ex: name@instance.com)"),
            Arg::new("player-args")
                .long("player-args")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Arguments to be passed to the player"),
            Arg::new("player")
                .short('p')
                .long("player")
                .num_args(1)
                .help("Player to play the videos with"),
            Arg::new("torrent-downloader")
                .long("torrent-downloader")
                .num_args(1)
                .help("Choose the torrent software to download the videos with"),
            Arg::new("torrent-downloader-arguments")
                .long("torrent-downloader-args")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Arguments to be passed to the torrent downloader"),
            Arg::new("instance")
                .short('i')
                .long("instance")
                .num_args(1)
                .help("Instance to be browsed"),
            Arg::new("search-engine")
                .long("search-engine")
                .num_args(1)
                .conflicts_with("instance")
                .help("Use a search engine (like sepiasearch)"),
            Arg::new("config-file")
                .short('c')
                .long("config")
                .num_args(1)
                .help("Sets a custom config file"),
            Arg::new("let-nsfw")
                .long("let-nsfw")
                .conflicts_with("tag-nsfw")
                .action(ArgAction::SetTrue)
                .help("Don't tag nsfw results"),
            Arg::new("tag-nsfw")
                .long("tag-nsfw")
                .conflicts_with("block-nsfw")
                .action(ArgAction::SetTrue)
                .help("Tag nsfw results. This is the default behavior. This flag is only useful to override the config file"),
            Arg::new("block-nsfw")
                .long("block-nsfw")
                .conflicts_with("let-nsfw")
                .action(ArgAction::SetTrue)
                .help("Block nsfw search results"),
            Arg::new("no-color")
                .long("no-color")
                .action(ArgAction::SetTrue)
                .help("Remove coloring of output"),
            Arg::new("color")
                .long("color")
                .action(ArgAction::SetTrue)
                .help("Force coloring of output if it is disabled in the config file"),
            Arg::new("initial-query")
                .num_args(1)
                .action(ArgAction::Append)
                .conflicts_with("trending")
                .conflicts_with("chandle")
                .index(1)
                .help("Initial query to be searched.\nIf it is a url, it will try to play it as a video"),
        ]
    )
}
