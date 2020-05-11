use clap::{App, ArgMatches, Values};
use dirs::config_dir;
use toml::{
    de::Error as TomlError,
    value::{Table, Value},
};

use std::collections::HashSet;
use std::default::Default;
use std::fmt::{self, Display};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use std::{error, io};

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
    UseTorrentAndNoInfo,
    NotATable,
    NotAString,
    IncorrectTag(String),
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
            ConfigLoadError::IncorrectTag(name) => write!(
                f,
                "Option {} didn't have a correct tag\nUsing default tag",
               name 
            ),
            ConfigLoadError::NotATable => write!(
                f,
                "The config file is malformed, it should be a TOML table\nUsing default config"
            ),
            ConfigLoadError::NotAString => write!(
                f,
                "Command arguments and blacklisted instances need to be a table of String\n Ignoring bad arguments"
            ),
            ConfigLoadError::UseTorrentAndNoInfo=> write!(
                f,
                "--use-torrent requires a torrent to be set"
            ),
        }
    }
}

impl error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ConfigLoadError::UnreadableFile(err, _) => Some(err),
            ConfigLoadError::NotATable
            | ConfigLoadError::NotAString
            | ConfigLoadError::TomlError(_)
            | ConfigLoadError::UseTorrentAndNoInfo
            | ConfigLoadError::IncorrectTag(_) => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NsfwBehavior {
    Block,
    Tag,
    Let,
}

impl NsfwBehavior {
    pub fn is_block(self) -> bool {
        self == NsfwBehavior::Block
    }
}

/// Config for the cli interface
#[derive(Debug)]
pub struct Config {
    player: PlayerConf,
    instance: String,
    torrent: Option<(TorrentConf, bool)>,
    listed_instances: HashSet<String>,
    is_whitelist: bool,

    nsfw: NsfwBehavior,

    select_quality: bool,

    max_hist_lines: usize,
}

impl Config {
    pub fn new() -> (Config, Option<String>, Vec<ConfigLoadError>) {
        let yml = load_yaml!("clap_app.yml");
        let app = App::from_yaml(yml);
        let cli_args = app.get_matches();

        if cli_args.is_present("PRINTDEFAULTCONFIG") {
            print!("{}", include_str!("default_config.toml"));
            exit(0);
        }

        if cli_args.is_present("PRINTFULLCONFIG") {
            print!("{}", include_str!("full_config.toml"));
            exit(0);
        }

        let initial_query = cli_args.values_of("initial-query").map(concat);

        // Parse config as an String with default to empty string
        let (mut config, mut load_errors) = if let Some(c) = cli_args.value_of("config-file") {
            Config::from_config_file(&PathBuf::from(c))
        } else {
            match config_dir() {
                Some(mut d) => {
                    d.push("peertube-viewer-rs");
                    d.push("config.toml");
                    Config::from_config_file(&d)
                }
                None => (Config::default(), Vec::new()),
            }
        };

        load_errors.append(&mut config.update_with_args(cli_args));

        (config, initial_query, load_errors)
    }

    fn from_config_file(path: &PathBuf) -> (Config, Vec<ConfigLoadError>) {
        let mut temp = Config::default();
        let mut load_errors = Vec::new();

        /* ---File parsing--- */

        let config_str = read_to_string(path)
            .map_err(|e| load_errors.push(ConfigLoadError::UnreadableFile(e, path.clone())))
            .unwrap_or_default();

        // Parse config as TOML with default to empty
        let config = match config_str.parse() {
            Ok(Value::Table(t)) => t,
            Ok(_) => {
                load_errors.push(ConfigLoadError::NotATable);
                Table::new()
            }
            Err(e) => {
                load_errors.push(ConfigLoadError::TomlError(e));
                Table::new()
            }
        };

        /* ---Player configuration --- */
        let (player_cmd, player_args, use_raw_urls) =
            if let Some(Value::Table(t)) = config.get("player") {
                (
                    t.get("command")
                        .map(|cmd| cmd.as_str())
                        .flatten()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "mpv".to_string()),
                    get_string_array(t, "args", &mut load_errors),
                    t.get("use-raw-urls")
                        .map(|b| b.as_bool())
                        .flatten()
                        .unwrap_or(false),
                )
            } else {
                ("mpv".to_string(), Vec::new(), false)
            };

