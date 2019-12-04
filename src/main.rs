use anyhow::{Context, Result};
use rusteam::game::Game;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not open config: {0}")]
    OpenConfig(#[source] std::io::Error),

    #[error("could not parse config: {0}")]
    ParseConfig(#[source] toml::de::Error),

    #[error("could not serialize config: {0}")]
    SerializeConfig(#[from] toml::ser::Error),

    #[error("failed to create config directory: {0}")]
    CreateConfigDir(#[source] std::io::Error),

    #[error("could not write config: {0}")]
    WriteConfig(#[source] std::io::Error),

    #[error("could not write to stdout: {0}")]
    WriteStdout(#[source] std::io::Error),

    #[error(transparent)]
    LibraryError(#[from] rusteam::Error),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default = "GamesRoot::default")]
    games_root: GamesRoot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GamesRoot(PathBuf);

impl Default for GamesRoot {
    fn default() -> Self {
        let home_dir = directories::BaseDirs::new()
            .map(|base_dirs| base_dirs.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/root"));
        Self(home_dir.join("Games"))
    }
}

mod cli {
    use std::path::PathBuf;
    use structopt::StructOpt;

    #[derive(StructOpt)]
    pub struct CLI {
        #[structopt(subcommand)]
        pub cmd: Command,
    }

    #[derive(StructOpt)]
    pub enum Command {
        #[structopt(name = "list", visible_alias = "ls", about = "List your games")]
        List {
            #[structopt(help = "substrings of game name")]
            patterns: Vec<String>,
        },
        #[structopt(
            name = "play",
            visible_aliases = &["run", "launch"],
            about = "Run a game"
        )]
        Play {
            #[structopt(help = "substrings of game name", required = true)]
            patterns: Vec<String>,
        },
        #[structopt(name = "install", about = "Install a game")]
        Install {
            #[structopt(help = "the path to game files")]
            path: PathBuf,
        },
        #[structopt(name = "completions", about = "Install shell completion files")]
        Completion(rusteam::Shell),
        #[structopt(name = "config", about = "Manage your configuration")]
        Config(Config),
    }

    #[derive(StructOpt)]
    pub enum Config {
        #[structopt(name = "init", about = "Initialize a default configuration file")]
        Init,
        #[structopt(name = "show", about = "Display your current configuration")]
        Show,
    }
}

impl Config {
    const CONFIG_FILENAME: &'static str = "config.toml";

    fn directory() -> Result<PathBuf> {
        directories::ProjectDirs::from("space", "brightone", env!("CARGO_PKG_NAME"))
            .map(|project_dirs| project_dirs.config_dir().to_path_buf())
            .context("no .config directory")
    }

    fn path() -> Result<PathBuf> {
        Ok(Self::directory()?.join(Self::CONFIG_FILENAME))
    }

    fn fetch() -> Result<Self> {
        let path = Self::path()?;
        let file = &fs::read(&path).map_err(Error::OpenConfig)?;
        let config = String::from_utf8_lossy(file);
        let toml = toml::from_str(&config).map_err(Error::ParseConfig)?;
        Ok(toml)
    }

    fn show(&self) -> Result<()> {
        let toml = toml::to_vec(&self).context("could not serialize {self:?} to TOML")?;
        std::io::stdout()
            .write_all(&toml)
            .map_err(Error::WriteStdout)?;
        Ok(())
    }

    fn init() -> Result<()> {
        Self::ensure_directory()?;
        Self::default().write()
    }

    /// Ensures the package config directory is present.
    /// **Does** create `~/.config` if needed.
    fn ensure_directory() -> Result<()> {
        let path = Self::directory()?;
        fs::create_dir_all(&path).map_err(Error::CreateConfigDir)?;
        Ok(())
    }

    fn write(&self) -> Result<()> {
        let toml = toml::to_string_pretty(&self).map_err(Error::SerializeConfig)?;
        let path = Self::path()?;
        fs::write(&path, &toml).map_err(Error::WriteConfig)?;
        Ok(())
    }
}

use cli::{Command, CLI};

/// Prints game names to stdout. Formatting pending.
fn print_games(games: &[Game]) {
    // TODO: fancy formatting of some sort
    // TODO: maybe move it to Rusteam?
    for game in games.iter() {
        println!("{}", game);
    }
}

#[paw::main]
fn main(args: CLI) -> Result<()> {
    match args.cmd {
        cmd => {
            let config = Config::fetch().unwrap_or_default();
            let GamesRoot(games_root) = &config.games_root;

            match cmd {
                Command::List { patterns } => {
                    let games = rusteam::list_games(&games_root, &patterns.join(" "));
                    print_games(&games);
                }
                Command::Play { patterns } => rusteam::play_game(&games_root, patterns.join(" "))?,
                Command::Install { path } => rusteam::install_game(&path)?,
                Command::Completion(shell) => rusteam::print_completion(&mut CLI::clap(), shell),
                Command::Config(config_action) => match config_action {
                    cli::Config::Init => Config::init()?,
                    cli::Config::Show => config.show()?,
                },
            }
        }
    }

    Ok(())
}
