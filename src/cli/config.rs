// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

mod frontend_url_parser;

use clap::ArgMatches;
use directories::ProjectDirs;
use rustyline::config::EditMode;
use toml::{
    de::Error as TomlError,
    value::{Table, Value},
};

use frontend_url_parser::{ParsedUrl, UrlType};
use peertube_viewer_utils::to_https;

use std::collections::HashSet;
use std::default::Default;
use std::env::{var, vars_os};
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{error, io};

use super::clap_app::gen_app;

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
    NotAString(String),
    NonUtf8EnvironmentVariable {
        name: &'static str,
        provided: OsString,
    },
    IncorrectTag {
        name: &'static str,
        provided: String,
        allowed: &'static [&'static str],
    },
    ConflicingOptions(String, String),
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
                "The config was not parsable as TOML:\n{e}\nUsing default config"
            ),
            ConfigLoadError::NonUtf8EnvironmentVariable{name,provided:_} => write!(
                f,
                "Environnment variable {name} is not utf8." ,
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
            ConfigLoadError::NotAString(s) => write!(
                f,
                "{s} needs to be Strings\n Ignoring bad arguments"
            ),
            ConfigLoadError::UseTorrentAndNoInfo=> write!(
                f,
                "--use-torrent requires a torrent to be set\nUsing player instead of torrent"
            ),
            ConfigLoadError::ConflicingOptions(first, second) => write!(
                f,
                "{first} and {second} cannot appear at the same time in the config.\nUsing the search engine provided",
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
            | ConfigLoadError::ConflicingOptions(_, _)
            | ConfigLoadError::UseTorrentAndNoInfo
            | ConfigLoadError::NotATable
            | ConfigLoadError::NotAString(_) => None,
        }
    }
}

/// Test because otherwise it's unused and produces a warning
/// Might become useful outside of tests at some point
#[cfg(test)]
impl ConfigLoadError {
    pub fn is_unreadable(&self) -> bool {
        matches!(self, Self::UnreadableFile(_, _))
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
    VideoUrl(String),
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
    is_search_engine: bool,
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
        let app = gen_app();
        let cli_args = app.get_matches();
        Self::new_with_args(cli_args)
    }

    fn new_with_args(cli_args: clap::ArgMatches) -> (Config, InitialInfo, Vec<ConfigLoadError>) {
        if cli_args.get_flag("print-default-config") {
            print!("{}", include_str!("default_config.toml"));
            exit(0);
        }

        if cli_args.get_flag("print-full-config") {
            print!("{}", include_str!("full_config.toml"));
            exit(0);
        }

        // Parse config as an String with default to empty string
        let (mut config, mut load_errors) =
            if let Some(c) = cli_args.get_one::<String>("config-file") {
                Config::from_config_file(&PathBuf::from(c), false)
            } else {
                match ProjectDirs::from("", "peertube-viewer-rs", "peertube-viewer-rs") {
                    Some(dirs) => {
                        let mut d = dirs.config_dir().to_owned();
                        d.push("config.toml");
                        Config::from_config_file(&d, true)
                    }
                    None => (Config::default(), Vec::new()),
                }
            };

        let initial_info = if cli_args.get_flag("trending") {
            InitialInfo::Trending
        } else if let Some(s) = cli_args.get_one::<String>("chandle") {
            InitialInfo::Handle(s.to_string())
        } else if cli_args.get_flag("channels") {
            InitialInfo::Channels(concat(
                cli_args
                    .get_many::<String>("initial-query")
                    .into_iter()
                    .flatten(),
            ))
        } else if let Some(s) = cli_args
            .get_many("initial-query")
            .map(|it| it.map(String::as_str).collect::<Vec<&str>>())
        {
            match ParsedUrl::from_url(s[0]) {
                Some(parsed) => {
                    config.instance = parsed.instance;
                    match parsed.url_data {
                        UrlType::Video(_) | UrlType::Channel(_) => config.is_search_engine = false,
                        _ => config.is_search_engine = true,
                    }

                    match parsed.url_data {
                        UrlType::Video(uuid) => InitialInfo::VideoUrl(uuid),
                        UrlType::Channel(chandle) => InitialInfo::Handle(chandle),
                        UrlType::Search(search) => InitialInfo::Query(search),
                        UrlType::LandingPage => InitialInfo::Query(concat(s)),
                    }
                }
                None => InitialInfo::Query(concat(s)),
            }
        } else {
            InitialInfo::None
        };

        load_errors.append(&mut config.update_with_args(cli_args));

        (config, initial_info, load_errors)
    }

