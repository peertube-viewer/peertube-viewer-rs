//use cli_ui::Editor;
use peertube_api::Instance;

use std::env;
use std::rc::Rc;

use tokio::process::Command;
use tokio::runtime;
use tokio::task::{spawn_local, LocalSet};

#[macro_use]
extern crate clap;

mod config;
mod display;
mod history;
mod input;

use history::History;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut basic_rt = runtime::Builder::new()
        .enable_all()
        .basic_scheduler()
        .build()?;
    basic_rt.block_on(async {
        let local = LocalSet::new();
        local.run_until(async { run().await }).await
    })
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (config, initial_query) = config::Config::new();
    let mut history = History::new();

    let home = env::split_paths(&env::var("HOME").unwrap())
        .next()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut history_location = home.clone();
    history_location.push_str("/.cache/peertube-viewer-rs/history");
    history.load_file(&history_location)?;
    let mut rl = input::Editor::new();
    let display = display::Display::new();
    let inst = Instance::new(config.instance().to_string());
    display.welcome(config.instance());

    let query = match initial_query {
        Some(q) => q,
        None => rl.readline_static(">> ").await?.unwrap(),
    };

    let mut search_results = inst.search_videos(&query).await.unwrap();
    let mut results_rc = Vec::new();
    for video in search_results.drain(..) {
        let video_stored = Rc::new(video);
        let cl1 = video_stored.clone();
        #[allow(unused_must_use)]
        spawn_local(async move {
            cl1.load_description().await;
        });
        if config.select_quality() {
            let cl2 = video_stored.clone();
            #[allow(unused_must_use)]
            spawn_local(async move {
                cl2.load_resolutions().await;
            });
        }
        results_rc.push(video_stored);
    }
    display.search_results(&results_rc, &history);

    let choice = rl.readline_static(">> ").await?.unwrap();
    let choice = choice.parse::<usize>().unwrap();
    let video = &results_rc[choice - 1];
    let video_url = if config.select_quality() {
        display.resolutions(video.resolutions().await?);
        let choice = rl.readline_static(">> ").await?.unwrap();
        let choice = choice.parse::<usize>().unwrap();
        video.resolution_url(choice - 1).await
    } else {
        video.watch_url()
    };
    display.info(video).await;
    history.add_video(video.uuid().clone());

    Command::new(config.player())
        .arg(video_url)
        .args(config.player_args())
        .spawn()
        .unwrap()
        .await?;
    history.write_file(&history_location, config.max_hist_lines());
    Ok(())
}
