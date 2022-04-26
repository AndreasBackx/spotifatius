# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Fixed hardcoded location for rspotify cache.

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
