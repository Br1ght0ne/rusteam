use crate::filesystem::entries;
use crate::game::Game;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod filesystem;
pub mod game;

const IGNORE_FILENAME: &str = ".rusteam-ignore";

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
    iter.filter(|game: &Game| {
        game.name
            .clone()
            .map_or(false, |name| matches(&name, &pattern))
        // REVIEW: is contains enough for now? Yes it is.
    })
    .collect::<Vec<Game>>()
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
