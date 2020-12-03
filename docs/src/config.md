Configuration


The config file for **peertube-viewer-rs** can be placed

| Platform | Location                                                                                            |
| -------  | -------------------------------------                                                               |
| Linux    | `$XDG_CONFIG_HOME/peertube-viewer-rs/config.toml` or `$HOME`/.config/peertube-viewer-rs/config.toml |
| macOS    | `$HOME`/Library/Preferences/peertube-viewer-rs/config.toml                                          |
| Windows  | `{FOLDERID_RoamingAppData}\peertube-viewer-rs\config.toml`                                          |

Syntax
------

The config file is written in [TOML](https://github.com/toml-lang/toml)
An example of config file with all configuration option showed is available [lower](#full-configuration) or can be obtained with `peertube-viewer-rs --print-full-example-config`

Format
------

- [[`general`]](#general)
    - [`nsfw`](#nsfw) behavior for nsfw content
    - [`colors`](#colors) coloring of the output
    - [`select-quality`](#select-quality) regarding quality selection
    - [`edit-mode`](#edit-mode) to set the input mode to vi style keybindings
    - [`browser`](#browser) set the browser to be used
    - [`user-agent`](#user-agent) set the user agent to be used

- [[`player`]](#player)
    - [`command`](#command) video player command
    - [`args`](#args) video player arguments
    - [`use-raw-urls`](#use-raw-urls) see [command line arguments](cli/args.md)
    - [`prefer-hls`](#prefer-hls) prefer [hls streams](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) to static files

- [[`instances`]](#instances)
    - [`main`](#main) main instance to search
    - [`blocklist`](#blocklist-allowlist) instance blocklist
    - [`allowlist`](#blocklist-allowlist) instance blocklist

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

#### edit-mode
Set the editing mode.
This allows the use of either vi or emacs keybinds. If you don't know what it is you don't need to worry about this option.

- `"emacs"`: the default
- `"vi"`

#### browser
Set the browser to use when opening items with the `:browser` command.
If this variable isn't set, the `BROWSER` environment variable is used.
If the environment variable isn't available, Firefox is the default

#### user-agent
Set the [`User-Agent`](https://en.wikipedia.org/wiki/User_agent) string to be used when making http requests.
If can be a string, which will be used as is, or a boolean (false means remove the `User-Agent` header).

Defaults to: `peertube-viewer-rs/<version>`


Example:
```toml
[general]
nsfw = "block"
colors = "enable"
select-quality = true
edit-mode = "vi"
browser = "qutebrowser"
user-agent = false
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

#### prefer-hls
Peertube has support for live streaming. Some regular videos are also made available through [hls](https://en.wikipedia.org/wiki/HTTP_Live_Streaming).

This settings tells `peertube-viewer` which one it should prefer if both are available.

- `false`
- `true`: the default

*Note: this option doesn't do anything if either `select-quality` is `true` or if `use-raw-urls` is `false`*

Example:
```toml
[player]
command = "vlc"
args = ["--no-audio", "-f"]
use-raw-urls = true
prefer-hls = false
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

#### blocklist-allowlist

- `blocklist`: an array of strings of instances to be blocked
- `allowlist`: an array of strings of instances to be allowed

Both can't be present at the same time.
When blocklist is present, no video from the instances listed will be shown.
When allowlist is present, only videos from the instances listed will be shown.

---

Full configuration
-------------------

This is an example of all configuration options that can be set

``` TOML
{{#include ../../src/cli/full_config.toml}}
```
