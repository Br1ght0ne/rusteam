use crate::filesystem::entries;
use crate::game::Game;
use std::path::{Path, PathBuf};
use std::process::Command;

mod filesystem;
mod game;

pub struct Rusteam;

impl Rusteam {
    const IGNORE_FILENAME: &'static str = ".rusteam-ignore";

    pub fn list_games(root: &Path, pattern: Option<String>) -> Vec<Game> {
        Self::find_games(root, pattern)
    }

    pub fn play_game(root: &Path, pattern: String) {
        if let Some(game) = Self::find_games(root, Some(pattern)).first() {
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

    fn find_games(root: &Path, pattern: Option<String>) -> Vec<Game> {
        let iter = Self::games_iter(root);
        if let Some(pattern) = pattern {
            iter.filter(|game: &Game| {
                game.clone() // HACK: I hate seeing clone
                    .name
                    .map_or(false, |name| Self::matches(&name, &pattern))
                // REVIEW: is contains enough for now?
            })
            .collect::<Vec<Game>>()
        } else {
            iter.collect::<Vec<Game>>()
        }
    }

    fn games_iter(root: &Path) -> impl Iterator<Item = Game> + '_ {
        entries(root)
            .into_iter()
            .filter(move |e| !Self::ignore(&root.join(e)))
            .map(move |e| Game::from_path(root.join(e)))
    }

    fn ignore(filepath: &Path) -> bool {
        entries(filepath).into_iter().any(|e: PathBuf| {
            e.file_name()
                .map_or(false, |filename| filename == Self::IGNORE_FILENAME)
        })
    }

    fn matches(haystack: &str, needle: &str) -> bool {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}
