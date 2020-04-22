extern crate peertube_api;
extern crate rustyline;

use peertube_api::Instance;

use std::process::Stdio;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = rustyline::Editor::<()>::new();
    let query = match rl.readline(">> ") {
        Ok(line) => line,
        Err(_) => panic!("No input"),
    };
    let mut inst = Instance::new("https://video.ploud.fr".to_string());
    let search_results = inst.search_videos(&query).await.unwrap();
    for (id, video) in search_results.iter().enumerate() {
        println!("{}:{}-{}", id + 1, video.name(), video.duration());
    }
    let choice = match rl.readline(">> ") {
        Ok(line) => line.parse::<usize>().unwrap(),
        Err(_) => panic!("No input"),
    };
    let video = &search_results[choice - 1];
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
