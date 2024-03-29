// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

mod clap_app;
mod config;
mod display;
mod history;
mod input;
mod parser;
mod preloadables;

pub use config::ConfigLoadError;
use config::{Blocklist, Config, InitialInfo};
use display::Display;
use history::History;
use input::Editor;
use parser::ParsedQuery;

use crate::error::Error;

use rustyline::error::ReadlineError;

use peertube_api::{error::Error as ApiError, Instance, VideoState};

use preloadable_list::PreloadableList;
use preloadables::{Channels, Comments, Videos};

use std::fs::create_dir;
use std::sync::Arc;

use directories::ProjectDirs;
use std::process::Command;

const SEARCH_TOTAL: usize = 20;

pub struct Cli {
    config: Config,
    history: History,
    dirs: Option<ProjectDirs>,
    rl: Editor,
    display: Display,
    instance: Arc<Instance>,
    initial_info: InitialInfo,
}

impl Cli {
    /// Loads an instance of the cli
    pub fn init() -> Result<Cli, Error> {
        let (config, initial_info, load_errors) = Config::new();
        let display = Display::new(config.colors());

        let mut err_iter = load_errors.into_iter();
        if let Some(err) = err_iter.next() {
            display.err(&err);
        }

        for err in err_iter {
            display.message("");
            display.err(&err);
        }

        let mut history = History::new();

        let dirs = ProjectDirs::from("", "peertube-viewer-rs", "peertube-viewer-rs");
        let mut rl = Editor::new(config.edit_mode(), config.colors())?;

        // Loads the history if available
        if let Some(d) = dirs.as_ref() {
            let cache = d.cache_dir();

            create_dir(cache).unwrap_or(());

            let mut view_hist_file = cache.to_owned();
            view_hist_file.push("history");
            let mut cmd_hist_file = cache.to_owned();
            cmd_hist_file.push("cmd_history");

            history.load_file(&view_hist_file).unwrap_or(()); // unwrap_or to ignore the unused_must_use warnings
            rl.load_history(&cmd_hist_file).unwrap_or(()); // we don't care if the loading failed
        }

        // If the initial query is a url, connect to the corresponding instance

        let instance_domain = config.instance().to_string();

        let instance = Instance::new(
            if config.is_blocked(&instance_domain[8..]).is_some() {
                let err = Error::BlockedInstance(instance_domain[8..].to_string());
                display.err(&err);
                return Err(err);
            } else {
                instance_domain
            },
            !config.nsfw().is_block(),
            config.local(),
            config.user_agent(),
            config.is_search_engine(),
        );

        if !matches!(initial_info, InitialInfo::VideoUrl(_)) {
            display.welcome(instance.host());
        }

        Ok(Cli {
            config,
            history,
            dirs,
            rl,
            display,
            instance,
            initial_info,
        })
    }

    /// Main loop for he cli interface
    fn main_loop(&mut self) -> Result<(), Error> {
        let mode = Mode::Temp; //Placeholder that will be changed just after anyway on the first loop run
        let action = match self.initial_info.take() {
            InitialInfo::VideoUrl(s) => {
                self.play_vid(&self.instance.single_video(self.instance.host(), &s)?)?;
                return Ok(());
            }
            InitialInfo::Query(s) => ParsedQuery::Query(s),
            InitialInfo::Channels(s) => ParsedQuery::Channels(s),
            InitialInfo::Handle(s) => ParsedQuery::Chandle(s),
            InitialInfo::Trending => ParsedQuery::Trending,
            InitialInfo::None => {
                self.display.info("Search for videos (:h for help)");
                self.rl.first_readline(">> ".to_string())?
            }
        };
        let changed_action = true;

        let mut data = LoopData {
            mode,
            action,
            changed_action,
            stop: false,
        };

        // Main loop
        while !data.stop {
            if let Err(err) = self.one_loop(&mut data) {
                if let Some(err) = self.handle_err(err) {
                    return Err(err);
                } else {
                    data.changed_action = true;
                    data.mode = Mode::Temp;
                    self.display.continue_despite_error();
                    data.action = self.rl.readline(">> ".to_string(), None)?;
                };
            }
        }
        Ok(())
    }