        temp.player = PlayerConf {
            client: player_cmd,
            args: player_args,
            use_raw_urls,
        };

        /* ---Torrent configuration --- */
        let torrent = if let Some(Value::Table(t)) = config.get("torrent") {
            if let Some(s) = t
                .get("command")
                .map(|cmd| cmd.as_str())
                .flatten()
                .map(|s| s.to_string())
            {
                Some(TorrentConf {
                    client: s,
                    args: get_string_array(t, "args", &mut load_errors),
                })
            } else {
                None
            }
        } else {
            None
        };

        /* ---Nsfw configuration --- */
        temp.nsfw = if let Some(Value::Table(t)) = config.get("general") {
            if let Some(Value::String(s)) = t.get("nsfw") {
                if s == "block" {
                    NsfwBehavior::Block
                } else if s == "let" {
                    NsfwBehavior::Let
                } else if s == "tag" {
                    NsfwBehavior::Tag
                } else {
                    load_errors.push(ConfigLoadError::IncorrectTag("nsfw".to_string()));
                    NsfwBehavior::Tag
                }
            } else {
                NsfwBehavior::Tag
            }
        } else {
            NsfwBehavior::Tag
        };

        /* ---Blacklist configuration --- */
        let (list, is_whitelist) = if let Some(Value::Table(t)) = config.get("instances") {
            if t.contains_key("whitelist") {
                (
                    get_string_array(t, "whitelist", &mut load_errors)
                        .into_iter()
                        .collect(),
                    true,
                )
            } else {
                (
                    get_string_array(t, "blacklist", &mut load_errors)
                        .into_iter()
                        .collect(),
                    false,
                )
            }
        } else {
            (HashSet::new(), false)
        };

        match config.get("instances") {
            Some(Value::Table(t)) => {
                if let Some(Value::String(s)) = t.get("main") {
                    temp.instance = correct_instance(s);
                }
            }
            _ => {}
        }

        temp.listed_instances = list;
        temp.is_whitelist = is_whitelist;

        temp.torrent = torrent.map(|t| (t, false));