    fn from_config_file(path: &Path, is_default: bool) -> (Config, Vec<ConfigLoadError>) {
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
            .map_err(|e| {
                if !is_default && matches!(e.kind(), io::ErrorKind::NotFound) {
                    load_errors.push(ConfigLoadError::UnreadableFile(e, path.to_path_buf()))
                }
            })
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
                        .and_then(|cmd| cmd.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "mpv".to_string()),
                    get_string_array(t, "args", &mut load_errors),
                    t.get("use-raw-urls")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false),
                    t.get("prefer-hls")
                        .and_then(|b| b.as_bool())
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
            t.get("command")
                .and_then(|cmd| cmd.as_str())
                .map(|s| TorrentConf {
                    client: s.to_string(),
                    args: get_string_array(t, "args", &mut load_errors),
                })
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
            match (t.get("main"), t.get("search-engine")) {
                (None, None) => {}
                (Some(_), Some(Value::String(search))) => {
                    temp.instance = to_https(search).into_owned();
                    temp.is_search_engine = true;
                    load_errors.push(ConfigLoadError::ConflicingOptions(
                        "instances: main".to_string(),
                        "instances: search-engine".to_string(),
                    ));
                }
                (None, Some(Value::String(search))) => {
                    temp.instance = to_https(search).into_owned();
                    temp.is_search_engine = true
                }
                (Some(Value::String(instance)), None) => {
                    temp.instance = to_https(instance).into_owned();
                    temp.is_search_engine = true
                }
                (Some(_), Some(_)) | (None, Some(_)) | (Some(_), None) => {
                    load_errors.push(ConfigLoadError::NotAString(
                        "instance : search-engine and instance: main".to_owned(),
                    ));
                }
            }
        }

        temp.listed_instances = list;
        temp.is_allowlist = is_allowlist;

        temp.torrent = torrent.map(|t| (t, false));

        (temp, load_errors)
    }

    fn update_with_args(&mut self, args: ArgMatches) -> Vec<ConfigLoadError> {
        let mut load_errors = Vec::new();

        if args.get_flag("let-nsfw") {
            self.nsfw = NsfwBehavior::Let
        } else if args.get_flag("block-nsfw") {
            self.nsfw = NsfwBehavior::Block
        } else if args.get_flag("tag-nsfw") {
            self.nsfw = NsfwBehavior::Tag
        }

        self.local = args.get_flag("local");

        if let Some(i) = args.get_one::<String>("instance") {
            self.instance = to_https(i).into_owned();
            self.is_search_engine = false;
        } else if let Some(s) = args.get_one::<String>("search-engine") {
            self.instance = to_https(s).into_owned();
            self.is_search_engine = true;
        }

        /* ---Torrent configuration --- */
        let client = args
            .get_one::<String>("torrent-downloader")
            .map(|s| s.to_owned());
        let torrent_args = args
            .get_many("torrent-downloader-arguments")
            .map(|v| v.map(|s: &String| s.to_string()).collect::<Vec<String>>());

        let use_torrent = args.get_flag("torrent");
        if self.torrent.is_none() && use_torrent && !args.get_flag("torrent-downloader") {
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
        if let Some(c) = args.get_one::<String>("player") {
            self.player.client = c.to_string();
        }
        args.get_many::<String>("player-args").map(|v| {
            v.map(|s| self.player.args.push(s.to_string()))
                .any(|_| false)
        });

        if args.get_flag("use-raw-urls") {
            self.player.use_raw_urls = true;
        }

        if args.get_flag("select-quality") {
            self.select_quality = true;
        }

        if args.get_flag("color") {
            self.colors = true;
        } else if args.get_flag("no-color") {
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

    pub fn is_search_engine(&self) -> bool {
        self.is_search_engine
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
            instance: "https://sepiasearch.org".to_string(),
            is_search_engine: true,
            torrent: None,
            user_agent: Some(USER_AGENT.into()),
            nsfw: NsfwBehavior::Tag,
            listed_instances: HashSet::new(),
            is_allowlist: false,
            edit_mode: EditMode::Emacs,
            browser: var("BROWSER").unwrap_or_else(|_| "firefox".to_string()),
            colors: true,
            select_quality: false,
            local: false,
            max_hist_lines: 2000,
        }
    }
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
            Some(format!("Blocked video from: {}", video.host()))
        } else {
            None
        }
    }
}

