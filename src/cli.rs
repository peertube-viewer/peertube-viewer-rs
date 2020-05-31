mod config;
mod display;
mod history;
mod input;

pub use config::ConfigLoadError;
use config::{Blacklist, Config, InitialInfo};
use display::Display;
use history::History;
use input::{Action, Editor};

use crate::error::Error;

use rustyline::error::ReadlineError;

use peertube_api::{Instance, PreloadableList, TrendingList, VideoSearch};

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
        let instance_domain = match initial_info.initial_query.as_ref() {
            Some(s) if s.starts_with("http://") || s.starts_with("https://") => {
                match s.split('/').nth(2) {
                    Some(domain) => {
                        let instance_temp =
                            format!("https://{}", domain.split(' ').next().expect("Unreachable"));
                        match s.split('/').nth(5) {
                            Some(uuid) => {
                                is_single_url = true;
                                initial_info.initial_query =
                                    Some(uuid.split(' ').next().expect("Unreachable").to_string());
                            }

                            None => {
                                initial_info.initial_query =
                                    s.splitn(2, ' ').nth(1).map(|s| s.to_string())
                            }
                        }
                        instance_temp
                    }
                    None => config.instance().to_string(),
                }
            }
            None | Some(_) => config.instance().to_string(),
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
        let mut mode;
        let mut query = if !self.initial_info.start_trending {
            // Check if the initital query is a video url
            let query_str = match self.initial_info.initial_query.take() {
                Some(q) => q,
                None => self.rl.readline(">> ".to_string()).await?,
            };

            if self.is_single_url {
                self.play_vid(&self.instance.single_video(&query_str).await?)
                    .await?;
                return Ok(());
            }
            if query_str == ":trending" {
                let mut trend_tmp = self.instance.trending(SEARCH_TOTAL);
                trend_tmp.next_videos().await?;

                mode = Mode::Trending(trend_tmp);
                Action::Query(query_str)
            } else {
                self.rl.add_history_entry(&query_str);
                let mut search_tmp = self.instance.search(&query_str, SEARCH_TOTAL);
                search_tmp.next_videos().await?;

                mode = Mode::Search(search_tmp);
                Action::Query(query_str)
            }
        } else {
            let mut trending_tmp = self.instance.trending(SEARCH_TOTAL);
            trending_tmp.next_videos().await?;
            mode = Mode::Trending(trending_tmp);
            Action::Query(":trending".to_string())
        };

        let mut changed_query = false;
        // Main loop
        loop {
            let video;
            if changed_query {
                match &query {
                    Action::Query(s) => {
                        if s == ":trending" {
                            let mut trending_tmp = self.instance.trending(SEARCH_TOTAL);
                            trending_tmp.next_videos().await?;
                            mode = Mode::Trending(trending_tmp);
                            query = Action::Query(":trending".to_string());
                        } else {
                            let mut search_tmp = self.instance.search(&s, SEARCH_TOTAL);
                            search_tmp.next_videos().await?;
                            self.rl.add_history_entry(&s);
                            mode = Mode::Search(search_tmp);
                        }
                    }
                    Action::Quit => break,
                    Action::Next => match &mut mode {
                        Mode::Search(search) => {
                            search.next_videos().await?;
                        }
                        Mode::Trending(trending) => {
                            trending.next_videos().await?;
                        }
                    },
                    Action::Prev => match &mut mode {
                        Mode::Search(search) => {
                            search.prev();
                        }
                        Mode::Trending(trending) => {
                            trending.prev();
                        }
                    },
                    _ => unreachable!(),
                };
            }
            match &mut mode {
                Mode::Search(search) => {
                    self.display
                        .video_list(search.current(), &self.history, &self.config);
                    self.display.mode_info(
                        "Search",
                        search.expected_total(),
                        search.offset(),
                        search.current_len(),
                    );

                    search.preload_res(self.config.select_quality() || self.config.use_raw_url());
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
                Mode::Trending(trending) => {
                    self.display
                        .video_list(trending.current(), &self.history, &self.config);
                    self.display.mode_info(
                        "Trending",
                        trending.expected_total(),
                        trending.offset(),
                        trending.current_len(),
                    );

                    trending.preload_res(self.config.select_quality() || self.config.use_raw_url());
                    let choice;
                    match self
                        .rl
                        .autoload_readline(">> ".to_string(), trending)
                        .await?
                    {
                        Action::Id(id) => choice = id,
                        res => {
                            query = res;
                            changed_query = true;
                            continue;
                        }
                    };

                    video = trending.current()[choice - 1].clone();
                }
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
    Search(VideoSearch),
    Trending(TrendingList),
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
