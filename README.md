# Spotifatius

_Spotify + status + some Latin = Spotifatius._

A simple Spotify CLI primarily made for monitoring what songs you're listening to and displaying that in your bar of choice like [waybar](https://github.com/Alexays/Waybar) or [polybar](https://github.com/polybar/polybar).


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
