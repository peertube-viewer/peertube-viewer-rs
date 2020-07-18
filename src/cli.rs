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
use input::{Action, Editor};

use crate::error::Error;

use rustyline::error::ReadlineError;

use peertube_api::{error::Error as ApiError, Instance};

use preloadable_list::PreloadableList;
use preloadables::{Channels, Comments, Videos};

use std::fs::create_dir;
use std::rc::Rc;

use directories::ProjectDirs;
use tokio::process::Command;
use tokio::runtime;
use tokio::task::LocalSet;

const SEARCH_TOTAL: usize = 20;

pub struct Cli {
    config: Config,
    history: History,
    dirs: Option<ProjectDirs>,
    rl: Editor,
    display: Display,
    instance: Rc<Instance>,
    initial_info: InitialInfo,
    is_single_url: bool,
}

impl Cli {
    /// Loads an instance of the cli
    pub fn init() -> Result<Cli, Error> {
        let (config, mut initial_info, load_errors) = Config::new();
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
        let mut rl = Editor::new(config.edit_mode());

        // Loads the history if available
        if let Some(d) = dirs.as_ref() {
            let cache = d.cache_dir();

            create_dir(&cache).unwrap_or(());

            let mut view_hist_file = cache.to_owned();
            view_hist_file.push("history");
            let mut cmd_hist_file = cache.to_owned();
            cmd_hist_file.push("cmd_history");

            history.load_file(&view_hist_file).unwrap_or(()); // unwrap_or to ignore the unused_must_use warnings
            rl.load_history(&cmd_hist_file).unwrap_or(()); // we don't care if the loading failed
        }

        // If the initial query is a url, connect to the corresponding instance
        let mut is_single_url = false;
        let instance_domain = match &initial_info {
            InitialInfo::Query(s) | InitialInfo::Channels(Some(s))
                if s.starts_with("http://") || s.starts_with("https://") =>
            {
                match s.split('/').nth(2) {
                    Some(domain) => {
                        let instance_temp =
                            format!("https://{}", domain.split(' ').next().expect("Unreachable"));
                        match s.split('/').nth(5) {
                            Some(uuid) => {
                                is_single_url = true;
                                initial_info = InitialInfo::Query(
                                    uuid.split(' ').next().expect("Unreachable").to_string(),
                                );
                            }

                            None => {
                                initial_info = if let Some(s) =
                                    s.splitn(2, ' ').nth(1).map(|s| s.to_string())
                                {
                                    InitialInfo::Query(s)
                                } else {
                                    InitialInfo::None
                                };
                            }
                        }
                        instance_temp
                    }
                    None => config.instance().to_string(),
                }
            }
            _ => config.instance().to_string(),
        };

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
        );

        if !is_single_url {
            display.welcome(instance.host());
        }

