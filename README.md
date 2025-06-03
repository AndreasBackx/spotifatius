# Spotifatius

_Spotify + status + some Latin = Spotifatius._

A simple Spotify CLI primarily made for monitoring what songs you're listening to and displaying that in your bar of choice like [waybar](https://github.com/Alexays/Waybar) or [polybar](https://github.com/polybar/polybar).

## Usage

First make sure the `RSPOTIFY_CLIENT_ID` and `RSPOTIFY_CLIENT_SECRET` environment
variables are available in your shell environment. They need to contain the
Spotify developer Client ID and secret. Follow [Spotify's Developer Documentation](https://developer.spotify.com/documentation/web-api/concepts/apps)
on how to set that up, make sure the redirect URI is set to `https://127.0.0.1:8000`.
Once done, run `spotifatius monitor` once to setup Spotify access tokens:


```shell
$ spotifatius monitor
Opened https://accounts.spotify.com/authorize?[...] in your browser.
Please enter the URL you were redirected to:
[INSERT URL HERE]
{"text":"Twenty One Pilots - Bounce Man","tooltip":"Scaled And Icy","class":["playing"]}
```

From then on, this step will no longer be required. You can then add
`spotifatius monitor` to your favourite bar, see [Bar Integration](#bar-integration).
The first invocation of this command will start a server that is required by
other commands. If there is already a server running, it will mirror the output
from said server, this is useful if you want the monitoring to show on
multiple displays.

To toggle the liked state anywhere, use `toggle-liked`.

```shell
$ spotifatius toggle-liked
Added to library!
```

That will update the monitoring server/client:

```shell
{"text":"Added to library!","class":["added"]}
{"text":"Twenty One Pilots + Bounce Man","tooltip":"Scaled And Icy","class":["liked","playing"]}
```

Doing that again will remove the liked state:

```shell
$ spotifatius toggle-liked
Removed from library!
```

```shell
{"text":"Removed from library!","class":["removed"]}
{"text":"Twenty One Pilots - Bounce Man","tooltip":"Scaled And Icy","class":["playing"]}
```

By default, liked songs also have a `+` instead of a `-` between the artist and song title.

## Bar Integration

Here are some configuration examples for the supported bars.

### Waybar

Add the following to make it available as a module:

```json
"custom/spotify": {
    "format": "{}",
    "return-type": "json",
    "on-click-right": "spotifatius toggle-liked",
    "exec": "spotifatius monitor"
}
```

The following classes are supported:
* `playing`: the current song is playing.
* `paused`: the current song is paused.
* `stopped`: the current song is stopped.
* `liked`: the current song is in your liked songs.
* `added`: there's a message being displayed saying the song was just added to your liked songs.
* `removed`: there's a message being displayed saying the song was just removed to your liked songs.

### Polybar

```ini
[module/spotify]
type = custom/script
exec = spotifatius monitor --output-type polybar
tail = true
click-right = spotifatius toggle-liked
```

## config.toml options

### format

🗒 **Note:** These formatting options are not yet final and might change in upcoming versions. They won't be considered breaking changes as a result, though a minor version bump will be done if this changes. [More info.](https://github.com/AndreasBackx/spotifatius/pull/8)

The JSON output can be formatted using the `format` var in config.toml:

Example 1:
```toml
format = "{status} {title} {separator} {artist}" # {"text":"  Bounce Man + Twenty One Pilots","tooltip":"Scaled And Icy","class":["liked", "playing"]}
```

Example 2:
```toml
format = "{artist} -- {title}" # {"text":"Twenty One Pilots -- Bounce Man","tooltip":"Scaled And Icy","class":["liked", "playing"]}
```

Default format is `{artist} {separator} {title}`

Available options:
| Name        | Function |
| ---         | --- |
| {status}    | Unicode symbol for playing/paused track |
| {title}     | Current track title |
| {artist}    | Current track artist |
| {separator} | + if current track is a liked song, - if not |


### polybar

Polybar maps the classes from the [waybar](#waybar) output to colors that you can define in your config file `~/.config/spotifatius/config.toml`:

```toml
[polybar]
[polybar.colors]
# added = ""
# liked = ""
paused = "#6E6E6E"
playing = "#CECECE"
# removed = ""
```

_By default there are no colors set for polybar._

Some example output:

```shell
$ spotifatius monitor --output-type polybar
# Output for playing unliked song.
%{F#CECECE}Twenty One Pilots - Bounce Man%{F-}
# Output for playing liked song.
%{F#CECECE}Twenty One Pilots + Bounce Man%{F-}
# Output for paused liked song.
%{F#6E6E6E}Twenty One Pilots + Bounce Man%{F-}
```

## Server/Client via gRPC

Spotifatius' monitor command will be default because a gRPC server that is streaming monitor updates, see [proto/service.proto](proto/service.proto). If a monitor instance detects the port is already used by another monitor instance, it will start listening over gRPC so all instances are in sync. As of writing, closing the server instance will also close the client.

## Installation

If you would like spotifatius to be available on your distro's package manager, feel free to make an issue if you have some time to help.

### Arch User Repository (AUR)

```zsh
paru -S spotifatius
```

### Cargo (crates.io)

```zsh
cargo install spotifatius --locked
```

### Manually

```zsh
git clone git@github.com:AndreasBackx/spotifatius.git
cd spotifatius
cargo install --path . --locked
```

## Logging

Pass `RUST_LOG` with either `trace`, `debug`, `info`, `warn`, or `error` to set the logging level, default is `error`. See [tracing-subcriber documentation for more info](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables).
