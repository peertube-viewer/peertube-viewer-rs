mod config;
mod display;
mod history;
mod input;

use config::Config;
pub use config::ConfigLoadError;
use display::Display;
use history::History;
use input::Editor;

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

    async fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.display.welcome(self.config.instance());

        let mut initial_query = None;
        swap(&mut initial_query, &mut self.initial_query);

        let query = match initial_query {
            Some(q) => q,
            None => match self.rl.readline(">> ".to_string()).await {
                Ok(l) => l,
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => return Ok(()),
                Err(e) => {
                    self.display.err(&format!("Unexpected input error:\n{}", e));
                    return Err(Box::new(e));
                }
            },
        };

        self.rl.add_history_entry(&query);

        let mut search_results = self.instance.search_videos(&query).await.unwrap();
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
        self.display.search_results(&results_rc, &self.history);

        let choice = self.rl.readline(">> ".to_string()).await.unwrap();
        let choice = choice.parse::<usize>().unwrap();
        let video = &results_rc[choice - 1];
        let video_url = if self.config.select_quality() {
            self.display.resolutions(video.resolutions().await?);
            let choice = self.rl.readline(">> ".to_string()).await.unwrap();
            let choice = choice.parse::<usize>().unwrap();
            video.resolution_url(choice - 1).await
        } else {
            video.watch_url()
        };
        self.display.info(video).await;
        self.history.add_video(video.uuid().clone());

        Command::new(self.config.player())
            .arg(video_url)
            .args(self.config.player_args())
            .spawn()
            .unwrap()
            .await?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut basic_rt = runtime::Builder::new()
            .enable_all()
            .basic_scheduler()
            .build()?;
        basic_rt.block_on(async {
            let local = LocalSet::new();
            local.run_until(self.main_loop()).await
        })
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
