extern crate dirs;
extern crate serde;
extern crate snafu;
extern crate structopt;

use crate::structopt::StructOpt;
use rusteam::Rusteam;
use std::io::Write;

use colored::*;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use std::fs;
use std::path::{Path, PathBuf};

use snafu::{ErrorCompat, OptionExt, Snafu};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("could not open config at {}: {}", config_path.display(), source))]
    OpenConfig {
        config_path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("could not parse config at {}: {}", config_path.display(), source))]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Config {
    #[serde(default = "GamesRoot::default")]
    games_root: GamesRoot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GamesRoot(Option<PathBuf>);

impl Default for GamesRoot {
    fn default() -> Self {
        Self(dirs::home_dir().map(|home| home.join("Games")))
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
            #[structopt(help = "substrings of game name")]
            patterns: Vec<String>,
        },
        #[structopt(name = "play", about = "Run a game")]
        Play {
            #[structopt(help = "substrings of game name")]
            patterns: Vec<String>,
        },
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

fn show_config(config: &Config) {
    std::io::stdout()
        .write(&toml::to_vec(config).expect("can't pretty config"))
        .expect("failed writing to stdout");
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
        Command::Config(cli::Config::Init) => write_config(&Config::default(), &config_path)?,
        cmd => {
            let config = fetch_config(&config_path)?;
            let games_root = &config
                .clone()
                .games_root
                .0
                .context(GamesRootNotDefined)
                .unwrap();
            match cmd {
                Command::List { patterns } => {
                    let games = Rusteam::list_games(&games_root, patterns.join(" "));
                    for game in games.iter() {
                        println!("{}", game);
                    }
                }
                Command::Play { patterns } => Rusteam::play_game(&games_root, patterns.join(" ")),
                Command::Config(cmd) => match cmd {
                    cli::Config::Init => unreachable!(),
                    cli::Config::Show => show_config(&config),
                },
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = cli() {
        eprintln!("{}\n{}", "An error occured:".red(), e);
        match &e {
            Error::OpenConfig {
                config_path,
                source: _,
            } => eprintln!(
                "Please run `rusteam config init` to get the default configuration at {}",
                format!("{}", config_path.display()).green()
            ),
            _ => (),
        }
        if let Some(backtrace) = ErrorCompat::backtrace(&e) {
            println!("{}", backtrace);
        }
    }
}
