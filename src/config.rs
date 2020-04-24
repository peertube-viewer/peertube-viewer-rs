use clap::{App, Arg};
use toml::value::Value;

use std::collections::HashSet;
use std::default::Default;
use std::env;

struct TorrentConf {
    pub client: String,
    pub args: String,
}

struct PlayerConf {
    pub client: String,
    pub args: String,
}

pub struct Config {
    player: PlayerConf,
    instance: String,
    torrent: Option<(TorrentConf, bool)>,
    listed_instances: HashSet<String>,

    select_quality: bool,
    use_raw_url: bool,

    max_hist_lines: usize,
}

impl Config {
    pub fn new() -> Config {
        let cli_args = clap_app!(("peertube-viewer-rs") =>
            (version: "1.0")
            (author: "Sosthène Guédon <sosthene.gued@gmail.com>")
            (about: "Peertube cli client")
            (@arg USERAWURL:--("use-raw-url")  "the raw url will be passed to the player. It may be neccessary for players without native support for peertube such as vlc. Some players (ex : mpv) may be able to show the video title in their interface if this option isn't used")
            (@arg PRINTDEFAULTCONFIG: --("print-default-config")  "print the default confing to stdout")
            (@arg SELECTQUALITY: --("select-quality") -s  "When playing a video with this option, the user will be prompted to chose the video quality")
            (@arg ("torrent"):--("use-torrent")  "will download the video via the torrent downloader instead of playing it")
            (@arg ("player args"):--("player-args")  +takes_value  "arguments to be passed to the player")
            (@arg player:-p --player +takes_value "player to play the videos with")
            (@arg ("torrent downloader"):--("torrent-downloader")  +takes_value   "choose the torrent software to download the videos with")
            (@arg ("torrent downloader arguments"):--("torrent-downloader-args")  +takes_value  "arguments to be passed to the torrent downloader")
            (@arg instance: -i --instance +takes_value "instance to be browsed")
            (@arg ("config file"): -c --config +takes_value "Sets a custom config file")
            (@arg ("initial query"): -q --query +takes_value ... "initial query to be searched.\nIf it is a url, it will try to play it as a video")
        )
        .get_matches();

        let home = env::split_paths(&env::var("HOME").unwrap())
            .nth(0)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let config_file = if let Some(c) = cli_args.value_of("config file") {
            c.to_string()
        } else {
            format!("{}/.config/peertube-viewer-rs/config.toml", home)
        };
        let config_str = std::fs::read_to_string(config_file).unwrap();
        let config = if let Value::Table(t) = config_str.parse().unwrap() {
            t
        } else {
            panic!("Config file is not a table");
        };

        let (config_player_cmd, config_player_args) =
            if let Some(Value::Table(t)) = config.get("player") {
                (
                    t.get("command")
                        .map(|cmd| cmd.to_string())
                        .unwrap_or("mpv".to_string()),
                    t.get("args")
                        .map(|cmd| cmd.to_string())
                        .unwrap_or("".to_string()),
                )
            } else {
                ("mpv".to_string(), "".to_string())
            };
        let client = cli_args
            .value_of("player")
            .map(|c| c.to_string())
            .unwrap_or(config_player_cmd);
        let args = cli_args
            .value_of("player args")
            .map(|c| c.to_string())
            .unwrap_or(config_player_args);
        let player = PlayerConf { client, args };

        let (config_torrent_cmd, config_torrent_args) =
            if let Some(Value::Table(t)) = config.get("torrent") {
                (
                    t.get("command")
                        .map(|cmd| cmd.to_string())
                        .unwrap_or("".to_string()),
                    t.get("args")
                        .map(|cmd| cmd.to_string())
                        .unwrap_or("".to_string()),
                )
            } else {
                ("".to_string(), "".to_string())
            };
        let client = cli_args
            .value_of("torrent")
            .map(|c| c.to_string())
            .unwrap_or(config_torrent_cmd);
        let args = cli_args
            .value_of("torrent args")
            .map(|c| c.to_string())
            .unwrap_or(config_torrent_args);
        let torrent = TorrentConf { client, args };

        let mut temp = Config::default();
        temp.player = player;
        if torrent.client != "" {
            temp.torrent = Some((torrent, false));
        };
        temp
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            player: PlayerConf {
                client: "mpv".to_string(),
                args: String::new(),
            },
            instance: "video.ploud.fr".to_string(),
            torrent: None,
            listed_instances: HashSet::new(),
            select_quality: false,
            use_raw_url: false,
            max_hist_lines: 2000,
        }
    }
}
