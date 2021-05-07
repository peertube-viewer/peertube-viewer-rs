use clap::{App, AppSettings, Arg};

pub fn gen_app() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author("Sosthène Guédon <dev@peertube-viewer.com>")
    .about("PeerTube CLI client")
    .setting(AppSettings::StrictUtf8)
    .args(
        &[
            Arg::with_name("use-raw-urls")
                .long("use-raw-urls")
                .help("the raw url to the video will be passed to the player instead of the url to web interface to watch it. It may be necessary for players without native support for peertube such as vlc. Some players (ex : mpv) may be able to show the video title in their interface if this option isn't used"),
            Arg::with_name("print-default-config")
                .long("print-default-config")
                .conflicts_with("print-full-config")
                .help("print the default confing to stdout and exit"),
            Arg::with_name("print-full-config")
                .long("print-full-example-config")
                .help("print an example of all possible config options and exit"),
            Arg::with_name("select-quality")
                .short("s")
                .long("select-quality")
                .help("When playing a video with this option, the user will be prompted to chose the video quality\n Note: this implies --use-raw-urls"),
            Arg::with_name("local")
                .long("local")
                .help("Only browse video hosted on the instance you are connected to"),
            Arg::with_name("torrent")
                .long("use-torrent")
                .conflicts_with("use-raw-urls")
                .help("will download the video via the torrent downloader instead of playing it"),
            Arg::with_name("trending")
                .short("t")
                .long("trending")
                .conflicts_with("chandle")
                .help("will start browsing trending videos"),
            Arg::with_name("channels")
                .long("channels")
                .takes_value(true)
                .multiple(true)
                .conflicts_with("trending")
                .help("will start searching video channels"),
            Arg::with_name("chandle")
                .long("chandle")
                .conflicts_with("channels")
                .takes_value(true)
                .help("start browsing the videos of a channel with its handle (ex: name@instance.com)"),
            Arg::with_name("player-args")
                .long("player-args")
                .takes_value(true)
                .multiple(true)
                .help("arguments to be passed to the player"),
            Arg::with_name("player")
                .short("p")
                .long("player")
                .takes_value(true)
                .help("player to play the videos with"),
            Arg::with_name("torrent-downloader")
                .long("torrent-downloader")
                .takes_value(true)
                .help("choose the torrent software to download the videos with"),
            Arg::with_name("torrent-downloader-arguments")
                .long("torrent-downloader-args")
                .takes_value(true)
                .multiple(true)
                .help("arguments to be passed to the torrent downloader"),
            Arg::with_name("instance")
                .short("i")
                .long("instance")
                .takes_value(true)
                .help("instance to be browsed"),
            Arg::with_name("search-engine")
                .long("search-engine")
                .takes_value(true)
                .conflicts_with("instance")
                .help("use a search engine (like sepiasearch)"),
            Arg::with_name("config-file")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("Sets a custom config file"),
            Arg::with_name("let-nsfw")
                .long("let-nsfw")
                .conflicts_with("tag-nsfw")
                .help("Don't tag nsfw results"),
            Arg::with_name("tag-nsfw")
                .long("tag-nsfw")
                .conflicts_with("block-nsfw")
                .help("Tag nsfw results. This is the default behavior. This flag is only useful to override the config file"),
            Arg::with_name("block-nsfw")
                .long("block-nsfw ")
                .conflicts_with("let-nsfw")
                .help("Block nsfw search results"),
            Arg::with_name("no-color")
                .long("no-color")
                .help("remove coloring of output"),
            Arg::with_name("color")
                .long("color")
                .help("force coloring of output if it is disabled in the config file"),
            Arg::with_name("initial-query")
                .takes_value(true)
                .multiple(true)
                .help("initial query to be searched.\nIf it is a url, it will try to play it as a video"),
        ]
    )
}