        Ok(Cli {
            config,
            history,
            rl,
            dirs,
            display,
            instance,
            initial_info,
            is_single_url,
        })
    }

    /// Main loop for he cli interface
    async fn main_loop(&mut self) -> Result<(), Error> {
        let mode = Mode::Temp; //Placeholder that will be changed just after anyway on the first loop run
        let action = match self.initial_info.take() {
            InitialInfo::Query(s) if self.is_single_url => {
                self.play_vid(&self.instance.single_video(&s).await?)
                    .await?;
                return Ok(());
            }
            InitialInfo::Query(s) => Action::Query(s),
            InitialInfo::Channels(Some(s)) => Action::Query(format!(":channels {}", s)),
            InitialInfo::Channels(None) => Action::Query(format!(
                ":channels {}",
                self.rl.readline(">> ".to_string()).await?
            )),
            InitialInfo::Handle(s) => Action::Query(format!(":chandle {}", s)),
            InitialInfo::Trending => Action::Query(":trending".to_string()),
            InitialInfo::None => Action::Query(self.rl.readline(">> ".to_string()).await?),
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
            if let Err(err) = self.one_loop(&mut data).await {
                if let Some(err) = self.handle_err(err) {
                    return Err(err);
                } else {
                    data.changed_action = true;
                    data.mode = Mode::Temp;
                    self.display.continue_despite_error();
                    data.action = Action::Query(self.rl.readline(">> ".to_string()).await?);
                };
            }
        }
        Ok(())
    }

    async fn one_loop(&mut self, data: &mut LoopData) -> Result<(), Error> {
        self.parse_action(data).await?;
        match &mut data.mode {
            Mode::Videos(videos) => {
                self.video_prompt(videos, &mut data.action, &mut data.changed_action)
                    .await?
            }
            Mode::Channels(channels) => {
                self.channel_prompt(channels, &mut data.action, &mut data.changed_action)
                    .await?
            }
            Mode::Comments(comments) => {
                self.comments_prompt(comments, &mut data.action, &mut data.changed_action)
                    .await?
            }
            Mode::Temp => unreachable!(),
        };
        Ok(())
    }

    async fn parse_action(&mut self, data: &mut LoopData) -> Result<(), Error> {
        if data.changed_action {
            match &data.action {
                Action::Query(s) => {
                    self.parse_query(&mut data.mode, &s).await?;
                    data.mode.ensure_init().await?;
                }
                Action::Quit => {
                    data.stop = true;
                    return Ok(());
                }
                Action::Next => match &mut data.mode {
                    Mode::Videos(search) => {
                        search.next().await?;
                    }
                    Mode::Channels(channels) => {
                        channels.next().await?;
                    }
                    Mode::Comments(comments) => {
                        comments.next().await?;
                    }
                    Mode::Temp => unreachable!(),
                },
                Action::Prev => match &mut data.mode {
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
                _ => unreachable!(),
            };
        }
        Ok(())
    }

    async fn video_prompt(
        &mut self,
        videos: &mut PreloadableList<Videos>,
        action: &mut Action,
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
            .loader_mut()
            .preload_res(self.config.select_quality() || self.config.use_raw_url());
        let choice;
        match self.rl.autoload_readline(">> ".to_string(), videos).await? {
            Action::Id(id) => choice = id,
            new_action => {
                *action = new_action;
                *changed_action = true;
                return Ok(());
            }
        };

        let video = videos.current()[choice - 1].clone();
        self.play_vid(&video).await?;
        *changed_action = false;
        Ok(())
    }

    async fn channel_prompt(
        &mut self,
        channels: &mut PreloadableList<Channels>,
        action: &mut Action,
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
        match self
            .rl
            .autoload_readline(">> ".to_string(), channels)
            .await?
        {
            Action::Id(id) => {
                *action =
                    Action::Query(format!(":chandle {}", channels.current()[id - 1].handle()));
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

    async fn comments_prompt(
        &mut self,
        comments: &mut PreloadableList<Comments>,
        action: &mut Action,
        changed_action: &mut bool,
    ) -> Result<(), Error> {
        self.display.comment_list(comments.current());
        self.display.mode_info(
            "Browsing video comments",
            comments.expected_total(),
            comments.offset(),
            comments.current_len(),
        );
        match self
            .rl
            .autoload_readline(">> ".to_string(), comments)
            .await?
        {
            Action::Id(id) => {
                *action = Action::Query(format!(":browser {}", id));
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

    async fn parse_query(&mut self, mode: &mut Mode, s: &str) -> Result<(), Error> {
        if s == ":trending" {
            let trending_tmp =
                PreloadableList::new(Videos::new_trending(self.instance.clone()), SEARCH_TOTAL);
            *mode = Mode::Videos(trending_tmp);
        } else if let Some(q) = parser::channels(&s) {
            let channels_tmp =
                PreloadableList::new(Channels::new(self.instance.clone(), q), SEARCH_TOTAL);
            *mode = Mode::Channels(channels_tmp);
            self.rl.add_history_entry(s);
        } else if let Some(handle) = parser::chandle(s) {
            let chandle_tmp = PreloadableList::new(
                Videos::new_channel(self.instance.clone(), handle),
                SEARCH_TOTAL,
            );
            *mode = Mode::Videos(chandle_tmp);
            self.rl.add_history_entry(s);
        } else if let Some(id) = parser::info(s, mode.current_len()) {
            self.info(&mode, id).await?;
            self.rl.add_history_entry(s);
            return Ok(());
        } else if let Some(id) = parser::browser(s, mode.current_len()) {
            self.open_browser(mode, id).await?;
            self.rl.add_history_entry(s);
            return Ok(());
        } else if let Some(id) = parser::comments(s, mode.current_len()) {
            self.comments(mode, id).await?;
            self.rl.add_history_entry(s);
            return Ok(());
        } else {
            self.rl.add_history_entry(&s);
            let search_tmp =
                PreloadableList::new(Videos::new_search(self.instance.clone(), &s), SEARCH_TOTAL);
            *mode = Mode::Videos(search_tmp);
        }

        Ok(())
    }

    async fn comments(&mut self, mode: &mut Mode, id: usize) -> Result<(), Error> {
        match mode {
            Mode::Videos(v) => {
                self.display.video_info(&v.current()[id - 1]).await;
                let comments_tmp = PreloadableList::new(
                    Comments::new(self.instance.clone(), v.current()[id - 1].uuid()),
                    SEARCH_TOTAL,
                );
                *mode = Mode::Comments(comments_tmp);
            }
            Mode::Channels(_) => self.display.err(&"Channels don't have comments"),
            Mode::Comments(_) => self.display.err(&"Comments don't have comments"),
            Mode::Temp => panic!("Bad use of temp"),
        }
        Ok(())
    }

    async fn info(&mut self, mode: &Mode, id: usize) -> Result<(), Error> {
        match mode {
            Mode::Videos(v) => self.display.video_info(&v.current()[id - 1]).await,
            Mode::Channels(c) => self.display.channel_info(&c.current()[id - 1]).await,
            Mode::Comments(_) => self.display.warn(&"No additionnal info available"),
            Mode::Temp => panic!("Bad use of temp"),
        }
        self.rl
            .std_in("Press enter to continue".to_string())
            .await?;
        Ok(())
    }

    async fn open_browser(&mut self, mode: &Mode, id: usize) -> Result<(), Error> {
        match &mode {
            Mode::Videos(v) => {
                self.display.video_info(&v.current()[id - 1]).await;
                Command::new(self.config.browser())
                    .arg(&v.current()[id - 1].watch_url())
                    .spawn()
                    .map_err(Error::BrowserLaunch)?
                    .await
                    .map_err(Error::BrowserLaunch)?;
            }
            Mode::Channels(c) => {
                self.display.channel_info(&c.current()[id - 1]).await;
                Command::new(self.config.browser())
                    .arg(self.instance.channel_url(&c.current()[id - 1]))
                    .spawn()
                    .map_err(Error::BrowserLaunch)?
                    .await
                    .map_err(Error::BrowserLaunch)?;
            }
            Mode::Comments(c) => {
                Command::new(self.config.browser())
                    .arg(&c.current()[id - 1].url())
                    .spawn()
                    .map_err(Error::BrowserLaunch)?
                    .await
                    .map_err(Error::BrowserLaunch)?;
            }
            Mode::Temp => panic!("Bad use of temp"),
        }

        Ok(())
    }

    async fn play_vid(&mut self, video: &peertube_api::Video) -> Result<(), Error> {
        // Resolution selection
        self.display.video_info(&video).await;
        if self.config.is_blocked(video.host()).is_some() {
            self.display.err(&"This video is from a blocked instance.");
            let confirm = self
                .rl
                .readline("Play it anyway ? [y/N]: ".to_string())
                .await?;
            if confirm != "y" && confirm != "Y" {
                return Ok(());
            }
        }
        let video_url = if self.config.select_quality() {
            let resolutions = video.resolutions().await?;
            let nb_resolutions = resolutions.len();
            self.display.resolutions(resolutions);
            let choice;
            loop {
                match self.rl.readline(">> ".to_string()).await?.parse::<usize>() {
                    Ok(id) if id <= nb_resolutions && id > 0 => {
                        choice = id;
                        break;
                    }
                    Ok(0) => self
                        .display
                        .message("0 is not a valid choice for a resolution"),
                    Ok(_) => self.display.message(&format!(
                        "Choice must be inferior to the number of available resolutions: {}",
                        nb_resolutions
                    )),
                    Err(_) => self
                        .display
                        .message("Enter a number to select the resolution"),
                }
            }
            if self.config.use_torrent() {
                video.torrent_url(choice - 1).await
            } else {
                video.resolution_url(choice - 1).await
            }
        } else if self.config.use_torrent() {
            video.load_resolutions().await?;
            video.torrent_url(0).await
        } else if self.config.use_raw_url() {
            video.load_resolutions().await?;
            video.resolution_url(0).await
        } else {
            video.watch_url()
        };
        self.history.add_video(video.uuid().to_string());

        Command::new(self.config.player())
            .args(self.config.player_args())
            .arg(video_url)
            .spawn()
            .map_err(Error::VideoLaunch)?
            .await
            .map_err(Error::VideoLaunch)?;
        Ok(())
    }

    /// Returns None if the error was dealt with
    fn handle_err(&mut self, err: Error) -> Option<Error> {
        match &err {
            Error::Api(ApiError::Reqwest(req_err)) => {
                if let Some(code) = req_err.status() {
                    if code.as_u16() >= 400 && code.as_u16() < 500 {
                        self.display.err(&format!(
                            "\
                            Error encountered while connecting to the desired instance: {}\n\
                            This is likely because the server you are trying to connect isn't a PeerTube instance.\
                        ",
                            code
                        ));
                        self.display.report_error(err);
                        return None;
                    } else if code.as_u16() >= 500 {
                        self.display.err(&format!(
                            "\
                            The server you are trying to connect failed to process the request: {}\n\
                            This likely isn't a bug from peertube-viewer-rs.
                        ",
                            code
                        ));
                        self.display.report_error(err);
                        return None;
                    }
                } else {
                    return Some(err);
                }
                None
            }
            _ => Some(err),
        }
    }

    pub fn run(&mut self) {
        let mut basic_rt = match runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                self.display.err(&Error::RuntimeInit(e));
                return;
            }
        };

        basic_rt
            .block_on(async {
                let local = LocalSet::new();
                local.run_until(self.main_loop()).await
            })
            .unwrap_or_else(|e| match e {
                Error::Readline(ReadlineError::Interrupted)
                | Error::Readline(ReadlineError::Eof) => (),
                err => self.display.err(&err),
            });
    }
}

struct LoopData {
    mode: Mode,
    action: Action,
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
    pub fn current_len(&self) -> usize {
        match self {
            Mode::Videos(v) => v.current().len(),
            Mode::Channels(c) => c.current().len(),
            Mode::Comments(c) => c.current().len(),
            Mode::Temp => 0,
        }
    }

    pub async fn ensure_init(&mut self) -> Result<(), Error> {
        match self {
            Mode::Videos(v) => Ok(v.ensure_init().await?),
            Mode::Channels(c) => Ok(c.ensure_init().await?),
            Mode::Comments(c) => Ok(c.ensure_init().await?),
            Mode::Temp => panic!("Bad use of temp"),
        }
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
