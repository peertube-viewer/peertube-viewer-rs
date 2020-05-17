Configuration


The config file for **peertube-viewer-rs** can be placed

| Platform | Location                                                                                            |
| -------  | -------------------------------------                                                               |
| Linux    | `$XDG_CONFIG_HOME/peertube-viewer-rs/config.toml` or `$HOME`/.config/peertube-viewer-rs/config.toml |
| macOS    | `$HOME`/Library/Preferences/peertube-viewer-rs/config.toml                                          |
| Windows  | `{FOLDERID_RoamingAppData}/peertube-viewer-rs/config.toml`                                          |

Syntax
------

The config file is written in [TOML](https://github.com/toml-lang/toml)
An example of config file with all configuration option showed can be obtained with `peertube-viewer-rs --print-full-example-config`

Format
------

- [[`general`]](#general)
    - [`nsfw`](#nsfw) behavior for nsfw content
    - [`colors`](#colors) coloring of the output
    - [`select-quality`](#select-quality) regarding quality selection

- [[`player`]](#player)
    - [`command`](#command) video player command
    - [`args`](#args) video player arguments
    - [`use-raw-urls`](#use-raw-urls) see [command line arguments](cli/args.md)

- [[`instances`]](#instances)
    - [`main`](#main) main instance to search
    - [`blacklist`](#blacklist-whitelist) instance blacklist
    - [`whitelist`](#blacklist-whitelist) instance whitelist

- [[`torrent`]](#torrent)
    - [`command`](#command) torrent downloader command
    - [`args`](#args) torrent downloader arguments


### General
Configuration for **peertube-viewer-rs** general behavior

#### nsfw
Set the behavior for nsfw content. It can be:

- `"block"`: block all nsfw content
- `"tag"`: the default, add a red nsfw tag next to the video
- `"let"`: treat nsfw content the same

This relies on the videos being properly tagged by the instance.


#### colors
Set whether output should be colored options:

- `"enable"`: the default
- `"disable"`



#### select-quality
Set whether the `--select-quality` flag is enabled by default

- `true`
- `false`: the default


Example:
```toml
[general]
nsfw = "block"
colors = "enable"
select-quality = true
```
### Player
Configuration for the player

The videos are played with the command: `player <player-args> <url>`.

#### command
Sets the command for the player, it is expected to be a string
#### args
Sets the arguments for the player, it is expected to be an array of strings
#### use-raw-urls
Set whether the `--use-raw-urls` flag is enabled by default

- `true`
- `false`: the default

Example:
```toml
[player]
command = "vlc"
args = ["--no-audio", "-f"]
use-raw-urls = true
```

### Torrent
Sets the command for the torrent downloader, it is expected to be a string
#### args
Sets the arguments for the torrent downloader, it is expected to be an array of strings

Example:
```toml
[player]
command = "transmission-remote"
args = ["-a"]
```

### instances

Instance settings

#### main
The main instance to be connected to, it is expected to be a string.

#### blacklist-whitelist

- `blacklist`: an array of strings of instances to be blacklisted
- `whitelist`: an array of strings of instances to be whitelisted

Both can't be present at the same time.
When blacklist is present, no video from the instances listed will be shown.
When whitelist is present, only videos from the instances listed will be shown.


