# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8](https://github.com/ratatui-org/crates-tui/compare/v0.1.7...v0.1.8) - 2024-03-25

### Fixed
- restore bracketed paste in tui::restore ([#37](https://github.com/ratatui-org/crates-tui/pull/37))

### Other
- *(deps)* bump toml from 0.8.10 to 0.8.12 ([#50](https://github.com/ratatui-org/crates-tui/pull/50))
- *(deps)* bump uuid from 1.7.0 to 1.8.0 ([#49](https://github.com/ratatui-org/crates-tui/pull/49))
- *(deps)* bump color-eyre from 0.6.2 to 0.6.3 ([#47](https://github.com/ratatui-org/crates-tui/pull/47))
- *(deps)* bump serde_with from 3.6.1 to 3.7.0 ([#43](https://github.com/ratatui-org/crates-tui/pull/43))
- *(deps)* bump tokio-stream from 0.1.14 to 0.1.15 ([#44](https://github.com/ratatui-org/crates-tui/pull/44))
- *(deps)* bump figment from 0.10.14 to 0.10.15 ([#45](https://github.com/ratatui-org/crates-tui/pull/45))
- *(deps)* bump clap from 4.5.2 to 4.5.3 ([#46](https://github.com/ratatui-org/crates-tui/pull/46))
- *(deps)* bump clap from 4.5.1 to 4.5.2 ([#41](https://github.com/ratatui-org/crates-tui/pull/41))
- *(deps)* bump webbrowser from 0.8.12 to 0.8.13 ([#42](https://github.com/ratatui-org/crates-tui/pull/42))
- *(deps)* bump chrono from 0.4.34 to 0.4.35 ([#40](https://github.com/ratatui-org/crates-tui/pull/40))
- *(deps)* bump strum from 0.26.1 to 0.26.2 ([#39](https://github.com/ratatui-org/crates-tui/pull/39))
- *(deps)* bump serde from 1.0.196 to 1.0.197 ([#38](https://github.com/ratatui-org/crates-tui/pull/38))
- *(deps)* bump textwrap from 0.16.0 to 0.16.1 ([#36](https://github.com/ratatui-org/crates-tui/pull/36))
- *(deps)* bump clap from 4.5.0 to 4.5.1 ([#35](https://github.com/ratatui-org/crates-tui/pull/35))
- jm/refactor tui ([#33](https://github.com/ratatui-org/crates-tui/pull/33))

## [0.1.7](https://github.com/ratatui-org/crates-tui/compare/v0.1.6...v0.1.7) - 2024-02-12

### Other
- clippy ([#32](https://github.com/ratatui-org/crates-tui/pull/32))
- simplify help.rs ([#27](https://github.com/ratatui-org/crates-tui/pull/27))
- *(deps)* bump chrono from 0.4.33 to 0.4.34 ([#30](https://github.com/ratatui-org/crates-tui/pull/30))
- *(deps)* bump ratatui from 0.26.1-alpha.1 to 0.26.1 ([#31](https://github.com/ratatui-org/crates-tui/pull/31))
- remove the specific crossterm events ([#28](https://github.com/ratatui-org/crates-tui/pull/28))

## [0.1.6](https://github.com/ratatui-org/crates-tui/compare/v0.1.5...v0.1.6) - 2024-02-11

### Added
- refactor app.rs ([#21](https://github.com/ratatui-org/crates-tui/pull/21))
- Increase spacing between top and bottom in summary

### Fixed
- version string even without git ([#24](https://github.com/ratatui-org/crates-tui/pull/24))

### Other
- Add AUR instructions ([#22](https://github.com/ratatui-org/crates-tui/pull/22))

## [0.1.5](https://github.com/ratatui-org/crates-tui/compare/v0.1.4...v0.1.5) - 2024-02-09

### Added
- Show cargo copy in demo
- Show help in demo
- Add vhs tape
- Better help menu with offset and UX for new users
- Add Action::Ignore

### Fixed
- Don't update crate info when scrolling help
- Change resolution in tape
- Missing enter in vhs tape

### Other
- Update gitignore to only ignore log files
- Update README.md with new demo
- more tweaks to the demo by scrolling help
- Tweak resolution and timing of the demo
- Increase resolution of demo
- Change demo to move up faster at the end

## [0.1.4](https://github.com/ratatui-org/crates-tui/compare/v0.1.3...v0.1.4) - 2024-02-09

### Fixed
- version string

## [0.1.3](https://github.com/ratatui-org/crates-tui/compare/v0.1.2...v0.1.3) - 2024-02-09

### Other
- Remove musl automated build

## [0.1.2](https://github.com/ratatui-org/crates-tui/compare/v0.1.1...v0.1.2) - 2024-02-09

### Other
- Add token to checkout

## [0.1.1](https://github.com/ratatui-org/crates-tui/compare/v0.1.0...v0.1.1) - 2024-02-09

### Added

- Open crates io pages from summary view
- Color theme support and configurable colors
- Better popup scroll
- Add copy cargo add command to clipboard
- Always show spinner in top right
- Add page number
- Better prompt
- Add summary screen
- Only show keywords instead of versions

### Fixed

- Popup scroll bug

### Other

- simplify popup ([#12](https://github.com/ratatui-org/crates-tui/pull/12))
- better keybinds ([#11](https://github.com/ratatui-org/crates-tui/pull/11))
- use cfg_if crate for better cfg checks ([#9](https://github.com/ratatui-org/crates-tui/pull/9))
- move events from tui to events module ([#8](https://github.com/ratatui-org/crates-tui/pull/8))
- simplify tui, events, errors ([#7](https://github.com/ratatui-org/crates-tui/pull/7))
- cleanup config.rs ([#6](https://github.com/ratatui-org/crates-tui/pull/6))
