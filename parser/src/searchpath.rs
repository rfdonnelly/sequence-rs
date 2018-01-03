use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct SearchPath {
    /// Search path for `import`
    pub paths: Vec<PathBuf>,
}

impl SearchPath {
    pub fn new(paths: Vec<PathBuf>) -> SearchPath {
        SearchPath {
            paths,
        }
    }

    /// Sets the search path used for `import`
    ///
    /// The string must be a colon separated list of paths.
    ///
    /// # Errors
    ///
    /// Error will be returned for parsed paths that do not exist.  If the search path string contains
    /// a mix of paths that do and do not exist, the paths that do exist will be added to the internal
    /// search path.
    pub fn from_string(s: &str) -> io::Result<SearchPath> {
        let mut paths: Vec<PathBuf> = Vec::new();
        let mut error_paths: Vec<PathBuf> = Vec::new();

        for path in s.split(':') {
            if !path.is_empty() {
                let path = Path::new(path);

                if path.exists() {
                    paths.push(path.to_path_buf());
                } else {
                    error_paths.push(path.to_path_buf());
                }
            }
        }

        if error_paths.len() > 0 {
            Err(io::Error::new(io::ErrorKind::NotFound,
                               format!("Paths not found:\n{}",
                                       error_paths.iter()
                                       .map(|path| format!("   {:?}", path))
                                       .collect::<Vec<String>>()
                                       .join("\n"))))
        } else {
            Ok(SearchPath::new(paths))
        }
    }

    pub fn add(&mut self, path: &Path) {
        self.paths.push(path.to_path_buf());
    }
}