    fn one_loop(&mut self, data: &mut LoopData) -> Result<(), Error> {
        self.parse_action(data)?;
        if data.stop {
            return Ok(());
        }

        data.mode.ensure_init()?;

        match &mut data.mode {
            Mode::Videos(videos) => {
                self.video_prompt(videos, &mut data.action, &mut data.changed_action)?
            }
            Mode::Channels(channels) => {
                self.channel_prompt(channels, &mut data.action, &mut data.changed_action)?
            }
            Mode::Comments(comments) => {
                self.comments_prompt(comments, &mut data.action, &mut data.changed_action)?
            }
            Mode::Temp => unreachable!(),
        };
        Ok(())
    }

    fn parse_action(&mut self, data: &mut LoopData) -> Result<(), Error> {
        if data.changed_action {
            match &data.action {
                ParsedQuery::Query(s) => {
                    self.rl.add_history_entry(s);
                    let search_tmp = PreloadableList::new(
                        Videos::new_search(self.instance.clone(), s),
                        SEARCH_TOTAL,
                    );
                    data.mode = Mode::Videos(search_tmp);
                }
                ParsedQuery::Quit => {
                    data.stop = true;
                    return Ok(());
                }
                ParsedQuery::Next => match &mut data.mode {
                    Mode::Videos(search) => {
                        search.try_next()?;
                    }
                    Mode::Channels(channels) => {
                        channels.try_next()?;
                    }
                    Mode::Comments(comments) => {
                        comments.try_next()?;
                    }
                    Mode::Temp => unreachable!(),
                },
                ParsedQuery::Previous => match &mut data.mode {
                    Mode::Videos(search) => {
                        search.prev();
                    }
                    Mode::Channels(channels) => {
                        channels.prev();
                    }
                    Mode::Comments(comments) => {
                        comments.prev();
                    }
                    Mode::Temp => unreachable!(),
                },
                ParsedQuery::Trending => {
                    if self.config.is_search_engine() {
                        self.display.warn(&"Trending results are not available when using a search engine such as sepia.");
                        if data.mode.is_temp() {
                            self.display.info("Search for videos (:h for help)");
                            data.action = self.rl.first_readline(">> ".to_string())?;
                            data.changed_action = true;
                            self.parse_action(data)?;
                        }
                    } else {
                        let trending_tmp = PreloadableList::new(
                            Videos::new_trending(self.instance.clone()),
                            SEARCH_TOTAL,
                        );
                        data.mode = Mode::Videos(trending_tmp);
                        self.rl.add_history_entry(":trending");
                    }
                }
                ParsedQuery::Help => {
                    self.display.help();
                    self.rl.std_in("Press enter to continue".to_string())?;
                    if data.mode.is_temp() {
                        self.display.info("Search for videos (:h for help)");
                        data.action = self.rl.first_readline(">> ".to_string())?;
                        data.changed_action = true;
                        self.parse_action(data)?;
                    }
                }
                ParsedQuery::Channels(q) => {
                    let channels_tmp =
                        PreloadableList::new(Channels::new(self.instance.clone(), q), SEARCH_TOTAL);
                    data.mode = Mode::Channels(channels_tmp);
                    self.rl.add_history_entry(&format!(":channels {q}"));
                }
                ParsedQuery::Chandle(handle) => {
                    let chandle_tmp = PreloadableList::new(
                        Videos::new_channel(self.instance.clone(), handle),
                        SEARCH_TOTAL,
                    );
                    data.mode = Mode::Videos(chandle_tmp);
                    self.rl.add_history_entry(&format!(":chandle {handle}"));
                }
                ParsedQuery::Info(id) => {
                    self.info(&data.mode, *id)?;
                    self.rl.add_history_entry(&format!(":info {id}"));
                }
                ParsedQuery::Browser(id) => {
                    self.open_browser(&data.mode, *id)?;
                    self.rl.add_history_entry(&format!(":browser {id}"));
                }
                ParsedQuery::Comments(id) => {
                    self.comments(&mut data.mode, *id);
                    self.rl.add_history_entry(&format!(":comments {id}"));
                }
                ParsedQuery::Id(_) => unreachable!(),
            };
        }
        Ok(())
    }

