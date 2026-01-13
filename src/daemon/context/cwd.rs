use std::fs;
use std::path::Path;

use anyhow::Result;
use tracing::debug;

/// List files in the current working directory
pub fn list_files(cwd: &Path, max_files: usize) -> Result<Vec<String>> {
    if !cwd.exists() || !cwd.is_dir() {
        debug!("CWD does not exist or is not a directory: {}", cwd.display());
        return Ok(Vec::new());
    }

    let mut entries: Vec<FileEntry> = Vec::new();

    let dir_entries = fs::read_dir(cwd)?;

    for entry in dir_entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files by default
        if file_name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let is_symlink = metadata.as_ref().map(|m| m.is_symlink()).unwrap_or(false);

        // Get extension for sorting
        let extension = Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        entries.push(FileEntry {
            name: file_name,
            is_dir,
            is_symlink,
            extension,
        });
    }

    // Sort: directories first, then by extension, then by name
    entries.sort_by(|a, b| {
        // Directories first
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // Then by extension
                match (&a.extension, &b.extension) {
                    (Some(ea), Some(eb)) => ea.cmp(eb).then(a.name.cmp(&b.name)),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => a.name.cmp(&b.name),
                }
            }
        }
    });

    // Limit and format
    let result: Vec<String> = entries
        .into_iter()
        .take(max_files)
        .map(|e| format_entry(&e))
        .collect();

    Ok(result)
}

#[derive(Debug)]
struct FileEntry {
    name: String,
    is_dir: bool,
    is_symlink: bool,
    extension: Option<String>,
}

/// Format a file entry (similar to ls -F)
fn format_entry(entry: &FileEntry) -> String {
    let suffix = if entry.is_dir {
        "/"
    } else if entry.is_symlink {
        "@"
    } else {
        ""
    };

    format!("{}{}", entry.name, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_list_files() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // Create some test files
        fs::write(dir_path.join("file1.txt"), "").unwrap();
        fs::write(dir_path.join("file2.rs"), "").unwrap();
        fs::create_dir(dir_path.join("subdir")).unwrap();
        fs::write(dir_path.join(".hidden"), "").unwrap();

        let files = list_files(dir_path, 10).unwrap();

        // Should have 3 entries (hidden file excluded)
        assert_eq!(files.len(), 3);

        // Directory should come first
        assert!(files[0].ends_with('/'));
        assert_eq!(files[0], "subdir/");
    }

    #[test]
    fn test_format_entry() {
        let dir_entry = FileEntry {
            name: "src".to_string(),
            is_dir: true,
            is_symlink: false,
            extension: None,
        };
        assert_eq!(format_entry(&dir_entry), "src/");

        let file_entry = FileEntry {
            name: "main.rs".to_string(),
            is_dir: false,
            is_symlink: false,
            extension: Some("rs".to_string()),
        };
        assert_eq!(format_entry(&file_entry), "main.rs");
    }
}