        (temp, load_errors)
    }

    fn update_with_args(&mut self, args: ArgMatches) -> Vec<ConfigLoadError> {
        let mut load_errors = Vec::new();

        if args.is_present("let-nsfw") {
            self.nsfw = NsfwBehavior::Let
        } else if args.is_present("block-nsfw") {
            self.nsfw = NsfwBehavior::Block
        } else if args.is_present("tag-nsfw") {
            self.nsfw = NsfwBehavior::Tag
        }

        if let Some(i) = args.value_of("instance") {
            self.instance = correct_instance(i);
        }

        /* ---Torrent configuration --- */
        let client = args.value_of("torrent-downloader").map(|c| c.to_string());
        let torrent_args = args
            .values_of("torrent-downloader-arguments")
            .map(|v| v.map(|s| s.to_string()).collect::<Vec<String>>());

        let use_torrent = args.is_present("TORRENT");
        if self.torrent.is_none() && use_torrent && !args.is_present("torrent-downloader") {
            load_errors.push(ConfigLoadError::UseTorrentAndNoInfo);
        }

        match (self.torrent.take(), client, torrent_args) {
            (None, Some(client), None) => {
                self.torrent = Some((
                    TorrentConf {
                        client,
                        args: Vec::new(),
                    },
                    use_torrent,
                ))
            }

            (None, Some(client), Some(a)) => {
                self.torrent = Some((TorrentConf { client, args: a }, use_torrent))
            }
            (Some((conf, _)), client, torrent_args) => {
                let mut conf_args = conf.args;
                self.torrent = Some((
                    TorrentConf {
                        client: client.unwrap_or(conf.client),
                        args: torrent_args
                            .map(|mut a| {
                                a.append(&mut conf_args);
                                a
                            })
                            .unwrap_or(conf_args),
                    },
                    use_torrent,
                ))
            }
            _ => {}
        }

        /* ---Player configuration --- */
        args.value_of("player")
            .map(|c| self.player.client = c.to_string());
        args.values_of("player-args").map(|v| {
            v.map(|s| self.player.args.push(s.to_string()))
                .any(|_| false)
        });
        self.player.use_raw_urls = args.is_present("USERAWURL");

        self.select_quality = args.is_present("SELECTQUALITY");

        load_errors
    }

    pub fn player(&self) -> &str {
        match &self.torrent {
            Some((tor, true)) => &tor.client,
            _ => &self.player.client,
        }
    }

    pub fn use_raw_url(&self) -> bool {
        self.player.use_raw_urls
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

    pub fn is_blacklisted(&self, instance: &str) -> bool {
        if self.is_whitelist {
            !self.listed_instances.contains(instance)
        } else {
            self.listed_instances.contains(instance)
        }
    }

    pub fn nsfw(&self) -> NsfwBehavior {
        self.nsfw
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

fn concat(mut v: Values) -> String {
    let mut concatenated = String::new();
    if let Some(s) = v.next() {
        concatenated.push_str(s);
    }
    for s in v {
        concatenated.push(' ');
        concatenated.push_str(s);
    }
    concatenated
}

fn get_string_array(t: &Table, name: &str, load_errors: &mut Vec<ConfigLoadError>) -> Vec<String> {
    t.get(name)
        .map(|cmd| cmd.as_array())
        .flatten()
        .map(|v| {
            v.iter()
                .filter_map(|s| {
                    let res = s.as_str().map(|s| s.to_string());
                    if res.is_none() {
                        load_errors.push(ConfigLoadError::NotAString)
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
            nsfw: NsfwBehavior::Tag,
            listed_instances: HashSet::new(),
            is_whitelist: false,
            select_quality: false,
            max_hist_lines: 2000,
        }
    }
}

#[cfg(test)]
mod config {
    use super::*;

    #[test]
    fn load_config_then_args() {
        let path = PathBuf::from("src/cli/full_config.toml");
        let (mut config, mut errors) = Config::from_config_file(&path);
        let yml = load_yaml!("clap_app.yml");
        assert_eq!(errors.len(), 0);
        assert_eq!(config.nsfw(), NsfwBehavior::Block);
        assert_eq!(config.player(), "mpv");
        assert_eq!(*config.player_args(), vec!["--volume=30"]);
        assert_eq!(config.instance(), "https://skeptikon.fr");
        assert_eq!(config.is_blacklisted("peertube.social"), true);
        assert_eq!(config.use_raw_url(), false);
        assert_eq!(config.select_quality(), false);

        let app = App::from_yaml(yml);
        let matches = app
            .get_matches_from_safe(vec![
                "peertube-viewer-rs",
                "--player",
                "args-player",
                "--player-args=--no-video",
                "--instance=args.ploud.fr",
                "--use-raw-url",
                "--let-nsfw",
                "-s",
            ])
            .unwrap();
        errors = config.update_with_args(matches);
        assert_eq!(errors.len(), 0);
        assert_eq!(config.nsfw(), NsfwBehavior::Let);
        assert_eq!(config.player(), "args-player");
        assert_eq!(*config.player_args(), vec!["--volume=30", "--no-video"]);
        assert_eq!(config.instance(), "https://args.ploud.fr");
        assert_eq!(config.select_quality(), true);
        assert_eq!(config.use_raw_url(), true);
    }

    #[test]
    fn torrent_options() {
        let path = PathBuf::from("src/cli/full_config.toml");
        let (mut config, mut errors) = Config::from_config_file(&path);
        let yml = load_yaml!("clap_app.yml");
        assert_eq!(errors.len(), 0);
        assert_eq!(config.nsfw(), NsfwBehavior::Block);
        assert_eq!(config.player(), "mpv");
        assert_eq!(*config.player_args(), vec!["--volume=30"]);
        assert_eq!(config.instance(), "https://skeptikon.fr");
        assert_eq!(config.is_blacklisted("peertube.social"), true);
        assert_eq!(config.use_raw_url(), false);
        assert_eq!(config.select_quality(), false);
        assert_eq!(config.use_torrent(), false);

        let app = App::from_yaml(yml);
        let matches = app
            .get_matches_from_safe(vec![
                "peertube-viewer-rs",
                "--torrent-downloader",
                "args-downloader",
                "--torrent-downloader-args=test",
                "--use-torrent",
            ])
            .unwrap();
        errors = config.update_with_args(matches);
        assert_eq!(errors.len(), 0);
        assert_eq!(config.player(), "args-downloader");
        assert_eq!(*config.player_args(), vec!["test", "-a"]);
        assert_eq!(config.use_torrent(), true);
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
