mod config;
mod display;
mod history;
mod input;

use config::Config;
pub use config::ConfigLoadError;
use display::Display;
use history::History;
use input::Editor;

use crate::error::Error;

use rustyline::error::ReadlineError;

use peertube_api::Instance;

use std::fs::create_dir;
use std::mem::swap;
use std::path::PathBuf;
use std::rc::Rc;

use dirs::cache_dir;
use tokio::process::Command;
use tokio::runtime;
use tokio::task::{spawn_local, LocalSet};

pub struct Cli {
    config: Config,
    history: History,
    query_offset: usize,
    rl: Editor,
    cache: Option<PathBuf>,
    display: Display,
    instance: Rc<Instance>,
    initial_query: Option<String>,
}

impl Cli {
    /// Loads an instance of the cli
    pub fn init() -> Result<Cli, Error> {
        let (config, initial_query, load_error) = Config::new();
        let display = Display::new();

        if let Some(err) = load_error {
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
        let instance_domain = match initial_query.as_ref() {
            Some(s) if s.starts_with("http://") || s.starts_with("https://") => {
                match s.split('/').nth(2) {
                    Some(s) => format!("https://{}", s),
                    None => config.instance().to_string(),
                }
            }
            None | Some(_) => config.instance().to_string(),
        };

        let instance = Instance::new(if config.is_blacklisted(&instance_domain[8..]) {
            let err = Error::BlacklistedInstance(instance_domain[8..].to_string());
            display.err(&err);
            return Err(err);
        } else {
            instance_domain
        });

        Ok(Cli {
            config,
            history,
            rl,
            query_offset: 0,
            cache,
            display,
            instance,
            initial_query,
        })
    }

    /// Main loop for he cli interface
    async fn main_loop(&mut self) -> Result<(), Error> {
        self.display.welcome(self.instance.host());

        let mut initial_query = None;
        swap(&mut initial_query, &mut self.initial_query);

        let is_url = initial_query
            .as_ref()
            .map(|s| s.starts_with("http://") || s.starts_with("https://"))
            == Some(true);

        // Check if the initital query is a video url
        let (mut query, is_single_url) = match initial_query {
            Some(v) if is_url => match v.split(' ').nth(1) {
                Some(q) => (q.to_string(), false),
                None => match v.split('/').nth(5) {
                    Some(uuid) => (uuid.to_string(), true),
                    None => (self.rl.readline(">> ".to_string()).await?, false),
                },
            },
            Some(q) => (q, false),
            None => (self.rl.readline(">> ".to_string()).await?, false),
        };

        let mut old_query = query.clone();

        let mut changed_query = true;

        let mut results_rc = Vec::new();

        // Main loop
        loop {
            let video;
            if !is_single_url {
                if changed_query {
                    changed_query = false;
                    if query == ":q" {
                        break;
                    } else if query == ":n" {
                        self.query_offset += 20;
                        query = old_query.clone();
                    } else if query == ":p" {
                        self.query_offset = if self.query_offset < 20 {
                            0
                        } else {
                            self.query_offset - 20
                        };
                        query = old_query.clone();
                    } else {
                        old_query = query.clone();
                        self.query_offset = 0;
                        self.rl.add_history_entry(&query);
                    }
                    results_rc = self.search(&query).await?;
                }
                self.display.search_results(&results_rc, &self.history);

                let choice;

                // Getting the choice among the search results
                // If the user doesn't input a number, it is a new query
                loop {
                    let s = self.rl.readline(">> ".to_string()).await?;
                    match s.parse::<usize>() {
                        Ok(id) if id <= results_rc.len() => {
                            choice = id;
                            break;
                        }
                        Ok(_) => continue,
                        Err(_) => {
                            query = s;
                            choice = 0;
                            changed_query = true;
                            break;
                        }
                    }
                }

                if changed_query {
                    continue;
                }

                video = results_rc[choice - 1].clone();
            } else {
                video = Rc::new(self.instance.single_video(&query).await?)
            }

            // Resolution selection
            let video_url = if self.config.select_quality() {
                let resolutions = video.resolutions().await?;
                let nb_resolutions = resolutions.len();
                self.display.resolutions(resolutions);
                let choice;
                loop {
                    match self.rl.readline(">> ".to_string()).await?.parse::<usize>() {
                        Ok(id) if id < nb_resolutions => {
                            choice = id;
                            break;
                        }
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

            changed_query = false;
            Command::new(self.config.player())
                .args(self.config.player_args())
                .arg(video_url)
                .spawn()
                .map_err(Error::VideoLaunch)?
                .await
                .map_err(Error::VideoLaunch)?;

            if is_single_url {
                break;
            }
        }
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

    /// Performs a search and launches asynchronous loading of additionnal video info
    async fn search(&mut self, query: &str) -> Result<Vec<Rc<peertube_api::Video>>, Error> {
        let mut search_results = self
            .instance
            .search_videos(&query, 20, self.query_offset)
            .await?;
        let mut results_rc = Vec::new();
        for video in search_results
            .drain(..)
            .filter(|v| !self.config.is_blacklisted(v.host()))
        {
            let video_stored = Rc::new(video);
            let cl1 = video_stored.clone();
            #[allow(unused_must_use)]
            spawn_local(async move {
                cl1.load_description().await;
            });
            if self.config.select_quality() || self.config.use_raw_url() {
                let cl2 = video_stored.clone();
                #[allow(unused_must_use)]
                spawn_local(async move {
                    cl2.load_resolutions().await;
                });
            }
            results_rc.push(video_stored);
        }

        Ok(results_rc)
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
