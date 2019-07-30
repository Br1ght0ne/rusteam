use std::path::{Path, PathBuf};

/// Gets a Vec of absolute entry paths in a directory.
pub fn entries(directory: &Path) -> Vec<PathBuf> {
    directory
        .read_dir()
        .expect("failed to read dir")
        .map(|entry| entry.expect("failed to read entry"))
        .map(|entry| entry.file_name())
        .map(|filename| directory.join(filename.to_str().expect("failed to_str")))
        .collect::<Vec<PathBuf>>()
}

/// Checks if the last two components of the `filepath` are equal.
///
/// # Caveats
///
/// Is automatically `false` when:
///
/// - there is no parent directory
///
/// # Examples
///
/// ```
/// # use rusteam::filesystem::has_same_name_as_parent_dir;
/// # use std::path::PathBuf;
/// let path = PathBuf::from("/path/to/entry/entry");
/// assert!(has_same_name_as_parent_dir(&path))
/// ```
pub fn has_same_name_as_parent_dir(filepath: &Path) -> bool {
    filepath.parent().map_or(false, |parent_dir| {
        let parent_file_name = parent_dir.file_name();
        parent_file_name.and(filepath.file_name()) == parent_file_name
    })
}