fn concat(v: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut concatenated = String::new();
    let mut it = v.into_iter();
    if let Some(s) = it.next() {
        concatenated.push_str(s.as_ref());
    }
    for s in it {
        concatenated.push(' ');
        concatenated.push_str(s.as_ref());
    }
    concatenated
}

fn get_string_array(t: &Table, name: &str, load_errors: &mut Vec<ConfigLoadError>) -> Vec<String> {
    t.get(name)
        .and_then(|cmd| cmd.as_array())
        .map(|v| {
            v.iter()
                .filter_map(|s| {
                    let res = s.as_str().map(|s| s.to_string());
                    if res.is_none() {
                        load_errors.push(ConfigLoadError::NotAString(format!(
                            "The elements of {name}"
                        )))
                    }
                    res
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;
    use pretty_assertions::assert_eq;

    #[test]
    fn load_config_then_args() {
        let path = PathBuf::from("src/cli/full_config.toml");
        let (mut config, mut errors) = Config::from_config_file(&path, false);
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

        let app = gen_app();
        let matches = app
            .try_get_matches_from(vec![
                "peertube-viewer-rs",
                "--player",
                "args-player",
                "--player-args=--no-video",
                "--instance=args.ploud.fr",
                "--use-raw-urls",
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
        let (mut config, mut errors) = Config::from_config_file(&path, false);
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

        let app = gen_app();
        let matches = app
            .try_get_matches_from(vec![
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
        let (config, errors) = Config::from_config_file(&path, true);
        assert_eq!(config.prefer_hls(), true);
        assert_eq!(errors.len(), 0);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn unknown_file() {
        let path = PathBuf::from("does/not/exists.toml");
        let (config, errors) = Config::from_config_file(&path, false);
        assert_eq!(config.prefer_hls(), true);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].is_unreadable());
        assert_eq!(config, Config::default());
    }

    #[test]
    fn default_no_file() {
        let path = PathBuf::from("does/not/exists.toml");
        let (config, errors) = Config::from_config_file(&path, true);
        assert_eq!(config.prefer_hls(), true);
        assert_eq!(errors.len(), 0);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn conflicting_args1() {
        let app = gen_app();
        assert_eq!(
            app.try_get_matches_from(vec!["peertube-viewer-rs", "--block-nsfw", "--let-nsfw"])
                .unwrap_err()
                .kind(),
            ErrorKind::ArgumentConflict
        );
    }
    #[test]
    fn conflicting_args2() {
        let app = gen_app();
        assert_eq!(
            app.try_get_matches_from(vec!["peertube-viewer-rs", "--block-nsfw", "--tag-nsfw"])
                .unwrap_err()
                .kind(),
            ErrorKind::ArgumentConflict
        );
    }

    #[test]
    fn conflicting_args3() {
        let app = gen_app();
        assert_eq!(
            app.try_get_matches_from(vec!["peertube-viewer-rs", "--let-nsfw", "--tag-nsfw"])
                .unwrap_err()
                .kind(),
            ErrorKind::ArgumentConflict
        );
    }

    #[test]
    fn initial_query() {
        let app = gen_app();
        let args = app
            .try_get_matches_from(vec![
                "peertube-viewer-rs",
                "-c",
                "src/cli/default_config.toml",
                "What",
                "is",
                "peertube",
            ])
            .unwrap();
        let (_, initial_info, errors) = Config::new_with_args(args);
        assert_eq!(errors.len(), 0, "{errors:?}");
        assert_eq!(initial_info, InitialInfo::Query("What is peertube".into()));
    }

    #[test]
    fn initial_channel_search() {
        let app = gen_app();
        let args = app
            .try_get_matches_from(vec![
                "peertube-viewer-rs",
                "-c",
                "src/cli/default_config.toml",
                "--channels",
                "What",
                "is",
                "peertube",
            ])
            .unwrap();
        let (_, initial_info, errors) = Config::new_with_args(args);
        assert_eq!(errors.len(), 0, "{errors:?}");
        assert_eq!(
            initial_info,
            InitialInfo::Channels("What is peertube".into())
        );
    }

    #[test]
    fn initial_and_chandle() {
        let app = gen_app();
        assert!(app
            .try_get_matches_from(vec![
                "peertube-viewer-rs",
                "-c",
                "src/cli/default_config.toml",
                "--chandle",
                "What",
                "is"
            ])
            .is_err());
    }
}
