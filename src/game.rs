use crate::filesystem::{entries, has_same_name_as_parent_dir};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Platform {
    Native,
    Wine,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Genre {
    Action,
    Platformer,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Game {
    pub name: Option<String>,
    pub platform: Option<Platform>,
    pub directory: PathBuf,
    pub genres: Vec<Genre>,
    pub launchers: Vec<PathBuf>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let default = &String::from("a game with no name");
        let name = self.name.as_ref().unwrap_or(default);

        write!(f, "{}", name)
    }
}

impl Game {
    pub fn from_path(directory: PathBuf) -> Self {
        let (platform, launchers) = Self::find_launchers(&directory);

        Self {
            // Name of the game is the name of its directory.
            name: Self::basename(&directory),
            // Genres is beyond us for now.
            genres: vec![],
            platform,
            launchers,
            directory,
        }
    }

    fn find_launchers(directory: &Path) -> (Option<Platform>, Vec<PathBuf>) {
        // We check for knows launchers in the root of the directory.

        let launchers = entries(directory)
            .into_iter()
            .filter(|filepath| Self::is_launcher(filepath))
            .collect::<Vec<PathBuf>>();

        // We can tell the platform if all found launchers belong to it.

        (Self::same_platform(launchers.as_slice()), launchers)
    }

    fn same_platform(launchers: &[PathBuf]) -> Option<Platform> {
        if launchers.is_empty() {
            None
        } else {
            Self::platform(&launchers[0]).filter(|first_platform| {
                launchers
                    .iter()
                    .all(|l| Self::platform(l).filter(|p| p == first_platform).is_some())
            })
        }
    }

    fn platform(file: &Path) -> Option<Platform> {
        match file {
            file if Self::is_native(file) => Some(Platform::Native),
            file if Self::is_wine(file) => Some(Platform::Wine),
            _ => None,
        }
    }

    fn is_launcher(filepath: &Path) -> bool {
        !Self::is_uninstall(filepath)
            && (Self::is_native(filepath)
                || Self::is_wine(filepath)
                || has_same_name_as_parent_dir(filepath))
    }

    fn is_uninstall(file: &Path) -> bool {
        file.file_name()
            .map(|f| f.to_string_lossy())
            .map_or(false, |f| f.contains("uninstall"))
    }

    fn is_native(file: &Path) -> bool {
        Self::extension_in(file, &["sh", "x86", "x86_64"])
    }

    fn is_wine(file: &Path) -> bool {
        Self::extension_in(file, &["exe"])
    }

    fn extension_in(file: &Path, extensions: &[&str]) -> bool {
        file.extension()
            .map(|ext| ext.to_str().expect("failed to_str"))
            .map_or(false, |ext| extensions.contains(&ext))
    }

    fn basename(directory: &Path) -> Option<String> {
        directory
            .file_name()
            .and_then(|f| f.to_str())
            .map(String::from)
    }
}
