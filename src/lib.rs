#![doc(html_root_url = "https://docs.rs/rusteam/0.5.1")]

use crate::filesystem::entries;
use crate::game::Game;
use snafu::{OptionExt, ResultExt, Snafu};
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::{clap, StructOpt};

pub mod filesystem;
pub mod game;

const IGNORE_FILENAME: &str = ".rusteam-ignore";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to run command {:?}: {}", command, source))]
    CommandSpawnFailed {
        command: String,
        source: std::io::Error,
    },
    #[snafu(display("no game found for pattern {}", pattern))]
    GameNotFound { pattern: String },
    #[snafu(display("no launcher found for game {}", game))]
    LauncherNotFound { game: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

// FIXME: DRY?
#[derive(StructOpt)]
pub enum Shell {
    #[structopt(name = "bash", about = "Print bash completion")]
    Bash,
    #[structopt(name = "elvish", about = "Print elvish completion")]
    Elvish,
    #[structopt(name = "fish", about = "Print fish completion")]
    Fish,
    #[structopt(name = "zsh", about = "Print zsh completion")]
    Zsh,
}

pub fn print_completion(app: &mut clap::App, shell: Shell) {
    // FIXME: DRY?
    let shell = match shell {
        Shell::Bash => clap::Shell::Bash,
        Shell::Elvish => clap::Shell::Elvish,
        Shell::Fish => clap::Shell::Fish,
        Shell::Zsh => clap::Shell::Zsh,
    };
    app.gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut std::io::stdout())
}

pub fn play_game(root: &Path, pattern: String) -> Result<()> {
    let games = list_games(root, &pattern);
    let game = games.first().context(GameNotFound { pattern })?;
    let launcher = game.launchers.first().context(LauncherNotFound {
        game: format!("{}", game),
    })?;
    spawn(&mut Command::new(dbg!(launcher)))
}

pub fn list_games(root: &Path, pattern: &str) -> Vec<Game> {
    let iter = games_iter(root);
    let mut games = iter
        .filter(|game: &Game| {
            game.name
                .clone()
                .map_or(false, |name| matches(&name, pattern))
            // REVIEW: is contains enough for now? Yes it is.
        })
        .collect::<Vec<Game>>();
    // REVIEW: best way to sort?
    games.sort_unstable();
    games
}

pub fn install_game(path: &Path) -> Result<()> {
    // Find installers
    // REVIEW: very similar to Game::find_launchers
    let installers: Vec<PathBuf> = entries(path)
        .into_iter()
        .filter(|e| is_installer(e))
        .collect();

    // Run every installer in order
    let results: Result<Vec<_>, _> = installers
        .iter()
        .map(|installer| spawn(Command::new("sh").arg(dbg!(installer))))
        .collect();

    results.map(|_| ())
}

fn spawn(command: &mut Command) -> Result<()> {
    command
        .spawn()
        .and_then(|mut child| child.wait().map(|_| ()))
        .context(CommandSpawnFailed {
            command: format!("{:?}", command),
        })
}

fn is_installer(filepath: &Path) -> bool {
    let filename = filepath.file_name().map(|ext| ext.to_string_lossy());
    let extension = filepath.extension().map(|ext| ext.to_string_lossy());
    extension
        .filter(|ext| (&["sh"]).contains(&ext.as_ref()))
        .and(filename.filter(|f| f.contains("install") || f.starts_with("gog_")))
        .is_some()
}

fn games_iter(root: &Path) -> impl Iterator<Item = Game> + '_ {
    entries(root)
        .into_iter()
        .filter(|e| e.is_dir())
        .filter(move |e| !ignore(&root.join(e)))
        .map(move |e| Game::from_path(root.join(e)))
}

fn ignore(filepath: &Path) -> bool {
    entries(filepath).into_iter().any(|e: PathBuf| {
        e.file_name()
            .map_or(false, |filename| filename == IGNORE_FILENAME)
    })
}

fn matches(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}
