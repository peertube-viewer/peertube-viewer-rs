mod config;
mod display;
mod history;
mod input;

use config::Config;
pub use config::ConfigLoadError;
use display::Display;
use history::History;
use input::{Editor, Message};

use crate::error::Error;

use rustyline::error::ReadlineError;

use peertube_api::Instance;

use std::fs::create_dir;
use std::path::PathBuf;
use std::rc::Rc;

use dirs::cache_dir;
use tokio::process::Command;
use tokio::runtime;
use tokio::task::{spawn_local, JoinHandle, LocalSet};

const SEARCH_TOTAL: usize = 20;

pub struct Cli {
    config: Config,
    history: History,
    query_offset: usize,
    rl: Editor,
    cache: Option<PathBuf>,
    display: Display,
    instance: Rc<Instance>,
    initial_query: Option<String>,
    is_single_url: bool,
}

impl Cli {
    /// Loads an instance of the cli
    pub fn init() -> Result<Cli, Error> {
        let (config, mut initial_query, load_errors) = Config::new();
        let display = Display::new(config.nsfw(), config.colors());

        let mut err_iter = load_errors.into_iter();
        if let Some(err) = err_iter.next() {
            display.err(&err);
        }

        for err in err_iter {
            display.message("");
            display.err(&err);
        }

        let mut history = History::new();

        let mut cache = cache_dir();

        let mut rl = Editor::new();

        // Loads the history if available
        if let Some(cache) = cache.as_mut() {
            cache.push("peertube-viewer-rs");
            create_dir(&cache).unwrap_or(());

            let mut view_hist_file = cache.clone();
            view_hist_file.push("history");
            let mut cmd_hist_file = cache.clone();
            cmd_hist_file.push("cmd_history");

            history.load_file(&view_hist_file).unwrap_or(()); // unwrap_or to ignore the unused_must_use warnings
            rl.load_history(&cmd_hist_file).unwrap_or(()); // we don't care if the loading failed
        }

        // If the initial query is a url, connect to the corresponding instance
        let mut is_single_url = false;
        let instance_domain = match initial_query.as_ref() {
            Some(s) if s.starts_with("http://") || s.starts_with("https://") => {
                match s.split('/').nth(2) {
                    Some(domain) => {
                        let instance_temp =
                            format!("https://{}", domain.split(' ').next().expect("Unreachable"));
                        match s.split('/').nth(5) {
                            Some(uuid) => {
                                is_single_url = true;
                                initial_query =
                                    Some(uuid.split(' ').next().expect("Unreachable").to_string());
                            }

                            None => initial_query = s.splitn(2, ' ').nth(1).map(|s| s.to_string()),
                        }
                        instance_temp
                    }
                    None => config.instance().to_string(),
                }
            }
            None | Some(_) => config.instance().to_string(),
        };

        let instance = Instance::new(
            if config.is_blacklisted(&instance_domain[8..]) {
                let err = Error::BlacklistedInstance(instance_domain[8..].to_string());
                display.err(&err);
                return Err(err);
            } else {
                instance_domain
            },
            !config.nsfw().is_block(),
        );

        if !is_single_url {
            display.welcome(instance.host());
        }

        Ok(Cli {
            config,
            history,
            rl,
            query_offset: 0,
            cache,
            display,
            instance,
            initial_query,
            is_single_url,
        })
    }

