use clap::{App, ArgMatches, Values};
use directories::ProjectDirs;
use rustyline::config::EditMode;
use toml::{
    de::Error as TomlError,
    value::{Table, Value},
};

use std::collections::HashSet;
use std::default::Default;
use std::env::vars_os;
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use std::{error, io};

pub trait Blocklist<T: ?Sized> {
    fn is_blocked(&self, instance: &T) -> Option<String>;
}

impl<T: ?Sized> Blocklist<T> for () {
    fn is_blocked(&self, _: &T) -> Option<String> {
        None
    }
}

#[derive(Debug, PartialEq)]
struct TorrentConf {
    pub client: String,
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct PlayerConf {
    pub client: String,
    pub args: Vec<String>,
    pub use_raw_urls: bool,
    pub prefer_hls: bool,
}

const NSFW_ALLOWED: [&str; 3] = ["tag", "block", "let"];
const COLORS_ALLOWED: [&str; 2] = ["enable", "disable"];
const EDIT_MODE_ALLOWED: [&str; 2] = ["emacs", "vi"];
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub enum ConfigLoadError {
    UnreadableFile(io::Error, PathBuf),
    TomlError(TomlError),
    UseTorrentAndNoInfo,
    NotATable,
    NotAString,
    NonUtf8EnvironmentVariable {
        name: &'static str,
        provided: OsString,
    },
    IncorrectTag {
        name: &'static str,
        provided: String,
        allowed: &'static [&'static str],
    },
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
            ConfigLoadError::NonUtf8EnvironmentVariable{name,provided:_} => write!(
                f,
                "Environnment variable {} is not utf8.",
                name ,
            ),
            ConfigLoadError::IncorrectTag{name,provided,allowed} => write!(
                f,
                "\"{}\" is not a valid tag for {}\nValid tags are: {:?}\nUsing default: \"{}\"",
                provided,
                name ,
                allowed,
                allowed[0],
            ),
            ConfigLoadError::NotATable => write!(
                f,
                "The config file is malformed, it should be a TOML table\nUsing default config"
            ),
            ConfigLoadError::NotAString => write!(
                f,
                "Command arguments and blocklisted instances need to be a table of String\n Ignoring bad arguments"
            ),
            ConfigLoadError::UseTorrentAndNoInfo=> write!(
                f,
                "--use-torrent requires a torrent to be set\nUsing player instead of torrent"
            ),
        }
    }
}

impl error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ConfigLoadError::UnreadableFile(e, _) => Some(e),
            ConfigLoadError::TomlError(e) => Some(e),
            ConfigLoadError::IncorrectTag {
                name: _,
                provided: _,
                allowed: _,
            }
            | ConfigLoadError::NonUtf8EnvironmentVariable {
                name: _,
                provided: _,
            }
            | ConfigLoadError::UseTorrentAndNoInfo
            | ConfigLoadError::NotATable
            | ConfigLoadError::NotAString => None,
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum InitialInfo {
    None,
    Query(String),
    Channels(String),
    Handle(String),
    Trending,
}

impl InitialInfo {
    pub fn take(&mut self) -> InitialInfo {
        let mut tmp = InitialInfo::None;
        std::mem::swap(&mut tmp, self);
        tmp
    }
}

/// Config for the cli interface
#[derive(Debug, PartialEq)]
pub struct Config {
    player: PlayerConf,
    instance: String,
    torrent: Option<(TorrentConf, bool)>,
    listed_instances: HashSet<String>,
    is_allowlist: bool,

    user_agent: Option<String>,

    edit_mode: EditMode,
    browser: String,

    nsfw: NsfwBehavior,
    select_quality: bool,
    colors: bool,
    local: bool,

    max_hist_lines: usize,
}

