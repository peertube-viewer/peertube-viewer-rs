extern crate peertube_api;
extern crate rustyline;

use peertube_api::Instance;

use std::rc::Rc;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::stream::StreamExt;
use tokio::task::{spawn_local, LocalSet};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local = LocalSet::new();
    local.run_until(async { run().await }).await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut std_lines = stdin.lines();

    stdout.write_all(b">> ").await;
    stdout.flush().await;
    let query = std_lines.next().await.unwrap()?;
    let inst = Instance::new("https://video.ploud.fr".to_string());
    let mut search_results = inst.search_videos(&query).await.unwrap();
    let mut results_rc = Vec::new();
    for (id, video) in search_results.drain(..).enumerate() {
        println!(
            "{}:{}-{}-{}",
            id + 1,
            video.name(),
            video.duration(),
            video
                .published()
                .map(|t| t.format("%a:%b:%Y").to_string())
                .unwrap_or_default()
        );
        let video_stored = Rc::new(video);
        let video_sent = video_stored.clone();
        results_rc.push(video_stored);
        spawn_local(async move {
            video_sent.description().await;
        });
    }
    stdout.write_all(b">> ").await;
    stdout.flush().await;
    let choice = std_lines.next().await.unwrap()?;

    let choice = choice.parse::<usize>().unwrap();
    let video = &results_rc[choice - 1];
    println!("Playing: {}", video.name());
    println!(
        "{}",
        video
            .description()
            .await?
            .as_ref()
            .get_or_insert(&"".to_string())
    );
    let mut video_url = "https://video.ploud.fr".to_string();
    video_url.push_str("/videos/watch/");
    video_url.push_str(video.uuid());
    Command::new("mpv").arg(video_url).spawn().unwrap().await?;
    Ok(())
}
