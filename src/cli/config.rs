use clap::{Arg, Values};
use dirs::config_dir;
use toml::{
    de::Error as TomlError,
    value::{Table, Value},
};

use std::collections::HashSet;
use std::default::Default;
use std::fmt::{self, Display, Formatter};
use std::fs::read_to_string;
use std::{env, error, io};

#[derive(Debug)]
struct TorrentConf {
    pub client: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
struct PlayerConf {
    pub client: String,
    pub args: Vec<String>,
    pub use_raw_urls: bool,
}

#[derive(Debug)]
pub enum ConfigLoadError {
    UnreadableFile(io::Error),
    TomlError(TomlError),
    NotATable,
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigLoadError::UnreadableFile(_) => write!(
                f,
                "Unable to read the config file, using default config\nUsing default config"
            ),
            ConfigLoadError::TomlError(_) => write!(
                f,
                "The config was not parsable as TOML\nUsing default config"
            ),
            ConfigLoadError::NotATable => write!(
                f,
                "The config file is malformed, it should be a TOML table\nUsing default config"
            ),
        }
    }
}

impl error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ConfigLoadError::UnreadableFile(err) => Some(err),
            ConfigLoadError::TomlError(err) => Some(err),
            ConfigLoadError::NotATable => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    player: PlayerConf,
    instance: String,
    torrent: Option<(TorrentConf, bool)>,
    listed_instances: HashSet<String>,

    select_quality: bool,

    max_hist_lines: usize,
}

impl Config {
    pub fn new() -> (Config, Option<String>, Option<ConfigLoadError>) {
        let app = clap_app!(("peertube-viewer-rs") =>
            (version: "1.0")
            (author: "Sosthène Guédon <sosthene.gued@gmail.com>")
            (about: "Peertube cli client")
            (@arg USERAWURL:--("use-raw-url")  "the raw url will be passed to the player. It may be neccessary for players without native support for peertube such as vlc. Some players (ex : mpv) may be able to show the video title in their interface if this option isn't used")
            (@arg PRINTDEFAULTCONFIG: --("print-default-config")  "print the default confing to stdout")
            (@arg SELECTQUALITY: --("select-quality") -s  "When playing a video with this option, the user will be prompted to chose the video quality")
            (@arg TORRENT:--("use-torrent")  "will download the video via the torrent downloader instead of playing it")
            (@arg ("player args"):--("player-args") +takes_value ... "arguments to be passed to the player")
            (@arg player:-p --player +takes_value "player to play the videos with")
            (@arg ("torrent downloader"):--("torrent-downloader")  +takes_value   "choose the torrent software to download the videos with")
            (@arg ("torrent downloader arguments"):--("torrent-downloader-args")  +takes_value ... "arguments to be passed to the torrent downloader")
            (@arg instance: -i --instance +takes_value "instance to be browsed")
            (@arg ("config file"): -c --config +takes_value "Sets a custom config file")
        ).arg(
            Arg::with_name("initial query" ).multiple(true).index(1).short("q").long("query").long_help("initial query to be searched.\nIf it is a url, it will try to play it as a video")
        );
        let cli_args = app.get_matches();

        let mut load_error = None;
        let config_str = if let Some(c) = cli_args.value_of("config file") {
            read_to_string(c.to_string())
                .map_err(|err| {
                    load_error = Some(ConfigLoadError::UnreadableFile(err));
                })
                .unwrap_or_default()
        } else {
            match config_dir() {
                Some(mut d) => {
                    d.push("peertube-viewer-rs");
                    d.push("config.toml");
                    read_to_string(&d)
                        .map_err(|err| {
                            load_error = Some(ConfigLoadError::UnreadableFile(err));
                        })
                        .unwrap_or_default()
                }
                None => String::new(),
            }
        };
        let config = match config_str.parse() {
            Ok(Value::Table(t)) => t,
            Ok(_) => {
                load_error = Some(ConfigLoadError::NotATable);
                Table::new()
            }
            Err(e) => {
                load_error = Some(ConfigLoadError::TomlError(e));
                Table::new()
            }
        };
        let (config_player_cmd, config_player_args, use_raw_urls) =
            if let Some(Value::Table(t)) = config.get("player") {
                (
                    t.get("command")
                        .map(|cmd| cmd.as_str())
                        .flatten()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "mpv".to_string()),
                    t.get("args")
                        .map(|cmd| cmd.as_array())
                        .flatten()
                        .map(|v| v.iter().map(|s| s.to_string()).collect())
                        .unwrap_or_default(),
                    t.get("use-raw-urls")
                        .map(|b| b.as_bool())
                        .flatten()
                        .unwrap_or(false),
                )
            } else {
                ("mpv".to_string(), Vec::new(), false)
            };
        let client = cli_args
            .value_of("player")
            .map(|c| c.to_string())
            .unwrap_or(config_player_cmd);
        let args = cli_args
            .values_of("player args")
            .map(|v| v.map(|s| s.to_string()).collect())
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
                        .map(|cmd| cmd.as_array())
                        .flatten()
                        .map(|v| v.iter().map(|s| s.to_string()).collect())
                        .unwrap_or_default(),
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
                .values_of("torrent args")
                .map(|v| v.map(|s| s.to_string()).collect())
                .unwrap_or(conf.args);
            Some(TorrentConf { client, args })
        } else {
            let client = cli_args
                .value_of("torrent")
                .map(|c| c.to_string())
                .unwrap_or_default();
            let args = cli_args
                .values_of("torrent args")
                .map(|v| v.map(|s| s.to_string()).collect())
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
        temp.select_quality = cli_args.is_present("SELECTQUALITY");

        let initial_query = cli_args.values_of("initial query").map(concat);

        (temp, initial_query, load_error)
    }

    pub fn player(&self) -> &str {
        &self.player.client
    }

    pub fn player_args(&self) -> &Vec<String> {
        &self.player.args
    }

    pub fn instance(&self) -> &str {
        &self.instance
    }

    pub fn max_hist_lines(&self) -> usize {
        self.max_hist_lines
    }

    pub fn select_quality(&self) -> bool {
        self.select_quality
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

fn concat(v: Values) -> String {
    let mut concatenated = String::new();
    for s in v {
        concatenated.push(' ');
        concatenated.push_str(s);
    }
    concatenated
}

impl Default for Config {
    fn default() -> Config {
        Config {
            player: PlayerConf {
                client: "mpv".to_string(),
                args: Vec::new(),
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