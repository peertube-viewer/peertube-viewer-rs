mod config;
mod display;
mod history;
mod input;

use config::Config;
pub use config::ConfigLoadError;
use display::Display;
use history::History;
use input::Editor;

use crate::error;

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
    rl: Editor,
    cache: Option<PathBuf>,
    display: Display,
    instance: Rc<Instance>,
    initial_query: Option<String>,
}

impl Cli {
    pub fn init() -> Cli {
        let (config, initial_query, load_error) = Config::new();
        let display = Display::new();

        if let Some(err) = load_error {
            display.err(&err);
        }

        let mut history = History::new();

        let mut cache = cache_dir();

        let mut rl = Editor::new();

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
        let instance = Instance::new(config.instance().to_string());

        Cli {
            config,
            history,
            rl,
            cache,
            display,
            instance,
            initial_query,
        }
    }

    async fn main_loop(&mut self) -> Result<(), error::Error> {
        self.display.welcome(self.config.instance());

        let mut initial_query = None;
        swap(&mut initial_query, &mut self.initial_query);

        let mut query = match initial_query {
            Some(q) => q,
            None => self.rl.readline(">> ".to_string()).await?,
        };

        let mut changed_query = true;

        let mut results_rc = Vec::new();
        loop {
            if changed_query {
                if query == ":q" {
                    continue;
                }
                self.rl.add_history_entry(&query);
                results_rc = self.search(&query).await?;
            }
            self.display.search_results(&results_rc, &self.history);

            let mut choice = None;
            loop {
                let s = self.rl.readline(">> ".to_string()).await?;
                match s.parse::<usize>() {
                    Ok(id) if id < results_rc.len() => {
                        choice = Some(id);
                        break;
                    }
                    Ok(_) => continue,
                    Err(_) => {
                        query = s;
                        choice = None;
                        changed_query = true
                    }
                }
            }

            let choice = match choice {
                Some(id) => id,
                None => continue,
            };

            let video = &results_rc[choice - 1];
            let video_url = if self.config.select_quality() {
                let resolutions = video.resolutions().await?;
                let nb_resolutions = resolutions.len();
                self.display.resolutions(resolutions);
                let mut choice = 0;
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
                video.resolutions().await?;
                video.torrent_url(choice - 1).await
            } else {
                video.watch_url()
            };
            self.display.info(video).await;
            self.history.add_video(video.uuid().clone());

            changed_query = false;
            Command::new(self.config.player())
                .args(self.config.player_args())
                .arg(video_url)
                .spawn()
                .map_err(error::Error::VideoLaunch)?
                .await
                .map_err(error::Error::VideoLaunch)?;
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
                self.display.err(&error::Error::RuntimeInit(e));
                return;
            }
        };

        basic_rt
            .block_on(async {
                let local = LocalSet::new();
                local.run_until(self.main_loop()).await
            })
            .map_err(|e| match e {
                error::Error::Readline(ReadlineError::Interrupted)
                | error::Error::Readline(ReadlineError::Eof) => return,
                err @ _ => {
                    self.display.err(&err);
                    return;
                }
            });
    }

    async fn search(&mut self, query: &str) -> Result<Vec<Rc<peertube_api::Video>>, error::Error> {
        let mut search_results = self.instance.search_videos(&query).await?;
        let mut results_rc = Vec::new();
        for video in search_results.drain(..) {
            let video_stored = Rc::new(video);
            let cl1 = video_stored.clone();
            #[allow(unused_must_use)]
            spawn_local(async move {
                cl1.load_description().await;
            });
            if self.config.select_quality() {
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