    fn video_prompt(
        &mut self,
        videos: &mut PreloadableList<Videos>,
        action: &mut ParsedQuery,
        changed_action: &mut bool,
    ) -> Result<(), Error> {
        self.display
            .video_list(videos.current(), &self.history, &self.config);
        self.display.mode_info(
            videos.loader().name(),
            videos.expected_total(),
            videos.offset(),
            videos.current_len(),
        );
        videos
            .loader()
            .preload_res(self.config.select_quality() || self.config.use_raw_url());
        let choice = match self.rl.autoload_readline(">> ".to_string(), videos)? {
            ParsedQuery::Id(id) => id,
            new_action => {
                *action = new_action;
                *changed_action = true;
                return Ok(());
            }
        };

        let video = videos.current()[choice - 1].clone();
        self.play_vid(&video)?;
        *changed_action = false;
        Ok(())
    }

    fn channel_prompt(
        &mut self,
        channels: &mut PreloadableList<Channels>,
        action: &mut ParsedQuery,
        changed_action: &mut bool,
    ) -> Result<(), Error> {
        self.display
            .channel_list(channels.current(), &self.history, &self.config);
        self.display.mode_info(
            "Channel search",
            channels.expected_total(),
            channels.offset(),
            channels.current_len(),
        );
        match self.rl.autoload_readline(">> ".to_string(), channels)? {
            ParsedQuery::Id(id) => {
                *action = ParsedQuery::Chandle(channels.current()[id - 1].handle());
                *changed_action = true;
                Ok(())
            }
            new_action => {
                *action = new_action;
                *changed_action = true;
                Ok(())
            }
        }
    }

    fn comments_prompt(
        &mut self,
        comments: &mut PreloadableList<Comments>,
        action: &mut ParsedQuery,
        changed_action: &mut bool,
    ) -> Result<(), Error> {
        self.display.comment_list(comments.current());
        self.display.mode_info(
            "Browsing video comments",
            comments.expected_total(),
            comments.offset(),
            comments.current_len(),
        );
        match self.rl.autoload_readline(">> ".to_string(), comments)? {
            ParsedQuery::Id(id) => {
                *action = ParsedQuery::Browser(id);
                *changed_action = true;
                Ok(())
            }
            new_action => {
                *action = new_action;
                *changed_action = true;
                Ok(())
            }
        }
    }

    fn comments(&mut self, mode: &mut Mode, id: usize) {
        match mode {
            Mode::Videos(v) => {
                self.display.video_info(&v.current()[id - 1]);
                let comments_tmp = PreloadableList::new(
                    Comments::new(
                        self.instance.clone(),
                        v.current()[id - 1].host().to_owned(),
                        v.current()[id - 1].uuid().to_owned(),
                    ),
                    SEARCH_TOTAL,
                );
                *mode = Mode::Comments(comments_tmp);
            }
            Mode::Channels(_) => self.display.err(&"Channels don't have comments"),
            Mode::Comments(_) => self.display.err(&"Comments don't have comments"),
            Mode::Temp => panic!("Bad use of temp"),
        }
    }

    fn info(&mut self, mode: &Mode, id: usize) -> Result<(), Error> {
        match mode {
            Mode::Videos(v) => self.display.video_info(&v.current()[id - 1]),
            Mode::Channels(c) => self.display.channel_info(&c.current()[id - 1]),
            Mode::Comments(_) => self.display.warn(&"No additional info available"),
            Mode::Temp => panic!("Bad use of temp"),
        }
        self.rl.std_in("Press enter to continue".to_string())?;
        Ok(())
    }

