//use cli_ui::Editor;
use peertube_api::Instance;

use std::rc::Rc;

use tokio::process::Command;
use tokio::runtime;
use tokio::task::{spawn_local, LocalSet};

#[macro_use]
extern crate clap;

mod config;
mod display;
mod input;

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
    let config = config::Config::new();
    let mut rl = input::Editor::new();
    let display = display::Display::new();
    let inst = Instance::new(config.instance().to_string());
    display.welcome(config.instance());

    let query = rl.readline_static(">> ").await?.unwrap();

    let mut search_results = inst.search_videos(&query).await.unwrap();
    let mut results_rc = Vec::new();
    for video in search_results.drain(..) {
        let video_stored = Rc::new(video);
        let video_sent = video_stored.clone();
        results_rc.push(video_stored);
        #[allow(unused_must_use)]
        spawn_local(async move {
            video_sent.description().await;
        });
    }
    display.search_results(&results_rc);

    let choice = rl.readline_static(">> ").await?.unwrap();

    let choice = choice.parse::<usize>().unwrap();
    let video = &results_rc[choice - 1];
    display.info(video).await;

    let mut video_url = "https://video.ploud.fr".to_string();
    video_url.push_str("/videos/watch/");
    video_url.push_str(video.uuid());
    Command::new(config.player())
        .arg(video_url)
        .arg(config.player_args())
        .spawn()
        .unwrap()
        .await?;
    Ok(())
}
