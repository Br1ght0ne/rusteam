<h1 align="center">rusteam</h1>
<p>
  <img alt="Version" src="https://img.shields.io/crates/v/rusteam" />
  <img alt="Downloads" src="https://img.shields.io/crates/d/rusteam" />
  <img alt="License" src="https://img.shields.io/github/license/filalex77/rusteam" />
  <a href="https://docs.rs/rusteam">
    <img alt="Documentation" src="https://img.shields.io/badge/documentation-yes-brightgreen.svg" target="_blank" />
  </a>
</p>

> Manage your games on the terminal

`rusteam` is a little Rust CLI utility to help geeks like me manage their
favourite games with ease.

## Features

- list and filter games
- run games with automatic launcher and platform detection
- cross-platform configuration
- shell completion generator

## Roadmap

- install games from downloaded files
- support more launchers, search methods, etc.

## Install

```sh
cargo install rusteam
```

Then, generate a config file:

```sh
rusteam config init
```

This places a config file in `~/.config/rusteam/config.toml`, which you can
start editing.

## Usage

```sh
# Get the list of possible subcommands
rusteam help

# Get completions for your shell. zsh in my case
rusteam completion zsh > /some/dir/on/your/fpath

# List all your games, sorted alphabetically
rusteam ls

# Find games with names containing a pattern
rusteam ls rpg

# Run a game. For example, Slay the Spire
rusteam play spire
```

## Configuration reference

| `games_root` | Where all your games are located. Default: "~/Games" |

## Author

**Oleksii Filonenko**

- Github: [@filalex77](https://github.com/filalex77)

# Contributing

Contributions, issues and feature requests are welcome!

Feel free to check [open issues](https://github.com/filalex77/rusteam/issues).