    /// Main loop for he cli interface
    async fn main_loop(&mut self) -> Result<(), Error> {
        // Check if the initital query is a video url
        let mut query = match self.initial_query.take() {
            Some(q) => q,
            None => self.rl.readline(">> ".to_string()).await?,
        };

        let mut old_query = query.clone();

        let mut changed_query = true;

        let mut results_rc = Vec::new();
        let mut next_results = SearchResults::None;
        let mut prev_results = SearchResults::None;

        if self.is_single_url {
            self.play_vid(&self.instance.single_video(&query).await?)
                .await?;
            return Ok(());
        }

        // Main loop
        loop {
            let video;
            if changed_query {
                let mut should_search = true;
                changed_query = false;
                if query == ":q" {
                    break;
                } else if query == ":n" {
                    self.query_offset += SEARCH_TOTAL;
                    query = old_query.clone();

                    if let SearchResults::Loading(f) = next_results {
                        prev_results = SearchResults::Loaded(results_rc);
                        results_rc = f
                            .await
                            .unwrap()?
                            .into_iter()
                            .filter(|v| !self.config.is_blacklisted(v.host()))
                            .map(Rc::new)
                            .collect();
                        next_results = SearchResults::None;
                        should_search = false;
                    } else if let SearchResults::Loaded(res) = next_results {
                        prev_results = SearchResults::Loaded(results_rc);
                        next_results = SearchResults::None;
                        results_rc = res;
                        should_search = false;
                    }
                } else if query == ":p" {
                    let is_start = self.query_offset == 0;
                    if is_start {
                        should_search = false;
                    } else {
                        self.query_offset = self.query_offset.saturating_sub(SEARCH_TOTAL);
                        query = old_query.clone();

                        if let SearchResults::Loading(f) = prev_results {
                            next_results = SearchResults::Loaded(results_rc);
                            results_rc = f
                                .await
                                .unwrap()?
                                .into_iter()
                                .filter(|v| !self.config.is_blacklisted(v.host()))
                                .map(Rc::new)
                                .collect();
                            prev_results = SearchResults::None;
                            should_search = false;
                        } else if let SearchResults::Loaded(res) = prev_results {
                            next_results = SearchResults::Loaded(results_rc);
                            prev_results = SearchResults::None;
                            results_rc = res;
                            should_search = false;
                        }
                    }
                } else {
                    old_query = query.clone();
                    self.query_offset = 0;
                    next_results = SearchResults::None;
                    prev_results = SearchResults::None;
                    self.rl.add_history_entry(&query);
                }
                if should_search {
                    results_rc = self
                        .instance
                        .search_videos(&query, SEARCH_TOTAL, self.query_offset)
                        .await?
                        .into_iter()
                        .filter(|v| !self.config.is_blacklisted(v.host()))
                        .map(Rc::new)
                        .collect()
                }
            }
            self.display.search_results(&results_rc, &self.history);

            let choice;

            // Getting the choice among the search results
            // If the user doesn't input a number, it is a new query
            // Get what the user is typing and load
            // the corresponding video in the background
            let mut handle = self
                .rl
                .helped_readline(">> ".to_string(), Some(results_rc.len()));
            loop {
                match handle.next().await {
                    Message::Over(res) => {
                        let s = res?;
                        match s.parse::<usize>() {
                            Ok(id) if id > 0 && id <= results_rc.len() => {
                                choice = id;
                                break;
                            }
                            Err(_) | Ok(_) => {
                                query = s;
                                choice = 0;
                                changed_query = true;
                                break;
                            }
                        }
                    }

                    Message::Number(id) => {
                        let video_cloned = results_rc[id - 1].clone();
                        spawn_local(async move { video_cloned.load_description().await });
                        if self.config.select_quality() || self.config.use_raw_url() {
                            let cl2 = results_rc[id - 1].clone();
                            #[allow(unused_must_use)]
                            spawn_local(async move {
                                cl2.load_resolutions().await;
                            });
                        }
                    }
                    Message::CommandNext if next_results.is_none() => {
                        let instance_cloned = self.instance.clone();
                        let query_cloned = query.clone();
                        let offset = self.query_offset;
                        next_results = SearchResults::Loading(spawn_local(async move {
                            instance_cloned
                                .search_videos(&query_cloned, SEARCH_TOTAL, offset + SEARCH_TOTAL)
                                .await
                        }));
                    }
                    Message::CommandPrev if self.query_offset != 0 && prev_results.is_none() => {
                        let instance_cloned = self.instance.clone();
                        let query_cloned = query.clone();
                        let offset = self.query_offset;
                        prev_results = SearchResults::Loading(spawn_local(async move {
                            instance_cloned
                                .search_videos(
                                    &query_cloned,
                                    SEARCH_TOTAL,
                                    offset.saturating_sub(SEARCH_TOTAL),
                                )
                                .await
                        }));
                    }
                    _ => {}
                }
            }

            if changed_query {
                continue;
            }

            video = results_rc[choice - 1].clone();

            self.play_vid(&video).await?;
        }
        Ok(())
    }

    async fn play_vid(&mut self, video: &peertube_api::Video) -> Result<(), Error> {
        // Resolution selection
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
        self.display.info(&video).await;
        self.history.add_video(video.uuid().clone());

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

impl Drop for Cli {
    fn drop(&mut self) {
        if let Some(cache) = self.cache.as_ref() {
            let mut view_hist_file = cache.clone();
            view_hist_file.push("history");
            self.history
                .save(&view_hist_file, self.config.max_hist_lines())
                .unwrap_or(());
            let mut cmd_hist_file = cache.clone();
            cmd_hist_file.push("cmd_history");
            self.rl.save_history(&cmd_hist_file).unwrap_or(());
        }
    }
}

enum SearchResults {
    None,
    Loading(JoinHandle<Result<Vec<peertube_api::Video>, peertube_api::error::Error>>),
    Loaded(Vec<Rc<peertube_api::Video>>),
}

impl SearchResults {
    pub fn is_none(&self) -> bool {
        if let SearchResults::None = self {
            true
        } else {
            false
        }
    }
}
