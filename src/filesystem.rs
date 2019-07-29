use std::path::{Path, PathBuf};

pub fn entries(directory: &Path) -> Vec<PathBuf> {
    directory
        .read_dir()
        .expect("failed to read dir")
        .map(|entry| entry.expect("failed to read entry"))
        .map(|entry| entry.file_name())
        .map(|filename| directory.join(filename.to_str().expect("failed to_str")))
        .collect::<Vec<PathBuf>>()
}

pub fn has_same_name_as_parent_dir(filepath: &Path) -> bool {
    if let Some(parent_dir) = filepath.parent() {
        let parent_file_name = parent_dir.file_name();

        parent_file_name
            .and(filepath.file_name())
            .map_or(false, |filename| filename == parent_file_name.unwrap())
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_name_name_as_parent_dir() {
        assert!(has_same_name_as_parent_dir(&PathBuf::from(
            "/path/to/Something/Something"
        )))
    }
}