    fn open_browser(&mut self, mode: &Mode, id: usize) -> Result<(), Error> {
        match &mode {
            Mode::Videos(v) => {
                self.display.video_info(&v.current()[id - 1]);
                Command::new(self.config.browser())
                    .arg(v.current()[id - 1].watch_url())
                    .spawn()
                    .map_err(Error::BrowserLaunch)?
                    .wait()
                    .map_err(Error::BrowserLaunch)?;
            }
            Mode::Channels(c) => {
                self.display.channel_info(&c.current()[id - 1]);
                let c = &c.current()[id - 1];
                Command::new(self.config.browser())
                    .arg(self.instance.channel_url(c.host(), c))
                    .spawn()
                    .map_err(Error::BrowserLaunch)?
                    .wait()
                    .map_err(Error::BrowserLaunch)?;
            }
            Mode::Comments(c) => {
                Command::new(self.config.browser())
                    .arg(c.current()[id - 1].url())
                    .spawn()
                    .map_err(Error::BrowserLaunch)?
                    .wait()
                    .map_err(Error::BrowserLaunch)?;
            }
            Mode::Temp => panic!("Bad use of temp"),
        }

        Ok(())
    }

    fn play_vid(&mut self, video: &peertube_api::Video) -> Result<(), Error> {
        // Resolution selection
        self.display.video_info(video);
        if self.config.is_blocked(video.host()).is_some() {
            self.display.err(&"This video is from a blocked instance.");
            let confirm = self.rl.std_in("Play it anyway ? [y/N]: ".to_string())?;
            if confirm != "y" && confirm != "Y" {
                return Ok(());
            }
        }

        if !matches!(
            video.state(),
            VideoState::Published | VideoState::None | VideoState::Unknown(_, _)
        ) {
            match video.state() {
                VideoState::ToTranscode | VideoState::ToImport => {
                    self.display.warn(&"This video is not yet available")
                }
                VideoState::WaitingForLive => {
                    self.display.warn(&"This livestream has not started yet")
                }
                VideoState::LiveEnded => self.display.warn(&"This livestream is over"),
                _ => unreachable!(),
            }

            return Ok(());
        }

        let video_url = if self.config.select_quality() {
            let resolutions = video.resolutions()?;
            let nb_resolutions = resolutions.len();

            if nb_resolutions == 0 {
                if video.has_streams()? {
                    self.display
                        .warn(&"No resolutions available\nVideo will be played with an HLS stream");
                    video.stream_url(0)?
                } else if self.config.use_torrent() {
                    self.display
                        .warn(&"Unable to fetch torrent url\nThis video will be skipped");
                    return Ok(());
                } else {
                    self.display
                        .warn(&"Unable to fetch resolutions\nAttempting to play with watch url");
                    video.watch_url()
                }
            } else {
                self.display.resolutions(resolutions);
                let choice = self
                    .rl
                    .readline_id(">> ".to_string(), Some(nb_resolutions + 1))?;
                if self.config.use_torrent() {
                    video.torrent_url(choice - 1)?
                } else {
                    video.resolution_url(choice - 1)?
                }
            }
        } else if self.config.use_torrent() {
            video.load_resolutions()?;

            match video.torrent_url(0) {
                Ok(url) => url,
                Err(peertube_api::error::Error::OutOfBound(_)) => {
                    self.display
                        .warn(&"Unable to fetch torrent url\nThis video will be skipped");
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            }
        } else if self.config.use_raw_url() {
            video.load_resolutions()?;
            if (self.config.prefer_hls() && video.has_streams()?) || !video.has_files()? {
                match video.stream_url(0) {
                    Ok(url) => url,
                    Err(peertube_api::error::Error::OutOfBound(_)) => {
                        self.display
                            .warn(&"Unable to fetch streaming video url\nAttempting to play with watch url");
                        video.watch_url()
                    }
                    Err(err) => return Err(err.into()),
                }
            } else {
                match video.resolution_url(0) {
                    Ok(url) => url,
                    Err(peertube_api::error::Error::OutOfBound(_)) => {
                        self.display.warn(
                            &"Unable to fetch raw video url\nAttempting to play with watch url",
                        );
                        video.watch_url()
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        } else {
            video.watch_url()
        };
        self.history.add_video(video.uuid().to_string());

        Command::new(self.config.player())
            .args(self.config.player_args())
            .arg(video_url)
            .spawn()
            .map_err(Error::VideoLaunch)?
            .wait()
            .map_err(Error::VideoLaunch)?;
        Ok(())
    }

    /// Returns None if the error was dealt with
    fn handle_err(&mut self, err: Error) -> Option<Error> {
        match &err {
            Error::Api(ApiError::Ureq(e)) => {
                if let ureq::Error::Status(code, _) = &**e {
                    if *code >= 400 && *code < 500 {
                        self.display.err(&format!(
                            "\
                            Error encountered while connecting to the desired instance: {}\n\
                            This is likely because the server you are trying to connect isn't a PeerTube instance.\
                        ",
                            *code
                        ));
                        self.display.report_error(err, self.instance.host());
                        None
                    } else if *code >= 500 {
                        self.display.err(&format!(
                            "\
                            The server you are trying to connect failed to process the request: {}\n\
                            This likely isn't a bug from peertube-viewer-rs.
                        ",
                            *code
                        ));
                        self.display.report_error(err, self.instance.host());
                        None
                    } else {
                        Some(err)
                    }
                } else {
                    Some(err)
                }
            }
            Error::Api(ApiError::Serde(_)) => {
                self.display.err(&"\
                            peertube-viewer-rs was not capable of understanding the response from the server\n\
                            This is might happen if the server you are trying to connect isn't a PeerTube instance.\n\
                            If not, it is a bug from peertube-viewer-rs".to_string()
                        );
                self.display.report_error(err, self.instance.host());
                None
            }
            Error::Api(ApiError::NoContent) => {
                self.display.err(&"\
                            No content seems to be available for this video\n\
                            This is might happen if video you are trying to play comes from a instance that is down\n\
                            If not, it is a bug from peertube-viewer-rs".to_string()
                        );
                self.display.report_error(err, self.instance.host());
                None
            }

            _ => Some(err),
        }
    }

    fn top_level_err(&mut self, err: Error) {
        match &err {
            Error::Readline(ReadlineError::Interrupted) | Error::Readline(ReadlineError::Eof) => {}
            err => {
                self.display.err(&format!(
                    "\
                Unexpected error: {err}\n\
                This is likely an error with peertube-viewer-rs.\n\
                "
                ));
                self.display.report_error(err, self.instance.host());
            }
        }
    }

    pub fn run(&mut self) {
        self.main_loop().unwrap_or_else(|e| self.top_level_err(e));
    }
}

struct LoopData {
    mode: Mode,
    action: ParsedQuery,
    changed_action: bool,
    stop: bool,
}

enum Mode {
    Videos(PreloadableList<Videos>),
    Channels(PreloadableList<Channels>),
    Comments(PreloadableList<Comments>),
    Temp,
}

impl Mode {
    pub fn ensure_init(&mut self) -> Result<(), Error> {
        match self {
            Mode::Videos(v) => Ok(v.ensure_init()?),
            Mode::Channels(c) => Ok(c.ensure_init()?),
            Mode::Comments(c) => Ok(c.ensure_init()?),
            Mode::Temp => panic!("Bad use of temp"),
        }
    }

    pub fn is_temp(&self) -> bool {
        matches!(self, Mode::Temp)
    }
}

impl Drop for Cli {
    fn drop(&mut self) {
        if let Some(d) = self.dirs.as_ref() {
            let mut view_hist_file = d.cache_dir().to_owned();
            view_hist_file.push("history");
            self.history
                .save(&view_hist_file, self.config.max_hist_lines())
                .unwrap_or(());
            let mut cmd_hist_file = d.cache_dir().to_owned();
            cmd_hist_file.push("cmd_history");
            self.rl.save_history(&cmd_hist_file).unwrap_or(());
        }
    }
}
