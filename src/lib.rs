#![doc(html_root_url = "https://docs.rs/rusteam/0.4.3")]

use crate::filesystem::entries;
use crate::game::Game;
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::{clap, StructOpt};

pub mod filesystem;
pub mod game;

const IGNORE_FILENAME: &str = ".rusteam-ignore";

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

pub fn play_game(root: &Path, pattern: String) {
    if let Some(game) = list_games(root, pattern).first() {
        if let Some(launcher) = game.launchers.first() {
            let command = Command::new(dbg!(launcher))
                .current_dir(&game.directory)
                .spawn();
            if let Ok(mut child) = command {
                child.wait().expect("game wasn't running");
            } else {
                panic!("{} didn't start", game)
            }
        }
    }
}

pub fn list_games(root: &Path, pattern: String) -> Vec<Game> {
    let iter = games_iter(root);
    let mut games = iter
        .filter(|game: &Game| {
            game.name
                .clone()
                .map_or(false, |name| matches(&name, &pattern))
            // REVIEW: is contains enough for now? Yes it is.
        })
        .collect::<Vec<Game>>();
    // REVIEW: best way to sort?
    games.sort_unstable();
    games
}

fn games_iter(root: &Path) -> impl Iterator<Item = Game> + '_ {
    entries(root)
        .into_iter()
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
