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
    pub use_raw_urls: bool,
}

pub struct Config {
    player: PlayerConf,
    instance: String,
    torrent: Option<(TorrentConf, bool)>,
    listed_instances: HashSet<String>,

    select_quality: bool,

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
            (@arg TORRENT:--("use-torrent")  "will download the video via the torrent downloader instead of playing it")
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
            .next()
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

        let (config_player_cmd, config_player_args, use_raw_urls) =
            if let Some(Value::Table(t)) = config.get("player") {
                (
                    t.get("command")
                        .map(|cmd| cmd.as_str())
                        .flatten()
                        .map(|s| s.to_string())
                        .unwrap_or("mpv".to_string()),
                    t.get("args")
                        .map(|cmd| cmd.as_str())
                        .flatten()
                        .map(|s| s.to_string())
                        .unwrap_or("".to_string()),
                    t.get("use-raw-urls")
                        .map(|b| b.as_bool())
                        .flatten()
                        .unwrap_or(false),
                )
            } else {
                ("mpv".to_string(), "".to_string(), false)
            };
        let client = cli_args
            .value_of("player")
            .map(|c| c.to_string())
            .unwrap_or(config_player_cmd);
        let args = cli_args
            .value_of("player args")
            .map(|c| c.to_string())
            .unwrap_or(config_player_args);
        let use_raw_urls = cli_args.is_present("USERAWURL") & use_raw_urls;
        let player = PlayerConf {
            client,
            args,
            use_raw_urls,
        };

        let torrent_config = if let Some(Value::Table(t)) = config.get("torrent") {
            if let Some(s) = t
                .get("command")
                .map(|cmd| cmd.as_str())
                .flatten()
                .map(|s| s.to_string())
            {
                Some(TorrentConf {
                    client: s,
                    args: t
                        .get("args")
                        .map(|cmd| cmd.to_string())
                        .unwrap_or("".to_string()),
                })
            } else {
                None
            }
        } else {
            None
        };

        let torrent = if let Some(conf) = torrent_config {
            let client = cli_args
                .value_of("torrent")
                .map(|c| c.to_string())
                .unwrap_or(conf.client);
            let args = cli_args
                .value_of("torrent args")
                .map(|c| c.to_string())
                .unwrap_or(conf.args);
            Some(TorrentConf { client, args })
        } else {
            let client = cli_args
                .value_of("torrent")
                .map(|c| c.to_string())
                .unwrap_or_default();
            let args = cli_args
                .value_of("torrent args")
                .map(|c| c.to_string())
                .unwrap_or_default();
            Some(TorrentConf { client, args })
        };

        let instance = if let Some(i) = cli_args.value_of("instance") {
            i
        } else {
            match config.get("instance") {
                Some(Value::Table(t)) => {
                    if let Some(Value::String(s)) = t.get("main") {
                        s
                    } else {
                        "video.ploud.fr"
                    }
                }
                _ => "video.ploud.fr",
            }
        };

        let mut temp = Config::default();
        temp.player = player;
        temp.instance = Config::correct_instance(instance);
        temp.torrent = torrent.map(|t| (t, cli_args.is_present("TORRENT")));
        temp
    }

    pub fn player(&self) -> &str {
        &self.player.client
    }

    pub fn instance(&self) -> &str {
        &self.instance
    }

    fn correct_instance(s: &str) -> String {
        let mut s = if s.starts_with("https://") {
            s.to_string()
        } else if s.starts_with("http://") {
            format!("https://{}", &s[7..])
        } else {
            format!("https://{}", s)
        };
        if let Some(c) = s.pop() {
            if c != '/' {
                s.push(c);
            }
        }

        s
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            player: PlayerConf {
                client: "mpv".to_string(),
                args: String::new(),
                use_raw_urls: false,
            },
            instance: "video.ploud.fr".to_string(),
            torrent: None,
            listed_instances: HashSet::new(),
            select_quality: false,
            max_hist_lines: 2000,
        }
    }
}