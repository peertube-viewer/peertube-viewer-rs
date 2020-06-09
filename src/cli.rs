mod config;
mod display;
mod history;
mod input;
mod parser;
mod preloadables;

pub use config::ConfigLoadError;
use config::{Blacklist, Config, InitialInfo};
use display::Display;
use history::History;
use input::{Action, Editor};

use crate::error::Error;

use rustyline::error::ReadlineError;

use peertube_api::Instance;

use preloadable_list::PreloadableList;
use preloadables::{Channels, Videos};

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
        let mut rl = Editor::new();

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
            if config.is_blacklisted(&instance_domain[8..]).is_some() {
                let err = Error::BlacklistedInstance(instance_domain[8..].to_string());
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
        let mut mode = Mode::Temp; //Placeholder that will be changed just after anyway on the first loop run
        let mut query = match self.initial_info.take() {
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
        let mut changed_query = true;

        // Main loop
        loop {
            let video;
            if changed_query {
                match &query {
                    Action::Query(s) => {
                        if s == ":trending" {
                            let mut trending_tmp = PreloadableList::new(
                                Videos::new_trending(self.instance.clone()),
                                SEARCH_TOTAL,
                            );
                            trending_tmp.next().await?;
                            mode = Mode::Videos(trending_tmp);
                            query = Action::Query(":trending".to_string());
                        } else if let Some(q) = parser::channels(&s) {
                            let mut channels_tmp = PreloadableList::new(
                                Channels::new(self.instance.clone(), q),
                                SEARCH_TOTAL,
                            );
                            channels_tmp.next().await?;
                            mode = Mode::Channels(channels_tmp);
                            self.rl.add_history_entry(s);
                        } else if let Some(handle) = parser::chandle(s) {
                            let mut chandle_tmp = PreloadableList::new(
                                Videos::new_channel(self.instance.clone(), handle),
                                SEARCH_TOTAL,
                            );
                            chandle_tmp.next().await?;
                            mode = Mode::Videos(chandle_tmp);
                            self.rl.add_history_entry(s);
                        } else {
                            self.rl.add_history_entry(&s);
                            let mut search_tmp = PreloadableList::new(
                                Videos::new_search(self.instance.clone(), &s),
                                SEARCH_TOTAL,
                            );
                            search_tmp.next().await?;
                            mode = Mode::Videos(search_tmp);
                        }
                    }
                    Action::Quit => break,
                    Action::Next => match &mut mode {
                        Mode::Videos(search) => {
                            search.next().await?;
                        }
                        Mode::Channels(channels) => {
                            channels.next().await?;
                        }
                        Mode::Temp => unreachable!(),
                    },
                    Action::Prev => match &mut mode {
                        Mode::Videos(search) => {
                            search.prev();
                        }
                        Mode::Channels(channels) => {
                            channels.prev();
                        }
                        Mode::Temp => unreachable!(),
                    },
                    _ => unreachable!(),
                };
            }
            match &mut mode {
                Mode::Videos(search) => {
                    self.display
                        .video_list(search.current(), &self.history, &self.config);
                    self.display.mode_info(
                        search.loader().name(),
                        search.expected_total(),
                        search.offset(),
                        search.current_len(),
                    );

                    search
                        .loader_mut()
                        .preload_res(self.config.select_quality() || self.config.use_raw_url());
                    let choice;
                    match self.rl.autoload_readline(">> ".to_string(), search).await? {
                        Action::Id(id) => choice = id,
                        res => {
                            query = res;
                            changed_query = true;
                            continue;
                        }
                    };

                    video = search.current()[choice - 1].clone();
                }
                Mode::Channels(channels) => {
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
                            query = Action::Query(format!(
                                ":chandle {}",
                                channels.current()[id - 1].handle()
                            ));
                            changed_query = true;
                            continue;
                        }
                        res => {
                            query = res;
                            changed_query = true;
                            continue;
                        }
                    };
                }
                Mode::Temp => unreachable!(),
            }
            changed_query = false;
            self.play_vid(&video).await?;
        }
        Ok(())
    }

    async fn play_vid(&mut self, video: &peertube_api::Video) -> Result<(), Error> {
        // Resolution selection
        self.display.info(&video).await;
        if self.config.is_blacklisted(video.host()).is_some() {
            self.display
                .err(&"This video is from a blacklisted instance.");
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

enum Mode {
    Videos(PreloadableList<Videos>),
    Channels(PreloadableList<Channels>),
    Temp,
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
