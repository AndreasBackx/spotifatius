# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2022-04-29
### Changed
- Fixed polybar formatting.

## [0.2.0] - 2022-04-27
### Added
- Added `added` and `removed` classes for when adding and removing songs from liked songs.

### Changed
- Fixed hardcoded location for rspotify cache.
- Changed naming from `saved` to `liked` to avoid confusion with the fact that something can be saved/added to your liked songs. Liked songs is not what the API uses, but it's what the GUI uses. This is reflected in both the code, the now `toggle-liked` command, and class output.

## [0.1.2] - 2022-04-25
### Added
- Added config, by default and optional at `~/.config/spotifatius/config.toml`. Can be changed by passing `--config` to `monitor`.
- Monitoring supports coloured formatting for polybar with config.

### Changed
- When toggling the saved state, that is correctly reflected in the output.

## [0.1.1] - 2022-04-24
### Changed
- When connecting a monitoring client, immediately get a response.
- Fixed `toggle-saved` not working due to not formatting address.

## [0.1.0] - 2022-04-24
### Added
- Initial release.