impl Config {
    pub fn new() -> (Config, InitialInfo, Vec<ConfigLoadError>) {
        let yml = load_yaml!("clap_app.yml");
        let app = App::from_yaml(yml);
        let cli_args = app.get_matches();

        if cli_args.is_present("print-default-config") {
            print!("{}", include_str!("default_config.toml"));
            exit(0);
        }

        if cli_args.is_present("print-full-config") {
            print!("{}", include_str!("full_config.toml"));
            exit(0);
        }

        let initial_info = if cli_args.is_present("trending") {
            InitialInfo::Trending
        } else if let Some(handle) = cli_args.value_of("chandle") {
            InitialInfo::Handle(handle.to_owned())
        } else if let Some(s) = cli_args.values_of("channels").map(concat) {
            InitialInfo::Channels(s)
        } else if let Some(s) = cli_args.values_of("initial-query").map(concat) {
            InitialInfo::Query(s)
        } else {
            InitialInfo::None
        };

        // Parse config as an String with default to empty string
        let (mut config, mut load_errors) = if let Some(c) = cli_args.value_of("config-file") {
            Config::from_config_file(&PathBuf::from(c))
        } else {
            match ProjectDirs::from("", "peertube-viewer-rs", "peertube-viewer-rs") {
                Some(dirs) => {
                    let mut d = dirs.config_dir().to_owned();
                    d.push("config.toml");
                    Config::from_config_file(&d)
                }
                None => (Config::default(), Vec::new()),
            }
        };

        load_errors.append(&mut config.update_with_args(cli_args));

        (config, initial_info, load_errors)
    }

