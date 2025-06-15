# Brainrot TD

A terminal-based "Italianbrainrot" style random tower defense game inspired by "Lucky Defense."

## Overview

Brainrot TD is a roguelike tower defense game playable entirely in your terminal. Place, merge, and upgrade quirky "allies" to defend against waves of enemies. The game features random ally generation, merging mechanics, and a unique Italian meme-inspired theme.

Built with [Ratatui](https://ratatui.rs) for the TUI and event-driven architecture.

## Features

- Randomized ally spawning with unique elements and abilities
- Ally merging and leveling system
- Configurable ally/enemy stats via `config.toml`
- Animated terminal UI with avatars and effects
- Logging and debugging support

## Requirements

- Rust (edition 2024)
- A terminal emulator

## Installation

Clone the repo and build with Cargo:

```sh
git clone https://github.com/yourusername/nycu-gdc-game-jam-0th.git
cd nycu-gdc-game-jam-0th
cargo build --release
```

## Usage

Run the game:

```sh
cargo run --release
```

## Controls

- **Arrow keys**: Move cursor
- **Space**: Buy (spawn) a random ally (costs coins)
- **Enter**: Select or merge allies
- **Q / Esc / Ctrl+C**: Quit

## Game Operation

- Place allies on the grid to defend against incoming enemies.
- Select two allies to merge them (if compatible) for upgrades or new abilities.
- Each ally has unique stats and effects based on their element(s).
- Survive all enemy waves to win!

## Configuration

You can customize ally stats and types in `config.toml`:

```toml
[general]
atk = 10
range = 2
aoe_range = 0
atk_speed = 1.0
levelup_ratio = 1.5
special_value = 0

[allies.basic]
# Inherit from [general]

[allies.slow]
atk = 7
atk_speed = 0.8

[allies.AOE]
atk = 5
aoe_range = 1

[allies.Dot]
atk = 7
special_value = 3

[allies.Critical]
special_value = 2.0
```

Missing fields inherit from `[general]`. You can tweak these values for testing or balancing.

## Development

- Logging is enabled and outputs to `.data/nycu-gdc-game-jam-0th.log`
- Images for avatars should be placed in `assets/avatars/`
- See `src/game.rs` for core game logic and mechanics

## License

Copyright (c) Bogay <pojay11523@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE

