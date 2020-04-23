//use cli_ui::Editor;
use peertube_api::Instance;
use rustyline::Editor;

use std::rc::Rc;
use std::sync::{mpsc as sync_mpsc, Arc, Mutex};

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::runtime;
use tokio::stream::StreamExt;
use tokio::sync::mpsc as async_mpsc;
use tokio::task::{spawn_blocking, spawn_local, LocalSet};

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
    });
    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = input::Editor::new();
    let display = display::Display::new();

    let query = rl.readline(">> ").await?.unwrap();

    let inst = Instance::new("https://video.ploud.fr".to_string());
    let mut search_results = inst.search_videos(&query).await.unwrap();
    let mut results_rc = Vec::new();
    for (id, video) in search_results.drain(..).enumerate() {
        let video_stored = Rc::new(video);
        let video_sent = video_stored.clone();
        results_rc.push(video_stored);
        spawn_local(async move {
            video_sent.description().await;
        });
    }
    display.search_results(&results_rc);

    let choice = rl.readline(">> ").await?.unwrap();

    let choice = choice.parse::<usize>().unwrap();
    let video = &results_rc[choice - 1];
    display.info(video).await;

    let mut video_url = "https://video.ploud.fr".to_string();
    video_url.push_str("/videos/watch/");
    video_url.push_str(video.uuid());
    Command::new("mpv").arg(video_url).spawn().unwrap().await?;
    Ok(())
}
