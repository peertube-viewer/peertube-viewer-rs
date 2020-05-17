peertube-viewer-rs
===

a command line program to view browse peertube, inspired by the youtube-viewer utility



Compiling
---

Compiling requires a rust toolchain to be installed

```bash
cargo build --locked --release
```

The executable can then be found in `target/release/peertube-viewer-rs`, while autocompletion for a few shells will be found in in the `completions` directory.



Usage
---
Here is an example of basic usage :

The user wants to use the peertube instance video.ploud.fr. They search for videos about mastodon then they select the first search result. Additional information about the video is displayed and the video is launched. If no player has been chosen as argument and no player is selected in the config file, it will default to `mpv` to play the video. When the video is ended, the user can search other videos, play another search result or quit with :q.
![Screenshot of basic usage](docs/src/screenshot.png?raw=true "Exemple usage")

To see all available options see:
```bash
peertube-viewer-rs -h
```
or read the manpage

Contributing
===

If you have a feature idea and want to implement it you are welcome.
If you want to contribute, do not hesitate to take a look at the feature that will be added in [TODO.md](TODO.md)
The master branch is the latest released version, while the development happens on the dev branch.

