extern crate dirs;
extern crate serde;
extern crate snafu;
extern crate structopt;

use crate::structopt::StructOpt;
use rusteam::Rusteam;

use colored::*;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use std::fs;
use std::path::{Path, PathBuf};

use snafu::{ErrorCompat, OptionExt, Snafu};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("could not open config from {}: {}", config_path.display(), source))]
    OpenConfig {
        config_path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("could not parse config from {}: {}", config_path.display(), source))]
    ParseConfig {
        config_path: PathBuf,
        source: toml::de::Error,
    },
    #[snafu(display("could not write config to {}: {}", config_path.display(), source))]
    WriteConfig {
        config_path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("games_root is not defined"))]
    GamesRootNotDefined,
    #[snafu(display("no .config directory"))]
    NoConfigDir,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default = "GamesRoot::default")]
    games_root: GamesRoot,
    #[serde(default = "Config::default_wait_for_game_process")]
    wait_for_game_process: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct GamesRoot(Option<PathBuf>);

impl Default for GamesRoot {
    fn default() -> Self {
        GamesRoot(dirs::home_dir().map(|home| home.join("Games")))
    }
}

impl Config {
    fn default_wait_for_game_process() -> bool {
        true
    }
}

mod cli {
    use structopt::StructOpt;

    #[derive(StructOpt)]
    pub struct CLI {
        #[structopt(subcommand)]
        pub cmd: Command,
    }

    #[derive(StructOpt)]
    pub enum Command {
        #[structopt(name = "ls", about = "List your games")]
        List {
            #[structopt(help = "substring of game name")]
            pattern: Option<String>,
        },
        #[structopt(name = "play", about = "Run a game")]
        Play {
            #[structopt(help = "substring of game name")]
            pattern: String,
        },
        #[structopt(name = "config", about = "Manage your configuration")]
        Config(Config),
    }

    #[derive(StructOpt)]
    pub enum Config {
        #[structopt(name = "create", about = "Create a default configuration file")]
        Create,
    }
}

fn fetch_config(config_path: &Path) -> Result<Config> {
    let file = &fs::read(&config_path).context(OpenConfig { config_path })?;
    let config =
        toml::from_str(&String::from_utf8_lossy(file)).context(ParseConfig { config_path })?;
    Ok(config)
}

fn write_config(config: &Config, config_path: &Path) -> Result<()> {
    fs::write(
        config_path,
        toml::to_string_pretty(config).expect("can't pretty config"),
    )
    .context(WriteConfig { config_path })?;
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    dirs::config_dir()
        .context(NoConfigDir)
        .map(|config_dir| config_dir.join("rusteam/config.toml"))
}

use cli::{Command, CLI};

fn cli() -> Result<()> {
    let cli = CLI::from_args();
    let config_path = config_path()?;

    match cli.cmd {
        Command::Config(cmd) => match cmd {
            cli::Config::Create => write_config(&Config::default(), &config_path)?,
        },
        cmd => {
            let config = fetch_config(&config_path)?;
            let Config {
                games_root,
                wait_for_game_process,
            } = config;
            let games_root = &games_root.0.context(GamesRootNotDefined).unwrap();
            match cmd {
                Command::List { pattern } => {
                    let games = Rusteam::list_games(&games_root, pattern);
                    for game in games.iter() {
                        println!("{}", game);
                    }
                }
                Command::Play { pattern } => {
                    Rusteam::play_game(&games_root, pattern, wait_for_game_process)
                }
                Command::Config(_) => unreachable!(),
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = cli() {
        eprintln!("{} {}", "An error occured:".red(), e);
        if let Some(backtrace) = ErrorCompat::backtrace(&e) {
            println!("{}", backtrace);
        }
    }
}
