extern crate dirs;
extern crate serde;
extern crate snafu;
extern crate structopt;

use rusteam::{game::Game, Rusteam};
use std::io::Write;
use structopt::StructOpt;

use colored::*;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use std::fs;
use std::path::{Path, PathBuf};

use snafu::{ErrorCompat, OptionExt, Snafu};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("could not open config at {}: {}", path.display(), source))]
    OpenConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("could not parse config at {}: {}", path.display(), source))]
    ParseConfig {
        path: PathBuf,
        source: toml::de::Error,
    },
    // TODO: what something?
    #[snafu(display("could not serialize something to TOML: {}", source))]
    TomlSerialization { source: toml::ser::Error },
    #[snafu(display("could not write config to {}: {}", path.display(), source))]
    WriteConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    WriteStdout {
        data: Vec<u8>,
        source: std::io::Error,
    },
    #[snafu(display("no .config directory"))]
    NoConfigDir,
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
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        Self(home_dir.join("Games"))
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

impl Config {
    const CONFIG_SUBDIR: &'static str = env!("CARGO_PKG_NAME");
    const CONFIG_FILENAME: &'static str = "config.toml";

    fn path() -> Result<PathBuf> {
        let config_home = dirs::config_dir().context(NoConfigDir)?;
        Ok(config_home
            .join(Self::CONFIG_SUBDIR)
            .join(Self::CONFIG_FILENAME))
    }

    fn fetch(path: &Path) -> Result<Self> {
        let file = &fs::read(&path).context(OpenConfig { path })?;
        let config = String::from_utf8_lossy(file);
        let toml = toml::from_str(&config).context(ParseConfig { path })?;
        Ok(toml)
    }

    fn show(&self) -> Result<()> {
        let toml = toml::to_vec(&self).context(TomlSerialization)?;
        std::io::stdout()
            .write_all(&toml)
            .context(WriteStdout { data: toml })?;
        Ok(())
    }

    fn write(&self, path: &Path) -> Result<()> {
        let toml = toml::to_string_pretty(&self).context(TomlSerialization)?;
        fs::write(path, &toml).context(WriteConfig { path })?;
        Ok(())
    }
}

use cli::{Command, CLI};

fn cli() -> Result<()> {
    let cli = CLI::from_args();
    let config_path = Config::path()?;

    match cli.cmd {
        cmd => {
            let config = Config::fetch(&config_path).unwrap_or_default();
            let games_root = &config.games_root.0;

            match cmd {
                Command::List { patterns } => {
                    let games = Rusteam::list_games(&games_root, patterns.join(" "));
                    print_games(&games);
                }
                Command::Play { patterns } => Rusteam::play_game(&games_root, patterns.join(" ")),
                Command::Config(config_action) => match config_action {
                    cli::Config::Init => config.write(&config_path)?,
                    cli::Config::Show => config.show()?,
                },
            }
        }
    }

    Ok(())
}

/// Prints game names to stdout. Formatting pending.
fn print_games(games: &[Game]) {
    // TODO: fancy formatting of some sort
    for game in games.iter() {
        println!("{}", game);
    }
}

fn main() {
    if let Err(e) = cli() {
        eprintln!("{}\n{}", "An error occured:".red(), e);
        match &e {
            Error::OpenConfig { path, .. } => eprintln!(
                "Please run `rusteam config init` to get the default configuration at {}",
                format!("{}", path.display()).green()
            ),
            _ => unimplemented!(),
        }
        if let Some(backtrace) = ErrorCompat::backtrace(&e) {
            println!("{}", backtrace);
        }
    }
}