    fn from_config_file(path: &PathBuf) -> (Config, Vec<ConfigLoadError>) {
        let mut temp = Config::default();
        let mut load_errors = Vec::new();

        for (key, value) in vars_os() {
            if key == "BROWSER" {
                match value.into_string() {
                    Ok(b) => temp.browser = b,
                    Err(ostr) => load_errors.push(ConfigLoadError::NonUtf8EnvironmentVariable {
                        name: "BROWSER",
                        provided: ostr,
                    }),
                }
            }
        }

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
        let (player_cmd, player_args, use_raw_urls, prefer_hls) =
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
                    t.get("prefer-hls")
                        .map(|b| b.as_bool())
                        .flatten()
                        .unwrap_or(true),
                )
            } else {
                ("mpv".to_string(), Vec::new(), false, true)
            };
        temp.player = PlayerConf {
            client: player_cmd,
            args: player_args,
            use_raw_urls,
            prefer_hls,
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

        /* ---General configuration --- */
        if let Some(Value::Table(t)) = config.get("general") {
            if let Some(Value::String(s)) = t.get("nsfw") {
                if s == "block" {
                    temp.nsfw = NsfwBehavior::Block;
                } else if s == "let" {
                    temp.nsfw = NsfwBehavior::Let;
                } else if s == "tag" {
                    temp.nsfw = NsfwBehavior::Tag;
                } else {
                    load_errors.push(ConfigLoadError::IncorrectTag {
                        name: "nsfw",
                        provided: s.to_string(),
                        allowed: &NSFW_ALLOWED,
                    });
                }
            }

            if let Some(Value::String(s)) = t.get("colors") {
                if s == "enable" {
                    temp.colors = true;
                } else if s == "disable" {
                    temp.colors = false;
                } else {
                    load_errors.push(ConfigLoadError::IncorrectTag {
                        name: "colors",
                        provided: s.to_string(),
                        allowed: &COLORS_ALLOWED,
                    });
                }
            }

            if let Some(Value::Boolean(true)) = t.get("select-quality") {
                temp.select_quality = true;
            }

            if let Some(Value::Boolean(false)) = t.get("user-agent") {
                temp.user_agent = None;
            }

            if let Some(Value::String(s)) = t.get("user-agent") {
                temp.user_agent = Some(s.into());
            }

            if let Some(Value::String(s)) = t.get("edit-mode") {
                if s == "vi" {
                    temp.edit_mode = EditMode::Vi;
                } else if s == "emacs" {
                    temp.edit_mode = EditMode::Emacs;
                } else {
                    load_errors.push(ConfigLoadError::IncorrectTag {
                        name: "edit-mode",
                        provided: s.to_string(),
                        allowed: &EDIT_MODE_ALLOWED,
                    });
                }
            }

            if let Some(Value::String(s)) = t.get("browser") {
                temp.browser = s.to_owned();
            }
        }

        /* ---Blocklist configuration --- */
        let (list, is_allowlist) = if let Some(Value::Table(t)) = config.get("instances") {
            if t.contains_key("allowlist") {
                (
                    get_string_array(t, "allowlist", &mut load_errors)
                        .into_iter()
                        .collect(),
                    true,
                )
            } else if t.contains_key("blocklist") {
                (
                    get_string_array(t, "blocklist", &mut load_errors)
                        .into_iter()
                        .collect(),
                    false,
                )
            } else if t.contains_key("whitelist") {
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

        if let Some(Value::Table(t)) = config.get("instances") {
            if let Some(Value::String(s)) = t.get("main") {
                temp.instance = correct_instance(s);
            }
        }

        temp.listed_instances = list;
        temp.is_allowlist = is_allowlist;

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

        self.local = args.is_present("local");

        if let Some(i) = args.value_of("instance") {
            self.instance = correct_instance(i);
        }

        /* ---Torrent configuration --- */
        let client = args.value_of("torrent-downloader").map(|c| c.to_string());
        let torrent_args = args
            .values_of("torrent-downloader-arguments")
            .map(|v| v.map(|s| s.to_string()).collect::<Vec<String>>());

        let use_torrent = args.is_present("torrent");
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
        if let Some(c) = args.value_of("player") {
            self.player.client = c.to_string();
        }
        args.values_of("player-args").map(|v| {
            v.map(|s| self.player.args.push(s.to_string()))
                .any(|_| false)
        });

        if args.is_present("use-raw-urls") {
            self.player.use_raw_urls = true;
        }

        if args.is_present("select-quality") {
            self.select_quality = true;
        }

        if args.is_present("color") {
            self.colors = true;
        } else if args.is_present("no-color") {
            self.colors = false;
        }

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

    pub fn prefer_hls(&self) -> bool {
        self.player.prefer_hls
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

    pub fn browser(&self) -> &str {
        &self.browser
    }

    pub fn use_torrent(&self) -> bool {
        matches!(self.torrent, Some((_, true)))
    }

    pub fn max_hist_lines(&self) -> usize {
        self.max_hist_lines
    }

    pub fn select_quality(&self) -> bool {
        self.select_quality
    }

    pub fn local(&self) -> bool {
        self.local
    }

    pub fn colors(&self) -> bool {
        self.colors
    }

    pub fn nsfw(&self) -> NsfwBehavior {
        self.nsfw
    }

    pub fn edit_mode(&self) -> EditMode {
        self.edit_mode
    }

    pub fn user_agent(&self) -> Option<String> {
        self.user_agent.clone()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            player: PlayerConf {
                client: "mpv".to_string(),
                args: Vec::new(),
                use_raw_urls: false,
                prefer_hls: true,
            },
            instance: "https://video.ploud.fr".to_string(),
            torrent: None,
            user_agent: Some(USER_AGENT.into()),
            nsfw: NsfwBehavior::Tag,
            listed_instances: HashSet::new(),
            is_allowlist: false,
            edit_mode: EditMode::Emacs,
            browser: "firefox".to_string(),
            colors: true,
            select_quality: false,
            local: false,
            max_hist_lines: 2000,
        }
    }
}

fn correct_instance(s: &str) -> String {
    let mut s = if s.starts_with("https://") {
        s.to_string()
    } else if let Some(stripped) = s.strip_prefix("http://") {
        format!("https://{}", stripped)
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

impl Blocklist<str> for Config {
    fn is_blocked(&self, instance: &str) -> Option<String> {
        if self.is_allowlist ^ self.listed_instances.contains(instance) {
            Some(instance.to_string())
        } else {
            None
        }
    }
}

impl Blocklist<peertube_api::Video> for Config {
    fn is_blocked(&self, video: &peertube_api::Video) -> Option<String> {
        if self.is_allowlist ^ self.listed_instances.contains(video.host()) {
            Some(format!("Blocked video from: {}", video.host().to_string()))
        } else {
            None
        }
    }
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

#[cfg(test)]
mod config {
    use super::*;
    use clap::ErrorKind;

    #[test]
    fn load_config_then_args() {
        let path = PathBuf::from("src/cli/full_config.toml");
        let (mut config, mut errors) = Config::from_config_file(&path);
        assert_eq!(errors.len(), 0);
        assert_eq!(config.nsfw(), NsfwBehavior::Block);
        assert_eq!(config.player(), "mpv");
        assert_eq!(*config.player_args(), vec!["--volume=30",]);
        assert_eq!(config.instance(), "https://skeptikon.fr");
        assert_eq!(config.browser(), "qutebrowser");
        assert!(config.is_blocked("peertube.social").is_some());
        assert_eq!(config.use_raw_url(), true);
        assert_eq!(config.select_quality(), true);
        assert_eq!(config.prefer_hls(), false);
        assert_eq!(config.edit_mode(), EditMode::Vi);
        assert_eq!(
            config.user_agent(),
            Some("Mozilla/5.0 (X11; Linux x86_64; rv:79.0) Gecko/20100101 Firefox/79.0".into())
        );

        let yml = load_yaml!("clap_app.yml");
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
        assert_eq!(config.edit_mode(), EditMode::Vi);
    }

    #[test]
    fn torrent_options() {
        let path = PathBuf::from("src/cli/full_config.toml");
        let (mut config, mut errors) = Config::from_config_file(&path);
        assert_eq!(errors.len(), 0);
        assert_eq!(config.nsfw(), NsfwBehavior::Block);
        assert_eq!(config.player(), "mpv");
        assert_eq!(*config.player_args(), vec!["--volume=30"]);
        assert_eq!(config.instance(), "https://skeptikon.fr");
        assert!(config.is_blocked("peertube.social").is_some());
        assert_eq!(config.use_raw_url(), true);
        assert_eq!(config.select_quality(), true);
        assert_eq!(config.use_torrent(), false);
        assert_eq!(config.colors(), false);

        let yml = load_yaml!("clap_app.yml");
        let app = App::from_yaml(yml);
        let matches = app
            .get_matches_from_safe(vec![
                "peertube-viewer-rs",
                "--torrent-downloader",
                "args-downloader",
                "--torrent-downloader-args=test",
                "--use-torrent",
                "--color",
            ])
            .unwrap();
        errors = config.update_with_args(matches);
        assert_eq!(errors.len(), 0);
        assert_eq!(config.player(), "args-downloader");
        assert_eq!(*config.player_args(), vec!["test", "-a"]);
        assert_eq!(config.use_torrent(), true);
        assert_eq!(config.colors(), true);
    }

    #[test]
    fn default_config_example() {
        let path = PathBuf::from("src/cli/default_config.toml");
        let (config, errors) = Config::from_config_file(&path);
        assert_eq!(config.prefer_hls(), true);
        assert_eq!(errors.len(), 0);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn conflicting_args1() {
        let yml = load_yaml!("clap_app.yml");
        let app = App::from_yaml(yml);
        assert_eq!(
            app.get_matches_from_safe(vec!["peertube-viewer-rs", "--block-nsfw", "--let-nsfw"])
                .unwrap_err()
                .kind,
            ErrorKind::ArgumentConflict
        );
    }
    #[test]
    fn conflicting_args2() {
        let yml = load_yaml!("clap_app.yml");
        let app = App::from_yaml(yml);
        assert_eq!(
            app.get_matches_from_safe(vec!["peertube-viewer-rs", "--block-nsfw", "--tag-nsfw"])
                .unwrap_err()
                .kind,
            ErrorKind::ArgumentConflict
        );
    }

    #[test]
    fn conflicting_args3() {
        let yml = load_yaml!("clap_app.yml");
        let app = App::from_yaml(yml);
        assert_eq!(
            app.get_matches_from_safe(vec!["peertube-viewer-rs", "--let-nsfw", "--tag-nsfw"])
                .unwrap_err()
                .kind,
            ErrorKind::ArgumentConflict
        );
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
