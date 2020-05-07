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
use std::path::PathBuf;
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
    UnreadableFile(io::Error, PathBuf),
    TomlError(TomlError),
    NotATable,
    NotAString,
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigLoadError::UnreadableFile(e, path) => write!(
                f,
                "Unable to read the config file at \"{}\":\n{}\nUsing default config",
                path.display(),
                e
            ),
            ConfigLoadError::TomlError(e) => write!(
                f,
                "The config was not parsable as TOML:\n{}\nUsing default config",
                e
            ),
            ConfigLoadError::NotATable => write!(
                f,
                "The config file is malformed, it should be a TOML table\nUsing default config"
            ),
            ConfigLoadError::NotAString => write!(
                f,
                "Command arguments need to be a table of String\n Ignoring bad arguments"
            ),
        }
    }
}

impl error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ConfigLoadError::UnreadableFile(err, _) => Some(err),
            ConfigLoadError::TomlError(err) => Some(err),
            ConfigLoadError::NotATable | ConfigLoadError::NotAString => None,
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
                    load_error = Some(ConfigLoadError::UnreadableFile(err, c.into()));
                })
                .unwrap_or_default()
        } else {
            match config_dir() {
                Some(mut d) => {
                    d.push("peertube-viewer-rs");
                    d.push("config.toml");
                    read_to_string(&d)
                        .map_err(|err| {
                            load_error = Some(ConfigLoadError::UnreadableFile(err, d));
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
                    get_args(t, &mut load_error),
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
                    args: get_args(t, &mut load_error),
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
        temp.instance = correct_instance(instance);
        temp.torrent = torrent.map(|t| (t, cli_args.is_present("TORRENT")));
        temp.select_quality = cli_args.is_present("SELECTQUALITY");

        let initial_query = cli_args.values_of("initial query").map(concat);

        (temp, initial_query, load_error)
    }

    pub fn player(&self) -> &str {
        match &self.torrent {
            Some((tor, true)) => &tor.client,
            _ => &self.player.client,
        }
    }

    pub fn player_args(&self) -> &Vec<String> {
        match &self.torrent {
            Some((tor, true)) => &tor.args,
            _ => &self.player.args,
        }
    }

    pub fn instance(&self) -> &str {
        &self.instance
    }

    pub fn use_torrent(&self) -> bool {
        if let Some((_, true)) = self.torrent {
            true
        } else {
            false
        }
    }

    pub fn max_hist_lines(&self) -> usize {
        self.max_hist_lines
    }

    pub fn select_quality(&self) -> bool {
        self.select_quality
    }
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

fn concat(v: Values) -> String {
    let mut concatenated = String::new();
    for s in v {
        concatenated.push(' ');
        concatenated.push_str(s);
    }
    concatenated
}

fn get_args(t: &Table, load_error: &mut Option<ConfigLoadError>) -> Vec<String> {
    t.get("args")
        .map(|cmd| cmd.as_array())
        .flatten()
        .map(|v| {
            v.iter()
                .filter_map(|s| {
                    let res = s.as_str().map(|s| s.to_string());
                    if res.is_none() {
                        *load_error = Some(ConfigLoadError::NotAString)
                    }
                    res
                })
                .collect()
        })
        .unwrap_or_default()
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

#[cfg(test)]
mod helpers {

    use super::*;

    #[test]
    fn instance_correction() {
        assert_eq!(correct_instance("http://foo.bar/"), "https://foo.bar");
        assert_eq!(correct_instance("foo.bar"), "https://foo.bar");
        assert_eq!(correct_instance("foo.bar/"), "https://foo.bar");
        assert_eq!(correct_instance("https://foo.bar/"), "https://foo.bar");
    }
}
